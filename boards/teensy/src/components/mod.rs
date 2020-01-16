mod gpio;
mod led;
mod spi;
mod alarm;
mod console;
//mod xconsole;
mod rnga;

pub use self::gpio::GpioComponent;
pub use self::led::LedComponent;
pub use self::spi::VirtualSpiComponent;
pub use self::alarm::AlarmComponent;
pub use self::console::ConsoleComponent;
//pub use self::xconsole::XConsoleComponent;
pub use self::rnga::RngaComponent;
