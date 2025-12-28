#![allow(dead_code)]
#![no_main]
#![no_std]

mod driver;
mod exception;
mod mmio;

use aarch64_cpu::{
    asm,
    registers::{self, Readable, VBAR_EL1},
};
use core::{arch, panic::PanicInfo};
use driver::uart0;
use heapless::format;

fn kernel_main() -> ! {
    unsafe { exception::set_vbar_el1() };
    uart0::init();
    uart0::write_str("hi :3\n");
    uart0::write_str(
        format!(100; "running kernel_main at address {:p}\n", kernel_main as *const ())
            .unwrap()
            .as_str(),
    );
    let el = (&registers::CurrentEL).get();
    uart0::write_str(format!(20; "current EL: {}\n", el >> 2).unwrap().as_str());
    let vbar_el1 = VBAR_EL1.get();
    uart0::write_str(
        format!(100; "address in vbar_el1: {:p}\n", vbar_el1 as *const ())
            .unwrap()
            .as_str(),
    );
    unsafe { arch::asm!("svc 0") }
    unsafe { arch::asm!("svc 0") }
    panic!();
}

#[unsafe(no_mangle)]
pub unsafe fn _start_rust() -> ! {
    kernel_main()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        asm::wfe();
    }
}
