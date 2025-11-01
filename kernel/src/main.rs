#![allow(dead_code)]
#![no_main]
#![no_std]

mod driver;
mod mmio;

use aarch64_cpu::{
    asm,
    registers::{self, Readable},
};
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

#[unsafe(no_mangle)]
pub unsafe fn _start_rust() -> ! {
    crate::kernel_main()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        asm::wfe();
    }
}
