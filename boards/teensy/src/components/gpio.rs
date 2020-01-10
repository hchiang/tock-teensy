use capsules::gpio;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::gpio::InterruptValuePin;
use kernel::static_init;

pub struct GpioComponent {
    board_kernel: &'static kernel::Kernel,
}

impl GpioComponent {
    pub fn new(board_kernel: &'static kernel::Kernel) -> Self {
        GpioComponent {
            board_kernel: board_kernel
        }
    }
}

impl Component for GpioComponent {
    type StaticInput = &'static [&'static dyn InterruptValuePin];
    type Output = &'static gpio::GPIO<'static>;

    unsafe fn finalize(&mut self, pins: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let gpio = static_init!(
                gpio::GPIO<'static>,
                gpio::GPIO::new(pins, self.board_kernel.create_grant(&grant_cap))
            );

        for pin in pins.iter() {
            pin.set_client(gpio);
        }

        gpio
    }
}

