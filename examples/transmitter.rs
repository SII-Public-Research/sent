#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

//use stm32f1::stm32f103;
//use stm32f1::stm32f103::interrupt;

use cortex_m_rt::entry;
use stm32f1xx_hal::{delay::Delay, pac, prelude::*};

extern crate sent_driver;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Coucou !");

    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let pin = gpioc.pc10.into_push_pull_output(&mut gpioc.crh);
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .pclk1(36.mhz())
        .sysclk(72.mhz())
        .freeze(&mut flash.acr);
    let delay = Delay::new(cp.SYST, clocks);

    let mut sent = sent_driver::Sent::new_default(delay, pin);

    loop {
        sent.send_frame(15, [0, 9, 5, 13, 8, 4], 20);
    }
}
