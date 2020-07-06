//! Implementation of the Multipurpose Clock Generator
//!

use ::core::mem;

use regs::mcg::*;

pub use self::Control1::CLKS::Value as OscSource;
pub use self::Control1::FRDIV::Value as Frdiv;
pub use self::Control2::RANGE::Value as OscRange;

pub enum State {
    Fei(Fei),
    Fee(Fee),
    Fbi(Fbi),
    Fbe(Fbe),
    Pbe(Pbe),
    Pee(Pee),
    Blpi(Blpi),
    Blpe(Blpe),
    Stop,
}

#[derive(Copy,Clone)]
pub struct Fei;

#[derive(Copy,Clone)]
pub struct Fee;

#[derive(Copy,Clone)]
pub struct Fbi;

#[derive(Copy,Clone)]
pub struct Fbe;

#[derive(Copy,Clone)]
pub struct Pbe;

#[derive(Copy,Clone)]
pub struct Pee;

#[derive(Copy,Clone)]
pub struct Blpi;

#[derive(Copy,Clone)]
pub struct Blpe;

pub fn state() -> State {
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

pub fn set_fll_freq(freq: u32) {
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

//TODO FCRDIV can divide freq of internal reference clock
// modify Ircs to pass in freq?

pub enum OscClock {
    Oscillator,
    RTC32K,
    IRC48M,
}

#[derive(Copy,Clone)]
pub enum Ircs {
    SlowInternal,
    FastInternal,
}

pub struct Xtal {
    pub clock: OscClock,
    pub range: OscRange,
    pub frdiv: Frdiv,
    pub load: ::osc::OscCapacitance
}

pub mod xtals {
    use mcg::{OscClock, Xtal, OscRange, Frdiv};
    use osc::OscCapacitance;

    #[allow(non_upper_case_globals)]
    pub const Teensy16MHz: Xtal = Xtal {
        clock: OscClock::Oscillator,
        range: OscRange::VeryHigh,
        frdiv: Frdiv::Low16_High512,
        load: OscCapacitance::Load_10pF
    };

    #[allow(non_upper_case_globals)]
    pub const Teensy32KHz: Xtal = Xtal {
        clock: OscClock::RTC32K,
        range: OscRange::Low,
        frdiv: Frdiv::Low1_High32,
        load: OscCapacitance::Load_10pF
    };

    #[allow(non_upper_case_globals)]
    pub const Teensy48MHz: Xtal = Xtal {
        clock: OscClock::IRC48M,
        range: OscRange::VeryHigh,
        frdiv: Frdiv::Low128_High1536,
        load: OscCapacitance::Load_10pF
    };
}

// Source: https://branan.github.io/teensy/2017/01/28/uart.html
impl Fei {
    pub fn to_fbi(self, ircs: Ircs) -> Fbi {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c1.modify(Control1::CLKS::Internal);

        while !mcg.s.matches_all(Status::CLKST::Internal) {}

        mcg.c2.modify(Control2::IRCS.val(ircs as u8));

        mcg.sc.modify(StatusControl::FCRDIV.val(0 as u8));

        Fbi {}        
    }
    pub fn to_fee(self, xtal: Xtal) -> Fee {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c2.modify(Control2::RANGE.val(xtal.range as u8) +
                      Control2::EREFS::SET);

        mcg.c7.modify(Control7::OSCSEL.val(xtal.clock as u8));

        mcg.c1.modify(Control1::FRDIV.val(xtal.frdiv as u8) +
                     Control1::IREFS::CLEAR);

        while !mcg.s.matches_all(Status::OSCINIT0::SET +
                             Status::IREFST::External) {} 

        Fee {}
    }
    pub fn to_fbe(self, xtal: Xtal) -> Fbe {
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

        Fbe {}
    }
}

impl Fee {
    pub fn to_fei(self) -> Fei {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c1.modify(Control1::IREFS::SET);

        while !mcg.s.matches_all(Status::IRCST::Slow +
                             Status::IREFST::Internal) {} 
    
        Fei {}
    }
    pub fn to_fbi(self, ircs: Ircs) -> Fbi {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };
        
        mcg.c2.modify(Control2::IRCS.val(ircs as u8));

        mcg.c1.modify(Control1::CLKS::Internal + Control1::IREFS::SlowInternal);

        while !mcg.s.matches_all(Status::IRCST.val(ircs as u8) + Status::IREFST::Internal + Status::CLKST::Internal) {}

        mcg.sc.modify(StatusControl::FCRDIV.val(0 as u8));
    
        Fbi {}
    }
    pub fn to_fbe(self) -> Fbe {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c1.modify(Control1::CLKS::External);

        while !mcg.s.matches_all(Status::CLKST::External) {}

        Fbe {}
    }
}

impl Fbi {
    pub fn to_fei(self) -> Fei {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c1.modify(Control1::CLKS::LockedLoop);

        while !mcg.s.matches_all(Status::CLKST::Fll + 
                                 Status::IRCST::Slow) {} 

        Fei {}
    }
    pub fn to_fee(self, xtal: Xtal) -> Fee {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c2.modify(Control2::RANGE.val(xtal.range as u8) +
                      Control2::EREFS::SET);

        mcg.c7.modify(Control7::OSCSEL.val(xtal.clock as u8));

        mcg.c1.modify(Control1::CLKS::LockedLoop +
                     Control1::FRDIV.val(xtal.frdiv as u8) +
                     Control1::IREFS::External);

        while !mcg.s.matches_all(Status::OSCINIT0::SET +
                                 Status::CLKST::Fll + 
                                 Status::IREFST::External) {} 

        Fee {}
    }
    pub fn to_fbe(self, xtal: Xtal) -> Fbe {
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

        Fbe {}
    }
    pub fn to_blpi(self) -> Blpi {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };
    
        mcg.c2.modify(Control2::LP::SET);

        Blpi {}
    }
}

impl Fbe {
    pub fn to_pbe(self, freq: u32) -> Pbe {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        set_pll_freq(freq);

        mcg.c6.modify(Control6::PLLS::SET);

        // Wait for PLL to be selected and stable PLL lock
        while !mcg.s.matches_all(Status::PLLST::PllcsOutput + Status::LOCK0::SET) {}

        Pbe {}
    }
    pub fn to_blpe(self) -> Blpe {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c2.modify(Control2::LP::SET);

        Blpe {}
    }
    pub fn to_fee(self) -> Fee {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c1.modify(Control1::CLKS::LockedLoop);

        while !mcg.s.matches_all(Status::CLKST::Fll) {}

        Fee {}
    }
    pub fn to_fei(self) -> Fei {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c1.modify(Control1::CLKS::LockedLoop+
                     Control1::IREFS::SlowInternal);

        while !mcg.s.matches_all(Status::IREFST::Internal + Status::CLKST::Fll + 
                                 Status::IRCST::Slow) {}

        Fei{}
    }
    pub fn to_fbi(self, ircs: Ircs) -> Fbi {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };
        
        mcg.c1.modify(Control1::CLKS::Internal +
                      Control1::IREFS::SlowInternal);

        while !mcg.s.matches_all(Status::IREFST::Internal + Status::CLKST::Internal) {}

        mcg.c2.modify(Control2::IRCS.val(ircs as u8));

        mcg.sc.modify(StatusControl::FCRDIV.val(0 as u8));

        Fbi {}
    }
}

impl Pbe {
    pub fn to_fbe(self) -> Fbe {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c6.modify(Control6::PLLS::CLEAR);

        while !mcg.s.matches_all(Status::PLLST::Fll ) {}
        
        Fbe {}
    }
    pub fn to_blpe(self) -> Blpe {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };
        
        mcg.c2.modify(Control2::LP::SET);

        Blpe {}
    }
    pub fn to_pee(self) -> Pee {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c1.modify(Control1::CLKS::LockedLoop);

        while !mcg.s.matches_all(Status::CLKST::Pll) {}

        Pee {}
    }
}

impl Pee {
    pub fn to_pbe(self) -> Pbe {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c1.modify(Control1::CLKS::External);

        while !mcg.s.matches_all(Status::CLKST::External) {}

        Pbe {}
    }
}

impl Blpi {
    pub fn to_fbi(self) -> Fbi {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c2.modify(Control2::LP::CLEAR);

        while !mcg.s.matches_all(Status::IREFST::Internal) {}

        Fbi {}        
    }
}

impl Blpe {
    pub fn to_fbe(self) -> Fbe {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };
        
        mcg.c6.modify(Control6::PLLS::CLEAR);

        mcg.c2.modify(Control2::LP::CLEAR);

        while !mcg.s.matches_all(Status::PLLST::Fll) {}
        
        Fbe {}
    }
    pub fn to_pbe(self, freq: u32) -> Pbe {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        set_pll_freq(freq);

        mcg.c6.modify(Control6::PLLS::SET);

        mcg.c2.modify(Control2::LP::CLEAR);

        while !mcg.s.matches_all(Status::PLLST::PllcsOutput + Status::LOCK0::SET) {}

        Pbe {}
    }
}
