//! Implementation of the Multipurpose Clock Generator
//!

use ::core::mem;
use core::cell::Cell;
use osc;
use sim;

use regs::mcg::*;

use self::Control1::CLKS::Value as OscSource;

#[derive(Copy,Clone,PartialEq)]
enum State {
    Fei(Fei),
    Fee(Fee),
    Fbi(Fbi),
    Fbe(Fbe),
    Pbe(Pbe),
    Pee(Pee),
    Blpi(Blpi),
    Blpe(Blpe),
}

#[derive(Copy,Clone,PartialEq)]
struct Fei;

#[derive(Copy,Clone,PartialEq)]
struct Fee;

#[derive(Copy,Clone,PartialEq)]
struct Fbi;

#[derive(Copy,Clone,PartialEq)]
struct Fbe;

#[derive(Copy,Clone,PartialEq)]
struct Pbe;

#[derive(Copy,Clone,PartialEq)]
struct Pee;

#[derive(Copy,Clone,PartialEq)]
struct Blpi;

#[derive(Copy,Clone,PartialEq)]
struct Blpe;

#[derive(Copy,Clone)]
enum OscClock {
    Oscillator,
    RTC32K,
    IRC48M,
}

#[allow(dead_code)]
#[derive(Copy,Clone)]
enum OscRange {
    Low = 0,
    High = 1,
    VeryHigh = 2
}

#[allow(non_camel_case_types, dead_code)]
#[derive(Copy,Clone)]
enum OscCapacitance {
    Load_0pF = 0b0000,
    Load_2pF = 0b1000,
    Load_4pF = 0b0100,
    Load_6pF = 0b1100,
    Load_8pF = 0b0010,
    Load_10pF = 0b1010,
    Load_12pF = 0b0110,
    Load_14pF = 0b1110,
    Load_16pF = 0b0001,
    Load_18pF = 0b1001,
    Load_20pF = 0b0101,
    Load_22pF = 0b1101,
    Load_24pF = 0b0011,
    Load_26pF = 0b1011,
    Load_28pF = 0b0111,
    Load_30pF = 0b1111
}

#[allow(non_camel_case_types, dead_code)]
#[derive(Copy,Clone)]
enum Frdiv {
    Low1_High32 = 0,
    Low2_High64 = 1,
    Low4_High128 = 2,
    Low8_High256 = 3,
    Low16_High512 = 4,
    Low32_High1024 = 5,
    Low64_High1280 = 6,
    Low128_High1536 = 7
}

//TODO FCRDIV can divide freq of internal reference clock
// modify Ircs to pass in freq?
#[derive(Copy,Clone)]
enum Ircs {
    SlowInternal,
    FastInternal,
}

#[derive(Copy,Clone)]
struct Xtal {
    clock: OscClock,
    range: OscRange,
    frdiv: Frdiv,
    load: OscCapacitance
}

#[allow(non_upper_case_globals)]
const Teensy16MHz: Xtal = Xtal {
    clock: OscClock::Oscillator,
    range: OscRange::VeryHigh,
    frdiv: Frdiv::Low16_High512,
    load: OscCapacitance::Load_10pF
};

#[allow(non_upper_case_globals)]
const Teensy32KHz: Xtal = Xtal {
    clock: OscClock::RTC32K,
    range: OscRange::Low,
    frdiv: Frdiv::Low1_High32,
    load: OscCapacitance::Load_10pF
};

#[allow(non_upper_case_globals)]
const Teensy48MHz: Xtal = Xtal {
    clock: OscClock::IRC48M,
    range: OscRange::VeryHigh,
    frdiv: Frdiv::Low128_High1536,
    load: OscCapacitance::Load_10pF
};

trait ClockChange {
    fn to_fei(self, freq: u32) -> State;
    fn to_fee(self, freq: u32, xtal: Xtal) -> State;
    fn to_fbi(self, ircs: Ircs) -> State;
    fn to_blpi(self, ircs: Ircs) -> State;
    fn to_fbe(self, xtal: Xtal) -> State;
    fn to_pbe(self, freq: u32, xtal: Xtal) -> State;
    fn to_blpe(self, xtal: Xtal) -> State;
    fn to_pee(self, freq: u32, xtal: Xtal) -> State;
}

impl ClockChange for State {
    fn to_fei(self, freq: u32) -> State {
        match self {
            State::Fei(Fei) => Fei.to_fei(freq),
            State::Fee(Fee) => Fee.to_fei(freq),
            State::Fbi(Fbi) => Fbi.to_fei(freq),
            State::Fbe(Fbe) => Fbe.to_fei(freq),
            State::Pbe(Pbe) => Pbe.to_fei(freq),
            State::Pee(Pee) => Pee.to_fei(freq),
            State::Blpi(Blpi) => Blpi.to_fei(freq),
            State::Blpe(Blpe) => Blpe.to_fei(freq),
        }
    }
    fn to_fee(self, freq: u32, xtal: Xtal) -> State {
        match self {
            State::Fei(Fei) => Fei.to_fee(freq, xtal),
            State::Fee(Fee) => Fee.to_fee(freq, xtal),
            State::Fbi(Fbi) => Fbi.to_fee(freq, xtal),
            State::Fbe(Fbe) => Fbe.to_fee(freq, xtal),
            State::Pbe(Pbe) => Pbe.to_fee(freq, xtal),
            State::Pee(Pee) => Pee.to_fee(freq, xtal),
            State::Blpi(Blpi) => Blpi.to_fee(freq, xtal),
            State::Blpe(Blpe) => Blpe.to_fee(freq, xtal),
        }
    }
    fn to_fbi(self, ircs: Ircs) -> State {
        match self {
            State::Fei(Fei) => Fei.to_fbi(ircs),
            State::Fee(Fee) => Fee.to_fbi(ircs),
            State::Fbi(Fbi) => Fbi.to_fbi(ircs),
            State::Fbe(Fbe) => Fbe.to_fbi(ircs),
            State::Pbe(Pbe) => Pbe.to_fbi(ircs),
            State::Pee(Pee) => Pee.to_fbi(ircs),
            State::Blpi(Blpi) => Blpi.to_fbi(ircs),
            State::Blpe(Blpe) => Blpe.to_fbi(ircs),
        }
    }
    fn to_blpi(self, ircs: Ircs) -> State {
        match self {
            State::Fei(Fei) => Fei.to_blpi(ircs),
            State::Fee(Fee) => Fee.to_blpi(ircs),
            State::Fbi(Fbi) => Fbi.to_blpi(ircs),
            State::Fbe(Fbe) => Fbe.to_blpi(ircs),
            State::Pbe(Pbe) => Pbe.to_blpi(ircs),
            State::Pee(Pee) => Pee.to_blpi(ircs),
            State::Blpi(Blpi) => Blpi.to_blpi(ircs),
            State::Blpe(Blpe) => Blpe.to_blpi(ircs),
        }
    }
    fn to_fbe(self, xtal: Xtal) -> State {
        match self {
            State::Fei(Fei) => Fei.to_fbe(xtal),
            State::Fee(Fee) => Fee.to_fbe(xtal),
            State::Fbi(Fbi) => Fbi.to_fbe(xtal),
            State::Fbe(Fbe) => Fbe.to_fbe(xtal),
            State::Pbe(Pbe) => Pbe.to_fbe(xtal),
            State::Pee(Pee) => Pee.to_fbe(xtal),
            State::Blpi(Blpi) => Blpi.to_fbe(xtal),
            State::Blpe(Blpe) => Blpe.to_fbe(xtal),
        }
    }
    fn to_pbe(self, freq: u32, xtal: Xtal) -> State {
        match self {
            State::Fei(Fei) => Fei.to_pbe(freq, xtal),
            State::Fee(Fee) => Fee.to_pbe(freq, xtal),
            State::Fbi(Fbi) => Fbi.to_pbe(freq, xtal),
            State::Fbe(Fbe) => Fbe.to_pbe(freq, xtal),
            State::Pbe(Pbe) => Pbe.to_pbe(freq, xtal),
            State::Pee(Pee) => Pee.to_pbe(freq, xtal),
            State::Blpi(Blpi) => Blpi.to_pbe(freq, xtal),
            State::Blpe(Blpe) => Blpe.to_pbe(freq, xtal),
        }
    }
    fn to_blpe(self, xtal: Xtal) -> State {
        match self {
            State::Fei(Fei) => Fei.to_blpe(xtal),
            State::Fee(Fee) => Fee.to_blpe(xtal),
            State::Fbi(Fbi) => Fbi.to_blpe(xtal),
            State::Fbe(Fbe) => Fbe.to_blpe(xtal),
            State::Pbe(Pbe) => Pbe.to_blpe(xtal),
            State::Pee(Pee) => Pee.to_blpe(xtal),
            State::Blpi(Blpi) => Blpi.to_blpe(xtal),
            State::Blpe(Blpe) => Blpe.to_blpe(xtal),
        }
    }
    fn to_pee(self, freq: u32, xtal: Xtal) -> State {
        match self {
            State::Fei(Fei) => Fei.to_pee(freq, xtal),
            State::Fee(Fee) => Fee.to_pee(freq, xtal),
            State::Fbi(Fbi) => Fbi.to_pee(freq, xtal),
            State::Fbe(Fbe) => Fbe.to_pee(freq, xtal),
            State::Pbe(Pbe) => Pbe.to_pee(freq, xtal),
            State::Pee(Pee) => Pee.to_pee(freq, xtal),
            State::Blpi(Blpi) => Blpi.to_pee(freq, xtal),
            State::Blpe(Blpe) => Blpe.to_pee(freq, xtal),
        }
    }
}

fn state() -> State {
    let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

    let clks: OscSource = match mcg.c1.read(Control1::CLKS) {
        1 => OscSource::Internal,
        2 => OscSource::External,
        _ => OscSource::LockedLoop
    };

    let irefs = mcg.c1.is_set(Control1::IREFS);
    let plls = mcg.c6.is_set(Control6::PLLS);
    let lp = mcg.c2.is_set(Control2::LP);

    match (clks, irefs, plls, lp) {
        (OscSource::LockedLoop, true, false, _) => State::Fei(Fei),
        (OscSource::LockedLoop, false, false, _) => State::Fee(Fee),
        (OscSource::Internal, true, false, false) => State::Fbi(Fbi),
        (OscSource::External, false, false, false) => State::Fbe(Fbe),
        (OscSource::LockedLoop, false, true, _) => State::Pee(Pee),
        (OscSource::External, false, true, false) => State::Pbe(Pbe),
        (OscSource::Internal, true, false, true) => State::Blpi(Blpi),
        (OscSource::External, false, _, true) => State::Blpe(Blpe),
        _ => panic!("Not in a recognized power mode!")
    }
}

//TODO bus and flash dividers
fn set_pll_freq(freq: u32) {
    let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

    let (pll_mul, pll_div) = match freq {
        64 => (16, 2),
        68 => (17, 2),
        72 => (18, 2),
        76 => (19, 2),
        80 => (20, 2),
        84 => (21, 2),
        88 => (22, 2),
        92 => (23, 2),
        96 => (24, 2),
        100 => (25, 2),
        104 => (26, 2),
        108 => (27, 2),
        112 => (28, 2),
        116 => (29, 2),
        120 => (30, 2),

        128 => (16, 1),
        136 => (17, 1),
        144 => (18, 1),
        152 => (19, 1),
        160 => (20, 1),
        168 => (21, 1),
        176 => (22, 1),

        _ => panic!("Invalid pll frequency selected!")
    };

    mcg.c5.modify(Control5::PRDIV.val(pll_div - 1));

    mcg.c6.modify(Control6::VDIV.val(pll_mul - 16));
}

fn set_fll_freq(freq: u32) {
    let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

    let drs_val = match freq {
        24 => 0,
        48 => 1,
        72 => 2,
        96 => 4,
        _ => panic!("Invalid fll frequency selected!")
    };

    match state() {
        State::Fei(Fei) | State::Fbi(Fbi) => {
            mcg.c4.modify(Control4::DRST_DRS.val(drs_val as u8))
        }
        State::Fee(Fee) | State::Fbe(Fbe) => {
            mcg.c4.modify(Control4::DRST_DRS.val(drs_val as u8) +
                          Control4::DMX32::SET);
        }
        _ => {}
    };
}

// Source: https://branan.github.io/teensy/2017/01/28/uart.html
impl ClockChange for Fei {
    fn to_fei(self, freq: u32) -> State {
        set_fll_freq(freq);
        State::Fei(Fei)
    }
    fn to_fbi(self, ircs: Ircs) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c1.modify(Control1::CLKS::Internal);

        while !mcg.s.matches_all(Status::CLKST::Internal) {}

        mcg.c2.modify(Control2::IRCS.val(ircs as u8));

        mcg.sc.modify(StatusControl::FCRDIV.val(0 as u8));

        State::Fbi(Fbi)
    }
    fn to_fee(self, freq: u32, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };


        mcg.c2.modify(Control2::RANGE.val(xtal.range as u8) +
                      Control2::EREFS::SET);

        mcg.c7.modify(Control7::OSCSEL.val(xtal.clock as u8));

        mcg.c1.modify(Control1::FRDIV.val(xtal.frdiv as u8) +
                     Control1::IREFS::CLEAR);

        set_fll_freq(freq);

        while !mcg.s.matches_all(Status::OSCINIT0::SET +
                             Status::IREFST::External) {} 

        State::Fee(Fee)
    }
    fn to_blpi(self, ircs: Ircs) -> State { self.to_fbi(ircs) }
    fn to_fbe(self, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c2.modify(Control2::RANGE.val(xtal.range as u8) +
                      Control2::EREFS::SET);

        mcg.c7.modify(Control7::OSCSEL.val(xtal.clock as u8));

        mcg.c1.modify(Control1::CLKS::External +
                     Control1::FRDIV.val(xtal.frdiv as u8) +
                     Control1::IREFS::CLEAR);

        while !mcg.s.matches_all(Status::OSCINIT0::SET +
                             Status::IREFST::External +
                             Status::CLKST::External) {}

        State::Fbe(Fbe)
    }
    fn to_pbe(self, _freq: u32, xtal: Xtal) -> State { self.to_fbe(xtal) }
    fn to_blpe(self, xtal: Xtal) -> State { self.to_fbe(xtal) }
    fn to_pee(self, _freq: u32, xtal: Xtal) -> State { self.to_fbe(xtal) }
}

impl ClockChange for Fee {
    fn to_fei(self, freq: u32) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c1.modify(Control1::IREFS::SET);
        set_fll_freq(freq);

        while !mcg.s.matches_all(Status::IRCST::Slow +
                             Status::IREFST::Internal) {} 
    
        State::Fei(Fei)
    }
    fn to_fee(self, freq: u32, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c2.modify(Control2::RANGE.val(xtal.range as u8) +
                      Control2::EREFS::SET);

        mcg.c7.modify(Control7::OSCSEL.val(xtal.clock as u8));

        mcg.c1.modify(Control1::FRDIV.val(xtal.frdiv as u8) +
                     Control1::IREFS::CLEAR);

        set_fll_freq(freq);

        while !mcg.s.matches_all(Status::OSCINIT0::SET +
                             Status::IREFST::External) {} 
        State::Fee(Fee)

    }
    fn to_fbi(self, ircs: Ircs) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };
        
        mcg.c2.modify(Control2::IRCS.val(ircs as u8));

        mcg.c1.modify(Control1::CLKS::Internal + Control1::IREFS::SlowInternal);

        while !mcg.s.matches_all(Status::IRCST.val(ircs as u8) + Status::IREFST::Internal + Status::CLKST::Internal) {}

        mcg.sc.modify(StatusControl::FCRDIV.val(0 as u8));
    
        State::Fbi(Fbi)
    }
    fn to_blpi(self, ircs: Ircs) -> State { self.to_fbi(ircs) }
    fn to_fbe(self, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c7.modify(Control7::OSCSEL.val(xtal.clock as u8));

        mcg.c1.modify(Control1::CLKS::External);

        while !mcg.s.matches_all(Status::CLKST::External) {}

        State::Fbe(Fbe)
    }
    fn to_pbe(self, _freq: u32, xtal: Xtal) -> State { self.to_fbe(xtal) }
    fn to_blpe(self, xtal: Xtal) -> State { self.to_fbe(xtal) }
    fn to_pee(self, _freq: u32, xtal: Xtal) -> State { self.to_fbe(xtal) }
}

impl ClockChange for Fbi {
    fn to_fei(self, freq: u32) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c1.modify(Control1::CLKS::LockedLoop);

        set_fll_freq(freq);

        while !mcg.s.matches_all(Status::CLKST::Fll + 
                                 Status::IRCST::Slow) {} 

        State::Fei(Fei)
    }
    fn to_fee(self, freq: u32, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c2.modify(Control2::RANGE.val(xtal.range as u8) +
                      Control2::EREFS::SET);

        mcg.c7.modify(Control7::OSCSEL.val(xtal.clock as u8));

        mcg.c1.modify(Control1::CLKS::LockedLoop +
                     Control1::FRDIV.val(xtal.frdiv as u8) +
                     Control1::IREFS::External);

        set_fll_freq(freq);

        while !mcg.s.matches_all(Status::OSCINIT0::SET +
                                 Status::CLKST::Fll + 
                                 Status::IREFST::External) {} 

        State::Fee(Fee)
    }
    fn to_fbi(self, ircs: Ircs) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c2.modify(Control2::IRCS.val(ircs as u8));

        State::Fbi(Fbi)
    }
    fn to_blpi(self, ircs: Ircs) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };
    
        mcg.c2.modify(Control2::IRCS.val(ircs as u8) + Control2::LP::SET);

        State::Blpi(Blpi)
    }
    fn to_fbe(self, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c2.modify(Control2::RANGE.val(xtal.range as u8) +
                      Control2::EREFS::SET);

        mcg.c7.modify(Control7::OSCSEL.val(xtal.clock as u8));

        mcg.c1.modify(Control1::CLKS::External +
                     Control1::FRDIV.val(xtal.frdiv as u8) +
                     Control1::IREFS::External);

        while !mcg.s.matches_all(Status::OSCINIT0::SET +
                                 Status::CLKST::External + 
                                 Status::IREFST::External) {} 

        State::Fbe(Fbe)
    }
    fn to_pbe(self, _freq: u32, xtal: Xtal) -> State { self.to_fbe(xtal) }
    fn to_blpe(self, xtal: Xtal) -> State { self.to_fbe(xtal) }
    fn to_pee(self, _freq: u32, xtal: Xtal) -> State { self.to_fbe(xtal) }
}

impl ClockChange for Fbe {
    fn to_fei(self, freq: u32) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c1.modify(Control1::CLKS::LockedLoop+
                     Control1::IREFS::SlowInternal);

        set_fll_freq(freq);

        while !mcg.s.matches_all(Status::IREFST::Internal + Status::CLKST::Fll + 
                                 Status::IRCST::Slow) {}

        State::Fei(Fei)
    }
    fn to_fee(self, freq: u32, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c1.modify(Control1::CLKS::LockedLoop);

        while !mcg.s.matches_all(Status::CLKST::Fll) {}

        mcg.c2.modify(Control2::RANGE.val(xtal.range as u8) +
                      Control2::EREFS::SET);

        mcg.c7.modify(Control7::OSCSEL.val(xtal.clock as u8));

        mcg.c1.modify(Control1::FRDIV.val(xtal.frdiv as u8) +
                     Control1::IREFS::CLEAR);

        set_fll_freq(freq);

        State::Fee(Fee)
    }
    fn to_fbi(self, ircs: Ircs) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };
        
        mcg.c1.modify(Control1::CLKS::Internal +
                      Control1::IREFS::SlowInternal);

        while !mcg.s.matches_all(Status::IREFST::Internal + Status::CLKST::Internal) {}

        mcg.c2.modify(Control2::IRCS.val(ircs as u8));

        mcg.sc.modify(StatusControl::FCRDIV.val(0 as u8));

        State::Fbi(Fbi)
    }
    fn to_blpi(self, ircs: Ircs) -> State { self.to_fbi(ircs) }
    fn to_fbe(self, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        if mcg.c7.read(Control7::OSCSEL) !=  xtal.clock as u8 {
            self.to_fbi(Ircs::FastInternal)
        } else {
            State::Fbe(Fbe)
        }
    }
    fn to_pbe(self, freq: u32, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        if mcg.c7.read(Control7::OSCSEL) !=  xtal.clock as u8 {
            self.to_fbi(Ircs::FastInternal)
        } else {
            set_pll_freq(freq);

            mcg.c6.modify(Control6::PLLS::SET);

            // Wait for PLL to be selected and stable PLL lock
            while !mcg.s.matches_all(Status::PLLST::PllcsOutput + Status::LOCK0::SET) {}

            State::Pbe(Pbe)
        }
    }
    fn to_blpe(self, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        if mcg.c7.read(Control7::OSCSEL) !=  xtal.clock as u8 {
            self.to_fbi(Ircs::FastInternal)
        } else {
            mcg.c2.modify(Control2::LP::SET);
            State::Blpe(Blpe)
        }
    }
    fn to_pee(self, freq: u32, xtal: Xtal) -> State { 
        self.to_pbe(freq, xtal) 
    }
}

impl ClockChange for Pbe {
    fn to_fei(self, _freq: u32) -> State { self.to_fbe(Teensy32KHz) }
    fn to_fee(self, _freq: u32, xtal: Xtal) -> State { self.to_fbe(xtal) }
    fn to_fbi(self, _ircs: Ircs) -> State { self.to_fbe(Teensy32KHz) }
    fn to_blpi(self, _ircs: Ircs) -> State { self.to_fbe(Teensy32KHz) }
    fn to_fbe(self, _xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c6.modify(Control6::PLLS::CLEAR);

        while !mcg.s.matches_all(Status::PLLST::Fll ) {}
        
        State::Fbe(Fbe) 
    }
    fn to_pbe(self, _freq: u32, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };
        
        if mcg.c7.read(Control7::OSCSEL) !=  xtal.clock as u8 {
            //TODO if freq is different
            self.to_fbe(xtal) 
        } else { 
            State::Pbe(Pbe)
        }
    }
    fn to_blpe(self, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };
        
        if mcg.c7.read(Control7::OSCSEL) !=  xtal.clock as u8 {
            self.to_fbe(xtal) 
        } else {
            mcg.c2.modify(Control2::LP::SET);

            State::Blpe(Blpe)
        }
    }
    fn to_pee(self, _freq: u32, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        if mcg.c7.read(Control7::OSCSEL) !=  xtal.clock as u8 {
            self.to_fbe(xtal) 
            //TODO handle case where freq is different
        } else {
            mcg.c1.modify(Control1::CLKS::LockedLoop);

            while !mcg.s.matches_all(Status::CLKST::Pll) {}

            State::Pee(Pee)
        }
    }
}

impl ClockChange for Pee {
    fn to_fei(self, freq: u32) -> State { self.to_pbe(freq, Teensy32KHz) } 
    fn to_fee(self, freq: u32, xtal: Xtal) -> State {self.to_pbe(freq, xtal) }
    fn to_fbi(self, _ircs: Ircs) -> State { self.to_pbe(0, Teensy32KHz) }
    fn to_blpi(self, _ircs: Ircs) -> State { self.to_pbe(0, Teensy32KHz) }
    fn to_fbe(self, xtal: Xtal) -> State { self.to_pbe(0, xtal) }
    fn to_pbe(self, _freq: u32, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c7.modify(Control7::OSCSEL.val(xtal.clock as u8));

        mcg.c1.modify(Control1::CLKS::External);

        while !mcg.s.matches_all(Status::CLKST::External) {}

        State::Pbe(Pbe)
    }
    fn to_blpe(self, xtal: Xtal) -> State { self.to_pbe(0, xtal) }
    fn to_pee(self, _freq: u32, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        if mcg.c7.read(Control7::OSCSEL) !=  xtal.clock as u8 {
            self.to_fbe(xtal) 
            //TODO handle case where freq is different
        } else {
            State::Pee(Pee)
        }
    }
}

impl ClockChange for Blpi {
    fn to_fei(self, _freq: u32) -> State { self.to_fbi(Ircs::SlowInternal) }
    fn to_fee(self, _freq: u32, _xtal: Xtal) -> State { self.to_fbi(Ircs::FastInternal) }
    fn to_fbi(self, ircs: Ircs) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c2.modify(Control2::IRCS.val(ircs as u8) + Control2::LP::CLEAR);

        while !mcg.s.matches_all(Status::IREFST::Internal) {}

        State::Fbi(Fbi)
    }
    fn to_blpi(self, ircs: Ircs) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c2.modify(Control2::IRCS.val(ircs as u8));

        State::Blpi(Blpi)
    }
    fn to_fbe(self, _xtal: Xtal) -> State { self.to_fbi(Ircs::FastInternal) }
    fn to_pbe(self, _freq: u32, _xtal: Xtal) -> State { self.to_fbi(Ircs::FastInternal) }
    fn to_blpe(self, _xtal: Xtal) -> State { self.to_fbi(Ircs::FastInternal) }
    fn to_pee(self, _freq: u32, _xtal: Xtal) -> State { self.to_fbi(Ircs::FastInternal) }
}

impl ClockChange for Blpe {
    fn to_fei(self, _freq: u32) -> State { self.to_fbe(Teensy32KHz) }
    fn to_fee(self, _freq: u32, xtal: Xtal) -> State { self.to_fbe(xtal) }
    fn to_fbi(self, _ircs: Ircs) -> State { self.to_fbe(Teensy32KHz) }
    fn to_blpi(self, _ircs: Ircs) -> State { self.to_fbe(Teensy32KHz) }
    fn to_fbe(self, _xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };
        
        mcg.c6.modify(Control6::PLLS::CLEAR);

        mcg.c2.modify(Control2::LP::CLEAR);

        while !mcg.s.matches_all(Status::PLLST::Fll) {}
        
        State::Fbe(Fbe)
    }
    fn to_pbe(self, freq: u32, _xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        set_pll_freq(freq);

        mcg.c6.modify(Control6::PLLS::SET);

        mcg.c2.modify(Control2::LP::CLEAR);

        while !mcg.s.matches_all(Status::PLLST::PllcsOutput + Status::LOCK0::SET) {}

        State::Pbe(Pbe)
    }
    fn to_blpe(self, xtal: Xtal) -> State {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        if mcg.c7.read(Control7::OSCSEL) !=  xtal.clock as u8 {
            self.to_fbe(xtal) 
        } else {
            State::Blpe(Blpe)
        }
    }
    fn to_pee(self, freq: u32, xtal: Xtal) -> State { self.to_pbe(freq, xtal) }
}

#[derive(Copy,Clone,PartialEq)]
pub enum SystemClockSource {
    Oscillator,
    RTC32K,
    IRC48M,
    SlowInternal,
    FastInternal,
    FLL(u32),
    PLL(u32),
}

pub struct SystemClockManager {
    clock_source: Cell<SystemClockSource>,
}

pub static mut SCM: SystemClockManager = SystemClockManager::new(SystemClockSource::FLL(24));

#[allow(non_upper_case_globals)]
const MHz: u32 = 1_000_000;

// On reset, MCGOUTCLK is sourced from the 32kHz internal reference clock 
// multiplied by the FLL, which has a default multiplier of 640.
static mut CORECLK: u32 = 20_480_000;
static mut BUSCLK: u32 = 20_480_000;
static mut FLASHCLK: u32 = 10_240_000;

impl SystemClockManager {
    const fn new(clock_source: SystemClockSource) -> SystemClockManager {
        SystemClockManager {
            clock_source: Cell::new(clock_source),
        } 
    }

    fn configure_div(&self, core_freq: u32) {
        let mut bus_div = 1;
        while core_freq / bus_div > 60_000_000 {
            bus_div += 1;
        }
    
        let mut flash_div = 1;
        while core_freq / flash_div > 28_000_000 {
            flash_div += 1;
        }
    
        sim::set_dividers(1, bus_div, flash_div);
    
        unsafe {
            CORECLK = core_freq * MHz;
            BUSCLK = (core_freq * MHz) / bus_div; 
            FLASHCLK = (core_freq * MHz) / flash_div;
        }
    }

    pub unsafe fn change_system_clock(&self, clock_source: SystemClockSource) {
        let mut set_divisors: bool = false;
        let new_clock_freq = get_clock_frequency(clock_source);
        if new_clock_freq > CORECLK {
            self.configure_div(new_clock_freq);
            set_divisors = true;
        }

        let mut clock_state: State = state();
        match clock_source {
            SystemClockSource::Oscillator => {
                osc::enable(Teensy16MHz.load as u8);
                while clock_state != State::Blpe(Blpe) {
                    clock_state = clock_state.to_blpe(Teensy16MHz);
                }
            }
            SystemClockSource::RTC32K => {
                while clock_state != State::Blpe(Blpe) {
                    clock_state = clock_state.to_blpe(Teensy32KHz);
                }
            }
            SystemClockSource::IRC48M => {
                while clock_state != State::Blpe(Blpe) {
                    clock_state = clock_state.to_blpe(Teensy48MHz);
                }
            }
            SystemClockSource::SlowInternal => {
                while clock_state != State::Blpi(Blpi) {
                    clock_state = clock_state.to_blpi(Ircs::SlowInternal);
                }
            }
            SystemClockSource::FastInternal => {
                while clock_state != State::Blpi(Blpi) {
                    clock_state = clock_state.to_blpi(Ircs::FastInternal);
                }
            }
            SystemClockSource::FLL(freq) => {
                while clock_state != State::Fei(Fei) {
                    clock_state = clock_state.to_fei(freq);
                }
            }
            SystemClockSource::PLL(freq) => {
                while clock_state != State::Pee(Pee) {
                    clock_state = clock_state.to_pee(freq, Teensy32KHz);
                }
            }
        }

        if !set_divisors {
            self.configure_div(new_clock_freq);
        }

        if self.clock_source.get() == SystemClockSource::Oscillator {
            osc::disable();
        }
        self.clock_source.set(clock_source);
    }
}

pub fn get_clock_frequency(clock: SystemClockSource) -> u32 {
    match clock {
        SystemClockSource::Oscillator => 16_000_000,
        SystemClockSource::RTC32K => 32_000,
        SystemClockSource::IRC48M => 48_000_000,
        SystemClockSource::SlowInternal => 32_000,
        SystemClockSource::FastInternal => 4_000_000,
        SystemClockSource::FLL(freq) => freq * 1_000_000,
        SystemClockSource::PLL(freq) => freq * 1_000_000,
    }
}

pub fn peripheral_clock_hz() -> u32 {
    unsafe { BUSCLK }
}

pub fn bus_clock_hz() -> u32 {
    unsafe { BUSCLK }
}

pub fn flash_clock_hz() -> u32 {
    unsafe { FLASHCLK }
}

pub fn core_clock_hz() -> u32 {
    unsafe { CORECLK }
}

