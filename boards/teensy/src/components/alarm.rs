use capsules::alarm::AlarmDriver;
use kernel;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::time::Alarm;
use kernel::static_init;

pub struct AlarmComponent {
    board_kernel: &'static kernel::Kernel,
}

impl AlarmComponent {
    pub fn new(board_kernel: &'static kernel::Kernel) -> Self {
        AlarmComponent {
            board_kernel: board_kernel
        }
    }
}

impl Component for AlarmComponent {
    type StaticInput = ();
    type Output = &'static AlarmDriver<'static, mk66::pit::Pit<'static>>;

    unsafe fn finalize(&mut self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        mk66::pit::PIT.init();

        let alarm = static_init!(
                AlarmDriver<'static, mk66::pit::Pit>,
                AlarmDriver::new(&mk66::pit::PIT,
                                self.board_kernel.create_grant(&grant_cap))
            );
        mk66::pit::PIT.set_client(alarm);
        alarm
    }
}
