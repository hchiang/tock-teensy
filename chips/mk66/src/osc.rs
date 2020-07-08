use core::mem;
use regs::osc::*;

pub use self::Control::CAP::Value as OscCapacitance;

pub fn enable(capacitance: u8) {
    let regs: &mut Registers = unsafe { mem::transmute(OSC) };

    // Set the capacitance.
    regs.cr.modify(Control::CAP.val(capacitance));

    // Enable the oscillator.
    regs.cr.modify(Control::ERCLKEN::SET);
}

pub fn disable() {
    let regs: &mut Registers = unsafe { mem::transmute(OSC) };
    regs.cr.modify(Control::ERCLKEN::CLEAR);
}
