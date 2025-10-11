#![no_main]
#![no_std]

mod bsp;
mod cpu;

use core::panic::PanicInfo;

unsafe fn kernel_init() -> ! {
    cpu::spin_forever()
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unimplemented!()
}
