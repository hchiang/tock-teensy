#![no_std]
#![no_main]
#![feature(asm,const_fn,lang_items)]

extern crate capsules;

#[macro_use(debug, debug_gpio, static_init, register_bitfields, register_bitmasks)]
extern crate kernel;

#[allow(dead_code)]
extern crate mk66;

#[macro_use]
pub mod io;

#[allow(dead_code)]
mod tests;

#[allow(dead_code)]
mod spi;

#[allow(dead_code)]
mod components;

pub mod xconsole;

#[allow(dead_code)]
mod pins;

use components::*;

#[allow(unused)]
struct Teensy {
    xconsole: <XConsoleComponent as Component>::Output,
    adc: <AdcComponent as Component>::Output,
    nonvolatile_storage: <NonvolatileStorageComponent as Component>::Output,
    gpio: <GpioComponent as Component>::Output,
    led: <LedComponent as Component>::Output,
    alarm: <AlarmComponent as Component>::Output,
    clock_driver: <ClockComponent as Component>::Output,
    //spi: <VirtualSpiComponent as Component>::Output,
    rng: <RngaComponent as Component>::Output,
    ipc: kernel::ipc::IPC,
}

impl kernel::Platform for Teensy {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
        where F: FnOnce(Option<&kernel::Driver>) -> R
    {
        match driver_num {
            xconsole::DRIVER_NUM => f(Some(self.xconsole)),
            capsules::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules::nonvolatile_storage_driver::DRIVER_NUM => f(Some(self.nonvolatile_storage)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),

            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            //spi::DRIVER_NUM => f(Some(self.spi)),

            capsules::led::DRIVER_NUM => f(Some(self.led)),

            capsules::rng::DRIVER_NUM => f(Some(self.rng)),

            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            28 => f(Some(self.clock_driver)),
            _ => f(None),
        }
    }
}

#[link_section = ".flashconfig"]
#[no_mangle]
pub static FLASH_CONFIG_BYTES: [u8; 16] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xDE, 0xF9, 0xFF, 0xFF,
];

#[no_mangle]
pub unsafe fn reset_handler() {
    // Disable the watchdog.
    mk66::wdog::stop();

    // Relocate the text and data segments.
    mk66::init();

    // Configure the system clock.
    mk66::mcg::SCM.change_system_clock(mk66::mcg::SystemClockSource::PLL(120));

    // Enable the Port Control and Interrupt clocks.
    mk66::sim::enable_clock(mk66::sim::Clock::Clock5(mk66::sim::ClockGate5::PORTA));
    mk66::sim::enable_clock(mk66::sim::Clock::Clock5(mk66::sim::ClockGate5::PORTB));
    mk66::sim::enable_clock(mk66::sim::Clock::Clock5(mk66::sim::ClockGate5::PORTC));
    mk66::sim::enable_clock(mk66::sim::Clock::Clock5(mk66::sim::ClockGate5::PORTD));
    mk66::sim::enable_clock(mk66::sim::Clock::Clock5(mk66::sim::ClockGate5::PORTE));

    let (gpio_pins, led_pins) = pins::configure_all_pins();
    kernel::debug::assign_gpios(Some(gpio_pins[24]), Some(gpio_pins[25]), None);
    debug_gpio!(0, make_output);
    debug_gpio!(0, clear);
    debug_gpio!(1, make_output);
    debug_gpio!(1, clear);

    let clock_driver = ClockComponent::new().finalize().unwrap();

    let xconsole = XConsoleComponent::new().finalize().unwrap();
    let adc = AdcComponent::new().finalize().unwrap();
    let nonvolatile_storage = NonvolatileStorageComponent::new().finalize().unwrap();
    let gpio = GpioComponent::new()
                             .dependency(gpio_pins)
                             .finalize().unwrap();
    let led = LedComponent::new()
                           .dependency(led_pins)
                           .finalize().unwrap();
    let alarm = AlarmComponent::new().finalize().unwrap();
    //let spi = VirtualSpiComponent::new().finalize().unwrap();
    let rng = RngaComponent::new().finalize().unwrap();

    let teensy = Teensy {
        xconsole: xconsole,
        adc: adc,
        clock_driver: clock_driver,
        nonvolatile_storage: nonvolatile_storage,
        gpio: gpio,
        led: led,
        alarm: alarm,
        //spi: spi,
        rng: rng,
        ipc: kernel::ipc::IPC::new(),
    };

    let mut chip = mk66::chip::MK66::new();

    if tests::TEST {
        tests::test();
    }
    kernel::kernel_loop(&teensy, &mut chip, load_processes(), Some(&teensy.ipc));
}


unsafe fn load_processes() -> &'static mut [Option<&'static mut kernel::procs::Process<'static>>] {
    extern "C" {
        /// Beginning of the ROM region containing the app images.
        static _sapps: u8;
    }

    const NUM_PROCS: usize = 1;

    // Total memory allocated to the processes
    #[link_section = ".app_memory"]
    static mut APP_MEMORY: [u8; 1 << 17] = [0; 1 << 17];

    // How the kernel responds when a process faults
    const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

    static mut PROCESSES: [Option<&'static mut kernel::procs::Process<'static>>; NUM_PROCS] = [None];

    kernel::procs::load_processes(
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
    );

    &mut PROCESSES
}
