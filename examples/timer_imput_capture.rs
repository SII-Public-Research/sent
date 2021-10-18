// TIM2 CH1 PA0

#![no_main]
#![no_std]

// crates de gestion des messages de debug
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use stm32f1::stm32f103;
use stm32f1::stm32f103::interrupt;

use cortex_m_rt::entry;
use stm32f1xx_hal::{pac, prelude::*};

static mut tableau: [u32; 8] = [0; 8];
static mut COUNT: u32 = 0;

#[interrupt]
fn TIM2() {
    static mut NEW_COUNT: u32 = 0;
    let temps: u32 = *NEW_COUNT / 8; //frequence

    //let registre = tim2.ccr1.read().bits();
    rprintln!("On est dedans !");

    unsafe {
        //      *NEW_COUNT = *(0x40000034 as *mut u32);
        //*(0x40000024 as *mut u32) = 0;
        // on remet la valeur du bit CC1IF à 0 pour dire qu'on a gere l'interup
        *(0x40000010 as *mut u32) &= !(0x01 << 1);

        // on inverse la valeur de la led avec une opération XOR
        *(0x4001080C as *mut u32) ^= 0x01 << 8;
    }

    unsafe {
        if COUNT == 0 {
            COUNT += 1;
        } else if COUNT < 9 {
            tableau[(COUNT as usize) - 1] = temps;
            COUNT += 1;
        } else {
            rprintln!("frame : {:?} ", tableau);
            COUNT = 0;
        }
    }

    //rprintln!("la valeure de CCR1 est : {}", NEW_COUNT);

    // TIMx_CCR2 - TIMx_CCR1 donne la durée entre deux fronts
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Coucou !");

    /****************************************************************************************/
    /*****************              CONFIGURATION DE BASE               *********************/
    /****************************************************************************************/

    // Get access to the device specific peripherals from the peripheral access crate
    let dp = stm32f103::Peripherals::take().unwrap();

    let rcc = &dp.RCC;
    // allume le GPIOA
    rcc.apb2enr.modify(|_, w| w.iopaen().set_bit());
    // allume les fonctions alternatives
    rcc.apb2enr.modify(|_, w| w.afioen().set_bit());
    // allume TIM2
    rcc.apb1enr.modify(|_, w| w.tim2en().set_bit());

    let gpioa = &dp.GPIOA;
    // configure the pin as input
    gpioa.crl.modify(|_, w| w.mode0().input());
    // configure mode open drain
    gpioa.crl.modify(|_, w| w.cnf0().open_drain());

    // configure the pin as output
    gpioa.crh.modify(|_, w| w.mode8().output());
    // configure mode output push pull
    gpioa.crh.modify(|_, w| w.cnf8().push_pull());

    /****************************************************************************************/
    /*****************              ACTIVATION DE L'INTERUPTION         *********************/
    /****************************************************************************************/

    unsafe {
        stm32f103::NVIC::unmask(stm32f103::Interrupt::TIM2);
    }

    /****************************************************************************************/
    /*****************              CONFIGURATION DE L'INTERUPTION          *****************/
    /****************************************************************************************/

    let tim2 = &dp.TIM2;

    // TIM 2 registre CCMR1 bit CC1S permet de selectioner le mode (input)
    // CC1 channel is configured as input, IC1 is mapped on TI1.
    unsafe {
        tim2.ccmr1_input()
            .write(|w| w.cc1s().ti1().ic1f().no_filter().ic1psc().bits(0x0));
    }
    // on applique un filtre (registre CCMR1 bit IC1F) -> 0011
    //tim2.ccmr1_input().write(|w| w.ic1f().fck_int_n4()); // a modifer pour test
    // CC1P : Capture / Compare 1 output polarity (falling edge -> 1)
    tim2.ccer.write(|w| w.cc1p().set_bit().cc1e().set_bit());
    // input prescaler a 0 (registre CCMR1 bit IC1PS) -> 0
    //unsafe {
    //    tim2.ccmr1_input().write(|w| w.ic1psc().bits(0x0));
    //}
    // CC1E : Capture / Compare 1 output enable (enable -> 1)
    //tim2.ccer.write(|w| w.cc1e().set_bit());
    // registre TIMx_DIER bit CC1IE active l'interuption correspondante
    tim2.dier.write(|w| w.cc1ie().set_bit());
    // demarre le timer (bit CEN)
    tim2.cr1.write(|w| w.cen().set_bit());

    // TIM2_CCR1 devient read only - ca donne la valeur du dernier triger
    // donc ! CCR2 - CCR1 donne la durée entre deux fronts

    // registre SR bit CC1IF pour prendre en compte l'interuption
    // est remis à 0 si on lit la valeur de CCR1

    loop {
        //let registre = tim2.cnt.read().bits();
        let registre = tim2.ccr1.read().bits();
        rprintln!("La valeur du registre CCR1 est : {} ", registre);
        rprintln!("SR : {:#x}", tim2.sr.read().bits());
    }
}
