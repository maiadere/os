#![allow(dead_code)]
#![no_main]
#![no_std]

mod boot;
mod cpu;
mod driver;

use aarch64_cpu::registers::{self, Readable};
use core::panic::PanicInfo;
use driver::uart0;
use heapless::format;

fn kernel_main() -> ! {
    uart0::init();
    uart0::write_str("hi :3\n");
    let el = Readable::get(&registers::CurrentEL);
    uart0::write_str(format!(20; "current EL: {}\n", el >> 2).unwrap().as_str());
    panic!();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    cpu::spin_forever()
}
