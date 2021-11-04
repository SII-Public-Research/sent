#![no_main]
#![no_std]

use sent_driver::sent;
use sent_driver::{SettingClock, SettingDMA};

// crates de gestion des messages de debug
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
//use stm32f1xx_hal;
//use stm32f1xx_hal::rcc::APB1;
//use stm32f1::stm32f103;
//use stm32f1::stm32f103::interrupt;
//use stm32f1xx_hal::pac;

#[entry]
fn main() -> ! {
    // init de la session de debug
    rtt_init_print!();

    rprintln!("Coucou !");

    let dma_set: SettingDMA = SettingDMA::new();
    let clock: SettingClock = SettingClock::new();

    let mut dma = sent::init(dma_set);
    rprintln!("dma_cr = {}", dma.5.ch().cr.read().bits());
    let mut tab_time: [u16; 19] = [0; 19];

    let add_isr = unsafe { &(*stm32f1xx_hal::pac::DMA1::ptr()).isr };
    let add_ifcr = unsafe { &(*stm32f1xx_hal::pac::DMA1::ptr()).ifcr };

    loop {
        let mut ind = 0;
        let mut status_trame: bool = true;

        if (add_isr.read().bits() & 0x70000) == 0x70000 {
            tab_time = sent::time_stock(tab_time, dma_set);

            add_ifcr.write(|w| w.cgif5().set_bit());
            dma.5.stop();

            ind = sent::synchro(tab_time, ind, &mut status_trame);

            if !status_trame {
                rprintln!("Trame fausse1");
            } else {
                let tab_value = sent::convert_data(clock, tab_time, ind, &mut status_trame);
                //rprintln!("tab = {:?}", tab_value);
                if !status_trame {
                    rprintln!("Trame fausse2");
                } else if sent::check(tab_value) {
                    rprintln!("Trame juste");
                }
            }

            dma = sent::restart(dma_set, dma);
        }
    }
}
