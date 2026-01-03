#![allow(dead_code)]
#![no_main]
#![no_std]

mod driver;
mod exception;
mod mmio;
mod userspace;

use aarch64_cpu::{
    asm,
    registers::{self, Readable, VBAR_EL1},
};
use core::{arch, panic::PanicInfo};
use driver::{
    uart0,
    videocore::framebuffer::{Color, Framebuffer},
};
use heapless::format;

fn kernel_main() -> ! {
    // post boot init
    uart0::init();
    unsafe {
        exception::set_vbar_el1();
        memory::mmu::post_boot_mappings();
    }
    // at this point the exceptions, uart printing and virtual memory are set up
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
    let sp = registers::SP.get();
    uart0::write_str(
        format!(128; "current stack pointer: {:p}\n", sp as *const ())
            .unwrap()
            .as_str(),
    );

    let (width, height) = (1024, 600);
    let fb = Framebuffer::init(width, height, 32, 1).unwrap();

    uart0::write_str(format!(500; "{:?}\n", fb).unwrap().as_str());

    for y in 0..height {
        for x in 0..width {
            let r = x as f32 / width as f32;
            let g = y as f32 / height as f32;
            fb.set_pixel(x, y, Color::new(r, g, 0.0)).unwrap();
        }
    }

    // translates to:
    // nop
    // svc #0
    let test_code: [u8; _] = [0x1f, 0x20, 0x03, 0xd5, 0x01, 0x00, 0x00, 0xd4];
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
