//use stm32f1::stm32f103;
use stm32f1xx_hal::{
    pac::{self},
    prelude::*,
};

use crate::{SettingClock, SettingDMA};

pub fn init(dma_set: SettingDMA, dp: pac::Peripherals) -> stm32f1xx_hal::dma::dma1::Channels {
    //let dp = ::take().unwrap();
    //let cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    //////////////////////////////    GPIOA CONFIG     /////////////////////////////
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    gpiob.pb0.into_floating_input(&mut gpiob.crl);

    let mut flash = dp.FLASH.constrain();
    rcc.cfgr.sysclk(8.mhz()).freeze(&mut flash.acr);
    //let mut delay = Delay::new(cp.SYST, clock);

    ////////////////////////////     TIMER CONFIG      /////////////////////////////
    let timer = dp.TIM3;
    let add_apbenr = unsafe { &(*stm32f1xx_hal::pac::RCC::ptr()).apb1enr };
    add_apbenr.modify(|_, w| w.tim3en().set_bit());

    timer.ccmr2_input().write(|w| w.cc3s().ti3());
    // Capture enabled on falling edge
    timer.ccer.write(|w| w.cc3p().set_bit().cc3e().set_bit());
    unsafe {
        timer.dmar.write(|w| w.dmab().bits(0x81));
    }
    // demarre le timer (bit CEN)
    timer.cr1.write(|w| w.cen().set_bit());
    // interrupt request + DMA request
    timer.dier.write(|w| w.cc3de().set_bit());

    ////////////////////////////    DMA CONFIG     /////////////////////////////////
    let mut dma = dp.DMA1.split(&mut rcc.ahb);

    dma.2.set_peripheral_address(dma_set.add_periph, false);
    dma.2.set_memory_address(dma_set.add_mem, true);
    dma.2.set_transfer_length(dma_set.nb_data.into()); // into() -> met en usize
    unsafe {
        dma.2.ch().cr.write(|w| {
            w.dir()
                .clear_bit()
                .circ()
                .clear_bit()
                .pinc()
                .clear_bit()
                .minc()
                .set_bit()
                .psize()
                .bits(0x01)
                .msize()
                .bits(0x01)
                .pl()
                .low()
                .mem2mem()
                .clear_bit()
        });
    }
    dma.2.listen(stm32f1xx_hal::dma::Event::TransferComplete);
    dma.2.start();

    /*unsafe {
        pac::NVIC::unmask(pac::Interrupt::DMA1_CHANNEL5);
    }*/

    dma
}

pub fn time_stock(mut tab_time: [u16; 19], dma: SettingDMA) -> [u16; 19] {
    for i in 0..19 {
        unsafe {
            tab_time[(i as usize)] = *((dma.add_mem + (i * 0x02) as u32) as *mut u16);
        }
    }
    tab_time
}

pub fn synchro(tab_time: [u16; 19], mut ind: usize, status_trame: &mut bool) -> usize {
    let mut diff = 0;

    while !(1350..=1360).contains(&diff) {
        // = diff < 1350 || diff > 1360
        if ind < 11 {
            ind += 1;
            diff = diff_calcul(tab_time[ind - 1], tab_time[ind]);
        } else {
            *status_trame = false;
            break;
        }
    }

    ind
}

pub fn diff_calcul(x: u16, y: u16) -> u16 {
    if x > y {
        65535 - x + y
    } else {
        y - x
    }
}

pub fn convert_data(
    timer: SettingClock,
    tab_time: [u16; 19],
    mut ind: usize,
    status_trame: &mut bool,
) -> [u8; 8] {
    let mut tab_value: [u8; 8] = [0; 8];
    for k in 0..8 {
        let data = (diff_calcul(tab_time[ind], tab_time[ind + 1]) as f32 * timer.period) as u8;
        if !(12..=81).contains(&data) {
            *status_trame = false;
        } else {
            tab_value[k] = (data - 36) / 3;
            ind += 1;
        }
    }
    tab_value
}

pub fn check(tab_value: [u8; 8]) -> bool {
    let mut checksum: u8 = 5;
    let crclookup: [u8; 16] = [0, 13, 7, 10, 14, 3, 9, 4, 1, 12, 6, 11, 15, 2, 8, 5];

    for a in tab_value.iter().take(7).skip(1) {
        //= 1..7
        checksum = crclookup[(checksum as usize)];
        checksum ^= a;
    }
    checksum = crclookup[(checksum as usize)];

    tab_value[7] == checksum
}

pub fn restart(
    dma_set: SettingDMA,
    mut dma: stm32f1xx_hal::dma::dma1::Channels,
) -> stm32f1xx_hal::dma::dma1::Channels {
    dma.2.set_transfer_length(dma_set.nb_data.into());
    dma.2.ch().cr.modify(|_, w| w.tcie().set_bit());
    dma.2.start();

    dma
}
