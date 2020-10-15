use kernel::hil::clock_pm::{SetClock};
use mcg;

pub struct TeensyClockManager{
}

impl TeensyClockManager {

    pub const fn new() -> TeensyClockManager {
        TeensyClockManager {}
    }

    fn convert_to_clock(&self, clock: u32)-> mcg::SystemClockSource{
        //Roughly ordered in terms of least to most power consumption

        let system_clock = match clock {
            0 => mcg::SystemClockSource::RTC32K,
            1 => mcg::SystemClockSource::SlowInternal,
            2 => mcg::SystemClockSource::FastInternal,
            3 => mcg::SystemClockSource::Oscillator,
            4 => mcg::SystemClockSource::IRC48M,
            5 => mcg::SystemClockSource::FLL(24),
            6 => mcg::SystemClockSource::FLL(48),
            7 => mcg::SystemClockSource::FLL(72),
            8 => mcg::SystemClockSource::FLL(96),
            9 => mcg::SystemClockSource::PLL(64),
            10 => mcg::SystemClockSource::PLL(120),
            11 => mcg::SystemClockSource::PLL(180),
            _ => mcg::SystemClockSource::PLL(64),
        };

        return system_clock;
    }
}

//Allows userland code to change the clock
impl SetClock for TeensyClockManager {
    fn set_clock(&self, clock: u32) {
        let system_clock = self.convert_to_clock(clock);
        unsafe {
            mcg::SCM.change_system_clock(system_clock);
        }
    }
}

