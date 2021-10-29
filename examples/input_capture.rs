#![no_main]
#![no_std]

// crates de gestion des messages de debug
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use stm32f1::stm32f103;
use stm32f1::stm32f103::interrupt;

//extern crate tim_sent;

const ADD_TIM2_CCR1: u32 = 0x40000034;
const ADD_SRAM: u32 = 0x20000200;
const NB_DATA_DMA: u16 = 0x13;

const TIMER_FREQ: f32 = 8.0; // Timer freq  MHz
const TIMER_PERIOD: f32 = 1.0 / TIMER_FREQ; // Timer period us

static mut TRANSFERT_COMPLETED: bool = false;
static mut TAB_TIME: [u16; 19] = [0; 19];
static mut I: u16 = 0;

#[interrupt]
fn DMA1_CHANNEL5() {
    //rprintln! ("i = {}", i);
    unsafe {
        TRANSFERT_COMPLETED = true;

        TAB_TIME[(I as usize)] = *((ADD_SRAM + (I * 0x2) as u32) as *mut u16);
        I = I + 1;
        if I > 18 {
            //rprintln!("Time {:?}", TAB_TIME);
            //rprintln!("Time {:?} ", tab_time);
            I = 0;
            *(0x40020004 as *mut u32) |= 1 << 17; // met à 0 les flags
            *(0x40020058 as *mut u32) &= 0; // Disable DMA
        }
    }
}

#[entry]
fn main() -> ! {
    // init de la session de debug
    rtt_init_print!();
    //rprintln!("Coucou !");

    let dp = stm32f103::Peripherals::take().unwrap();

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
    //let dma_isr = &dp.DMA1.isr;
    //let dma_ifcr = &dp.DMA1.ifcr;
    // address TIM2_CCR1 register
    unsafe {
        dma5.par.write(|w| w.pa().bits(ADD_TIM2_CCR1));
        // address memory SRAM
        dma5.mar.write(|w| w.ma().bits(ADD_SRAM));
    }
    // DMA interrupt Transfert completed
    //dma5.cr.write(|w| w.tcie().set_bit());
    // total number of data transfered

    let dma5 = &dp.DMA1.ch5;
    dma5.ndtr.write(|w| w.ndt().bits(NB_DATA_DMA));
    //Ch priority : LOW 00, Memory - Peripheral size : 16bits 01, Mem incremente : enable 1, Dir transfer : read from periph 0, channel enable : enable 1

    //dma5.ndtr.write(|w| w.ndt().bits(NB_DATA_DMA));
    unsafe {
        dma5.cr.write(|w| {
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

    unsafe {
        stm32f103::NVIC::unmask(stm32f103::Interrupt::DMA1_CHANNEL5);
    }

    let mut tab_value: [u8; 8] = [0; 8];
    let mut diff = 0;

    loop {
        unsafe {
            let mut j = 0;
            let mut trame_failed: bool = false;

            if TRANSFERT_COMPLETED == true {
                while diff < 1350 || diff > 1360 {
                    j = j + 1;
                    diff = diff_calcul(TAB_TIME[j - 1], TAB_TIME[j]);
                }

                for k in 0..8 {
                    diff = diff_calcul(TAB_TIME[j], TAB_TIME[j + 1]);
                    tab_value[k] = (diff as f32 * TIMER_PERIOD) as u8;

                    if tab_value[k] > 81 {
                        trame_failed = true;
                        TRANSFERT_COMPLETED = false;
                        dma5.ndtr.write(|w| w.ndt().bits(NB_DATA_DMA));
                        dma5.cr.modify(|_, w| w.tcie().set_bit().en().set_bit());
                    } else if tab_value[(k as usize)] >= 36 {
                        tab_value[k] = (tab_value[k] - 36) / 3;
                    }
                    j = j + 1;
                }

                if trame_failed == false {
                    check(tab_value);
                    rprintln!("Trame juste");
                    TRANSFERT_COMPLETED = false;

                    dma5.ndtr.write(|w| w.ndt().bits(NB_DATA_DMA));
                    dma5.cr.modify(|_, w| w.tcie().set_bit().en().set_bit());
                }
            }
        }
    }
}

fn diff_calcul(x: u16, y: u16) -> u16 {
    let diff = if x > y { 65535 - x + y } else { y - x };

    diff
}

fn check(tab_value: [u8; 8]) -> bool {
    let mut checksum: u8 = 5;
    let crclookup: [u8; 16] = [0, 13, 7, 10, 14, 3, 9, 4, 1, 12, 6, 11, 15, 2, 8, 5];

    for a in 1..7 {
        checksum = crclookup[(checksum as usize)];
        checksum = checksum ^ tab_value[a];
    }
    checksum = crclookup[(checksum as usize)];

    if tab_value[7] == checksum {
        //tab_value[7] -> crc
        true
    } else {
        false
    }
}
