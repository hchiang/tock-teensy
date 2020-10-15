use mk66;
use capsules::clock_pm;
use components::Component;

pub struct ClockComponent;

impl ClockComponent {
    pub fn new() -> Self {
        ClockComponent {}
    }
}

impl Component for ClockComponent {
    type Output = &'static clock_pm::ClockCM<mk66::clock_pm::TeensyClockManager>;

    unsafe fn finalize(&mut self) -> Option<Self::Output> {

        let clock_driver = static_init!(
            clock_pm::ClockCM<mk66::clock_pm::TeensyClockManager>,
            clock_pm::ClockCM::new(
                mk66::clock_pm::TeensyClockManager::new()
            )
        );
        Some(clock_driver)
    }
}
