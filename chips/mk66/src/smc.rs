use kernel::common::regs::{ReadWrite, ReadOnly};
use kernel::common::StaticRef;

#[repr(C)]
pub struct Registers {
    pmprot: ReadWrite<u8>,
    pmctrl: ReadWrite<u8>,
    stopctrl: ReadWrite<u8, StopControl::Register>,
    pmstat: ReadOnly<u8>,
}

pub enum StopModes {
    STOP,
    PSTOP1,
    PSTOP2
}

register_bitfields![u8,
    StopControl [
        PSTOPO OFFSET(6) NUMBITS(2) [
            STOP = 0,
            PSTOP1 = 1,
            PSTOP2 = 2
        ],
        PORPO OFFSET(5) NUMBITS(1) [],
        RAM2POO OFFSET(4) NUMBITS(1) [],
        LLSM OFFSET(0) NUMBITS(3) []
    ]
];

pub const SMC_REGS: StaticRef<Registers> = unsafe { StaticRef::new(0x4007E000 as *mut Registers) };

pub fn set_partial_stop(stop: StopModes) {
    let regs: &Registers = &*SMC_REGS;
    regs.stopctrl.modify(StopControl::PSTOPO.val(stop as u8));
}
