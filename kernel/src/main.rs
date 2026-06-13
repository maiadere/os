#![allow(dead_code)]
#![no_main]
#![no_std]

mod console;
mod driver;
mod exception;
mod mmio;
mod userspace;

use aarch64_cpu::asm;
use core::{arch::asm, panic::PanicInfo};
use driver::{uart0, videocore::framebuffer::Framebuffer};
use heapless::format;

use crate::driver::{timer, videocore::framebuffer::Color};

fn kernel_main() -> ! {
    // post boot init
    uart0::init();
    unsafe {
        exception::set_vbar_el1();
        memory::mmu::post_boot_mappings();
    }

    let fb = Framebuffer::init(1024, 600, 32, 1).unwrap();

    loop {
        log!(100; "time: {}\n", timer::get_system_time() as f32 / 1000.0);
        fb.fill(Color::new(255, 255, 255));
        timer::delay_ms(1000);

        log!(100; "time: {}\n", timer::get_system_time() as f32 / 1000.0);
        fb.fill(Color::new(0, 0, 0));
        timer::delay_ms(1000);
    }
}

#[unsafe(no_mangle)]
pub unsafe fn _start_rust() -> ! {
    unsafe {
        asm!(
            ".macro ADR_REL register, symbol",
            "    adrp \\register, \\symbol",
            "    add \\register, \\register, #:lo12:\\symbol",
            ".endm",
            "    ADR_REL x0, __bss_start",
            "    ADR_REL x1, __bss_end",
            /* zero the bss segment */
            "0:",
            "    cmp x0, x1",
            "    b.eq 1f",
            "    stp xzr, xzr, [x0], #16",
            "    b 0b",
            "1:",
        );
    }
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
