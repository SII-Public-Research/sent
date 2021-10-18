#![no_main]
#![no_std]

//use std::fs;

// crates de gestion des messages de debug
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use stm32f1::stm32f103;

// Gestion des interruptions
use stm32f1::stm32f103::interrupt;

static mut FALLING_EDGE: bool = false;

#[interrupt]
fn EXTI15_10() {
    unsafe {
        FALLING_EDGE = true;
    }

    unsafe {
        // on met à 1 le bit 5 du registre pr pour valider l'interuption
        *(0x40010414 as *mut u32) |= 1 << 13;
    }
}

#[entry]
fn main() -> ! {
    // init de la session de debug
    rtt_init_print!();
    rprintln!("Coucou !");

    let dp = stm32f103::Peripherals::take().unwrap();

    // Interrupt enable
    unsafe {
        stm32f103::NVIC::unmask(stm32f103::Interrupt::EXTI15_10);
    }

    // access RCC registers
    let rcc = &dp.RCC;

    //unsafe{rcc.cfgr.write(|w| w.mco().bits(0x5));}
    // HSI clock active
    rcc.cr.modify(|_, w| w.hsion().set_bit());
    rcc.cr.write(|w| w.hsitrim().bits(0x10));
    // PLL active
    rcc.cr.modify(|_, w| w.pllon().set_bit());

    // PLL selected as system clock
    unsafe {
        rcc.cfgr.write(|w| w.sw().bits(0x2));
    }
    // AHB prescaler
    unsafe {
        rcc.cfgr.write(|w| w.hpre().bits(0x0));
    }

    unsafe {
        rcc.cfgr.write(|w| w.ppre1().bits(0x4));
    }
    // PLL multiplier
    rcc.cfgr.write(|w| w.pllmul().bits(0xD));

    // access RCC_APB2ENR register + GPIO C enable
    rcc.apb2enr.modify(|_, w| w.iopcen().set_bit());
    // AFIO enable
    rcc.apb2enr.modify(|_, w| w.afioen().set_bit());
    // TIM2 enable
    rcc.apb1enr.modify(|_, w| w.tim2en().set_bit());

    // access GPIOC registers
    let gpioc = &dp.GPIOC;
    // configure the pin 13 (PC12) as input
    gpioc.crh.modify(|_, w| w.mode12().input());
    gpioc.crh.modify(|_, w| w.mode10().output());
    // configure mode open drain
    gpioc.crh.modify(|_, w| w.cnf12().open_drain());
    gpioc.crh.modify(|_, w| w.cnf10().push_pull());

    ///////////////////////////// INTERRUPT //////////////////////////////////////

    // access AFIO registers
    let afio = &dp.AFIO;
    // select the source inout for EXTI12 -> PC12
    unsafe {
        afio.exticr4.write(|w| w.exti12().bits(0x02));
    }

    // access EXTI registers
    let exti = &dp.EXTI;
    // interrupt enable EXTI12
    exti.imr.write(|w| w.mr12().set_bit());
    // falling trigger enable for EXTI12
    exti.ftsr.write(|w| w.tr12().set_bit());

    // access TIM2 registers
    let tim2 = &dp.TIM2;
    // Capture 1 interrupt enable
    tim2.dier.write(|w| w.cc1ie().set_bit());
    // counter enable, will immediately start counting
    tim2.cr1.write(|w| w.cen().set_bit());
    // prescaler value
    //tim2.psc.write(|w| w.psc().bits(0x2));

    let mut count: u32 = 0;
    let mut value: [u32; 10] = [0; 10];
    let mut synchro: bool = false;
    let mut sum: u32 = 0;
    let mut br_b: u32 = 0;
    let mut br_h: u32 = 0;

    loop {
        unsafe {
            if FALLING_EDGE == true {
                //rprintln!("count : {}", count);
                gpioc.odr.write(|w| w.odr10().clear_bit());
                FALLING_EDGE = false;
                value[(count as usize)] = tim2.cnt.read().bits();

                if count == 0 {
                    //gpioc.odr.write(|w| w.odr10().clear_bit());
                    if value[(count as usize)] > (65535 - 1344) {
                        sum = value[(count as usize)] + 1344 - 65535;
                    } else {
                        sum = value[(count as usize)] + 1344;
                    }
                    br_b = sum - 10;
                    br_h = sum + 20;
                    //gpioc.odr.write(|w| w.odr10().set_bit());
                }

                if count == 1 {
                    //rprintln!("value0 : {}", value[0]);
                    //rprintln!("value1 : {}", value[1]);
                    //rprintln!("{}", sum);
                    //gpioc.odr.write(|w| w.odr10().clear_bit());
                    if value[(count as usize)] > br_b && value[(count as usize)] < br_h {
                        synchro = true;
                        sum = 0;
                    } else {
                        //  gpioc.odr.write(|w| w.odr10().set_bit());
                        value[0] = value[(count as usize)];
                        //gpioc.odr.write(|w| w.odr10().set_bit());
                        continue;
                    }
                    //gpioc.odr.write(|w| w.odr10().set_bit());
                }

                if synchro == true {
                    count = count + 1;

                    if count == 10 {
                        //gpioc.odr.write(|w| w.odr10().clear_bit());
                        count = 0;
                        //gpioc.odr.write(|w| w.odr10().set_bit());
                        //rprintln!("{:?}", value);
                    }
                    //rprintln!("Synchro !");
                    gpioc.odr.write(|w| w.odr10().set_bit());
                    continue;
                }

                count = count + 1;
                gpioc.odr.write(|w| w.odr10().set_bit());
            }
        }
    }
}
