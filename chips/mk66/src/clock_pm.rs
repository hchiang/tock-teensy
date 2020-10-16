use kernel::hil::clock_pm::*;
use mcg;

//const RTC32K: u32           = 0x001; 
const SLOWINTERNAL: u32     = 0x001; 
const FASTINTERNAL: u32     = 0x002; 
const OSCILLATOR: u32       = 0x004; 
const FLL24: u32            = 0x008;    
const FLL48: u32            = 0x010;    
const IRC48M: u32           = 0x020; 
const PLL64: u32            = 0x040; 
const FLL72: u32            = 0x080;    
const FLL96: u32            = 0x100;    
const PLL120: u32           = 0x200; 
const PLL180: u32           = 0x400; 
const ALL_CLOCKS: u32       = 0x7ff; 

pub struct TeensyClockManager;

#[allow(non_upper_case_globals)]
pub static TeensyCM: TeensyClockManager = TeensyClockManager::new();

impl TeensyClockManager {

    const fn new() -> TeensyClockManager {
        TeensyClockManager {}
    }

    fn convert_to_clock(&self, clock: u32) -> mcg::SystemClockSource {
        // Roughly ordered in terms of least to most power consumption
        return match clock {
            //RTC32K => mcg::SystemClockSource::RTC32K,
            SLOWINTERNAL => mcg::SystemClockSource::SlowInternal,
            FASTINTERNAL => mcg::SystemClockSource::FastInternal,
            OSCILLATOR => mcg::SystemClockSource::Oscillator,
            IRC48M => mcg::SystemClockSource::IRC48M,
            FLL24 => mcg::SystemClockSource::FLL(24),
            FLL48 => mcg::SystemClockSource::FLL(48),
            FLL72 => mcg::SystemClockSource::FLL(72),
            FLL96 => mcg::SystemClockSource::FLL(96),
            PLL64 => mcg::SystemClockSource::PLL(64),
            PLL120 => mcg::SystemClockSource::PLL(120),
            PLL180 => mcg::SystemClockSource::PLL(180),
            _ => mcg::SystemClockSource::PLL(180),
        }
    }
}

impl ClockConfigs for TeensyClockManager {

    fn get_num_clock_sources(&self) -> u32 {
        11 
    }

    fn get_max_freq(&self) -> u32 {
        180_000_000
    }

    fn get_all_clocks(&self) -> u32 {
        ALL_CLOCKS 
    }

    fn get_compute(&self) -> u32 {
        FLL96
    }

    fn get_noncompute(&self) -> u32 {
        SLOWINTERNAL
    }

    // Used to calculate acceptable clocks based on frequency range
    fn get_clockmask(&self, min_freq: u32, max_freq: u32) -> u32 {
        if min_freq > max_freq {
            return 0;
        }

        let mut clockmask: u32 = 0;

        if min_freq <= 32000 && max_freq >= 32000 { 
            clockmask |= SLOWINTERNAL;
        } 
        if min_freq <= 4_000_000 && max_freq >= 4_000_000 { 
            clockmask |= FASTINTERNAL;
        }
        if min_freq <= 16_000_000 && max_freq >= 16_000_000 { 
            clockmask |= OSCILLATOR;
        } 
        if min_freq <= 24_000_000 && max_freq >= 24_000_000 { 
            clockmask |= FLL24;
        }
        if min_freq <= 48_000_000 && max_freq >= 48_000_000 { 
            clockmask |= IRC48M + FLL48;
        }
        if min_freq <= 72_000_000 && max_freq >= 72_000_000 { 
            clockmask |= FLL72;
        }
        if min_freq <= 96_000_000 && max_freq >= 96_000_000 { 
            clockmask |= FLL96;
        }
        if min_freq <= 64_000_000 && max_freq >= 64_000_000 {
            clockmask |= PLL64;
        }
        if min_freq <= 120_000_000 && max_freq >= 120_000_000 {
            clockmask |= PLL120;
        }
        if min_freq <= 180_000_000 && max_freq >= 180_000_000 {
            clockmask |= PLL180;
        }

        clockmask
    }

    fn get_clock_frequency(&self, clock: u32) -> u32 {
        let system_clock = self.convert_to_clock(clock);
        mcg::get_clock_frequency(system_clock)
    }

    fn get_system_frequency(&self) -> u32 {
        mcg::core_clock_hz()
    }

    fn change_system_clock(&self, clock: u32) {
        let system_clock = self.convert_to_clock(clock);
        unsafe {
            mcg::SCM.change_system_clock(system_clock);
        }
    }

    fn get_intermediates_list(&self, clock:u32) -> IntermediateList {
        let external_clocks = OSCILLATOR | IRC48M;
        let pll = PLL64 | PLL120 | PLL180;
        let fll = FLL24 | FLL48 | FLL72 | FLL96;
        match clock {
            OSCILLATOR | IRC48M => IntermediateList::new(ALL_CLOCKS & !external_clocks, external_clocks & !clock),
            FLL24 | FLL48 | FLL72 | FLL96 | SLOWINTERNAL | FASTINTERNAL => IntermediateList::new(OSCILLATOR, pll),
            PLL64 | PLL120 | PLL180 => IntermediateList::new(OSCILLATOR, fll | SLOWINTERNAL | FASTINTERNAL),
            _ => IntermediateList::new(0, 0),
        }
    }
}

