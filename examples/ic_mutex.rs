#![no_main]
#![no_std]

use sent_driver::sent;
use sent_driver::{SettingClock, SettingDMA};

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;

use stm32f1::stm32f103;
use stm32f1::stm32f103::interrupt;

use core::cell::Cell;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

static MDMA: Mutex<RefCell<Option<stm32f103::DMA1>>> = Mutex::new(RefCell::new(None));
static MINT: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

static mut TAB_TIME: [u16; 19] = [0; 19];
static ADD_SRAM: u32 = 0x20000200;

#[interrupt]
fn DMA1_CHANNEL5() {
    //rprintln!("Interrupt ! ");

    //let mut tab_time: [u16;19] = [0;19];
    let mut i: u16 = 0;

    while i < 19 {
        unsafe {
            TAB_TIME[(i as usize)] = *((ADD_SRAM + (i * 0x2) as u32) as *mut u16);
        }
        i += 1;
    }

    //unsafe {rprintln!("Time = {:?}", TAB_TIME);}

    cortex_m::interrupt::free(|cs| MINT.borrow(cs).set(true));

    cortex_m::interrupt::free(|cs| {
        let dma = MDMA.borrow(cs).borrow();
        dma.as_ref().unwrap().ifcr.write(|w| w.ctcif5().set_bit());
        dma.as_ref()
            .unwrap()
            .ch5
            .cr
            .modify(|_, w| w.en().clear_bit());
    })
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Coucou !");

    let dma_set: SettingDMA = SettingDMA::new();
    let clock: SettingClock = SettingClock::new();

    let dp = stm32f103::Peripherals::take().unwrap();

    //let d = pac::Peripherals::take().unwrap();

    /////////////////////////////     RCC CONFIG     //////////////////////////////
    let rcc = &dp.RCC;
    // enable GPIO C + AFIO
    rcc.apb2enr
        .modify(|_, w| w.iopaen().set_bit().afioen().set_bit());
    // enable TIM2
    rcc.apb1enr.modify(|_, w| w.tim2en().set_bit());
    // enable DMA1 and SRAM
    rcc.ahbenr
        .modify(|_, w| w.dma1en().set_bit().sramen().set_bit());

    //////////////////////////////    GPIOA CONFIG     /////////////////////////////
    let gpioa = &dp.GPIOA;
    // configure the pin A0 (PA0) as input
    gpioa
        .crl
        .modify(|_, w| w.mode0().input().cnf0().open_drain());
    // configure the pin D7 (PA8) as output
    gpioa
        .crh
        .modify(|_, w| w.mode8().output().cnf8().push_pull());

    ////////////////////////////     TIMER CONFIG      /////////////////////////////
    let tim2 = &dp.TIM2;
    tim2.ccmr1_input().write(|w| w.cc1s().ti1());
    // Capture enabled on falling edge
    tim2.ccer.write(|w| w.cc1p().set_bit().cc1e().set_bit());
    //tim2.egr.write(|w| w.cc1g().set_bit().tg().set_bit());
    unsafe {
        tim2.dmar.write(|w| w.dmab().bits(0x81));
    }
    // demarre le timer (bit CEN)
    tim2.cr1.write(|w| w.cen().set_bit());
    // interrupt request + DMA request
    tim2.dier.write(|w| w.cc1de().set_bit());

    ////////////////////////////    DMA CONFIG     /////////////////////////////////
    let dma5 = &dp.DMA1.ch5;

    unsafe {
        dma5.par.write(|w| w.pa().bits(dma_set.add_periph));
        // address memory SRAM
        dma5.mar.write(|w| w.ma().bits(dma_set.add_mem));
    }
    dma5.ndtr.write(|w| w.ndt().bits(dma_set.nb_data));
    unsafe {
        stm32f103::NVIC::unmask(stm32f103::Interrupt::DMA1_CHANNEL5);
    }

    cortex_m::interrupt::free(|cs| MDMA.borrow(cs).replace(Some(dp.DMA1)));

    cortex_m::interrupt::free(|cs| {
        let dma = MDMA.borrow(cs).borrow();

        unsafe {
            dma.as_ref().unwrap().ch5.cr.write(|w| {
                w.tcie()
                    .set_bit()
                    .dir()
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
                    .en()
                    .set_bit()
            });
        }
    });

    let mut tab_value: [u8; 8] = [0; 8];
    let mut diff = 0;

    loop {
        let mut j = 0;
        let mut trame_failed: bool = false;

        if cortex_m::interrupt::free(|cs| MINT.borrow(cs).get()) {
            while !(1350..=1360).contains(&diff) {
                j += 1;
                unsafe {
                    diff = sent::diff_calcul(TAB_TIME[j - 1], TAB_TIME[j]);
                }
            }

            for k in 0..8 {
                unsafe {
                    diff = sent::diff_calcul(TAB_TIME[j], TAB_TIME[j + 1]);
                }

                tab_value[k] = (diff as f32 * clock.period) as u8;

                if tab_value[k] > 81 {
                    trame_failed = true;
                    reload(dma_set);
                } else if tab_value[(k as usize)] >= 36 {
                    tab_value[k] = (tab_value[k] - 36) / 3;
                }

                j += 1;
            }

            if !trame_failed {
                sent::check(tab_value);
                rprintln!("Trame juste");
                reload(dma_set);
            }
        }
    }
}
fn reload(dma_set: SettingDMA) {
    cortex_m::interrupt::free(|cs| MINT.borrow(cs).set(false));
    cortex_m::interrupt::free(|cs| {
        let dma = MDMA.borrow(cs).borrow();

        dma.as_ref()
            .unwrap()
            .ch5
            .ndtr
            .write(|w| w.ndt().bits(dma_set.nb_data));
        dma.as_ref()
            .unwrap()
            .ch5
            .cr
            .modify(|_, w| w.tcie().set_bit().en().set_bit());
    });
}
