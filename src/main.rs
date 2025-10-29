#![allow(dead_code)]
#![no_main]
#![no_std]

mod boot;
mod cpu;
mod driver;

use core::panic::PanicInfo;
use driver::uart0;

fn kernel_main() -> ! {
    uart0::init();
    uart0::write_str("hi :3");
    panic!();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    cpu::spin_forever()
}
