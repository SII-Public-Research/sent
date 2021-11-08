#![no_main]
#![no_std]

use sent_driver::sent;
use sent_driver::{SettingClock, SettingDMA};

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;

use stm32f1xx_hal::pac;
use stm32f1xx_hal::pac::dma1::isr::TCIF1_A::COMPLETE;

#[entry]
fn main() -> ! {
    // init de la session de debug
    rtt_init_print!();
    rprintln!("Start");

    let dma_set: SettingDMA = SettingDMA::new();
    let clock: SettingClock = SettingClock::new();

    let dp = pac::Peripherals::take().unwrap();
    // initialise the setting registers
    let mut dma = sent::init(dma_set, dp);

    let mut tab_time: [u16; 19] = [0; 19];

    let add_isr = unsafe { &(*stm32f1xx_hal::pac::DMA1::ptr()).isr };

    loop {
        let mut ind = 0;
        let mut status_trame: bool = true;

        if add_isr.read().tcif2() == COMPLETE {
            // stock the 19 memory time data in a table
            tab_time = sent::time_stock(tab_time, dma_set);

            dma = sent::stop_dma(dma);
            // check when the diff between two times is egal to 56 ticks (= synchro time)
            ind = sent::synchro(tab_time, ind, &mut status_trame);

            if !status_trame {
                rprintln!("Trame fausse1");
            } else {
                // convert time data in sensor's values
                let tab_value = sent::convert_data(clock, tab_time, ind, &mut status_trame);

                if !status_trame {
                    rprintln!("Trame fausse2");
                    // verify the crc
                } else if sent::check(tab_value) {
                    rprintln!("Trame juste");
                }
            }

            dma = sent::start_dma(dma_set, dma);
        }
    }
}
