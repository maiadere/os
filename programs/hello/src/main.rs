// test program for the load_program syscall, it asks the kernel to replace it with the program
// at the path stored after its code, which only returns if loading failed, in which case
// the error code the syscall left in x0 is reported with a register dump

#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;


// puts the program's entry point at the start of the .text section,
// so that the kernel can jump to it
global_asm!(
    ".pushsection .text._start, \"ax\"",
    ".global _start",
    "_start:",
    "    bl main",
    ".popsection",
);

fn print(message: &str) {
    unsafe {
        asm!(
            "svc #1",
            in("x0") message.as_ptr(),
            in("x1") message.len(),
            options(nostack),
        );
    }
}

fn exit() -> ! {
    unsafe {
        asm!("svc #0", options(noreturn, nostack));
    }
}

#[unsafe(no_mangle)]
extern "C" fn main() -> ! {
    print("hello from a program loaded off the sd card!\n");
    exit()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit()
}
