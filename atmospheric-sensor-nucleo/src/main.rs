#![no_main]
#![no_std]

use infrastructure as _;
use stm32f3xx_hal as hal;
use hal::prelude::*;

#[cortex_m_rt::entry]
fn main() -> ! {

    let board = hal::pac::Peripherals::take().unwrap();
    let mut rcc = board.RCC.constrain();
    let mut gpiob = board.GPIOB.split(&mut rcc.ahb);
    let mut ld2 = gpiob.pb13.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    ld2.set_low().unwrap();

    let mut flash = board.FLASH.constrain();
    let clocks = rcc.cfgr.sysclk(8.mhz()).freeze(&mut flash.acr);
    let mut delay = hal::delay::Delay::new(cortex_m::Peripherals::take().unwrap().SYST, clocks);

    loop {
        ld2.toggle().unwrap();
        delay.delay_ms(1000_u16);
    }
}
