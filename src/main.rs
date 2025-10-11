#![no_main]
#![no_std]

mod boot;
mod cpu;
mod driver;

use core::panic::PanicInfo;

unsafe fn kernel_init() -> ! {
    panic!();
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    cpu::spin_forever()
}
