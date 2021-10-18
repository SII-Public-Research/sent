#![no_main]
#![no_std]

// crates de gestion des messages de debug
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use stm32f1::stm32f103;

use stm32f1xx_hal::pac;

const ADD_TIM2_CCR1: u32 = 0x40000034;
const ADD_SRAM: u32 = 0x20000200;
const NB_DATA_DMA: u16 = 0x1E;

#[entry]
fn main() -> ! {
    // init de la session de debug
    rtt_init_print!();
    rprintln!("Coucou !");

    let dp = pac::Peripherals::take().unwrap();

    /////////////////////////////     RCC CONFIG     //////////////////////////////
    let rcc = dp.RCC;
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
    let dma_isr = &dp.DMA1.isr;
    let dma_ifcr = &dp.DMA1.ifcr;
    // address TIM2_CCR1 register
    dma5.par.write(|w| w.pa().bits(ADD_TIM2_CCR1));
    // address memory SRAM
    dma5.mar.write(|w| w.ma().bits(ADD_SRAM));
    // DMA interrupt Transfert completed
    dma5.cr.write(|w| w.tcie().set_bit());
    // total number of data transfered
    dma5.ndtr.write(|w| w.ndt().bits(NB_DATA_DMA));
    //Ch priority : LOW 00, Memory - Peripheral size : 16bits 01, Mem incremente : enable 1, Dir transfer : read from periph 0, channel enable : enable 1
    unsafe {
        dma5.cr.write(|w| {
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
                .en()
                .set_bit()
        });
    }
    rprintln!("CR : {:#x}", dma5.cr.read().bits());

    // enable DMA CH5
    //dma5.cr.write(|w| w.en().set_bit());
    rprintln!("CR : {:#x}", dma5.cr.read().bits());

    unsafe {
        stm32f103::NVIC::unmask(stm32f103::Interrupt::DMA1_CHANNEL5);
    }
    let mut address_sram: u32 = ADD_SRAM;
    //, 0x20000101, 0x20000102, 0x20000103, 0x20000104, 0x20000105,0x20000106, 0x20000107, 0x20000108, 0x20000109];
    //let mut sram: *mut u32 =  *(0x20000100 as *mut u32);

    let mut x: u16 = 0;
    //unsafe {x = *(address_sram as *mut u16);}

    //rprintln!{"ndtr value : {}", dma5.ndtr.read().bits()};

    loop {
        if (dma_isr.read().bits() & 0x20000) == 0x20000 {
            rprintln!("Transfert complet");

            for i in 0..50 {
                unsafe {
                    x = *(ADD_SRAM as *mut u16);
                }

                rprintln!("Data{} : {:#x}", i, x);
                address_sram = address_sram + 0x2;
            }

            dma_ifcr.write(|w| w.ctcif5().set_bit());
        }
    }
}
