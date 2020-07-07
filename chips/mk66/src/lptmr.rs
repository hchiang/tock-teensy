use core::cell::Cell;
use kernel::common::regs::ReadWrite;
use kernel::common::StaticRef;
use kernel::hil::time::{Client, Time, Alarm, Frequency};
use nvic;
use sim;

#[repr(C)]
pub struct LptmrRegisters {
    csr: ReadWrite<u32, ControlStatus::Register>,
    psr: ReadWrite<u32, Prescale::Register>,
    cmr: ReadWrite<u32, Compare::Register>,
    cnr: ReadWrite<u32, Counter::Register>,
}

register_bitfields![u32,
    ControlStatus [
        TCF OFFSET(7) NUMBITS(1) [],
        TIE OFFSET(6) NUMBITS(1) [],
        TPS OFFSET(4) NUMBITS(2) [],
        TPP OFFSET(3) NUMBITS(1) [],
        TFC OFFSET(2) NUMBITS(1) [],
        TMS OFFSET(1) NUMBITS(1) [],
        TEN OFFSET(0) NUMBITS(1) []
    ],
    Prescale [
        PRESCALE OFFSET(3) NUMBITS(4) [],
        PBYP OFFSET(2) NUMBITS(1) [],
        PCS OFFSET(0) NUMBITS(2) [
            MCGIRCLK = 0,
            LPO = 1,
            ERCLK32K = 2,
            OSCERCLK_UNDIV = 3
        ]
    ],
    Compare [
        COMPARE OFFSET(0) NUMBITS(16) []
    ],
    Counter [
        COUNTER OFFSET(0) NUMBITS(16) []
    ]
];
    

pub const LPTMR_ADDRS: StaticRef<LptmrRegisters> = unsafe { 
        StaticRef::new(0x4004_0000 as *const LptmrRegisters)};
pub static mut LPTMR: Lptmr<'static> = Lptmr::new();

pub struct Lptmr<'a> {
    pub client: Cell<Option<&'a Client>>,
    alarm: Cell<u32>,
    registers: StaticRef<LptmrRegisters>,
}

impl<'a> Lptmr<'a> {
    pub const fn new() -> Self {
        Lptmr {
            client: Cell::new(None),
            alarm: Cell::new(0),
            registers: LPTMR_ADDRS,
        }
    }

    pub fn init(&self) {
        unsafe { nvic::enable(nvic::NvicIdx::LPTMR); }
        sim::enable_clock(sim::Clock::Clock5(sim::ClockGate5::LPTMR));

        let regs: &LptmrRegisters = &*self.registers;

        // these values should only be altered when LPTMR is disabled
        // CNR is reset when CMR is reached, LPTMR in counter mode
        regs.csr.modify(ControlStatus::TFC::CLEAR + ControlStatus::TMS::CLEAR);
        // Bypass prescaler, select LPO as clock
        regs.psr.modify(Prescale::PBYP::SET + Prescale::PCS::LPO);
    }

    pub fn enable(&self) {
        let regs: &LptmrRegisters = &*self.registers;
        regs.csr.modify(ControlStatus::TEN::SET);
    }

    pub fn is_enabled(&self) -> bool {
        let regs: &LptmrRegisters = &*self.registers;
        regs.csr.is_set(ControlStatus::TEN)
    }

    pub fn enable_interrupt(&self) {
        let regs: &LptmrRegisters = &*self.registers;
        regs.csr.modify(ControlStatus::TIE::SET);
    }

    pub fn set_counter(&self, value: u32) {
        let regs: &LptmrRegisters = &*self.registers;
        regs.cmr.modify(Compare::COMPARE.val(value));
    }

    pub fn get_counter(&self)  -> u32 {
        let regs: &LptmrRegisters = &*self.registers;
        regs.cnr.read(Counter::COUNTER)
    }

    pub fn clear_pending(&self) {
        let regs: &LptmrRegisters = &*self.registers;
        regs.csr.modify(ControlStatus::TCF::SET);
    }

    pub fn disable(&self) {
        let regs: &LptmrRegisters = &*self.registers;
        regs.csr.modify(ControlStatus::TEN::CLEAR);
    }

    pub fn disable_interrupt(&self) {
        let regs: &LptmrRegisters = &*self.registers;
        regs.csr.modify(ControlStatus::TIE::CLEAR);
    }

    pub fn set_client(&self, client: &'a Client) {
        self.client.set(Some(client));
    }

    pub fn handle_interrupt(&self) {
        self.disable();
        self.disable_interrupt();
        self.clear_pending();
        self.client.get().map(|client| { client.fired(); });
    }
}

pub struct LptmrFrequency;
impl Frequency for LptmrFrequency {
    fn frequency() -> u32 {
        1000
    }
}

impl<'a> Time for Lptmr<'a> {
    type Frequency = LptmrFrequency;
    fn disable(&self) {
        Lptmr::disable(self);
        self.disable_interrupt();
        self.clear_pending();
    }

    fn is_armed(&self) -> bool {
        self.is_enabled()
    }
}

impl<'a> Alarm for Lptmr<'a> {
    fn now(&self) -> u32 {
        self.alarm.get()
    }

    fn set_alarm(&self, ticks: u32) {
        Time::disable(self);
        self.alarm.set(ticks.wrapping_sub(self.now()));
        self.set_counter(self.alarm.get());
        self.enable_interrupt();
        self.enable();
    }

    fn get_alarm(&self) -> u32 {
        self.alarm.get()
    }
}
