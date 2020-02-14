// On reset, MCGOUTCLK is sourced from the 32kHz internal reference clock 
// multiplied by the FLL, which has a default multiplier of 640.
static mut MCGOUTCLK: u32 = 20_480_000;

static mut CORECLK: u32 = 20_480_000;
static mut BUSCLK: u32 = 20_480_000;
static mut FLASHCLK: u32 = 10_240_000;

use osc;
use mcg;
use sim;

pub fn peripheral_clock_hz() -> u32 {
    unsafe { BUSCLK }
}

pub fn bus_clock_hz() -> u32 {
    unsafe { BUSCLK }
}

pub fn flash_clock_hz() -> u32 {
    unsafe { FLASHCLK }
}

pub fn core_clock_hz() -> u32 {
    unsafe { CORECLK }
}

#[allow(non_upper_case_globals)]
const MHz: u32 = 1_000_000;

fn configure_div(core_freq: u32) {
    let mut bus_div = 1;
    while core_freq / bus_div > 60 {
        bus_div += 1;
    }

    let mut flash_div = 1;
    while core_freq / flash_div > 28 {
        flash_div += 1;
    }

    sim::set_dividers(1, bus_div, flash_div);

    unsafe {
        MCGOUTCLK = core_freq * MHz;
        CORECLK = core_freq * MHz;
        BUSCLK = (core_freq * MHz) / bus_div; 
        FLASHCLK = (core_freq * MHz) / flash_div;
    }
}

pub fn configure(core_freq: u32) {
    if let mcg::State::Fei(fei) = mcg::state() {

        osc::enable(mcg::xtals::Teensy16MHz);

        configure_div(core_freq);

        //mcg::set_fll_freq(mcg::FLLFreq::FLL24M);
        let fbe = fei.to_fbe(mcg::xtals::Teensy16MHz);
        let pbe = fbe.to_pbe(core_freq);
        pbe.to_pee();


    } else {
        // We aren't in FEI mode, meaning that configuration has already occurred.
        // For now, just exit without changing the existing configuration.
        return;
    }
}
