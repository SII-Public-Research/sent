//use stm32f1::stm32f103;
use stm32f1xx_hal::{
    pac::{self},
    prelude::*,
};

use crate::{SettingClock, SettingDMA};

// initialise the setting registers
pub fn init(dma_set: SettingDMA, dp: pac::Peripherals) -> stm32f1xx_hal::dma::dma1::Channels {
    let mut rcc = dp.RCC.constrain();
    //////////////////////////////    GPIOA CONFIG     /////////////////////////////
    // enable GPIO B
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    // configure PB0 as input
    gpiob.pb0.into_floating_input(&mut gpiob.crl);

    ////////////////////////////     TIMER CONFIG      /////////////////////////////
    let timer = dp.TIM3;
    // enable TIM3
    let add_apbenr = unsafe { &(*stm32f1xx_hal::pac::RCC::ptr()).apb1enr };
    add_apbenr.modify(|_, w| w.tim3en().set_bit());

    //
    timer.ccmr2_input().write(|w| w.cc3s().ti3());
    // Capture enabled on falling edge
    timer.ccer.write(|w| w.cc3p().set_bit().cc3e().set_bit());
    unsafe {
        timer.dmar.write(|w| w.dmab().bits(0x81));
    }
    // start timer (bit CEN)
    timer.cr1.write(|w| w.cen().set_bit());
    // enable DMA request
    timer.dier.write(|w| w.cc3de().set_bit());

    ////////////////////////////    DMA CONFIG     /////////////////////////////////
    let mut dma = dp.DMA1.split(&mut rcc.ahb);

    // set peripheral address + don't enable address increment after each transfer
    dma.2.set_peripheral_address(dma_set.add_periph, false);
    // set memory address + enable address increment after each transfer
    dma.2.set_memory_address(dma_set.add_mem, true);
    // set how many data must be transfered
    dma.2.set_transfer_length(dma_set.nb_data.into()); // into() -> met en usize
    unsafe {
        // configure the dma transfer
        dma.2.ch().cr.modify(|_, w| {
            w.dir() // periph -> memory
                .clear_bit()
                .circ() // disable circular mode
                .clear_bit()
                .psize() // data size in periph = 16 bits
                .bits(0x01)
                .msize() // data size in mem = 16 bits
                .bits(0x01)
                .pl() // priority level == low
                .low()
        });
    }
    // enable interrupt when transfer complete
    dma.2.listen(stm32f1xx_hal::dma::Event::TransferComplete);
    // enable dma
    dma.2.start();

    dma
}

// stock the 19 memory time data in a table
pub fn time_stock(mut tab_time: [u16; 19], dma: SettingDMA) -> [u16; 19] {
    for i in 0..19 {
        unsafe {
            tab_time[(i as usize)] = *((dma.add_mem + (i * 0x02) as u32) as *mut u16);
        }
    }
    tab_time
}

// check if the function diff_calcul returns a result corresponding to the 56 ticks of the synchro data
pub fn synchro(tab_time: [u16; 19], mut ind: usize, status_trame: &mut bool) -> usize {
    let mut diff = 0;

    while !(1350..=1360).contains(&diff) {
        if ind < 11 {
            // after ind = 11, it's not possible to have a fair frame
            ind += 1;
            diff = diff_calcul(tab_time[ind - 1], tab_time[ind]);
        } else {
            *status_trame = false; // ind >= 11 = frame false
            break;
        }
    }

    ind
}

// calculate the difference between time n (x) and time n+1 (y)
// TIM3 is a auto-reload timer so we need to check if between time n and time n+1 there was a reload
pub fn diff_calcul(x: u16, y: u16) -> u16 {
    if x > y {
        // if x > y => reload  65535 = timer_value_max : TIM3 (16bits)
        65535 - x + y
    } else {
        y - x
    }
}

// convert times into data ( 0 - 15)
pub fn convert_data(
    timer: SettingClock,
    tab_time: [u16; 19],
    mut ind: usize,
    status_trame: &mut bool,
) -> [u8; 8] {
    let mut tab_value: [u8; 8] = [0; 8];
    for k in 0..8 {
        let data = (diff_calcul(tab_time[ind], tab_time[ind + 1]) as f32 * timer.period) as u8;
        if !(36..=81).contains(&data) {
            // data can take the next value : 36, 39, 42,..., 78, 81 = corresponding to time in us
            *status_trame = false;
        } else {
            tab_value[k] = (data - 36) / 3; // return the value
            ind += 1;
        }
    }
    tab_value
}

// verify the crc received by calculate a new crc with data received
pub fn check(tab_value: [u8; 8]) -> bool {
    let mut checksum: u8 = 5;
    let crclookup: [u8; 16] = [0, 13, 7, 10, 14, 3, 9, 4, 1, 12, 6, 11, 15, 2, 8, 5];

    for a in tab_value.iter().take(7).skip(1) {
        // calculate new crc by using only the data (6 -> tab_value[1] to tab_value[6])

        checksum = crclookup[(checksum as usize)];
        checksum ^= a;
    }
    checksum = crclookup[(checksum as usize)];

    tab_value[7] == checksum
}

// stop dma and clear the flag tcif
pub fn stop_dma(mut dma: stm32f1xx_hal::dma::dma1::Channels) -> stm32f1xx_hal::dma::dma1::Channels {
    let add_ifcr = unsafe { &(*stm32f1xx_hal::pac::DMA1::ptr()).ifcr };
    add_ifcr.write(|w| w.cgif2().set_bit()); // clear the flag tcif for channel 2
    dma.2.stop();
    dma
}

// start dma with sttings : tcie enable and nb_data to transfert
pub fn start_dma(
    dma_set: SettingDMA,
    mut dma: stm32f1xx_hal::dma::dma1::Channels,
) -> stm32f1xx_hal::dma::dma1::Channels {
    dma.2.set_transfer_length(dma_set.nb_data.into());
    dma.2.listen(stm32f1xx_hal::dma::Event::TransferComplete);
    dma.2.start();

    dma
}
