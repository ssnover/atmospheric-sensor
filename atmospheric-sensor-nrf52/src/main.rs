#![no_main]
#![no_std]

use infrastructure as _;
use nrf52840_hal::{
    self as hal,
    gpio::{p0::Parts as P0Parts, Level},
    Timer,
};
use embedded_hal::{
    blocking::delay::DelayMs,
    digital::v2::OutputPin,
};

#[cortex_m_rt::entry]
fn main() -> ! {
    let board = hal::pac::Peripherals::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);

    let pins = P0Parts::new(board.P0);

    let mut led_1 = pins.p0_13.into_push_pull_output(Level::Low);

    let delay_time = 1000u32;
    timer.delay_ms(delay_time);

    loop {
        led_1.set_high().unwrap();
        timer.delay_ms(delay_time);
        led_1.set_low().unwrap();
        timer.delay_ms(delay_time);
    }
}