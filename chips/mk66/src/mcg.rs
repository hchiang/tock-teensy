//! Implementation of the Multipurpose Clock Generator
//!

use ::core::mem;
use kernel::common::registers::{register_bitfields, ReadWrite, ReadOnly};

pub use self::Control1::CLKS::Value as OscSource;
pub use self::Control1::FRDIV::Value as Frdiv;
pub use self::Control2::RANGE::Value as OscRange;

  
pub const MCG: *mut Registers = 0x4006_4000 as *mut Registers;

#[repr(C)]
pub struct Registers {
    pub c1: ReadWrite<u8, Control1::Register>,
    pub c2: ReadWrite<u8, Control2::Register>,
    pub c3: ReadWrite<u8>,
    pub c4: ReadWrite<u8>,
    pub c5: ReadWrite<u8, Control5::Register>,
    pub c6: ReadWrite<u8, Control6::Register>,
    pub s: ReadOnly<u8, Status::Register>,
    _reserved0: ReadOnly<u8>,
    pub sc: ReadWrite<u8>,
    _reserved1: ReadOnly<u8>,
    pub atcvh: ReadWrite<u8>,
    pub atcvl: ReadWrite<u8>,
    pub c7: ReadWrite<u8, Control7::Register>,
    pub c8: ReadWrite<u8>,
    pub c9: ReadWrite<u8>,
    _reserved2: ReadOnly<u8>,
    pub c11: ReadWrite<u8>,
    pub c12: ReadWrite<u8>,
    pub s2: ReadOnly<u8>,
    pub t3: ReadWrite<u8>
}

register_bitfields![u8,
    Control1 [
        CLKS OFFSET(6) NUMBITS(2) [
            LockedLoop = 0,
            Internal = 1,
            External = 2
        ],
        FRDIV OFFSET(4) NUMBITS(2) [
            Low1_High32 = 0,
            Low2_High64 = 1,
            Low4_High128 = 2,
            Low8_High256 = 3,
            Low16_High512 = 4,
            Low32_High1024 = 5,
            Low64_High1280 = 6,
            Low128_High1536 = 7
        ],
        IREFS OFFSET(2) NUMBITS(1) [
            External = 0,
            SlowInternal = 1
        ],
        IRCLKEN OFFSET(1) NUMBITS(1) [
            Inactive = 0,
            Active = 1
        ],
        IREFSTEN OFFSET(0) NUMBITS(1) [
            IrefDisabledInStop = 0,
            IrefEnabledInStop = 1
        ]
    ],


    Control2 [
        LOCKRE0 OFFSET(7) NUMBITS(1) [],
        FCFTRIM OFFSET(6) NUMBITS(1) [],
        RANGE OFFSET(4) NUMBITS(2) [
            Low = 0,
            High = 1,
            VeryHigh = 2
        ],
        HGO OFFSET(3) NUMBITS(1) [
            LowPower = 0,
            HighGain = 1
        ],
        EREFS OFFSET(2) NUMBITS(1) [
            External = 0,
            Oscillator = 1
        ],
        LP OFFSET(1) NUMBITS(1) [],
        IRCS OFFSET(0) NUMBITS(1) [
            SlowInternal = 0,
            FastInternal = 1
        ]
    ],

    Control5 [
        PLLCLKEN OFFSET(6) NUMBITS(1) [],
        PLLSTEN OFFSET(5) NUMBITS(1) [],
        PRDIV OFFSET(0) NUMBITS(3) [
            Div1 = 0, Div2 = 1, Div3 = 2, Div4 = 3,
            Div5 = 4, Div6 = 5, Div7 = 6, Div8 = 7
        ]
    ],
    Control6 [
        LOLIE0 OFFSET(7) NUMBITS(1) [],
        PLLS OFFSET(6) NUMBITS(1) [
            Fll = 0,
            PllcsOutput = 1
        ],
        CME0 OFFSET(5) NUMBITS(1) [],
        VDIV OFFSET(0) NUMBITS(5) [
            Mul16 = 0, Mul17 = 1, Mul18 = 2, Mul19 = 3,
            Mul20 = 4, Mul21 = 5, Mul22 = 6, Mul23 = 7,
            Mul24 = 8, Mul25 = 9, Mul26 = 10, Mul27 = 11,
            Mul28 = 12, Mul29 = 13, Mul30 = 14, Mul31 = 15,
            Mul32 = 16, Mul33 = 17, Mul34 = 18, Mul35 = 19,
            Mul36 = 20, Mul37 = 21, Mul38 = 22, Mul39 = 23,
            Mul40 = 24, Mul41 = 25, Mul42 = 26, Mul43 = 27,
            Mul44 = 28, Mul45 = 29, Mul46 = 30, Mul47 = 31
        ]
    ],

    Status [
        LOLS0 OFFSET(7) NUMBITS(1) [],
        LOCK0 OFFSET(6) NUMBITS(1) [],
        PLLST OFFSET(5) NUMBITS(1) [
            Fll = 0,
            PllcsOutput = 1
        ],
        IREFST OFFSET(4) NUMBITS(1) [
            External = 0,
            Internal = 1
        ],
        CLKST OFFSET(2) NUMBITS(2) [
            Fll = 0,
            Internal = 1,
            External = 2,
            Pll = 3
        ],
        OSCINIT0 OFFSET(1) NUMBITS(1) [],
        IRCST OFFSET(0) NUMBITS(1) [
            Slow = 0,
            Fast = 1
        ]
    ],

    Control7 [
        OSCSEL OFFSET(0) NUMBITS(2) [
            Oscillator = 0,
            RTC32K = 1,
            IRC48M = 2
        ]
    ]
];


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

pub enum OscClock {
    Oscillator,
    RTC32K,
    IRC48M,
}

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
    use mcg::{OscClock, Ircs, Xtal, OscRange, Frdiv};
    use osc::OscCapacitance;

    #[allow(non_upper_case_globals)]
    pub const Teensy16MHz: Xtal = Xtal {
        clock: OscClock::Oscillator,
        range: OscRange::VeryHigh,
        frdiv: Frdiv::Low16_High512,
        load: OscCapacitance::Load_10pF
    };
}

//TODO c4 divider for FLL
// Source: https://branan.github.io/teensy/2017/01/28/uart.html
impl Fei {
    pub fn to_fbi(self, ircs: Ircs) -> Fbi {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c1.modify(Control1::CLKS::Internal);

        while !mcg.s.matches_all(Status::CLKST::Internal) {}

        mcg.c2.modify(Control2::IRCS.val(ircs as u8));

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

        while !mcg.s.matches_all(Status::IREFST::Internal + Status::CLKST::Internal) {}
    
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
    pub fn to_pbe(self, multiplier: u8, divider: u8) -> Pbe {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        if multiplier < 16 || multiplier > 47 {
            panic!("Invalid PLL VCO divide factor: {}", multiplier);
        }
        if divider < 1 || divider > 8 {
            panic!("Invalid PLL reference divide factor: {}", divider);
        }

        mcg.c5.modify(Control5::PRDIV.val(divider - 1));

        mcg.c6.modify(Control6::VDIV.val(multiplier - 16) +
                      Control6::PLLS::SET);

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
    pub fn to_pbe(self, xtal: Xtal) -> Pbe {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        mcg.c2.modify(Control2::RANGE.val(xtal.range as u8) +
                      Control2::EREFS::SET);

        mcg.c7.modify(Control7::OSCSEL.val(xtal.clock as u8));

        mcg.c1.modify(Control1::CLKS::External +
                      Control1::FRDIV.val(xtal.frdiv as u8));

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
    pub fn to_pbe(self, multiplier: u8, divider: u8) -> Pbe {
        let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

        if multiplier < 16 || multiplier > 47 {
            panic!("Invalid PLL VCO divide factor: {}", multiplier);
        }
        if divider < 1 || divider > 8 {
            panic!("Invalid PLL reference divide factor: {}", divider);
        }

        mcg.c5.modify(Control5::PRDIV.val(divider - 1));

        mcg.c6.modify(Control6::VDIV.val(multiplier - 16) +
                      Control6::PLLS::SET);

        mcg.c2.modify(Control2::LP::CLEAR);

        while !mcg.s.matches_all(Status::PLLST::PllcsOutput + Status::LOCK0::SET) {}

        Pbe {}
    }
}
