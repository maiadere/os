#![no_main]
#![no_std]

mod bsp;
mod cpu;

use core::panic::PanicInfo;

unsafe fn kernel_init() -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unimplemented!()
}
