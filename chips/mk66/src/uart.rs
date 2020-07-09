//! Implementation of the MK66 UART Peripheral

use core::cell::Cell;
use kernel::common::cells::TakeCell;
use kernel::hil;
use kernel::hil::uart;
use core::mem;
use nvic;
use regs::uart::*;
use mcg;
use sim;

#[derive(Copy, Clone, PartialEq)]
pub enum  UartState {
    Idle,
    Receiving,
    Transmitting,
}

pub struct Uart {
    index: usize,
    registers: *mut Registers,
    client: Cell<Option<&'static uart::Client>>,
    tx_buffer: TakeCell<'static, [u8]>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    rx_index: Cell<usize>,
    state: Cell<UartState>,
    baud_rate: Cell<u32>,
}

pub static mut UART0: Uart = Uart::new(0);
pub static mut UART1: Uart = Uart::new(1);
pub static mut UART2: Uart = Uart::new(2);
pub static mut UART3: Uart = Uart::new(3);
pub static mut UART4: Uart = Uart::new(4);

impl Uart {
    pub const fn new(index: usize) -> Uart {
        Uart {
            index: index,
            registers: UART_BASE_ADDRS[index],
            client: Cell::new(None),
            tx_buffer: TakeCell::empty(),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_index: Cell::new(0),
            state: Cell::new(UartState::Idle),
            baud_rate: Cell::new(0),
        }
    }

    pub fn handle_interrupt(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        // Read byte from data register; reading S1 and D clears interrupt
        if self.state.get() == UartState::Receiving && regs.s1.is_set(Status1::RDRF) {
            let datum: u8 = regs.d.get();

            // Put byte into buffer, trigger callback if buffer full
            let mut done = false;
            let mut index = self.rx_index.get();
            self.rx_buffer.map( |buf| {
                buf[index] = datum;
                index = index + 1;
                if index >= self.rx_len.get() {
                    done = true;
                }
                self.rx_index.set(index);
            });
            if done {
                self.state.set(UartState::Idle);
                self.disable_rx_interrupts();
                self.client.get().map(|client| {
                    match self.rx_buffer.take() {
                        Some(buf) => client.receive_complete(buf, index, uart::Error::CommandComplete),
                        None => ()
                    }
                });
                self.disable_clock();
            }
        }

        if self.state.get() == UartState::Transmitting && self.tx_ready() {
            self.state.set(UartState::Idle);
            self.disable_tx_interrupts();
            self.client.get().map(move |client|
                match self.tx_buffer.take() {
                    Some(buf) => client.transmit_complete(buf, uart::Error::CommandComplete),
                    None => ()
                }
            );
            self.disable_clock();
        }
    }

    pub fn handle_error(&self) {
        // TODO: implement
    }

    fn set_parity(&self, parity: hil::uart::Parity) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };

        let (pe, pt) = match parity {
            hil::uart::Parity::None => (Control1::PE::CLEAR, Control1::PT::Even),
            hil::uart::Parity::Even => (Control1::PE::SET, Control1::PT::Even),
            hil::uart::Parity::Odd => (Control1::PE::SET, Control1::PT::Odd)
        };

        // This basic procedure outlined in section 59.9.3.
        // Set control register 1, which configures the parity.
        regs.c1.write(pe + pt +
                      Control1::LOOPS::CLEAR +
                      Control1::UARTSWAI::CLEAR +
                      Control1::RSRC::CLEAR +
                      Control1::M::EightBit +
                      Control1::WAKE::Idle +
                      Control1::ILT::AfterStop);
    }

    fn set_stop_bits(&self, stop_bits: hil::uart::StopBits) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };

        let stop_bits = match stop_bits {
            hil::uart::StopBits::One => BaudRateHigh::SBNS::One,
            hil::uart::StopBits::Two => BaudRateHigh::SBNS::Two
        };

        regs.bdh.modify(stop_bits);
    }

    fn set_baud_rate(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };

        // Baud rate generation. Note that UART0 and UART1 are sourced from the core clock, not the
        // bus clock.
        let uart_clock = match self.index {
            0 | 1 => mcg::core_clock_hz(),
            _ => mcg::peripheral_clock_hz()
        };

        let baud_counter: u32 = (uart_clock >> 4) / self.baud_rate.get();

        // Set the baud rate.
        regs.c4.modify(Control4::BRFA.val(0));
        regs.bdh.modify(BaudRateHigh::SBR.val((baud_counter >> 8) as u8));
        regs.bdl.set(baud_counter as u8);
    }

    pub fn enable_rx(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.c1.modify(Control1::ILT::SET); // Idle after stop bit
        regs.c2.modify(Control2::RE::SET);  // Enable UART reception
    }

    pub fn enable_rx_interrupts(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.rwfifo.set(1);               // Issue interrupt on each byte
        regs.c5.modify(Control5::RDMAS::CLEAR); // Issue interrupt on RX data
        regs.c2.modify(Control2::RIE::SET);     // Enable interrupts
    }

    pub fn disable_rx_interrupts(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.c2.modify(Control2::RIE::CLEAR);    
    }

    pub fn enable_tx(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.c2.modify(Control2::TE::SET);
    }

    pub fn enable_tx_interrupts(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.c5.modify(Control5::TDMAS::CLEAR); // Issue interrupt on TX data
        regs.c2.modify(Control2::TIE::SET);
    }

    pub fn disable_tx_interrupts(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.c2.modify(Control2::TIE::CLEAR);
    }

    fn disable_clock(&self) {
        match self.index {
            0 => sim::disable_clock(sim::Clock::Clock4(sim::ClockGate4::UART0)),
            1 => sim::disable_clock(sim::Clock::Clock4(sim::ClockGate4::UART1)),
            2 => sim::disable_clock(sim::Clock::Clock4(sim::ClockGate4::UART2)),
            3 => sim::disable_clock(sim::Clock::Clock4(sim::ClockGate4::UART3)),
            4 => sim::disable_clock(sim::Clock::Clock1(sim::ClockGate1::UART4)),
            _ => unreachable!()
        };
    }

    fn enable_clock(&self) {
        match self.index {
            0 => sim::enable_clock(sim::Clock::Clock4(sim::ClockGate4::UART0)),
            1 => sim::enable_clock(sim::Clock::Clock4(sim::ClockGate4::UART1)),
            2 => sim::enable_clock(sim::Clock::Clock4(sim::ClockGate4::UART2)),
            3 => sim::enable_clock(sim::Clock::Clock4(sim::ClockGate4::UART3)),
            4 => sim::enable_clock(sim::Clock::Clock1(sim::ClockGate1::UART4)),
            _ => unreachable!()
        };
    }

    pub fn send_byte(&self, byte: u8) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };

        while !regs.s1.is_set(Status1::TRDE) {}
        regs.d.set(byte);
    }

    pub fn tx_ready(&self) -> bool {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.s1.is_set(Status1::TC)
    }
}


/// Implementation of kernel::hil::UART
impl hil::uart::UART for Uart {
    fn set_client(&self, client: &'static hil::uart::Client) {
        self.client.set(Some(client));
    }

    fn init(&self, params: uart::UARTParams) {
        self.enable_clock();
        match self.index {
            0 => unsafe {nvic::enable(nvic::NvicIdx::UART0)},
            1 => unsafe {nvic::enable(nvic::NvicIdx::UART1)},
            2 => unsafe {nvic::enable(nvic::NvicIdx::UART2)},
            3 => unsafe {nvic::enable(nvic::NvicIdx::UART3)},
            4 => unsafe {nvic::enable(nvic::NvicIdx::UART4)},
            _ => unreachable!()
        };

        self.set_parity(params.parity);
        self.set_stop_bits(params.stop_bits);
        self.baud_rate.set(params.baud_rate);

        self.disable_clock();
    }

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        self.state.set(UartState::Transmitting);
        self.enable_clock();
        self.set_baud_rate();
        self.enable_tx();
        self.enable_tx_interrupts();

        // This basic procedure outlined in section 59.9.3.
        for i in 0..tx_len {
            self.send_byte(tx_data[i]);
        }
        self.tx_buffer.put(Some(tx_data));
    }

    #[allow(unused_variables)]
    fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize) {
        self.state.set(UartState::Receiving);
        self.enable_clock();
        self.set_baud_rate();
        self.enable_rx();
        self.enable_rx_interrupts();

        let mut length = rx_len;
        if rx_len > rx_buffer.len() {
            length = rx_buffer.len();
        }

        self.rx_buffer.put(Some(rx_buffer));
        self.rx_len.set(length);
        self.rx_index.set(0);
    }

    fn abort_receive(&self) {
        unimplemented!();
    }
}
