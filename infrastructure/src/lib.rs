#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    cortex_m::asm::udf()
}

pub fn _exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
