//! Implementation of the flash controller.
//! Currently only setup for access to FlexNVM
//!
//! - Author:  Holly Chiang <hchiang1@stanford.edu>
//! - Date: June 18, 2020 

use core::cell::Cell;
use core::ops::{Index, IndexMut};
use deferred_call_tasks::Task;
use kernel::common::cells::{TakeCell, OptionalCell};
use kernel::common::deferred_call::DeferredCall;
use kernel::common::regs::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::hil::clock_pm::{ClockClient, ClockManager, ClientIndex};
use kernel::ReturnCode;
use sim;

/// FMC registers. Section 31.5 of the datasheet
#[repr(C)]
struct FMCRegisters {
    pfapr: ReadWrite<u32>,
    pfb01cr: ReadWrite<u32, FlashBank01Control::Register>,
    pfb23cr: ReadWrite<u32, FlashBank23Control::Register>,
}

register_bitfields![u32,
    FlashBank01Control [
        BORWSC OFFSET(28) NUMBITS(4) [],
        CLCK_WAY OFFSET(24) NUMBITS(4) [],
        CINV_WAY OFFSET(20) NUMBITS(4) [],
        S_B_INV OFFSET(19) NUMBITS(1) [],
        B0MW OFFSET(17) NUMBITS(2) [],
        CRC OFFSET(5) NUMBITS(3) [],
        B0DCE OFFSET(4) NUMBITS(1) [],
        B0ICE OFFSET(3) NUMBITS(1) [],
        B0DPE OFFSET(2) NUMBITS(1) [],
        B0IPE OFFSET(1) NUMBITS(1) []
    ],
    FlashBank23Control [
        B1RWSC OFFSET(28) NUMBITS(4) [],
        B1MW OFFSET(17) NUMBITS(2) [],
        B1DCE OFFSET(4) NUMBITS(1) [],
        B1ICE OFFSET(3) NUMBITS(1) [],
        B1DPE OFFSET(2) NUMBITS(1) [],
        B1IPE OFFSET(1) NUMBITS(1) []
    ]
];

/// Flash registers. Section 32.3.4 of the datasheet
#[repr(C)]
struct FlashRegisters {
    fstat: ReadWrite<u8, FlashStatus::Register>,
    fcnfg: ReadWrite<u8, FlashConfiguration::Register>,
    fsec: ReadOnly<u8, FlashSecurity::Register>,
    fopt: ReadOnly<u8>,
    fccob3: ReadWrite<u8, FlashCommonCommandObject::Register>,
    fccob2: ReadWrite<u8, FlashCommonCommandObject::Register>,
    fccob1: ReadWrite<u8, FlashCommonCommandObject::Register>,
    fccob0: ReadWrite<u8, FlashCommonCommandObject::Register>,
    fccob7: ReadWrite<u8, FlashCommonCommandObject::Register>,
    fccob6: ReadWrite<u8, FlashCommonCommandObject::Register>,
    fccob5: ReadWrite<u8, FlashCommonCommandObject::Register>,
    fccob4: ReadWrite<u8, FlashCommonCommandObject::Register>,
    fccobb: ReadWrite<u8, FlashCommonCommandObject::Register>,
    fccoba: ReadWrite<u8, FlashCommonCommandObject::Register>,
    fccob9: ReadWrite<u8, FlashCommonCommandObject::Register>,
    fccob8: ReadWrite<u8, FlashCommonCommandObject::Register>,
    fprot3: ReadWrite<u8, ProgramFlashProtection::Register>,
    fprot2: ReadWrite<u8, ProgramFlashProtection::Register>,
    fprot1: ReadWrite<u8, ProgramFlashProtection::Register>,
    fprot0: ReadWrite<u8, ProgramFlashProtection::Register>,
    _reserved0: [u8; 2],
    feprot: ReadWrite<u8, EEPROMProtection::Register>,
    fdprot: ReadWrite<u8, DataFlashProtection::Register>,
    xacch3: ReadOnly<u8, ExecuteOnlyAccess::Register>,
    xacch2: ReadOnly<u8, ExecuteOnlyAccess::Register>,
    xacch1: ReadOnly<u8, ExecuteOnlyAccess::Register>,
    xacch0: ReadOnly<u8, ExecuteOnlyAccess::Register>,
    xaccl3: ReadOnly<u8, ExecuteOnlyAccess::Register>,
    xaccl2: ReadOnly<u8, ExecuteOnlyAccess::Register>,
    xaccl1: ReadOnly<u8, ExecuteOnlyAccess::Register>,
    xaccl0: ReadOnly<u8, ExecuteOnlyAccess::Register>,
    sacch3: ReadOnly<u8, SupervisorOnlyAccess::Register>,
    sacch2: ReadOnly<u8, SupervisorOnlyAccess::Register>,
    sacch1: ReadOnly<u8, SupervisorOnlyAccess::Register>,
    sacch0: ReadOnly<u8, SupervisorOnlyAccess::Register>,
    saccl3: ReadOnly<u8, SupervisorOnlyAccess::Register>,
    saccl2: ReadOnly<u8, SupervisorOnlyAccess::Register>,
    saccl1: ReadOnly<u8, SupervisorOnlyAccess::Register>,
    saccl0: ReadOnly<u8, SupervisorOnlyAccess::Register>,
    facss: ReadOnly<u8, FlashAccessSegmentSize::Register>,
    _reserved1: [u8; 2],
    facsn: ReadOnly<u8, FlashAccessSegmentNumber::Register>,
}

register_bitfields![u8,
    FlashStatus [
        CCIF 7,
        RDCOLERR 6,
        ACCERR 5,
        FPVIOL 4,
        MGSTAT0 0
    ],
    FlashConfiguration [
        CCIE 7,
        RDCOLLIE 6,
        ERSAREQ 5,
        ERSSUSP 4,
        SWAP 3,
        PFLSH 2,
        RAMRDY 1,
        EEERDY 0
    ],
    FlashSecurity [
        KEYEN OFFSET(6) NUMBITS(2) [],
        MEEN OFFSET(4) NUMBITS(2) [],
        FSLACC OFFSET(2) NUMBITS(2) [],
        SEC OFFSET(0) NUMBITS(2) []
    ],
    FlashCommonCommandObject [
        CCOB OFFSET(0) NUMBITS(8) []
    ],
    ProgramFlashProtection [
        PROT OFFSET(0) NUMBITS(8) []
    ],
    EEPROMProtection [
        EPROT OFFSET(0) NUMBITS(8) []
    ],
    DataFlashProtection [
        DPROT OFFSET(0) NUMBITS(8) []
    ], 
    ExecuteOnlyAccess [
        XA OFFSET(0) NUMBITS(8) []
    ],
    SupervisorOnlyAccess [
        SA OFFSET(0) NUMBITS(8) []
    ],
    FlashAccessSegmentSize [
        SGSIZE OFFSET(0) NUMBITS(8) []
    ],
    FlashAccessSegmentNumber [
        NUMSG OFFSET(0) NUMBITS(8) []
    ]
];

/// Flash commands from 32.4.12
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum FlashCMD {
    ReadOnesBlock = 0x0,            // Check if the block has been erased
    ReadOnesSection = 0x1,          // Check if the section has been erased
    ProgramCheck = 0x2,             // Check a programed longword to see if it reads correctly
    ReadResource = 0x3,             // Read data from special registers such as IFR, Version ID
    ProgramPhrase = 0x7,            // Programs 8 previously erased bytes
    EraseFlashBlock = 0x8,          // Erases all addresses in a flash block
    EraseFlashSector = 0x9,         // Erases all addresses in a flash sector
    ProgramSection = 0xb,           // Programs a previously erased location from data loaded into RAM (up to 1KB)
    ReadOnesAllBlocks = 0x40,       // Checks if all blocks have been erased
    ReadOnce = 0x41,                // Reads program flash 0 IFR
    ProgramOnce = 0x43,             // Programs program flash 0 IFR
    EraseAllBlocks = 0x44,          // Erases all flash blocks
    VerifyBackdoorAccessKey = 0x45, // Releases security if key in FCCOB matches key in FCF
    SwapControl = 0x46,             // Handles activities related to swaping two halves of program flash memory
    ProgramPartition = 0x80,        // Prepares FlexNVM block for use as data flash, EEPROM backup, or a combo and initializes FlexRAM
    SetFlexRAMFunction = 0x81       // Changes function of FlexRAM
}

/// FlashState is used to track the current state and command of the flash.
#[derive(Clone, Copy, PartialEq)]
enum FlashState {
    Unconfigured,                   // Flash is unconfigured, call configure().
    Ready,                          // Flash is ready to complete a command.
    Read,                           // Performing a read operation.
    WriteSetRam { addr: usize },    // Make sure FlexRAM is available as RAM.
    WriteErasing { addr: usize },   // Waiting on the page to erase.
    WriteWriting { addr: usize , offset: usize }, // Waiting on the page to actually be written.
    EraseErasing,                   // Waiting on the erase to finish.
}

static DEFERRED_CALL: DeferredCall<Task> = unsafe {DeferredCall::new(Task::Flashcalw) };

const SECTOR_SIZE: usize = 4 * 1024;
const FLEXNVM_ADDR: usize = 0x1000_0000;
const FLEXNVM_SIZE: usize = 256 * 1024;
const FLEXRAM_ADDR: usize = 0x1400_0000;
/// From 32.4.12.8 only the lower quarter of the RAM can be used as a 
/// section program buffer
const PROGRAM_BUFFER_SIZE: usize = 1024;
/// From 32.4.12, address bit 23 is used to select between program flash
/// memory (=0) and data flash memory (=1)
const SELECT_DATA_FLASH: usize = 0x800000;

/// This is a wrapper around a u8 array that is sized to a single page for the
/// K66. Users of this module must pass an object of this type to use the
/// `hil::flash::Flash` interface.
///
/// An example looks like:
///
/// ```
/// static mut PAGEBUFFER: K66Sector = K66Sector::new();
/// ```
pub struct K66Sector(pub [u8; SECTOR_SIZE as usize]);

impl K66Sector {
    pub const fn new() -> K66Sector {
        K66Sector([0; SECTOR_SIZE as usize])
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<usize> for K66Sector {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl IndexMut<usize> for K66Sector {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl AsMut<[u8]> for K66Sector {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

// The FLASH memory module
pub struct FTFE {
    fmc_registers: StaticRef<FMCRegisters>,
    registers: StaticRef<FlashRegisters>,
    client: Cell<Option<&'static hil::flash::Client<FTFE>>>,
    current_state: Cell<FlashState>,
    buffer: TakeCell<'static, K66Sector>,

    clock_manager: OptionalCell<&'static ClockManager>,
    client_index: OptionalCell<&'static ClientIndex>,
}

const FMC_REGISTER_ADDRESS: StaticRef<FMCRegisters> =
    unsafe { StaticRef::new(0x4001f000 as *const FMCRegisters) };
const FLASH_REGISTER_ADDRESS: StaticRef<FlashRegisters> =
    unsafe { StaticRef::new(0x40020000 as *const FlashRegisters) };
// static instance for the board. Only one FTFE on chip.
pub static mut FLASH_CONTROLLER: FTFE = FTFE::new(FMC_REGISTER_ADDRESS,
                                                  FLASH_REGISTER_ADDRESS);

impl FTFE {
    const fn new(
        fmc_registers: StaticRef<FMCRegisters>,
        registers: StaticRef<FlashRegisters>,
    ) -> FTFE {
        FTFE {
            fmc_registers: fmc_registers,
            registers: registers,
            client: Cell::new(None),
            current_state: Cell::new(FlashState::Unconfigured),
            buffer: TakeCell::empty(),
            clock_manager: OptionalCell::empty(),
            client_index: OptionalCell::empty(),
        }
    }
    pub fn handle_interrupt(&self) {
        let regs: &FlashRegisters = &*self.registers;
        // Disable interrupt.
        regs.fcnfg.modify(FlashConfiguration::CCIE::CLEAR);

        // Check for errors and report to Client if there are any
        if self.is_error() {
            let attempted_operation = self.current_state.get();

            // Reset state now that we are ready to do a new operation.
            self.current_state.set(FlashState::Ready);

            self.client.get().map(|client| match attempted_operation {
                FlashState::Read => {
                    self.buffer.take().map(|buffer| {
                        client.read_complete(buffer, hil::flash::Error::FlashError);
                    });
                }
                FlashState::WriteSetRam { .. } | FlashState::WriteErasing { .. }
                | FlashState::WriteWriting { .. } => {
                    self.buffer.take().map(|buffer| {
                        client.write_complete(buffer, hil::flash::Error::FlashError);
                    });
                }
                FlashState::EraseErasing => {
                    client.erase_complete(hil::flash::Error::FlashError);
                }
                _ => {}
            });
        }
        // Part of a command succeeded -- continue onto next steps.
        match self.current_state.get() {
            FlashState::Read => {
                self.current_state.set(FlashState::Ready);

                self.client_index.map( |client_index|
                    self.clock_manager.map( |clock_manager|
                        clock_manager.disable_clock(client_index)
                    )
                );

                self.client.get().map(|client| {
                    self.buffer.take().map(|buffer| {
                        client.read_complete(buffer, hil::flash::Error::CommandComplete);
                    });
                });
            }
            FlashState::WriteSetRam { addr } => {
                self.current_state.set(FlashState::WriteErasing{ addr: addr });
                self.issue_command(FlashCMD::EraseFlashSector, addr);

//TODO add another flash state for this?
                self.client_index.map( |client_index|
                    self.clock_manager.map( |clock_manager| {
                        clock_manager.set_min_frequency(client_index, 1000000);
                        clock_manager.enable_clock(client_index)
                    })
                );
            }
//TODO disable for write_to_program_buffer?
            FlashState::WriteErasing { addr } => {
                self.current_state.set(
                    FlashState::WriteWriting{ addr: addr, offset: PROGRAM_BUFFER_SIZE });
                self.write_to_program_buffer(0);
                self.issue_command(FlashCMD::ProgramSection, addr);
            }
            FlashState::WriteWriting { addr, offset } => {
                if offset >= SECTOR_SIZE {
                    self.current_state.set(FlashState::Ready);

                    //flush FlexNVM cache
                    //let fmc_regs: &FMCRegisters = &*self.fmc_registers;
                    //fmc_regs.pfb01cr.modify(FlashBank01Control::CINV_WAY.val(0xf) +
                    //    FlashBank01Control::S_B_INV::SET);
                    //fmc_regs.pfb23cr.modify(FlashBank23Control::B1DCE::CLEAR + 
                    //                        FlashBank23Control::B1DPE::CLEAR); 

                    self.client_index.map( |client_index|
                        self.clock_manager.map( |clock_manager|
                            clock_manager.disable_clock(client_index)
                        )
                    );

                    self.client.get().map(|client| {
                        self.buffer.take().map(|buffer| {
                            client.write_complete(buffer, hil::flash::Error::CommandComplete);
                        });
                    });
                } else {
                    self.current_state.set(
                        FlashState::WriteWriting{ addr: addr,  
                            offset: offset + PROGRAM_BUFFER_SIZE });
                    self.write_to_program_buffer(offset);
                    self.issue_command(FlashCMD::ProgramSection, addr + offset);
                }
            }
            FlashState::EraseErasing => {
                self.current_state.set(FlashState::Ready);

                self.client_index.map( |client_index|
                    self.clock_manager.map( |clock_manager|
                        clock_manager.disable_clock(client_index) 
                    )
                );

                self.client.get().map(|client| {
                    client.erase_complete(hil::flash::Error::CommandComplete);
                });
            }
            _ => {
                self.current_state.set(FlashState::Ready);
            }
        }
    }

    /// Ftfe status
    fn is_error(&self) -> bool {
        let regs: &FlashRegisters = &*self.registers;
        regs.fstat.is_set(FlashStatus::RDCOLERR) | regs.fstat.is_set(FlashStatus::ACCERR)
            | regs.fstat.is_set(FlashStatus::FPVIOL)
    }

    /// Ftfe command control
    fn issue_command(&self, command: FlashCMD, argument: usize) {
        // This currently works for EraseFlashSector, ProgramSection, and SetFlexRamFunction
        let regs: &FlashRegisters = &*self.registers;

        if self.is_error() {
            if regs.fstat.is_set(FlashStatus::RDCOLERR) {
                regs.fstat.write(FlashStatus::RDCOLERR::SET);
            }
            if regs.fstat.is_set(FlashStatus::ACCERR) {
                regs.fstat.write(FlashStatus::ACCERR::SET);
            }
            if regs.fstat.is_set(FlashStatus::FPVIOL) {
                regs.fstat.write(FlashStatus::FPVIOL::SET);
            }
            regs.fstat.write(FlashStatus::CCIF::SET);
        }

        // Wait for commands to finish
        while !regs.fstat.is_set(FlashStatus::CCIF) {}
        
        // Enable interrupt.
        regs.fcnfg.modify(FlashConfiguration::CCIE::SET + FlashConfiguration::RDCOLLIE::SET);

        //// Setup the command registers
        regs.fccob0.write(FlashCommonCommandObject::CCOB.val(command as u8));
        if command == FlashCMD::SetFlexRAMFunction {
            regs.fccob1.write(FlashCommonCommandObject::CCOB.val((argument & 0xff) as u8));
        }
        if command == FlashCMD::EraseFlashSector || command == FlashCMD::ProgramSection {
            let address = argument | SELECT_DATA_FLASH;
            regs.fccob1.write(FlashCommonCommandObject::CCOB.val(((address >> 16) & 0xff) as u8));
            regs.fccob2.write(FlashCommonCommandObject::CCOB.val(((address >> 8) & 0xff) as u8));
            regs.fccob3.write(FlashCommonCommandObject::CCOB.val((address & 0xff) as u8));
        }
        if command == FlashCMD::ProgramSection {
            let num_double_phrases = PROGRAM_BUFFER_SIZE / 16; 
            regs.fccob4.write(
                FlashCommonCommandObject::CCOB.val(((num_double_phrases >> 8) & 0xff) as u8));
            regs.fccob5.write(FlashCommonCommandObject::CCOB.val((num_double_phrases & 0xff) as u8));
        }

        // launch the command
        regs.fstat.modify(FlashStatus::CCIF::SET);
    }

    fn write_to_program_buffer(&self, offset: usize) {
        let mut page_buffer: *mut u8 = FLEXRAM_ADDR as *mut u8;

        self.buffer.map(|buffer| {
            unsafe {
                use core::ptr;

                let mut start_buffer: *const u8 = &buffer[offset] as *const u8;
                let mut data_transfered: usize = 0;
                while data_transfered < PROGRAM_BUFFER_SIZE {
                    // real copy
                    ptr::copy(start_buffer, page_buffer, 8);
                    page_buffer = page_buffer.offset(8);
                    start_buffer = start_buffer.offset(8);
                    data_transfered += 8;
                }
            }
        });
    }

    pub fn configure(&mut self) {
        // Enable clock in case it's off.
        sim::enable_clock(sim::Clock::Clock6(sim::ClockGate6::FTF));

        self.current_state.set(FlashState::Ready);
    }

    // Address is some raw offset in FlexNVM that you want to read.
    fn read_range(
        &self,
        address: usize,
        size: usize,
        buffer: &'static mut K66Sector,
    ) -> ReturnCode {
        if self.current_state.get() == FlashState::Unconfigured {
            return ReturnCode::FAIL;
        }

        // Check that address makes sense and buffer has room.
        if address > FLEXNVM_ADDR + FLEXNVM_SIZE
            || address + size > FLEXNVM_ADDR + FLEXNVM_SIZE || address + size < size
            || buffer.len() < size
        {
            // invalid flash address
            return ReturnCode::EINVAL;
        }

        self.client_index.map( |client_index|
            self.clock_manager.map( |clock_manager| {
                clock_manager.set_min_frequency(client_index, 40000000);
                clock_manager.enable_clock(client_index);
            })
        );

        // Actually do a copy from flash into the buffer.
        let mut byte: *const u8 = address as *const u8;
        unsafe {
            for i in 0..size {
                buffer[i] = *byte;
                byte = byte.offset(1);
            }
        }

        self.current_state.set(FlashState::Read);
        // Hold on to the buffer for the callback.
        self.buffer.replace(buffer);

        // Since read() is synchronous, we need to schedule as if we had an 
        // interrupt so this function can return and then call the callback.
        DEFERRED_CALL.set();

        ReturnCode::SUCCESS
    }

    fn write_page(&self, addr: usize, data: &'static mut K66Sector) -> ReturnCode {
        match self.current_state.get() {
            FlashState::Unconfigured => return ReturnCode::FAIL,
            FlashState::Ready => {}
            _ => return ReturnCode::EBUSY,
        }

        self.buffer.replace(data);

        // Make sure FlexRAM is available as RAM 
        let regs: &FlashRegisters = &*self.registers;
        if !regs.fcnfg.is_set(FlashConfiguration::RAMRDY) {
            self.current_state.set(FlashState::WriteSetRam{ addr: addr});
            self.issue_command(FlashCMD::SetFlexRAMFunction, 0xFF);
        } else {
            self.current_state
                .set(FlashState::WriteErasing{ addr: addr });
            self.issue_command(FlashCMD::EraseFlashSector, addr);
        }
        ReturnCode::SUCCESS
    }

    fn erase_page(&self, addr: usize) -> ReturnCode {
        match self.current_state.get() {
            FlashState::Unconfigured => return ReturnCode::FAIL,
            FlashState::Ready => {}
            _ => return ReturnCode::EBUSY,
        }

        self.current_state.set(FlashState::EraseErasing);

//TODO add EraseClock state for this?
        self.issue_command(FlashCMD::EraseFlashSector, addr);

        self.client_index.map( |client_index|
            self.clock_manager.map( |clock_manager| {
                clock_manager.set_min_frequency(client_index, 1000000);
                clock_manager.enable_clock(client_index)
            })
        );

        ReturnCode::SUCCESS
    }
}

impl<C: hil::flash::Client<Self>> hil::flash::HasClient<'static, C> for FTFE {
    fn set_client(&self, client: &'static C) {
        self.client.set(Some(client));
    }
}

impl hil::flash::Flash for FTFE {
    type Page = K66Sector;

    fn read_page(&self, page_number: usize, buf: &'static mut Self::Page) -> ReturnCode {
        self.read_range(page_number * SECTOR_SIZE, buf.len(), buf)
    }

    fn write_page(&self, page_number: usize, buf: &'static mut Self::Page) -> ReturnCode {
        self.write_page(page_number * SECTOR_SIZE, buf)
    }

    fn erase_page(&self, page_number: usize) -> ReturnCode {
        self.erase_page(page_number * SECTOR_SIZE)
    }
}

impl ClockClient for FTFE {
    fn setup_client(&self, clock_manager: &'static ClockManager, client_index: &'static ClientIndex) {
        self.clock_manager.set(clock_manager);
        self.client_index.set(client_index);
        clock_manager.set_need_lock(client_index, false);
    }
    fn configure_clock(&self, _frequency: u32) {}
    fn clock_enabled(&self) {
        match self.current_state.get() {
            FlashState::WriteErasing{..} | FlashState::EraseErasing => self.handle_interrupt(),
            _ => {}
        }
    }
    fn clock_disabled(&self) {}
}

