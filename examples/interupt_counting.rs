
//#![deny(unsafe_code)]
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
};

use cortex_m::peripheral::DWT;

static mut tableau: [u32; 8] = [0; 8];
static mut COUNT: u32 = 0;


#[interrupt]
fn EXTI15_10() {

    static mut NEW_COUNT: u32 = 0;
    *NEW_COUNT = DWT::get_cycle_count();
    let temps: u32 = *NEW_COUNT / 32; //frequence

    unsafe {
        // je remet a 0 le counter DWT
        *(0xE0001004 as *mut u32) = 0;
        // on met à 1 le bit 12 du registre pr pour valider l'interuption 
        *(0x40010414 as *mut u32) |= 1 << 12;
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

}

#[entry]
fn main() -> ! {

    // init de la session de debug
    rtt_init_print!();
    rprintln!("Coucou !");

    let dp = pac::Peripherals::take().unwrap();
    let mut cp = pac::CorePeripherals::take().unwrap();

    /****************************************************************************************/
    /*****************              ACTIVATION DE L'INTERUPTION         *********************/
    /****************************************************************************************/

    unsafe {
        stm32f103::NVIC::unmask(stm32f103::Interrupt::EXTI15_10);
    }

    /****************************************************************************************/
    /*****************              ACTIVATION DES HORLOGES          ************************/
    /****************************************************************************************/

    let rcc = &dp.RCC;

    // allume le GPIOC
    rcc.apb2enr.modify(|_, w| w.iopcen().set_bit());
    // allume les fonctions alternatives 
    rcc.apb2enr.modify(|_, w| w.afioen().set_bit());


    /****************************************************************************************/
    /*****************              INITIALISATION DES GPIOS         ************************/
    /****************************************************************************************/
    
    let gpioc = &dp.GPIOC;

    // configure le pin en input
    gpioc.crh.modify(|_, w| w.mode12().input());
    // configure le mode input en open_drain
    gpioc.crh.modify(|_, w| w.cnf12().open_drain());


    /****************************************************************************************/
    /*****************              INITIALISATION DE L'INTERUPTION         *****************/
    /****************************************************************************************/
    
    let afio = &dp.AFIO;

    unsafe {
        // selectionne le GPIOC pin 12 en source
        afio.exticr4.write(|w| w.exti12().bits(0x02));
    }

    let exti = &dp.EXTI;

    // active les interuptions sur les pin 12
    exti.imr.write(|w| w.mr12().set_bit());
    // leve une interuption sur les fronts descendants
    exti.ftsr.write(|w| w.tr12().set_bit());

    cp.DWT.enable_cycle_counter();

    unsafe {
        // je remet a 0 le counter DWT
        *(0xE0001004 as *mut u32) = 0;
    }
    
    //rprintln!("{config termine !}");

    //let mut flash = dp.FLASH.constrain();
    //let mut rcc = dp.RCC.constrain();
    //let clocks = rcc.cfgr.freeze(&mut flash.acr);
    //let timer = MonoTimer::new(cp.DWT, clocks);
    // 0xE000_1000

    //let t = timer.now().elapsed();
    //hprintln!("nombre de ticks : {}", t).unwrap();
    //let freq = timer.frequency();
    //hprintln!("frequence : {:?} Hertz", freq.0).unwrap();
    loop {}
        
}

