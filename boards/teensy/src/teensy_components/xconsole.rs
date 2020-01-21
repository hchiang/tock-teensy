use mk66;
use kernel;
use xconsole;
use kernel::component::Component;
use kernel::hil::uart::{Transmit, Receive};

pub struct XConsoleComponent;

impl XConsoleComponent {
    pub fn new() -> Self {
        XConsoleComponent {}
    }
}

impl Component for XConsoleComponent {
    type StaticInput = ();
    type Output = &'static xconsole::XConsole<'static, mk66::uart::Uart>;

    unsafe fn finalize(&mut self, _s: Self::StaticInput) -> Self::Output {
        let xconsole = static_init!(
                xconsole::XConsole<mk66::uart::Uart>,
                xconsole::XConsole::new(&mk66::uart::UART0,
                                        115200,
                                        &mut xconsole::WRITE_BUF,
                                        &mut xconsole::READ_BUF,
                                        kernel::Grant::create())
            );
        mk66::uart::UART0.set_transmit_client(xconsole);
        xconsole.initialize();

        let kc = static_init!(
                xconsole::App,
                xconsole::App::default()
            );
        kernel::debug::assign_console_driver(Some(xconsole), kc);

        mk66::uart::UART0.enable_rx();
        mk66::uart::UART0.enable_rx_interrupts();

        xconsole
    }
}
