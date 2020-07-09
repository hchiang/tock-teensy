use cortexm4;
use pit;
use spi;
use gpio;
use uart;
use mpu;
use dma;
use adc;
use flash;
use sim;
use lptmr;
use smc;
use deferred_call_tasks::Task;
use nvic;

use kernel::common::deferred_call;
use kernel::Chip;

pub struct MK66 {
    pub mpu: mpu::Mpu,
    pub systick: cortexm4::systick::SysTick,
}

impl MK66 {
    pub unsafe fn new() -> MK66 {
        // Set up DMA channels
        adc::ADC0.set_dma(&mut dma::DMA_CHANNELS[0]);
        nvic::enable(nvic::NvicIdx::DMA0); 
        dma::DMA_CHANNELS[0].initialize(&mut adc::ADC0, dma::DMAPeripheral::ADC0);

        adc::ADC1.set_dma(&mut dma::DMA_CHANNELS[1]);
        nvic::enable(nvic::NvicIdx::DMA1);
        dma::DMA_CHANNELS[1].initialize(&mut adc::ADC1, dma::DMAPeripheral::ADC1);

        MK66 {
            mpu: mpu::Mpu::new(),
            systick: cortexm4::systick::SysTick::new(),
        }
    }
}

impl Chip for MK66 {
    type MPU = mpu::Mpu;
    type SysTick = cortexm4::systick::SysTick;

    fn service_pending_interrupts(&mut self) {
        use nvic::*;
        unsafe {
            if let Some(task) = deferred_call::DeferredCall::next_pending() {
                match task {
                    Task::Flashcalw => flash::FLASH_CONTROLLER.handle_interrupt(),
                }
            }
            while let Some(interrupt) = cortexm4::nvic::next_pending() {
                match interrupt {
                    DMA0 => dma::DMA_CHANNELS[0].handle_interrupt(),
                    DMA1 => dma::DMA_CHANNELS[1].handle_interrupt(),

                    FLASHCC => flash::FLASH_CONTROLLER.handle_interrupt(),
                    FLASHRC => flash::FLASH_CONTROLLER.handle_interrupt(),
                    ADC0 => adc::ADC0.handle_interrupt(),
                    ADC1 => adc::ADC1.handle_interrupt(),
                    PCMA => gpio::PA.handle_interrupt(),
                    PCMB => gpio::PB.handle_interrupt(),
                    PCMC => gpio::PC.handle_interrupt(),
                    PCMD => gpio::PD.handle_interrupt(),
                    PCME => gpio::PE.handle_interrupt(),
                    PIT2 => pit::PIT.handle_interrupt(),
                    SPI0 => spi::SPI0.handle_interrupt(),
                    SPI1 => spi::SPI1.handle_interrupt(),
                    SPI2 => spi::SPI2.handle_interrupt(),
                    UART0 => uart::UART0.handle_interrupt(),
                    UART1 => uart::UART1.handle_interrupt(),
                    LPTMR=> lptmr::LPTMR.handle_interrupt(),
                    _ => {}
                }

                let n = cortexm4::nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm4::nvic::has_pending() || deferred_call::has_tasks() }
    }

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn systick(&self) -> &Self::SysTick {
        &self.systick
    }

    fn sleep(&self) {
        if sim::deep_sleep_ready() {
            smc::set_vlps();
            unsafe {
                cortexm4::scb::set_sleepdeep();
            }
        }
        else {
            unsafe {
                cortexm4::scb::unset_sleepdeep();
            }
        }

        unsafe {
            cortexm4::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm4::support::atomic(f)
    }
}
