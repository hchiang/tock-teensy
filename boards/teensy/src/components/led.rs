use capsules::led;
use kernel::component::Component;
use kernel::hil::gpio::Pin;
use kernel::static_init;

pub struct LedComponent {
}

impl LedComponent {
    pub fn new() -> Self {
        LedComponent {
        }
    }
}

impl Component for LedComponent {
    type StaticInput = &'static [(&'static dyn Pin, led::ActivationMode)];
    type Output = &'static capsules::led::LED<'static>;

    unsafe fn finalize(&mut self, pins: Self::StaticInput) -> Self::Output {
        let led = static_init!(led::LED<'static>, led::LED::new(pins));

        led
    }
}
