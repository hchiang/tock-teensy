use mk66;
use kernel;
use components::Component;
use capsules::alarm::AlarmDriver;

pub struct AlarmComponent;

impl AlarmComponent {
    pub fn new() -> Self {
        AlarmComponent {}
    }
}

impl Component for AlarmComponent {
    type Output = &'static AlarmDriver<'static, mk66::lptmr::Lptmr<'static>>;

    unsafe fn finalize(&mut self) -> Option<Self::Output> {
        mk66::lptmr::LPTMR.init();

        let alarm = static_init!(
                AlarmDriver<'static, mk66::lptmr::Lptmr>,
                AlarmDriver::new(&mk66::lptmr::LPTMR,
                                 kernel::Grant::create())
            );
        mk66::lptmr::LPTMR.set_client(alarm);
        Some(alarm)
    }
}
