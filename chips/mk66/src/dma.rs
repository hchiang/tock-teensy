//! Implementation of the DMAMUX peripheral.

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::regs::ReadWrite;
use kernel::common::StaticRef;

/// Memory registers for a DMA channel. Section 23.4 of the datasheet.
#[repr(C)]
#[allow(dead_code)]
struct DMARegisters {
    chcfg: ReadWrite<u8, ChannelConfiguration::Register>,
}

register_bitfields![u8,
    ChannelConfiguration [
        /// DMA Channel Enable
        ENBL OFFSET(7) NUMBITS(1) [],
        /// DMA Channel Trigger Enable
        TRIG OFFSET(6) NUMBITS(1) [],
        /// DMA Channel Source
        SOURCE OFFSET(0) NUMBITS(6) []
    ]
];

/// The DMAMUX's base addresses in memory (Section 23.4 of manual).
const DMA_BASE_ADDR: usize = 0x40021000;

/// The number of bytes between each memory mapped DMA Channel (Section 23.4).
const DMA_CHANNEL_SIZE: usize = 0x1;

/// Shared counter that Keeps track of how many DMA channels are currently
/// active.
static mut NUM_ENABLED: usize = 0;

/// The DMA channel number. Each channel transfers data between memory and a
/// peripheral that is assigned to it. There are 32 available channels, but
/// channel i and i+16 share the same NVIC interrupt (Table 3-3).
#[derive(Copy, Clone)]
pub enum DMAChannelNum {
    DMAChannel00 = 0,
    DMAChannel01 = 1,
    DMAChannel02 = 2,
    DMAChannel03 = 3,
    DMAChannel04 = 4,
    DMAChannel05 = 5,
    DMAChannel06 = 6,
    DMAChannel07 = 7,
    DMAChannel08 = 8,
    DMAChannel09 = 9,
    DMAChannel10 = 10,
    DMAChannel11 = 11,
    DMAChannel12 = 12,
    DMAChannel13 = 13,
    DMAChannel14 = 14,
    DMAChannel15 = 15,
}

/// The peripheral request sources a channel can be assigned to (Table 23-1). 
/// `*_RX` means transfer data from peripheral to memory, `*_TX` means transfer
/// data from memory to peripheral.
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum DMAPeripheral {
    TSI0 = 1,
    UART0_RX = 2,
    UART0_TX = 3,
    UART1_RX = 4,
    UART1_TX = 5,
    UART2_RX = 6,
    UART2_TX = 7,
    UART3_RX = 8,
    UART3_TX = 9,
    UART4 = 10,
    I2S0_RX = 12,
    I2S0_TX = 13,
    SPI0_RX = 14,
    SPI0_TX = 15,
    SPI1_RX = 16,
    SPI1_tx = 17,
    I2C0 = 18,
    I2C1 = 19,
    FTM0_CH0 = 20, 
    FTM0_CH1 = 21, 
    FTM0_CH2 = 22, 
    FTM0_CH3 = 23, 
    FTM0_CH4 = 24, 
    FTM0_CH5 = 25, 
    FTM0_CH6 = 26, 
    FTM0_CH7 = 27, 
    FTM1_TPM1_CH0 = 28, 
    FTM1_TPM1_CH1 = 29, 
    FTM2_TPM2_CH0 = 30, 
    FTM2_TPM2_CH1 = 31, 
    FTM3_CH0 = 32, 
    FTM3_CH1 = 33, 
    FTM3_CH2 = 34, 
    FTM3_CH3 = 35, 
    FTM3_CH4 = 36, 
    FTM3_CH5 = 37, 
    FTM3_CH6_SPI2_RX = 38, 
    FTM3_CH7_SPI2_TX = 39, 
    ADC0 = 40,
    ADC1 = 41,
    CMP0 = 42,
    CMP1 = 43,
    CMP2_CMP3 = 44,
    DAC0 = 45,
    DAC1 = 46,
    CMT = 47,
    PDB = 48,
}

pub static mut DMA_CHANNELS: [DMAChannel; 16] = [
    DMAChannel::new(DMAChannelNum::DMAChannel00),
    DMAChannel::new(DMAChannelNum::DMAChannel01),
    DMAChannel::new(DMAChannelNum::DMAChannel02),
    DMAChannel::new(DMAChannelNum::DMAChannel03),
    DMAChannel::new(DMAChannelNum::DMAChannel04),
    DMAChannel::new(DMAChannelNum::DMAChannel05),
    DMAChannel::new(DMAChannelNum::DMAChannel06),
    DMAChannel::new(DMAChannelNum::DMAChannel07),
    DMAChannel::new(DMAChannelNum::DMAChannel08),
    DMAChannel::new(DMAChannelNum::DMAChannel09),
    DMAChannel::new(DMAChannelNum::DMAChannel10),
    DMAChannel::new(DMAChannelNum::DMAChannel11),
    DMAChannel::new(DMAChannelNum::DMAChannel12),
    DMAChannel::new(DMAChannelNum::DMAChannel13),
    DMAChannel::new(DMAChannelNum::DMAChannel14),
    DMAChannel::new(DMAChannelNum::DMAChannel15),
];

pub struct DMAChannel {
    registers: StaticRef<DMARegisters>,
    client: OptionalCell<&'static DMAClient>,
    periph: Cell<Option<DMAPeripheral>>,
    enabled: Cell<bool>,
}

pub trait DMAClient {
    fn transfer_done(&self, pid: DMAPeripheral);
}

impl DMAChannel {
    const fn new(channel: DMAChannelNum) -> DMAChannel {
        DMAChannel {
            registers: unsafe {
                StaticRef::new(
                    (DMA_BASE_ADDR + (channel as usize) * DMA_CHANNEL_SIZE) as *const DMARegisters,
                )
            },
            client: OptionalCell::empty(),
            periph: Cell::new(None),
            enabled: Cell::new(false),
        }
    }

    pub fn initialize(&self, client: &'static mut DMAClient, periph: DMAPeripheral) {
        self.client.set(client);
        self.periph.set(Some(periph));
    }

    pub fn enable(&self) {
        if !self.enabled.get() {
            let registers: &DMARegisters = &*self.registers;
            registers
                .chcfg
                .write(ChannelConfiguration::ENBL::SET + ChannelConfiguration::SOURCE.val(self.periph.get().unwrap() as u8)); 
            self.enabled.set(true);
        }
    }

    pub fn disable(&self) {
        if self.enabled.get() {
            let registers: &DMARegisters = &*self.registers;
            registers.chcfg.write(ChannelConfiguration::ENBL::CLEAR);
            self.enabled.set(false);
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }

    pub fn handle_interrupt(&mut self, channel: DMAPeripheral) {
        self.client.map(|client| {
            client.transfer_done(channel);
        });
    }
}
