//! Implementation of the K66 ADCIFE.
//!
//! This is an implementation of the K66 analog to digital converter. 
//!
//! Samples can either be collected individually or continuously at a specified
//! frequency.
//!
//! - Author: Holly Chiang <hchiang1@stanford.edu>
//! - Updated: Feb 24, 2020

use clock;
use core::cell::Cell;
use core::{cmp, mem, slice};
use dma;
use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::math;
use kernel::common::regs::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ReturnCode;

/// Representation of an ADC channel on the SAM4L.
pub struct AdcChannel {
    adc_num: u8,
    chan_num: u32,
}

/// K66 ADC channels.
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum Channel0 {
    ADC0_DP3_DM3 = 0x03, //DP3 is A10, DM3 is A11
    ADC0_SE4b = 0x04, //A9
    ADC0_SE5b = 0x05, //A0
    ADC0_SE6b = 0x06, //A6
    ADC0_SE7b = 0x07, //A7
    ADC0_SE8 = 0x08, //A2
    ADC0_SE9 = 0x09, //A3
    ADC0_SE10 = 0x10, 
    ADC0_SE11 = 0x11,
    ADC0_SE12 = 0x12, //A5
    ADC0_SE13 = 0x13, //A4
    ADC0_SE14 = 0x14, //A1
    ADC0_SE15 = 0x15, //A8
    ADC0_SE16 = 0x16, 
    ADC0_SE17 = 0x17, //A14
    ADC0_SE18 = 0x18, //A15
    ADC0_DM0 = 0x19, 
    ADC0_SE21 = 0x21, 
    ADC0_SE22 = 0x22,
    ADC0_SE23 = 0x23, //A21
    Temperature = 0x26,
    Bandgap = 0x27,
    VREFH = 0x29, //AREFH
    VREFL = 0x30,
    Disabled = 0x31,
}

#[allow(non_camel_case_types)]
enum Channel1 {
    ADC1_DP0_DM0 = 0x00, //DP0 is A10, DM0 is A11
    ADC1_SE4b = 0x04, //A16
    ADC1_SE5b = 0x05, //A17
    ADC1_SE6b = 0x06, //A18
    ADC1_SE7b = 0x07, //A19
    ADC1_SE8 = 0x08, //A2
    ADC1_SE9 = 0x09, //A3
    ADC1_SE10 = 0x10, //A23
    ADC1_SE11 = 0x11, //A24
    ADC1_SE12 = 0x12, 
    ADC1_SE13 = 0x13,
    ADC1_SE14 = 0x14, //A12
    ADC1_SE15 = 0x15, //A13
    ADC1_SE16 = 0x16,
    ADC1_SE17 = 0x17, //A20
    ADC1_SE18 = 0x18,
    ADC1_DM0 = 0x19,
    ADC1_SE23 = 0x23, //A22
    Temperature = 0x26,
    Bandgap = 0x27,
    VREFH = 0x29, //AREFH
    VREFL = 0x30,
    Disabled = 0x31,
}

/// Initialization of an ADC channel.
impl AdcChannel {
    /// Create a new ADC channel.
    ///
    /// - `channel`: Channel enum representing the channel number and whether it
    ///   is internal
    const fn new(adc_num: u8, channel: u32) -> AdcChannel {
        AdcChannel {
            adc_num: adc_num,
            chan_num: channel,
        }
    }
}

/// Statically allocated ADC channels. Used in board configurations to specify
/// which channels are used on the platform.
pub static mut CHANNEL0_A0: AdcChannel = AdcChannel::new(0, Channel0::ADC0_SE5b as u32);
pub static mut CHANNEL0_A1: AdcChannel = AdcChannel::new(0, Channel0::ADC0_SE14 as u32);
pub static mut CHANNEL0_A2: AdcChannel = AdcChannel::new(0, Channel0::ADC0_SE8 as u32);
pub static mut CHANNEL1_A2: AdcChannel = AdcChannel::new(1, Channel1::ADC1_SE8 as u32);
pub static mut CHANNEL0_A3: AdcChannel = AdcChannel::new(0, Channel0::ADC0_SE9 as u32);
pub static mut CHANNEL1_A3: AdcChannel = AdcChannel::new(1, Channel1::ADC1_SE9 as u32);
pub static mut CHANNEL0_A4: AdcChannel = AdcChannel::new(0, Channel0::ADC0_SE13 as u32);
pub static mut CHANNEL0_A5: AdcChannel = AdcChannel::new(0, Channel0::ADC0_SE12 as u32);
pub static mut CHANNEL0_A6: AdcChannel = AdcChannel::new(0, Channel0::ADC0_SE6b as u32);
pub static mut CHANNEL0_A7: AdcChannel = AdcChannel::new(0, Channel0::ADC0_SE7b as u32);
pub static mut CHANNEL0_A8: AdcChannel = AdcChannel::new(0, Channel0::ADC0_SE15 as u32);
pub static mut CHANNEL0_A9: AdcChannel = AdcChannel::new(0, Channel0::ADC0_SE4b as u32);
pub static mut CHANNEL0_A10: AdcChannel = AdcChannel::new(0, Channel0::ADC0_DP3_DM3 as u32);
pub static mut CHANNEL1_A10: AdcChannel = AdcChannel::new(1, Channel1::ADC1_DP0_DM0 as u32);
pub static mut CHANNEL0_A11: AdcChannel = AdcChannel::new(0, Channel0::ADC0_DP3_DM3 as u32);
pub static mut CHANNEL1_A11: AdcChannel = AdcChannel::new(1, Channel1::ADC1_DP0_DM0 as u32);
pub static mut CHANNEL1_A12: AdcChannel = AdcChannel::new(1, Channel1::ADC1_SE14 as u32);
pub static mut CHANNEL1_A13: AdcChannel = AdcChannel::new(1, Channel1::ADC1_SE15 as u32);
pub static mut CHANNEL0_A14: AdcChannel = AdcChannel::new(0, Channel0::ADC0_SE17 as u32);
pub static mut CHANNEL0_A15: AdcChannel = AdcChannel::new(0, Channel0::ADC0_SE18 as u32);
pub static mut CHANNEL1_A16: AdcChannel = AdcChannel::new(1, Channel1::ADC1_SE4b as u32);
pub static mut CHANNEL1_A17: AdcChannel = AdcChannel::new(1, Channel1::ADC1_SE5b as u32);
pub static mut CHANNEL1_A18: AdcChannel = AdcChannel::new(1, Channel1::ADC1_SE6b as u32);
pub static mut CHANNEL1_A19: AdcChannel = AdcChannel::new(1, Channel1::ADC1_SE7b as u32);
pub static mut CHANNEL1_A20: AdcChannel = AdcChannel::new(1, Channel1::ADC1_SE17 as u32);
pub static mut CHANNEL0_A21: AdcChannel = AdcChannel::new(0, Channel0::ADC0_SE23 as u32);
pub static mut CHANNEL1_A22: AdcChannel = AdcChannel::new(1, Channel1::ADC1_SE23 as u32);
pub static mut CHANNEL1_A23: AdcChannel = AdcChannel::new(1, Channel1::ADC1_SE10 as u32);
pub static mut CHANNEL1_A24: AdcChannel = AdcChannel::new(1, Channel1::ADC1_SE11 as u32);
pub static mut CHANNEL0_VREFH: AdcChannel = AdcChannel::new(0, Channel0::VREFH as u32);
pub static mut CHANNEL1_VREFH: AdcChannel = AdcChannel::new(1, Channel1::VREFH as u32);

/// Create a trait of both client types to allow a single client reference to
/// act as both
pub trait EverythingClient: hil::adc::Client + hil::adc::HighSpeedClient {}
impl<C: hil::adc::Client + hil::adc::HighSpeedClient> EverythingClient for C {}

/// ADC driver code for the SAM4L.
pub struct Adc {
    registers: StaticRef<AdcRegisters>,
    index: usize,

    // state tracking for the ADC
    adc_clk_freq: Cell<u32>,
    active: Cell<bool>,
    continuous: Cell<bool>,

    // DMA peripheral, buffers, and length
    rx_dma: OptionalCell<&'static dma::DMAChannel>,
    rx_dma_peripheral: dma::DMAPeripheral,
    rx_length: Cell<usize>,
    next_dma_buffer: TakeCell<'static, [u16]>,
    next_dma_length: Cell<usize>,
    stopped_buffer: TakeCell<'static, [u16]>,

    // ADC client to send sample complete notifications to
    client: OptionalCell<&'static EverythingClient>,
}

/// Memory mapped registers for the ADC.
#[repr(C)]
pub struct AdcRegisters {
    // From page 957 of K66 manual
    sc1a: ReadWrite<u32, Control::Register>,
    sc1b: ReadWrite<u32, Control::Register>,
    cfg1: ReadWrite<u32, Configuration1::Register>,
    cfg2: ReadWrite<u32, Configuration2::Register>,
    ra: ReadOnly<u32, DataResult::Register>,
    rb: ReadOnly<u32, DataResult::Register>,
    cv1: ReadWrite<u32, CompareValue::Register>,
    cv2: ReadWrite<u32, CompareValue::Register>,
    sc2: ReadWrite<u32, StatusControl2::Register>,
    sc3: ReadWrite<u32, StatusControl3::Register>,
    ofs: ReadWrite<u32, OffsetCorrection::Register>,
    pg: ReadWrite<u32, PlusSideGain::Register>,
    mg: ReadWrite<u32, MinusSideGain::Register>,
    clpd: ReadWrite<u32, CalibrationD::Register>,
    clps: ReadWrite<u32, CalibrationS::Register>,
    clp4: ReadWrite<u32, Calibration4::Register>,
    clp3: ReadWrite<u32, Calibration3::Register>,
    clp2: ReadWrite<u32, Calibration2::Register>,
    clp1: ReadWrite<u32, Calibration1::Register>,
    clp0: ReadWrite<u32, Calibration0::Register>,
    clmd: ReadWrite<u32, CalibrationD::Register>,
    clms: ReadWrite<u32, CalibrationS::Register>,
    clm4: ReadWrite<u32, Calibration4::Register>,
    clm3: ReadWrite<u32, Calibration3::Register>,
    clm2: ReadWrite<u32, Calibration2::Register>,
    clm1: ReadWrite<u32, Calibration1::Register>,
    clm0: ReadWrite<u32, Calibration0::Register>,
}

register_bitfields![u32,
    Control [
        /// Conversion Complete Flag
        COCO OFFSET(7) NUMBITS(1) [],
        /// Interrupt Enable
        AIEN OFFSET(6) NUMBITS(1) [],
        /// Differential Mode Enable
        DIFF OFFSET(5) NUMBITS(1) [],
        /// Input channel select
        ADCH OFFSET(0) NUMBITS(5) []
    ],

    Configuration1 [
        /// Low-Power Configuration
        ADLPC OFFSET(7) NUMBITS(1) [],
        /// Clock Divide Select
        ADIV OFFSET(5) NUMBITS(2) [
            Div1 = 0,
            Div2 = 1,
            Div4 = 2,
            Div8 = 3
        ],
        /// Sample Time Configuration
        ADLSMP OFFSET(4) NUMBITS(1) [
            Short = 0,
            Long = 1
        ],
        /// Conversion Mode Selection
        MODE OFFSET(2) NUMBITS(2) [
            Bit8or9 = 0,
            Bit12or13 = 1,
            Bit10or11 = 2, 
            Bit16 = 3 
        ],
        /// Input Clock Select
        ADICLK OFFSET(0) NUMBITS(2) [
            BUSCLK = 0,
            BUSCLKDIV2 = 1,
            ALTCLK = 2,
            ADACK = 3
        ]
    ],

    Configuration2 [
        ///ADC Mux Select
        MUXSEL OFFSET(4) NUMBITS(1) [
            ChannelA = 0,
            ChannelB = 1
        ],
        /// Asynchronous Clock Output Enable
        ADACKEN OFFSET(3) NUMBITS(1) [],
        /// High-Speed Configuration
        ADHSC OFFSET(2) NUMBITS(1) [
            Normal = 0,
            HighSpeed = 1
        ],
        /// Long Sample Time Select
        ADLSTS OFFSET(0) NUMBITS(2) [
            Cycles24 = 0,
            Cycles16 = 1,
            Cycles10 = 2,
            Cycles6 = 3
        ]
    ],

    DataResult [
        D OFFSET(0) NUMBITS(16) []
    ],

    CompareValue [
        CV OFFSET(0) NUMBITS(16) []
    ],

    StatusControl2 [
        /// Conversion Active
        ADACT OFFSET(7) NUMBITS(1) [],
        /// Conversion Trigger Select
        ADTRG OFFSET(6) NUMBITS(1) [
            Software = 0,
            Hardware = 1
        ],
        /// Compare Function Enable
        ACFE OFFSET(5) NUMBITS(1) [],
        /// Compare Function Greater Than Enable
        ACFGT OFFSET(4) NUMBITS(1) [
            LessThan = 0,
            GreaterThanEqual = 1
        ],      
        /// Compare Function Range Enable
        ACREN OFFSET(3) NUMBITS(1) [],
        /// DMA Enable
        DMAEN OFFSET(2) NUMBITS(1) [],
        /// Voltage Reference Selection
        REFSEL OFFSET(0) NUMBITS(2) [
            DefaultRef = 0,
            Alternate = 1
        ]
    ],

    StatusControl3 [
        /// Calibration
        CAL OFFSET(7) NUMBITS(1) [],
        /// Calibration Failed Flag
        CALF OFFSET(6) NUMBITS(1) [],
        ///Continuous Conversion Enable
        ADCO OFFSET(3) NUMBITS(1) [
            One = 0,
            Continuous = 1
        ],
        /// Hardware Average Enable
        AVGE OFFSET(2) NUMBITS(1) [],
        /// Hardware Average Select
        AVGS OFFSET(0) NUMBITS(2) [
            Avg4 = 0,
            Avg8 = 1,
            Avg16 = 2,
            Avg32 = 3
        ]
    ],

    OffsetCorrection [
        OFS OFFSET(0) NUMBITS(16) []
    ],

    PlusSideGain [
        PG OFFSET(0) NUMBITS(16) []
    ],

    MinusSideGain [
        MG OFFSET(0) NUMBITS(16) []
    ],
    
    CalibrationD [
        CLD OFFSET(0) NUMBITS(6) []
    ],
    
    CalibrationS [
        CLS OFFSET(0) NUMBITS(6) []
    ],
    
    Calibration4 [
        CL4 OFFSET(0) NUMBITS(10) []
    ],
    
    Calibration3 [
        CL3 OFFSET(0) NUMBITS(9) []
    ],
    
    Calibration2 [
        CL2 OFFSET(0) NUMBITS(8) []
    ],
    
    Calibration1 [
        CL1 OFFSET(0) NUMBITS(7) []
    ],
    
    Calibration0 [
        CL0 OFFSET(0) NUMBITS(6) []
    ]
];

// Page 957 of K66 data sheet
pub const ADC_ADDRS: [StaticRef<AdcRegisters>; 2] = [
    unsafe { StaticRef::new(0x4003_B000 as *const AdcRegisters)},
    unsafe { StaticRef::new(0x400B_B000 as *const AdcRegisters)}];

/// Statically allocated ADC driver. Used in board configurations to connect to
/// various capsules.
pub static mut ADC0: Adc = Adc::new(0, dma::DMAPeripheral::ADC0);
pub static mut ADC1: Adc = Adc::new(1, dma::DMAPeripheral::ADC1);

/// Functions for initializing the ADC.
impl Adc {
    /// Create a new ADC driver.
    ///
    /// - `index`: which ADC
    /// - `rx_dma_peripheral`: type used for DMA transactions
    const fn new(
        index: usize,
        rx_dma_peripheral: dma::DMAPeripheral,
    ) -> Adc {
        Adc {
            // pointer to memory mapped I/O registers
            registers: ADC_ADDRS[index],
            index: index,

            // status of the ADC peripheral
            adc_clk_freq: Cell::new(0),
            active: Cell::new(false),
            continuous: Cell::new(false),

            // DMA status and stuff
            rx_dma: OptionalCell::empty(),
            rx_dma_peripheral: rx_dma_peripheral,
            rx_length: Cell::new(0),
            next_dma_buffer: TakeCell::empty(),
            next_dma_length: Cell::new(0),
            stopped_buffer: TakeCell::empty(),

            // higher layer to send responses to
            client: OptionalCell::empty(),
        }
    }

    /// Sets the client for this driver.
    ///
    /// - `client`: reference to capsule which handles responses
    pub fn set_client<C: EverythingClient>(&self, client: &'static C) {
        self.client.set(client);
    }

    /// Sets the DMA channel for this driver.
    ///
    /// - `rx_dma`: reference to the DMA channel the ADC should use
    pub fn set_dma(&self, rx_dma: &'static dma::DMAChannel) {
        self.rx_dma.set(rx_dma);
    }

    pub fn enable_clock(&self) {
        use sim::{clocks, Clock};
        match self.index {
            0 => clocks::ADC0.enable(),
            1 => clocks::ADC1.enable(),
            _ => unreachable!()
        };
    }

    /// Calibrate the adc
    /// clock and frequency, sample time, high speed configuration must be set before calibration
    pub fn calibrate(&self) -> ReturnCode {
        let regs: &AdcRegisters = &*self.registers;

        // select software trigger
        regs.sc2.write(StatusControl2::ADTRG::Software);

        // start calibration
        regs.sc3.modify(StatusControl3::CAL::SET + StatusControl3::CALF::SET);

        while !regs.sc1a.is_set(Control::COCO) {}

        if regs.sc3.is_set(StatusControl3::CALF) {
            return ReturnCode::FAIL;
        }

        // calibrate
        let mut var: u16 = 0;
        var += regs.clp0.read(Calibration0::CL0) as u16;
        var += regs.clp1.read(Calibration1::CL1) as u16;
        var += regs.clp2.read(Calibration2::CL2) as u16;
        var += regs.clp3.read(Calibration3::CL3) as u16;
        var += regs.clp4.read(Calibration4::CL4) as u16;
        var += regs.clps.read(CalibrationS::CLS) as u16;
        var = var >> 1;
        var |= 1 << 15;
        regs.pg.write(PlusSideGain::PG.val(var as u32));

        let mut var: u16 = 0;
        var += regs.clm0.read(Calibration0::CL0) as u16;
        var += regs.clm1.read(Calibration1::CL1) as u16;
        var += regs.clm2.read(Calibration2::CL2) as u16;
        var += regs.clm3.read(Calibration3::CL3) as u16;
        var += regs.clm4.read(Calibration4::CL4) as u16;
        var += regs.clms.read(CalibrationS::CLS) as u16;
        var = var >> 1;
        var |= 1 << 15;
        regs.mg.write(MinusSideGain::MG.val(var as u32));

        regs.sc3.write(StatusControl3::CAL::CLEAR);

        ReturnCode::SUCCESS
    }

    /// Setup the adc clock
    pub fn set_clock_divisor(&self, frequency: u32) -> ReturnCode {
        let regs: &AdcRegisters = &*self.registers;
        let periph_freq = clock::peripheral_clock_hz();
        // see pg. 988 of the datasheet for conversion time
        // (5 ADCK cycles + 5 bus clock cycles) + 1*(20 + 0 + 2 ADCK cycles)
        let clock_freq = frequency * 32;
        let divisor = (periph_freq + clock_freq -1)/clock_freq;
        let divisor_pow2 = math::closest_power_of_two(divisor);
        let clock_divisor = cmp::min(
            math::log_base_two(divisor_pow2).checked_sub(2).unwrap_or(0), 3);

        let new_adc_clk_freq = periph_freq/(1 << clock_divisor);
        if self.adc_clk_freq.get() == new_adc_clk_freq {
           return ReturnCode::SUCCESS;
        }
        self.adc_clk_freq.set(new_adc_clk_freq);

        regs.cfg1.modify(Configuration1::ADIV.val(clock_divisor));

        ReturnCode::SUCCESS
    }

    /// Interrupt handler for the ADC.
    pub fn handle_interrupt(&mut self) {
        let regs: &AdcRegisters = &*self.registers;
        let status = regs.sc1a.is_set(Control::COCO);

        if self.active.get() {
            if status {
                let val = regs.ra.read(DataResult::D) as u16;
                self.client.map(|client| {
                    client.sample_ready(val);
                });

                if !self.continuous.get() {
                    self.active.set(false);
                    regs.sc1a.modify(Control::AIEN::CLEAR);
                }
            }
        } else {
            // we are inactive, why did we get an interrupt?
            // disable all interrupts, clear status, and just ignore it
            regs.sc1a.modify(Control::AIEN::CLEAR);
        }
    }
}

/// Implements an ADC capable reading ADC samples on any channel.
impl hil::adc::Adc for Adc {
    type Channel = AdcChannel;

    /// Enable and configure the ADC.
    /// This can be called multiple times with no side effects.
    fn initialize(&self) -> ReturnCode {
        self.enable_clock();
        self.calibrate()
    }

    /// Capture a single analog sample, calling the client when complete.
    /// Returns an error if the ADC is already sampling.
    ///
    /// - `channel`: the ADC channel to sample
    fn sample(&self, channel: &Self::Channel) -> ReturnCode {
        let regs: &AdcRegisters = &*self.registers;

        if self.active.get() {
            // only one operation at a time
            ReturnCode::EBUSY
        } else {
            self.active.set(true);
            self.continuous.set(false);

            // divide clock by 1, select short sample time, select 12 bit conversion, select bus clock as input
            regs.cfg1.write(Configuration1::ADIV::Div1 + Configuration1::ADLSMP::Short + 
                            Configuration1::MODE::Bit12or13 + Configuration1::ADICLK::BUSCLK);

            // select ADC channel b
            regs.cfg2.write(Configuration2::MUXSEL::ChannelB);

            let res = self.calibrate();
            if res != ReturnCode::SUCCESS {
                return res;
            }

            // enable end of conversion interrupt and select input channel
            // since software trigger selected, conversion starts following write to sc1a
            regs.sc1a.write(Control::AIEN::SET + Control::ADCH.val(channel.chan_num));

            ReturnCode::SUCCESS
        }
    }

    /// Request repeated analog samples on a particular channel, calling after
    /// each sample. Continuous mode is limited to 4K(?) samples per second.
    /// To sample faster, use the sample_highspeed function.
    ///
    /// - `channel`: the ADC channel to sample
    /// - `frequency`: the number of samples per second to collect
    fn sample_continuous(&self, channel: &Self::Channel, frequency: u32) -> ReturnCode {
        let regs: &AdcRegisters = &*self.registers;

        if self.active.get() {
            // only one operation at a time
            ReturnCode::EBUSY
        } else if frequency == 0 || frequency > 4000 {
            ReturnCode::EINVAL
        } else {
            self.active.set(true);
            self.continuous.set(true);

            self.set_clock_divisor(frequency);

            // select short sample time, select 12 bit conversion, select bus clock as input
            regs.cfg1.modify(Configuration1::ADLSMP::Short + 
                            Configuration1::MODE::Bit12or13 + Configuration1::ADICLK::BUSCLK);

            // select ADC channel b
            regs.cfg2.write(Configuration2::MUXSEL::ChannelB + Configuration2::ADHSC::HighSpeed);

            let res = self.calibrate();
            if res != ReturnCode::SUCCESS {
                return res;
            }

            //setup sc3 for continuous sample here
            regs.sc3.modify(StatusControl3::ADCO::Continuous);

            // enable end of conversion interrupt and select input channel
            // since software trigger selected, conversion starts following write to sc1a
            regs.sc1a.write(Control::AIEN::SET + Control::ADCH.val(channel.chan_num));

            ReturnCode::SUCCESS
        }
    }

    /// Stop continuously sampling the ADC.
    /// This is expected to be called to stop continuous sampling operations,
    /// but can be called to abort any currently running operation.
    fn stop_sampling(&self) -> ReturnCode {
        if !self.active.get() {
            // cannot cancel sampling that isn't running
            ReturnCode::EINVAL
        } else {
            // clean up state
            self.active.set(false);
            self.continuous.set(false);
        
            //Writing to any register besides sc1n aborts conversion
            let regs: &AdcRegisters = &*self.registers;
            regs.sc3.modify(StatusControl3::ADCO::One);
            regs.sc2.modify(StatusControl2::DMAEN::CLEAR);
            regs.sc1a.modify(Control::AIEN::CLEAR);

            // stop DMA transfer if going. This should safely return a None if
            // the DMA was not being used
            let dma_buffer = self.rx_dma.map_or(None, |rx_dma| {
                let dma_buf = rx_dma.abort_transfer();
                rx_dma.disable();
                dma_buf
            });
            self.rx_length.set(0);

            // store the buffer if it exists
            dma_buffer.map(|dma_buf| {
                // change buffer back into a [u16]
                // the buffer was originally a [u16] so this should be okay
                let buf_ptr = unsafe { mem::transmute::<*mut u8, *mut u16>(dma_buf.as_mut_ptr()) };
                let buf = unsafe { slice::from_raw_parts_mut(buf_ptr, dma_buf.len() / 2) };

                // we'll place it here so we can return it to the higher level
                // later in a `retrieve_buffers` call
                self.stopped_buffer.replace(buf);
            });

            ReturnCode::SUCCESS
        }
    }
}

/// Implements an ADC capable of continuous sampling
impl hil::adc::AdcHighSpeed for Adc {
    /// Capture buffered samples from the ADC continuously at a given
    /// frequency, calling the client whenever a buffer fills up. The client is
    /// then expected to either stop sampling or provide an additional buffer
    /// to sample into. 
    ///
    /// - `channel`: the ADC channel to sample
    /// - `frequency`: frequency to sample at
    /// - `buffer1`: first buffer to fill with samples
    /// - `length1`: number of samples to collect (up to buffer length)
    /// - `buffer2`: second buffer to fill once the first is full
    /// - `length2`: number of samples to collect (up to buffer length)
    fn sample_highspeed(
        &self,
        channel: &Self::Channel,
        frequency: u32,
        buffer1: &'static mut [u16],
        length1: usize,
        buffer2: &'static mut [u16],
        length2: usize,
    ) -> (
        ReturnCode,
        Option<&'static mut [u16]>,
        Option<&'static mut [u16]>,
    ) {
        let regs: &AdcRegisters = &*self.registers;

        if self.active.get() {
            // only one operation at a time
            (ReturnCode::EBUSY, Some(buffer1), Some(buffer2))
        } else if frequency == 0 || frequency > 500000 { //is 500kHz the max?
            (ReturnCode::EINVAL, Some(buffer1), Some(buffer2))
        } else if length1 == 0 {
            (ReturnCode::EINVAL, Some(buffer1), Some(buffer2))
        } else {
            self.active.set(true);
            self.continuous.set(true);

            self.set_clock_divisor(frequency);

            // select short sample time, select 12 bit conversion, select bus clock as input
            regs.cfg1.modify(Configuration1::ADLSMP::Short + 
                            Configuration1::MODE::Bit12or13 + Configuration1::ADICLK::BUSCLK);

            // select ADC channel b
            regs.cfg2.write(Configuration2::MUXSEL::ChannelB + Configuration2::ADHSC::HighSpeed);

            let res = self.calibrate();
            if res != ReturnCode::SUCCESS {
                return (res, Some(buffer1), Some(buffer2));
            }

            // setup sc3 for continuous sample here
            regs.sc3.modify(StatusControl3::ADCO::Continuous);

            // store the second buffer for later use 
            self.next_dma_buffer.replace(buffer2);
            self.next_dma_length.set(length2);

            let dma_len = cmp::min(buffer1.len(), length1);

            // change buffer into a [u8]
            // this is unsafe but acceptable for the following reasons
            //  * the buffer is aligned based on 16-bit boundary, so the 8-bit
            //    alignment is fine
            //  * the DMA is doing checking based on our expected data width to
            //    make sure we don't go past dma_buf.len()/width
            //  * we will transmute the array back to a [u16] after the DMA
            //    transfer is complete
            let dma_buf_ptr = unsafe { mem::transmute::<*mut u16, *mut u8>(buffer1.as_mut_ptr()) };
            let dma_buf = unsafe { slice::from_raw_parts_mut(dma_buf_ptr, buffer1.len() * 2) };

            regs.sc2.modify(StatusControl2::DMAEN::SET);
            self.rx_dma.map(move |dma| {
                dma.enable();
                self.rx_length.set(dma_len);
                let config = dma::TransferConfig::new(
                    0x4003B010, (&buffer1[0] as *const _) as u32, 2, dma_len as u16);
                dma.do_transfer(config, dma_buf);
            });

            // enable end of conversion interrupt and select input channel
            // since software trigger selected, conversion starts following write to sc1a
            regs.sc1a.write(Control::ADCH.val(channel.chan_num));

            (ReturnCode::SUCCESS, None, None)
        }
    }

    /// Provide a new buffer to send on-going buffered continuous samples to.
    /// This is expected to be called after the `samples_ready` callback.
    ///
    /// - `buf`: buffer to fill with samples
    /// - `length`: number of samples to collect (up to buffer length)
    fn provide_buffer(
        &self,
        buf: &'static mut [u16],
        length: usize,
    ) -> (ReturnCode, Option<&'static mut [u16]>) {
        if !self.active.get() {
            // cannot continue sampling that isn't running
            (ReturnCode::EINVAL, Some(buf))
        } else if !self.continuous.get() {
            // cannot continue a single sample operation
            (ReturnCode::EINVAL, Some(buf))
        } else if self.next_dma_buffer.is_some() {
            // we've already got a second buffer, we don't need a third yet
            (ReturnCode::EBUSY, Some(buf))
        } else {
            // store the buffer for later use
            self.next_dma_buffer.replace(buf);
            self.next_dma_length.set(length);

            (ReturnCode::SUCCESS, None)
        }
    }

    /// Reclaim buffers after the ADC is stopped.
    /// This is expected to be called after `stop_sampling`.
    fn retrieve_buffers(
        &self,
    ) -> (
        ReturnCode,
        Option<&'static mut [u16]>,
        Option<&'static mut [u16]>,
    ) {
        if self.active.get() {
            // cannot return buffers while running
            (ReturnCode::EINVAL, None, None)
        } else {
            // we're not running, so give back whatever we've got
            (
                ReturnCode::SUCCESS,
                self.next_dma_buffer.take(),
                self.stopped_buffer.take(),
            )
        }
    }
}

/// Implements a client of a DMA.
impl dma::DMAClient for Adc {
    /// Handler for DMA transfer completion.
    ///
    /// - `pid`: the DMA peripheral that is complete
    fn transfer_done(&self) {
        let regs: &AdcRegisters = &*self.registers;
        let status = regs.sc1a.is_set(Control::COCO);
        if status {
            // get buffer filled with samples from DMA
            let dma_buffer = self.rx_dma.map_or(None, |rx_dma| {
                let dma_buf = rx_dma.abort_transfer();
                rx_dma.disable();
                dma_buf
            });

            // get length of received buffer
            let length = self.rx_length.get();

            // start a new transfer with the next buffer
            // we need to do this quickly in order to keep from missing samples.
            self.next_dma_buffer.take().map(|buf| {
                // first determine the buffer's length in samples
                let dma_len = cmp::min(buf.len(), self.next_dma_length.get());

                // only continue with a nonzero length. If we were given a
                // zero-length buffer or length field, assume that the user knew
                // what was going on, and just don't use the buffer
                if dma_len > 0 {
                    // change buffer into a [u8]
                    // this is unsafe but acceptable for the following reasons
                    //  * the buffer is aligned based on 16-bit boundary, so the
                    //    8-bit alignment is fine
                    //  * the DMA is doing checking based on our expected data
                    //    width to make sure we don't go past
                    //    dma_buf.len()/width
                    //  * we will transmute the array back to a [u16] after the
                    //    DMA transfer is complete
                    let dma_buf_ptr =
                        unsafe { mem::transmute::<*mut u16, *mut u8>(buf.as_mut_ptr()) };
                    let dma_buf = unsafe { slice::from_raw_parts_mut(dma_buf_ptr, buf.len() * 2) };

                    // set up the DMA
                    self.rx_dma.map(move |dma| {
                        dma.enable();
                        self.rx_length.set(dma_len);
                        let config = dma::TransferConfig::new(
                            0x4003B010, (&buf[0] as *const _) as u32, 2, dma_len as u16);
                        dma.do_transfer(config, dma_buf);
                    });
                } else {
                    // if length was zero, just keep the buffer in the takecell
                    // so we can return it when `stop_sampling` is called
                    self.next_dma_buffer.replace(buf);
                }
            });

            // alert client
            self.client.map(|client| {
                dma_buffer.map(|dma_buf| {
                    // change buffer back into a [u16]
                    // the buffer was originally a [u16] so this should be okay
                    let buf_ptr =
                        unsafe { mem::transmute::<*mut u8, *mut u16>(dma_buf.as_mut_ptr()) };
                    let buf = unsafe { slice::from_raw_parts_mut(buf_ptr, dma_buf.len() / 2) };

                    // pass the buffer up to the next layer. It will then either
                    // send down another buffer to continue sampling, or stop
                    // sampling
                    client.samples_ready(buf, length);
                });
            });
        } 
    }
}
