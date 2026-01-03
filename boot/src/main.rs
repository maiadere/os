#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

use core::arch::global_asm;

use aarch64_cpu::asm;

global_asm!(include_str!("./boot.s"));

#[unsafe(no_mangle)]
pub unsafe fn _start_rust() -> ! {
    unsafe {
        memory::mmu::enable_mmu();
        // branch to kernel's entrypoint
        asm!(
            "ldr x0, =0xfffffffe00080000",
            "mov sp, x0",
            "ldr x0, =0xfffffffe00081000",
            "br x0",
        );
    }
    unreachable!();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        asm::wfe();
    }
}
