//! Implementation of the Multipurpose Clock Generator
//!

use cortexm4;
use ::core::mem;
use core::cell::Cell;
use osc;
use sim;
use smc;

use regs::mcg::*;

use self::Control1::CLKS::Value as OscSource;

#[derive(Copy,Clone,PartialEq)]
enum OscClock {
    Oscillator,
    RTC32K,
    IRC48M,
}

#[allow(dead_code)]
#[derive(Copy,Clone,PartialEq)]
enum OscRange {
    Low = 0,
    High = 1,
    VeryHigh = 2
}

#[allow(non_camel_case_types, dead_code)]
#[derive(Copy,Clone,PartialEq)]
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
#[derive(Copy,Clone,PartialEq)]
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
#[derive(Copy,Clone,PartialEq)]
enum Ircs {
    SlowInternal,
    FastInternal,
}

#[derive(Copy,Clone,PartialEq)]
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

#[derive(Copy,Clone,PartialEq)]
enum State {
    Fei,
    Fee(Xtal),
    Fbi(Ircs),
    Fbe(Xtal),
    Pbe(Xtal),
    Pee(Xtal),
    Blpi(Ircs),
    Blpe(Xtal),
}

trait ClockChange {
    fn to_fei(self) -> State;
    fn to_fee(self, xtal: Xtal) -> State;
    fn to_fbi(self, ircs: Ircs) -> State;
    fn to_blpi(self, ircs: Ircs) -> State;
    fn to_fbe(self, xtal: Xtal) -> State;
    fn to_pbe(self, xtal: Xtal) -> State;
    fn to_blpe(self, xtal: Xtal) -> State;
    fn to_pee(self, xtal: Xtal) -> State;
}

impl ClockChange for State {
    fn to_fei(self) -> State {
        match self {
            State::Fei => State::Fei,
            State::Fee(_xtal) => to_fei(),
            State::Fbi(_ircs) => to_fei(),
            State::Fbe(_xtal) => to_fei(),
            State::Pbe(xtal) => to_fbe(xtal),
            State::Pee(xtal) => to_pbe(xtal),
            State::Blpi(ircs) => to_fbi(ircs),
            State::Blpe(xtal) => to_fbe(xtal),
        }
    }
    fn to_fee(self, xtal: Xtal) -> State {
        match self {
            State::Fei => to_fee(xtal),
// OSCSEL (IRC48M vs Oscillator vs RTC32K) cannot change while in use
            State::Fee(old_xtal) => {
                if old_xtal == xtal { self }
                else { to_fei() }
            }
            State::Fbi(_ircs) => to_fee(xtal),
            State::Fbe(old_xtal) => {
                if old_xtal == xtal { to_fee(xtal) }
                else { to_fei() }
            }
            State::Pbe(old_xtal) => to_fbe(old_xtal),
            State::Pee(old_xtal) => to_pbe(old_xtal),
            State::Blpi(ircs) => to_fbi(ircs),
            State::Blpe(old_xtal) => to_fbe(old_xtal),
        }
    }
    fn to_fbi(self, ircs: Ircs) -> State {
        match self {
            State::Fei => to_fbi(ircs),
            State::Fee(_xtal) => to_fbi(ircs),
            State::Fbi(_ircs) => to_fbi(ircs),
            State::Fbe(_xtal) => to_fbi(ircs),
            State::Pbe(xtal) => to_fbe(xtal),
            State::Pee(xtal) => to_pbe(xtal),
            State::Blpi(_ircs) => to_fbi(ircs),
            State::Blpe(xtal) => to_fbe(xtal),
        }
    }
    fn to_blpi(self, ircs: Ircs) -> State {
        match self {
            State::Fei => to_fbi(ircs),
            State::Fee(_xtal) => to_fbi(ircs),
            State::Fbi(_ircs) => to_blpi(ircs),
            State::Fbe(_xtal) => to_fbi(ircs),
            State::Pbe(xtal) => to_fbe(xtal),
            State::Pee(xtal) => to_pbe(xtal),
            State::Blpi(_ircs) => to_blpi(ircs),
            State::Blpe(xtal) => to_fbe(xtal),
        }
    }
    fn to_fbe(self, xtal: Xtal) -> State {
        match self {
            State::Fei => to_fbe(xtal),
            State::Fee(old_xtal) => {
                if old_xtal == xtal { to_fbe(xtal) }
                else { to_fbi(Ircs::FastInternal) }
            }
            State::Fbi(_ircs) => to_fbe(xtal),
            State::Fbe(old_xtal) => {
                if old_xtal == xtal { self }
                else { to_fbi(Ircs::FastInternal) }
            }
            State::Pbe(old_xtal) => to_fbe(old_xtal),
            State::Pee(old_xtal) => to_pbe(old_xtal),
            State::Blpi(ircs) => to_fbi(ircs),
            State::Blpe(old_xtal) => to_fbe(old_xtal),
        }
    }
    fn to_pbe(self, xtal: Xtal) -> State {
        match self {
            State::Fei => to_fbe(xtal),
            State::Fee(old_xtal) => {
                if old_xtal == xtal { to_fbe(xtal) }
                else { to_fbi(Ircs::FastInternal) }
            }
            State::Fbi(_ircs) => to_fbe(xtal),
            State::Fbe(old_xtal) => {
                if old_xtal == xtal { to_pbe(xtal) }
                else { to_fbi(Ircs::FastInternal) }
            }
            State::Pbe(old_xtal) => {
                if old_xtal == xtal { self }
                else { to_fbe(old_xtal) }
            }
            State::Pee(old_xtal) => to_pbe(old_xtal),
            State::Blpi(ircs) => to_fbi(ircs),
            State::Blpe(old_xtal) => {
                if old_xtal == xtal { to_pbe(xtal) }
                else { to_fbe(old_xtal) }
            }
        }
    }
    fn to_blpe(self, xtal: Xtal) -> State {
        match self {
            State::Fei => to_fbe(xtal),
            State::Fee(old_xtal) => {
                if old_xtal == xtal { to_fbe(xtal) }
                else { to_fbi(Ircs::FastInternal) }
            }
            State::Fbi(_ircs) => to_fbe(xtal),
            State::Fbe(old_xtal) => {
                if old_xtal == xtal { to_blpe(xtal) }
                else { to_fbi(Ircs::FastInternal) }
            }
            State::Pbe(old_xtal) => {
                if old_xtal == xtal { to_blpe(xtal) }
                else { to_fbe(old_xtal) }
            }
            State::Pee(old_xtal) => to_pbe(old_xtal),
            State::Blpi(ircs) => to_fbi(ircs),
            State::Blpe(old_xtal) => {
                if old_xtal == xtal { self }
                else { to_fbe(old_xtal) }
            }
        }
    }
    fn to_pee(self, xtal: Xtal) -> State {
        match self {
            State::Fei => to_fbe(xtal),
            State::Fee(old_xtal) => {
                if old_xtal == xtal { to_fbe(xtal) }
                else { to_fbi(Ircs::FastInternal) }
            }
            State::Fbi(_ircs) => to_fbe(xtal),
            State::Fbe(old_xtal) => {
                if old_xtal == xtal { to_pbe(xtal) }
                else { to_fbi(Ircs::FastInternal) }
            }
            State::Pbe(old_xtal) => {
                if old_xtal == xtal { to_pee(xtal) }
                else { to_fbe(old_xtal) }
            }
            State::Pee(old_xtal) => {
                if old_xtal == xtal { self }
                else { to_pbe(old_xtal) }
            }
            State::Blpi(ircs) => to_fbi(ircs),
            State::Blpe(old_xtal) => {
                if old_xtal == xtal { to_pbe(xtal) }
                else { to_fbe(old_xtal) }
            }
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

    let xtal: Xtal = match mcg.c7.read(Control7::OSCSEL) {
        0 => Teensy16MHz,
        1 => Teensy32KHz,
        _ => Teensy48MHz,
    };

    let ircs: Ircs = match mcg.s.read(Status::IRCST) {
        0 => Ircs::SlowInternal,
        _ => Ircs::FastInternal,
    };

    match (clks, irefs, plls, lp) {
        (OscSource::LockedLoop, true, false, _) => State::Fei,
        (OscSource::LockedLoop, false, false, _) => State::Fee(xtal),
        (OscSource::Internal, true, false, false) => State::Fbi(ircs),
        (OscSource::External, false, false, false) => State::Fbe(xtal),
        (OscSource::LockedLoop, false, true, _) => State::Pee(xtal),
        (OscSource::External, false, true, false) => State::Pbe(xtal),
        (OscSource::Internal, true, false, true) => State::Blpi(ircs),
        (OscSource::External, false, _, true) => State::Blpe(xtal),
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
        180 => (45, 2),

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
    let drs_val = match freq {
        24 => 0,
        48 => 1,
        72 => 2,
        96 => 3,
        _ => panic!("Invalid fll frequency selected!")
    };

    let mcg: &mut Registers = unsafe { mem::transmute(MCG) };
    mcg.c4.modify(Control4::DRST_DRS.val(drs_val as u8) +
                  Control4::DMX32::SET);
}

fn to_fei() -> State {
    let mcg: &mut Registers = unsafe { mem::transmute(MCG) };
    mcg.c1.modify(Control1::CLKS::LockedLoop+
                 Control1::IREFS::SlowInternal);

    while !mcg.s.matches_all(Status::CLKST::Fll + Status::IREFST::Internal + 
                             Status::IRCST::Slow) {}
    
    State::Fei
}

fn to_fee(xtal: Xtal) -> State {
    let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

    mcg.c2.modify(Control2::RANGE.val(xtal.range as u8));

    match state() {
        State::Fei | State::Fbi(..)  => {
            if xtal == Teensy16MHz {
                mcg.c2.modify(Control2::EREFS::SET);
                while !mcg.s.matches_all(Status::OSCINIT0::SET) {}
            }
            mcg.c7.modify(Control7::OSCSEL.val(xtal.clock as u8));
        }
        _ => {}
    }

    mcg.c1.modify(Control1::CLKS::LockedLoop +
                 Control1::FRDIV.val(xtal.frdiv as u8) +
                 Control1::IREFS::External);

    while !mcg.s.matches_all(Status::CLKST::Fll + 
                             Status::IREFST::External) {} 

    State::Fee(xtal)
}

fn to_fbi(ircs: Ircs) -> State {
    let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

    mcg.sc.modify(StatusControl::FCRDIV.val(0 as u8));

    mcg.c2.modify(Control2::LP::CLEAR + Control2::IRCS.val(ircs as u8));

    mcg.c1.modify(Control1::CLKS::Internal + Control1::IREFS::SlowInternal);

    while !mcg.s.matches_all(Status::CLKST::Internal +
                             Status::IREFST::Internal +
                             Status::IRCST.val(ircs as u8)) {} 

    State::Fbi(ircs)
}

fn to_blpi(ircs: Ircs) -> State { 
    let mcg: &mut Registers = unsafe { mem::transmute(MCG) };
    
    mcg.c2.modify(Control2::IRCS.val(ircs as u8) + Control2::LP::SET);

    while !mcg.s.matches_all(Status::IRCST.val(ircs as u8)) {} 

    State::Blpi(ircs)
}

fn to_fbe(xtal: Xtal) -> State {
    let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

    mcg.c2.modify(Control2::LP::CLEAR +
                  Control2::RANGE.val(xtal.range as u8));

    match state() {
        State::Fei | State::Fbi(..)  => {
            if xtal == Teensy16MHz {
                mcg.c2.modify(Control2::EREFS::SET);
                while !mcg.s.matches_all(Status::OSCINIT0::SET) {}
            }
            mcg.c7.modify(Control7::OSCSEL.val(xtal.clock as u8));
        }
        _ => {}
    }

    mcg.c6.modify(Control6::PLLS::Fll);

    mcg.c1.modify(Control1::CLKS::External +
                 Control1::FRDIV.val(xtal.frdiv as u8) +
                 Control1::IREFS::External);

    while !mcg.s.matches_all(Status::PLLST::Fll +
                             Status::CLKST::External +
                             Status::IREFST::External) {} 

    State::Fbe(xtal)
}

fn to_pbe(xtal: Xtal) -> State { 
    let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

    match state() {
        State::Pee(..) => {
            mcg.c1.modify(Control1::CLKS::External);

            while !mcg.s.matches_all(Status::CLKST::External) {}
        } 
        _ => {
            mcg.c6.modify(Control6::PLLS::SET);

            mcg.c2.modify(Control2::LP::CLEAR);
        }
    }

    while !mcg.s.matches_all(Status::PLLST::PllcsOutput + Status::LOCK0::SET) {}

    State::Pbe(xtal)
}

fn to_blpe(xtal: Xtal) -> State { 
    let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

    mcg.c2.modify(Control2::LP::SET);

    State::Blpe(xtal)
}

fn to_pee(xtal: Xtal) -> State { 
    let mcg: &mut Registers = unsafe { mem::transmute(MCG) };

    mcg.c1.modify(Control1::CLKS::LockedLoop);

    while !mcg.s.matches_all(Status::CLKST::Pll) {}

    State::Pee(xtal)
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
    system_initial_configs: Cell<bool>,
}

pub static mut SCM: SystemClockManager = SystemClockManager::new(SystemClockSource::FLL(20));

// On reset, MCGOUTCLK is sourced from the 32kHz internal reference clock 
// multiplied by the FLL, which has a default multiplier of 640.
static mut CORECLK: u32 = 20_480_000;
static mut BUSCLK: u32 = 20_480_000;
static mut FLASHCLK: u32 = 10_240_000;

impl SystemClockManager {
    const fn new(clock_source: SystemClockSource) -> SystemClockManager {
        SystemClockManager {
            clock_source: Cell::new(clock_source),
            system_initial_configs: Cell::new(false),
        } 
    }

    fn configure_div(&self, core_freq: u32) {
        unsafe {
            cortexm4::systick::SysTick::set_hertz(core_freq);
        }

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
            CORECLK = core_freq ;
            BUSCLK = core_freq  / bus_div; 
            FLASHCLK = core_freq  / flash_div;
        }
    }

    pub unsafe fn change_system_clock(&self, clock_source: SystemClockSource) {
        if clock_source == self.clock_source.get() {
            return;
        }

        let mut set_divisors: bool = false;
        let new_clock_freq = get_clock_frequency(clock_source);
        if new_clock_freq > CORECLK {
            if new_clock_freq > 120_000_000 {
                if !self.system_initial_configs.get() {
                    smc::enable_power_modes(1,0,0,0);
                    self.system_initial_configs.set(true);
                }
                smc::hsrun_mode();
            } 
            self.configure_div(new_clock_freq);
            set_divisors = true;
        }

        let mut clock_state: State = state();
        match clock_source {
            SystemClockSource::Oscillator => {
                osc::enable(Teensy16MHz.load as u8);
                while clock_state != State::Blpe(Teensy16MHz) {
                    clock_state = clock_state.to_blpe(Teensy16MHz);
                }
            }
            SystemClockSource::RTC32K => {
                while clock_state != State::Blpe(Teensy32KHz) {
                    clock_state = clock_state.to_blpe(Teensy32KHz);
                }
            }
            SystemClockSource::IRC48M => {
                while clock_state != State::Blpe(Teensy48MHz) {
                    clock_state = clock_state.to_blpe(Teensy48MHz);
                }
            }
            SystemClockSource::SlowInternal => {
                while clock_state != State::Blpi(Ircs::SlowInternal) {
                    clock_state = clock_state.to_blpi(Ircs::SlowInternal);
                }
            }
            SystemClockSource::FastInternal => {
                while clock_state != State::Blpi(Ircs::FastInternal) {
                    clock_state = clock_state.to_blpi(Ircs::FastInternal);
                }
            }
            SystemClockSource::FLL(freq) => {
                while clock_state != State::Fei {
                    clock_state = clock_state.to_fei();
                }
                set_fll_freq(freq);
            }
            SystemClockSource::PLL(freq) => {
                osc::enable(Teensy16MHz.load as u8);
                set_pll_freq(freq);
                while clock_state != State::Pee(Teensy16MHz) {
                    clock_state = clock_state.to_pee(Teensy16MHz);
                }
            }
        }

        if !set_divisors {
            if CORECLK > 180_000_000 && new_clock_freq <= 120_000_000 {
                smc::run_mode();
            }
            self.configure_div(new_clock_freq);
        }

        match clock_source {
            SystemClockSource::Oscillator | SystemClockSource::PLL(..) => {}
            _ => { osc::disable(); }
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

