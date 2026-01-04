#![allow(dead_code)]
#![no_main]
#![no_std]

mod console;
mod driver;
mod exception;
mod mmio;
mod userspace;

use aarch64_cpu::{
    asm,
    registers::{self, Readable, VBAR_EL1},
};
use core::panic::PanicInfo;
use driver::{uart0, videocore::framebuffer::Framebuffer};
use heapless::format;

fn kernel_main() -> ! {
    // post boot init
    uart0::init();
    unsafe {
        exception::set_vbar_el1();
        memory::mmu::post_boot_mappings();
    }
    // at this point the exceptions, uart printing and virtual memory are set up
    log!(10; "hi :3\n");
    log!(100; "running kernel_main at address {:p}\n", kernel_main as *const ());
    log!(20; "current EL: {}\n", ((&registers::CurrentEL).get()) >> 2);
    log!(100; "address in vbar_el1: {:p}\n", VBAR_EL1.get() as *const ());
    log!(128; "current stack pointer: {:p}\n", registers::SP.get() as *const ());

    let (width, height) = (1024, 600);
    let fb = Framebuffer::init(width, height, 32, 1).unwrap();
    log!(500; "{:?}\n", fb);

    console::render(&fb);

    let test_code: [u8; _] = [
        0x1f, 0x20, 0x03, 0xd5, //nop
        0x01, 0x00, 0x00, 0xd4, //svc #0
    ];
    userspace::load_program(&test_code);
}

#[unsafe(no_mangle)]
pub unsafe fn _start_rust() -> ! {
    kernel_main()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(panic_message) = format!(2048;"{}\n", info).ok() {
        uart0::write_str(panic_message.as_str());
    } else {
        uart0::write_str("Kernel panic; panic message too long!");
    }
    loop {
        asm::wfe();
    }
}
