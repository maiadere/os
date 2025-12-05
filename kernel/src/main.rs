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
use driver::{uart0, videocore::framebuffer};
use heapless::format;

fn kernel_main() -> ! {
    uart0::init();
    uart0::write_str("hi :3\n");
    uart0::write_str(
        format!(100; "running kernel_main at address {:p}\n", kernel_main as *const ())
            .unwrap()
            .as_str(),
    );
    let el = Readable::get(&registers::CurrentEL);
    uart0::write_str(format!(20; "current EL: {}\n", el >> 2).unwrap().as_str());

    framebuffer::init(1024, 700, 24);
    let fb = framebuffer::get().expect("framebuffer should be initialized");

    for y in 0..300 {
        for x in 0..400 {
            unsafe {
                let fb = (fb + 3 * (x + y * 400)) as *mut u32;
                let r = (255.0 * (y as f32 / 300.0)) as u32;
                let g = (255.0 * (x as f32 / 400.0)) as u32;
                fb.write_volatile(((g & 0xff) << 8) | (r & 0xff));
            }
        }
    }

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
