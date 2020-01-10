//! Implementation of the MK66 random number generator accelerator (RNGA).
//!
//! This module implements a PRNG. It uses the RNGA peripheral to generate 256
//! 32-bit numbers with 1-2 bits of entropy each, and uses SHA-256 to hash this
//! data into a 256-bit key for the Twofish block cipher in counter mode.
//!
//! - Author: Conor McAvity <cmcavity@stanford.edu>

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadWrite, WriteOnly, ReadOnly};
use kernel::hil::entropy;
use kernel::ReturnCode;
use sha2::{Sha256, Digest};
use twofish::{Twofish, BlockCipher};
use block_cipher_trait::generic_array::GenericArray;
use sim;

#[repr(C)]
struct RngaRegisters {
    control: ReadWrite<u8, Control::Register>,
    _unused0: [u8; 3],
    status: ReadOnly<u8, Status::Register>,
    reg_level: ReadOnly<u8>,
    reg_size: ReadOnly<u8>,
    _unused1: u8,
    entropy: WriteOnly<u32>,
    output: ReadOnly<u32>,
}

register_bitfields! [
    u8,
    Control [
        GO 0,
        HA 1,
        INTM 2,
        CLRI 3,
        SLP 4
    ],
    Status [
        SECV 0,
        LRS 1,
        ORU 2,
        ERRI 3,
        SLP 4
    ]
];

const BASE_ADDRESS: *const RngaRegisters = 0x40029000 as *const RngaRegisters;

pub struct Entropy<'a> {
    regs: *const RngaRegisters,
    client: OptionalCell<&'a dyn entropy::Client32>,
    key: Cell<[u8; 32]>,
    counter: Cell<u128>,
}

pub static mut ENTROPY: Entropy<'static> = Entropy::new();

impl<'a> Entropy<'a> {
    const fn new() -> Entropy<'a> {
        Entropy {
            regs: BASE_ADDRESS,
            client: OptionalCell::empty(),
            key: Cell::new([0; 32]),
            counter: Cell::new(0),
        }
    }

    pub fn init(&mut self) {
        // set clock gate
        let sim = unsafe { &*sim::SIM };
        sim.scgc6.modify(sim::SystemClockGatingControl6::RNGA::SET);

        // start rnga
        let regs = unsafe { &*self.regs };
        regs.control.modify(Control::SLP::CLEAR);
        regs.control.modify(Control::INTM::SET + Control::HA::SET + Control::GO::SET);

        let mut msg: [u8; 1024] = [0; 1024];

        // collect data from rnga
        for i in 0..256 {
            while true {
                if regs.reg_level.get() != 1 {
                    continue
                }

                let rn = regs.output.get();

                let j = 4 * i;
                msg[j] = (rn >> 24) as u8;
                msg[j + 1] = (rn >> 16) as u8;
                msg[j + 2] = (rn >> 8) as u8;
                msg[j + 3] = rn as u8;

                break;
            }
        }

        let hash = Sha256::digest(&msg);

        let key = self.key.get_mut();

        for i in 0..32 {
            key[i] = hash[i];
        }

        // stop rnga
        regs.control.modify(Control::SLP::SET);
    }


    pub fn get_number(&self) -> Option<u32> {
        let key = GenericArray::clone_from_slice(&self.key.get());
        let counter = self.counter.replace(self.counter.get() + 1);

        let mut block: [u8; 16] = [0; 16];

        // put counter value into 128 bit block
        for i in 0..16 {
            block[i] = (counter >> (120 - 8 * i)) as u8;
        }

        let mut block = GenericArray::clone_from_slice(&block);

        let cipher: Twofish = BlockCipher::new(&key);
        cipher.encrypt_block(&mut block);

        let mut num = 0u32;

        // keeps the 32 least significant bits
        for i in 0..4 {
            let byte = block[15 - i] as u32;
            num |= byte << (8 * i);
        }

        Some(num)
    }
}

struct EntropyIter<'a, 'b: 'a>(&'a Entropy<'b>);

impl<'a, 'b> Iterator for EntropyIter<'a, 'b> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        self.0.get_number()
    }
}

impl<'a> entropy::Entropy32<'a> for Entropy<'a> {
    fn get(&self) -> ReturnCode {
        if self.client.is_none() {
            return ReturnCode::FAIL
        } 
        self.client.map(|client| {
            while true {
                let result = client.entropy_available(&mut EntropyIter(self), ReturnCode::SUCCESS);
                if let entropy::Continue::Done = result {
                    break;
                }
            }
        });
        ReturnCode::SUCCESS
    }

    fn set_client(&self, client: &'a dyn entropy::Client32) {
        self.client.replace(client);
    }

    fn cancel(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }
}
