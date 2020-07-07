use kernel::common::regs::{ReadWrite, ReadOnly};
use kernel::common::StaticRef;

#[repr(C)]
pub struct Registers {
    pmprot: ReadWrite<u8, PowerModeProtection::Register>,
    pmctrl: ReadWrite<u8, PowerModeControl::Register>,
    stopctrl: ReadWrite<u8, StopControl::Register>,
    pmstat: ReadOnly<u8, PowerModeStatus::Register>,
}

register_bitfields![u8,
    PowerModeProtection [
        AHSRUN OFFSET(7) NUMBITS(1) [],
        AVLP OFFSET(5) NUMBITS(1) [],
        ALLS OFFSET(3) NUMBITS(1) [],
        AVLLS OFFSET(1) NUMBITS(1) []
    ],
    PowerModeControl [
        RUNM OFFSET(5) NUMBITS(2) [],
        STOPA OFFSET(3) NUMBITS(1) [],
        STOPM OFFSET(0) NUMBITS(3) [
            STOP = 0,
            VLPS = 2,
            LLSx = 3,
            VLLSx = 4
        ]
    ],
    StopControl [
        PSTOPO OFFSET(6) NUMBITS(2) [
            STOP = 0,
            PSTOP1 = 1,
            PSTOP2 = 2
        ],
        PORPO OFFSET(5) NUMBITS(1) [],
        RAM2POO OFFSET(4) NUMBITS(1) [],
        LLSM OFFSET(0) NUMBITS(3) []
    ],
    PowerModeStatus [
        PMSTAT OFFSET(0) NUMBITS(8) []
    ]
];

pub const SMC_REGS: StaticRef<Registers> = unsafe { StaticRef::new(0x4007E000 as *mut Registers) };

pub fn set_vlps() {
    let regs: &Registers = &*SMC_REGS;
    regs.pmprot.modify(PowerModeProtection::AVLP::SET);
    regs.pmctrl.modify(PowerModeControl::STOPM::VLPS);
}
