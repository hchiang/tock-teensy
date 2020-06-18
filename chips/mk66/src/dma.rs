//! Implementation of the eDMA peripheral.

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;

/// DMA memory map. Section 24.3.4 of the datasheet.
#[repr(C)]
#[allow(dead_code)]
struct EDMABaseRegisters {
    cr: ReadWrite<u32, ControlRegister::Register>,
    es: ReadOnly<u32, ErrorStatus::Register>,
    _reserved0: ReadOnly<u32>,
    erq: ReadWrite<u32>,
    _reserved1: ReadOnly<u32>,
    eei: ReadWrite<u32>,
    ceei: WriteOnly<u8, ChannelSet::Register>,
    seei: WriteOnly<u8, ChannelSet::Register>,
    cerq: WriteOnly<u8, ChannelSet::Register>,
    serq: WriteOnly<u8, ChannelSet::Register>,
    cdne: WriteOnly<u8, ChannelSet::Register>,
    ssrt: WriteOnly<u8, ChannelSet::Register>,
    cerr: WriteOnly<u8, ChannelSet::Register>,
    cint: WriteOnly<u8, ChannelSet::Register>,
    _reserved2: ReadOnly<u32>,
    int: ReadWrite<u32, ChannelStatus::Register>,
    _reserved3: ReadOnly<u32>,
    err: ReadWrite<u32, ChannelStatus::Register>,
    _reserved4: ReadOnly<u32>,
    hrs: ReadOnly<u32, ChannelStatus::Register>,
}

#[repr(C)]
#[allow(dead_code)]
struct EDMATcdRegisters {
    saddr: ReadWrite<u32, SourceAddress::Register>,
    soff: ReadWrite<u16, SourceAddressOffset::Register>,
    attr: ReadWrite<u16, TransferAttributes::Register>,
    mlo: ReadWrite<u32, MinorLoopOffset::Register>,
    slast: ReadWrite<u32, LastSourceAddressAdjustment::Register>,
    daddr: ReadWrite<u32, DestinationAddress::Register>,
    doff: ReadWrite<u16, DestinationAddressOffset::Register>,
    citer: ReadWrite<u16, CurrentMinorLoopLink::Register>,
    dlastsga: ReadWrite<u32, LastDestinationAddressAdjustment::Register>,
    csr: ReadWrite<u16, ControlAndStatus::Register>,
    biter: ReadWrite<u16, BeginningMinorLoopLink::Register>,
}

/// Memory registers for a DMA channel. Section 23.4 of the datasheet.
#[repr(C)]
#[allow(dead_code)]
struct DMAMUXRegisters {
    chcfg: ReadWrite<u8, ChannelConfiguration::Register>,
}

register_bitfields![u8,
    ChannelSet[
        /// Enable all 
        AEN OFFSET(6) NUMBITS(1) [],
        EN OFFSET(0) NUMBITS(5) []
    ],
    ChannelConfiguration [
        /// DMA Channel Enable
        ENBL OFFSET(7) NUMBITS(1) [],
        /// DMA Channel Trigger Enable
        TRIG OFFSET(6) NUMBITS(1) [],
        /// DMA Channel Source
        SOURCE OFFSET(0) NUMBITS(6) []
    ]
];

register_bitfields![u16,
    SourceAddressOffset[
        SOFF OFFSET(0) NUMBITS(16) []
    ],
    TransferAttributes[
        /// Source address modulo
        SMOD OFFSET(11) NUMBITS(5) [],
        /// Source data transfer size
        SSIZE OFFSET(8) NUMBITS(3) [
            BITS8 = 0b000,
            BITS16 = 0b001,
            BITS32 = 0b010,
            BURST16 = 0b100,
            BURST32 = 0b101
        ],
        /// Destination address modulo
        DMOD OFFSET(3) NUMBITS(5) [],
        /// Destination data transfer size
        DSIZE OFFSET(0) NUMBITS(3) [
            BITS8 = 0b000,
            BITS16 = 0b001,
            BITS32 = 0b010,
            BURST16 = 0b100,
            BURST32 = 0b101
        ]
    ],
    DestinationAddressOffset[
        DOFF OFFSET(0) NUMBITS(16) []
    ],
    CurrentMinorLoopLink[
        /// Enable chancel-to-channel linking on minor-loop complete
        ELINK OFFSET(15) NUMBITS(1) [],
        /// Current major iteration count
        CITER OFFSET(0) NUMBITS(15) []
    ],
    ControlAndStatus[
        /// Disable request
        DREQ OFFSET(3) NUMBITS(1) [],
        /// Enable an interrupt when major iteration count completes
        INTMAJOR OFFSET(1) NUMBITS(1) []
    ],
    ///TODO this register configuration varies depending on if BITER is set
    BeginningMinorLoopLink[
        ELINK OFFSET(15) NUMBITS(1) [],
        BITER OFFSET(0) NUMBITS(15) []
    ]
    
];

register_bitfields![u32,
    ControlRegister[
        /// Enable Minor Loop Mapping
        EMLM OFFSET(7) NUMBITS(1) []
    ],
    ErrorStatus[
        /// Logical OR of all ERR status bits
        VLD OFFSET(31) NUMBITS(1) [],
        /// Transfer Canceled
        ECX OFFSET(16) NUMBITS(1) [],
        /// Group Priority Error
        GPE OFFSET(15) NUMBITS(1) [],
        /// Channel Priority Error
        CPE OFFSET(14) NUMBITS(1) [],
        /// Error Channel Number
        ERRCHN OFFSET(8) NUMBITS(5) [],
        /// Source Address Error 
        SAE OFFSET(7) NUMBITS(1) [],
        /// Source Offset Error 
        SOE OFFSET(6) NUMBITS(1) [],
        /// Destination Address Error 
        DAE OFFSET(5) NUMBITS(1) [],
        /// Destination Offset Error 
        DOE OFFSET(4) NUMBITS(1) [],
        /// NBYTES/CITER Configuration Error 
        NCE OFFSET(3) NUMBITS(1) [],
        /// Scatter/Gather Configuration Error
        SGE OFFSET(2) NUMBITS(1) [],
        /// Source Bus Error
        SBE OFFSET(1) NUMBITS(1) [],
        /// Destination Bus Error
        DBE OFFSET(0) NUMBITS(1) []
    ],
    ChannelStatus[
        C31 31,
        C30 30,
        C29 29,
        C28 28,
        C27 27,
        C26 26,
        C25 25,
        C24 24,
        C23 23,
        C22 22,
        C21 21,
        C20 20,
        C19 19,
        C18 18,
        C17 17,
        C16 16,
        C15 15,
        C14 14,
        C13 13,
        C12 12,
        C11 11,
        C10 10,
        C9 9,
        C8 8,
        C7 7,
        C6 6,
        C5 5,
        C4 4,
        C3 3,
        C2 2,
        C1 1,
        C0 0
    ],
    SourceAddress[
        SADDR OFFSET(0) NUMBITS(32) []
    ],
    ///TODO this register configuration varies depending on CR[EMLM] and SMLOE/DMLOE
    MinorLoopOffset[
        /// Source minor loop offset enable
        SMLOE OFFSET(31) NUMBITS(1) [],
        /// Destination minor loop offset enable
        DMLOE OFFSET(30) NUMBITS(1) [],
        /// Minor loop offset
        MLOFF OFFSET(10) NUMBITS(20) [],
        /// Minor byte transfer count
        NBYTES OFFSET(0) NUMBITS(10) []
    ],
    LastSourceAddressAdjustment[
        /// Adjustment value added to source address at completion of major iteration
        SLAST OFFSET(0) NUMBITS(32) []
    ],
    DestinationAddress[
        DADDR OFFSET(0) NUMBITS(32) []
    ],
    LastDestinationAddressAdjustment[
        DLASTSGA OFFSET(0) NUMBITS(32) []
    ]
];

/// The eDMA's base addresses in memory (Section 24.3.4 of manual).
const EDMA_BASE_ADDR: usize = 0x40008000;
const EDMA_TCD_ADDR: usize = 0x40009000;
const EDMA_TCD_SIZE: usize = 0x20;

/// The DMAMUX's base addresses in memory (Section 23.4 of manual).
const DMAMUX_BASE_ADDR: usize = 0x40021000;
/// The number of bytes between each memory mapped DMA Channel (Section 23.4).
const DMAMUX_CHANNEL_SIZE: usize = 0x1;

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

#[derive(Copy, Clone, PartialEq)]
pub struct TransferConfig {
    saddr: u32,
    soff: u16,
    ssize: u16,
    dsize: u16,
    nbytes: u32,
    slast: u32,
    daddr: u32,
    doff: u16, 
    citer: u16,
    dlastsga: u32,
    biter: u16
}

//TODO add accessor functions
impl TransferConfig {
    pub const fn new(saddr: u32, daddr:u32, nbytes: u16, nruns: u16) -> TransferConfig {
        TransferConfig {
            saddr: saddr,
            soff: 0,
            ssize: 1, //16-bit
            dsize: 1, //16-bit
            nbytes: nbytes as u32,
            slast: 0,
            daddr: daddr,
            doff: nbytes, 
            citer: nruns,
            dlastsga: 0, 
            biter: nruns,
        }
    }
    
}

pub static mut CHANNELS_ENABLED: u8 = 0;
/// 32 DMA channels
pub static mut DMA_CHANNELS: [DMAChannel; 32] = [
    DMAChannel::new(0),
    DMAChannel::new(1),
    DMAChannel::new(2),
    DMAChannel::new(3),
    DMAChannel::new(4),
    DMAChannel::new(5),
    DMAChannel::new(6),
    DMAChannel::new(7),
    DMAChannel::new(8),
    DMAChannel::new(9),
    DMAChannel::new(10),
    DMAChannel::new(11),
    DMAChannel::new(12),
    DMAChannel::new(13),
    DMAChannel::new(14),
    DMAChannel::new(15),
    DMAChannel::new(16),
    DMAChannel::new(17),
    DMAChannel::new(18),
    DMAChannel::new(19),
    DMAChannel::new(20),
    DMAChannel::new(21),
    DMAChannel::new(22),
    DMAChannel::new(23),
    DMAChannel::new(24),
    DMAChannel::new(25),
    DMAChannel::new(26),
    DMAChannel::new(27),
    DMAChannel::new(28),
    DMAChannel::new(29),
    DMAChannel::new(30),
    DMAChannel::new(31),
];

pub struct DMAChannel {
    registers: StaticRef<EDMABaseRegisters>,
    tcd_registers: StaticRef<EDMATcdRegisters>,
    dmamux_registers: StaticRef<DMAMUXRegisters>,
    client: OptionalCell<&'static DMAClient>,
    periph: Cell<Option<DMAPeripheral>>,
    channel: Cell<u8>,
    enabled: Cell<bool>,
}

pub trait DMAClient {
    fn transfer_done(&self);
}

impl DMAChannel {
    const fn new(channel: usize) -> DMAChannel {
        DMAChannel {
            registers: unsafe {
                StaticRef::new(
                    EDMA_BASE_ADDR as *const EDMABaseRegisters,
                )
            },
            tcd_registers: unsafe {
                StaticRef::new(
                    (EDMA_TCD_ADDR + channel * EDMA_TCD_SIZE) as *const EDMATcdRegisters,
                )
            },
            dmamux_registers: unsafe {
                StaticRef::new(
                    (DMAMUX_BASE_ADDR + channel * DMAMUX_CHANNEL_SIZE) as *const DMAMUXRegisters,
                )
            },
            client: OptionalCell::empty(),
            periph: Cell::new(None),
            channel: Cell::new(channel as u8),
            enabled: Cell::new(false),
        }
    }

    pub fn initialize(&self, client: &'static mut DMAClient, periph: DMAPeripheral) {
        self.client.set(client);
        self.periph.set(Some(periph));

    }

    pub fn enable(&self) {
        if self.enabled.get() {
            return;
        }
        
        let registers: &EDMABaseRegisters = &*self.registers;

        unsafe {
        if CHANNELS_ENABLED == 0 {
            use sim::{clocks, Clock};
            clocks::DMAMUX.enable();
            clocks::DMA.enable();
            registers.cr.modify(ControlRegister::EMLM::SET);
        }
        CHANNELS_ENABLED = CHANNELS_ENABLED + 1;
        }

        let dmamux_registers: &DMAMUXRegisters = &*self.dmamux_registers;
        dmamux_registers
            .chcfg
            .modify(ChannelConfiguration::ENBL::SET + 
                ChannelConfiguration::SOURCE.val(self.periph.get().unwrap() as u8)); 

        self.enabled.set(true);
    }

    pub fn disable(&self) {
        if !self.enabled.get() {
            return;
        }
        self.enabled.set(false);

        //Stop DMA
        let registers: &EDMABaseRegisters = &*self.registers;
        registers.cerq.write(ChannelSet::EN.val(self.channel.get()));

        //Disable DMAMUX
        let dmamux_registers: &DMAMUXRegisters = &*self.dmamux_registers;
        dmamux_registers.chcfg.write(ChannelConfiguration::ENBL::CLEAR);

        unsafe {
        CHANNELS_ENABLED = CHANNELS_ENABLED - 1;
        if CHANNELS_ENABLED == 0 {
            //TODO rewrite sim to implement disable
            //use sim::{clocks, Clock};
            //clocks::DMA.disable();
            //clocks::DMAMUX.disable();
        }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }

    pub fn prepare_transfer(&self, transfer_config: TransferConfig) { 
        let tcd_registers: &EDMATcdRegisters = &*self.tcd_registers;
        tcd_registers.saddr.write(
            SourceAddress::SADDR.val(transfer_config.saddr));
        tcd_registers.soff.write(
            SourceAddressOffset::SOFF.val(transfer_config.soff));
        tcd_registers.attr.write(
            TransferAttributes::SSIZE.val(transfer_config.ssize) +
            TransferAttributes::DSIZE.val(transfer_config.dsize));
        tcd_registers.mlo.write(
            MinorLoopOffset::NBYTES.val(transfer_config.nbytes));
        tcd_registers.slast.write(
            LastSourceAddressAdjustment::SLAST.val(transfer_config.slast));
        tcd_registers.daddr.write(
            DestinationAddress::DADDR.val(transfer_config.daddr));
        tcd_registers.doff.write(
            DestinationAddressOffset::DOFF.val(transfer_config.doff));
        tcd_registers.citer.write(
            CurrentMinorLoopLink::CITER.val(transfer_config.citer));
        tcd_registers.dlastsga.write(
            LastDestinationAddressAdjustment::DLASTSGA.val(transfer_config.dlastsga));
        tcd_registers.csr.write(ControlAndStatus::DREQ::SET + ControlAndStatus::INTMAJOR::SET);
        tcd_registers.biter.write(
            BeginningMinorLoopLink::BITER.val(transfer_config.biter));
    }

    pub fn start_transfer(&self) {
        let registers: &EDMABaseRegisters = &*self.registers;

        //Start DMA 
        registers.serq.write(ChannelSet::EN.val(self.channel.get()));
    }

    pub fn do_transfer(&self, transfer_config: TransferConfig) { 
        self.prepare_transfer(transfer_config);
        self.start_transfer();
    }

    pub fn abort_transfer(&self) {
        let tcd_registers: &EDMATcdRegisters = &*self.tcd_registers;
        tcd_registers.csr.modify(ControlAndStatus::DREQ::SET);
    }

    pub fn handle_interrupt(&mut self) {
        let registers: &EDMABaseRegisters = &*self.registers;
        registers.cint.write(ChannelSet::EN.val(self.channel.get()));

        self.client.map(|client| {
            client.transfer_done();
        });
    }
}