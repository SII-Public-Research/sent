#![no_main]
#![no_std]

// crates de gestion des messages de debug
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use stm32f1::stm32f103;
use stm32f1::stm32f103::interrupt;

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    pac,
    prelude::*,
};

#[interrupt]
fn TIM2() {
    rprintln!("On est dedans !");
    unsafe {
        // on remet la valeur du bit UIF à 0 pour dire qu'on a gere l'interup
        *(0x40000010 as *mut u32) &= !(0x01 << 0);
    }
}

#[entry]
fn main() -> ! {

    rtt_init_print!();
    rprintln!("Coucou !");

    /****************************************************************************************/
    /*****************              CONFIGURATION DE BASE               *********************/
    /****************************************************************************************/

    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Prepare the alternate function I/O registers
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);


    /****************************************************************************************/
    /*****************              ACTIVATION DE L'INTERUPTION         *********************/
    /****************************************************************************************/


    unsafe {
        stm32f103::NVIC::unmask(stm32f103::Interrupt::TIM2);
    }

    /****************************************************************************************/
    /*****************              CONFIGURATION DE L'INTERUPTION          *****************/
    /****************************************************************************************/

    // TIM 2 registre CCMR1 bit CC1S permet de selection er le mode (input)
    // TIM2_CCR1 devient read only - compteur -> 01


    // on applique un filtre (registre CCMR1 bit IC1F) -> 0011

    // on choisi le type de transition (registre CCER bit CC1P) 
    // -> 1 (front descendant)

    // input prescaler a 0 (registre CCMR bit IC1PS) -> 0

    // active la capture (registre CCER bit CC1E) -> 1

    // registre TIMx_DIER bit CC1IE active l'interuption correspondante

    loop {}
        
}
