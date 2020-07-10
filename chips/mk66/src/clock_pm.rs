use kernel::hil::clock_pm::*;
use mcg;

const RTC32K: u32           = 0x002; 
const SLOWINTERNAL: u32     = 0x004; 
const FASTINTERNAL: u32     = 0x008; 
const OSCILLATOR: u32       = 0x010; 
const IRC48M: u32           = 0x020; 
const FLL: u32              = 0x040;    
const PLL: u32              = 0x080; 

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
            0x02 => mcg::SystemClockSource::RTC32K,
            0x04 => mcg::SystemClockSource::SlowInternal,
            0x08 => mcg::SystemClockSource::FastInternal,
            0x10 => mcg::SystemClockSource::Oscillator,
            0x20 => mcg::SystemClockSource::IRC48M,
            0x40 => mcg::SystemClockSource::FLL(48),
            0x80 => mcg::SystemClockSource::PLL(64),
            _ => mcg::SystemClockSource::PLL(64),
        }
    }
}

impl ClockConfigs for TeensyClockManager {

    fn get_num_clock_sources(&self) -> u32 {
        return 8;
    }

    fn get_max_freq(&self) -> u32 {
        return 180_000_000;
    }

    fn get_all_clocks(&self) -> u32 {
        return 0xfe;
    }

    fn get_default(&self) -> u32 {
        return 0xff;
    }

    fn get_compute(&self) -> u32 {
        return PLL;
    }

    // Used to calculate acceptable clocks based on frequency range
    fn get_clockmask(&self, min_freq: u32, max_freq: u32) -> u32 {
        if min_freq > max_freq {
            return 0;
        }

        let mut clockmask: u32 = 0;

        if min_freq <= 32000 && max_freq >= 32000 { 
            clockmask |= RTC32K + SLOWINTERNAL;
        } 
        if min_freq <= 4_000_000 && max_freq >= 4_000_000 { 
            clockmask |= FASTINTERNAL;
        }
        if min_freq <= 16_000_000 && max_freq >= 16_000_000 { 
            clockmask |= OSCILLATOR;
        } 
        if min_freq <= 48_000_000 && max_freq >= 48_000_000 { 
            clockmask |= IRC48M + FLL;
        }
        if min_freq <= 64_000_000 && max_freq >= 64_000_000 {
            clockmask |= PLL;
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
}

