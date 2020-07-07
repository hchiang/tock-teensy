//! Implementation of the MK66 System Integration Module
use regs::sim::*;
use kernel::common::regs::FieldValue;
use kernel::ClockInterface;

#[derive(Copy, Clone, Debug)]
pub enum Clock {
    Clock1(ClockGate1),
    Clock2(ClockGate2),
    Clock3(ClockGate3),
    Clock4(ClockGate4),
    Clock5(ClockGate5),
    Clock6(ClockGate6),
    Clock7(ClockGate7),
}

#[derive(Copy, Clone, Debug)]
pub enum ClockGate1 {
    I2C2 = 6,
    I2C3,
    UART4 = 10,
}

#[derive(Copy, Clone, Debug)]
pub enum ClockGate2 {
    ENET = 0,
    LPUART0 = 4,
    TPM1 = 9,
    TPM2,
    DAC0 = 12,
    DAC1,
}

#[derive(Copy, Clone, Debug)]
pub enum ClockGate3 {
    RNGA,
    USBHS,
    USBHSPHY,
    USBHSDCD,
    FLEXCAN1,
    SPI2 = 12,
    SDHC = 17,
    FTM2 = 24,
    FTM3,
    ADC1 = 27,
}

#[derive(Copy, Clone, Debug)]
pub enum ClockGate4 {
    EWM = 1,
    CMT,
    I2C0 = 6,
    I2C1,
    UART0 = 10,
    UART1,
    UART2, 
    UART3,
    USBOTG = 18,
    CMP,
    VREF,
}

#[derive(Copy, Clone, Debug)]
pub enum ClockGate5 {
    LPTMR,
    TSI = 5,
    PORTA = 9,
    PORTB,
    PORTC,
    PORTD,
    PORTE,
}

#[derive(Copy, Clone, Debug)]
pub enum ClockGate6 {
    FTF,
    DMAMUX,
    FLEXCAN0 = 4,
    RNGA = 9,   
    SPI0 = 12,
    SPI1,
    I2S = 15,
    CRC = 18,
    USBDCD = 21,
    PDB,
    PIT,
    FTM0,
    FTM1,
    FTM2,
    ADC0, 
    RTC = 29,
    DAC0 = 31,
}

#[derive(Copy, Clone, Debug)]
pub enum ClockGate7 {
    FLEXBUS,
    DMA,
    MPU,
    SDRAMC,
}

impl ClockInterface for Clock {
    fn is_enabled(&self) -> bool {
        match self {
            &Clock::Clock1(v) => SIM_REGS.scgc1.get() & (1 << (v as u32)) != 0,
            &Clock::Clock2(v) => SIM_REGS.scgc2.get() & (1 << (v as u32)) != 0,
            &Clock::Clock3(v) => SIM_REGS.scgc3.get() & (1 << (v as u32)) != 0,
            &Clock::Clock4(v) => SIM_REGS.scgc4.get() & (1 << (v as u32)) != 0,
            &Clock::Clock5(v) => SIM_REGS.scgc5.get() & (1 << (v as u32)) != 0,
            &Clock::Clock6(v) => SIM_REGS.scgc6.get() & (1 << (v as u32)) != 0,
            &Clock::Clock7(v) => SIM_REGS.scgc7.get() & (1 << (v as u32)) != 0,
        }
    }
    fn enable(&self) {
        match self {
            &Clock::Clock1(v) => SIM_REGS.scgc1.set(SIM_REGS.scgc1.get() | 1 << (v as u32)),
            &Clock::Clock2(v) => SIM_REGS.scgc2.set(SIM_REGS.scgc2.get() | 1 << (v as u32)),
            &Clock::Clock3(v) => SIM_REGS.scgc3.set(SIM_REGS.scgc3.get() | 1 << (v as u32)),
            &Clock::Clock4(v) => SIM_REGS.scgc4.set(SIM_REGS.scgc4.get() | 1 << (v as u32)),
            &Clock::Clock5(v) => SIM_REGS.scgc5.set(SIM_REGS.scgc5.get() | 1 << (v as u32)),
            &Clock::Clock6(v) => SIM_REGS.scgc6.set(SIM_REGS.scgc6.get() | 1 << (v as u32)),
            &Clock::Clock7(v) => SIM_REGS.scgc7.set(SIM_REGS.scgc7.get() | 1 << (v as u32)),
        }
    }
    fn disable(&self) {
        match self {
            &Clock::Clock1(v) => SIM_REGS.scgc1.set(SIM_REGS.scgc1.get() & !(1 << (v as u32))),
            &Clock::Clock2(v) => SIM_REGS.scgc2.set(SIM_REGS.scgc2.get() & !(1 << (v as u32))),
            &Clock::Clock3(v) => SIM_REGS.scgc3.set(SIM_REGS.scgc3.get() & !(1 << (v as u32))),
            &Clock::Clock4(v) => SIM_REGS.scgc4.set(SIM_REGS.scgc4.get() & !(1 << (v as u32))),
            &Clock::Clock5(v) => SIM_REGS.scgc5.set(SIM_REGS.scgc5.get() & !(1 << (v as u32))),
            &Clock::Clock6(v) => SIM_REGS.scgc6.set(SIM_REGS.scgc6.get() & !(1 << (v as u32))),
            &Clock::Clock7(v) => SIM_REGS.scgc7.set(SIM_REGS.scgc7.get() & !(1 << (v as u32))),
        }
    }
}

pub fn enable_clock(clock: Clock) {
    match clock {
        Clock::Clock1(v) => SIM_REGS.scgc1.set(SIM_REGS.scgc1.get() | 1 << (v as u32)),
        Clock::Clock2(v) => SIM_REGS.scgc2.set(SIM_REGS.scgc2.get() | 1 << (v as u32)),
        Clock::Clock3(v) => SIM_REGS.scgc3.set(SIM_REGS.scgc3.get() | 1 << (v as u32)),
        Clock::Clock4(v) => SIM_REGS.scgc4.set(SIM_REGS.scgc4.get() | 1 << (v as u32)),
        Clock::Clock5(v) => SIM_REGS.scgc5.set(SIM_REGS.scgc5.get() | 1 << (v as u32)),
        Clock::Clock6(v) => SIM_REGS.scgc6.set(SIM_REGS.scgc6.get() | 1 << (v as u32)),
        Clock::Clock7(v) => SIM_REGS.scgc7.set(SIM_REGS.scgc7.get() | 1 << (v as u32)),
    }
}

pub fn disable_clock(clock: Clock) {
    match clock {
        Clock::Clock1(v) => SIM_REGS.scgc1.set(SIM_REGS.scgc1.get() & !(1 << (v as u32))),
        Clock::Clock2(v) => SIM_REGS.scgc2.set(SIM_REGS.scgc2.get() & !(1 << (v as u32))),
        Clock::Clock3(v) => SIM_REGS.scgc3.set(SIM_REGS.scgc3.get() & !(1 << (v as u32))),
        Clock::Clock4(v) => SIM_REGS.scgc4.set(SIM_REGS.scgc4.get() & !(1 << (v as u32))),
        Clock::Clock5(v) => SIM_REGS.scgc5.set(SIM_REGS.scgc5.get() & !(1 << (v as u32))),
        Clock::Clock6(v) => SIM_REGS.scgc6.set(SIM_REGS.scgc6.get() & !(1 << (v as u32))),
        Clock::Clock7(v) => SIM_REGS.scgc7.set(SIM_REGS.scgc7.get() & !(1 << (v as u32))),
    }
}

pub fn set_dividers(core: u32, bus: u32, flash: u32) {
    SIM_REGS.clkdiv1.modify(ClockDivider1::Core.val(core - 1) +
                            ClockDivider1::Bus.val(bus - 1) +
                            ClockDivider1::FlexBus.val(bus - 1) +
                            ClockDivider1::Flash.val(flash - 1));
}

pub fn deep_sleep_ready() -> bool {
    // From Table 8-1 and 8-2
    let clockgate2_mask: FieldValue<u32, SystemClockGatingControl2::Register> =
        SystemClockGatingControl2::DAC1::SET +
        SystemClockGatingControl2::DAC0::SET;
    let clockgate4_mask: FieldValue<u32, SystemClockGatingControl4::Register> =
        SystemClockGatingControl4::VREF::SET +
        SystemClockGatingControl4::CMP::SET;
    let clockgate5_mask: FieldValue<u32, SystemClockGatingControl5::Register> =
        SystemClockGatingControl5::PORTE::SET +
        SystemClockGatingControl5::PORTD::SET +
        SystemClockGatingControl5::PORTC::SET +
        SystemClockGatingControl5::PORTB::SET +
        SystemClockGatingControl5::PORTA::SET +
        SystemClockGatingControl5::TSI::SET +
        SystemClockGatingControl5::LPTMR::SET;
    let clockgate6_mask: FieldValue<u32, SystemClockGatingControl6::Register> =
        SystemClockGatingControl6::RTC::SET +
        SystemClockGatingControl6::DMAMUX::SET +
        SystemClockGatingControl6::FTF::SET; 
    let clockgate7_mask: FieldValue<u32, SystemClockGatingControl7::Register> =
        SystemClockGatingControl7::MPU::SET + 
        SystemClockGatingControl7::DMA::SET;
    
    let cg1 = SIM_REGS.scgc1.get() == 0;
    let cg2 = SIM_REGS.scgc2.get() & !clockgate2_mask.mask() == 0;
    let cg3 = SIM_REGS.scgc3.get() == 0;
    let cg4 = SIM_REGS.scgc4.get() & !clockgate4_mask.mask() == 0xf000_0030; 
    let cg5 = SIM_REGS.scgc5.get() & !clockgate5_mask.mask() == 0x40182;
    let cg6 = SIM_REGS.scgc6.get() & !clockgate6_mask.mask() == 0x4000_0000; 
    let cg7 = SIM_REGS.scgc7.get() & !clockgate7_mask.mask() == 0;

    cg1 && cg2 && cg3 && cg4 && cg5 && cg6 && cg7
}
