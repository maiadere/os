#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

use aarch64_cpu::asm;

use core::arch::global_asm;

global_asm!(include_str!("./boot.s"));

#[unsafe(no_mangle)]
pub unsafe fn _start_rust() -> ! {
    unsafe {
        // branch to kernel's entrypoint
        asm!("ldr x0, =0x81000");
        asm!("br x0");
    }
    panic!();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        asm::wfe();
    }
}
