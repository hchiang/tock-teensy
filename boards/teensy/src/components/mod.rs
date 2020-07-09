pub trait Component {
    type Output;

    unsafe fn finalize(&mut self) -> Option<Self::Output>;
}

pub trait ComponentWithDependency<D>: Component {
    fn dependency(&mut self, _dep: D) -> &mut Self { self }
}

mod adc;
mod clock_pm;
mod nonvolatile_storage;
mod gpio;
mod led;
mod spi;
mod alarm;
mod console;
mod xconsole;
mod rnga;

pub use self::adc::AdcComponent;
pub use self::clock_pm::ClockManagerComponent;
pub use self::nonvolatile_storage::NonvolatileStorageComponent;
pub use self::gpio::GpioComponent;
pub use self::led::LedComponent;
pub use self::spi::VirtualSpiComponent;
pub use self::alarm::AlarmComponent;
pub use self::console::UartConsoleComponent;
pub use self::xconsole::XConsoleComponent;
pub use self::rnga::RngaComponent;
