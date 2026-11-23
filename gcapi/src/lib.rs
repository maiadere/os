#![no_std]

use core::arch::{asm, naked_asm};

/// ends the program, never returns
pub fn exit() -> ! {
    unsafe { asm!("svc #0", options(noreturn, nostack)) }
}

/// prints a string to the kernel console (at most 4096 bytes)
pub fn print(s: &str) {
    unsafe {
        asm!(
            "svc #1",
            in("x0") s.as_ptr(),
            in("x1") s.len(),
            options(nostack, readonly, preserves_flags),
        )
    }
}

/// returns microseconds since boot
pub fn get_time() -> u64 {
    let time: u64;
    unsafe { asm!("svc #2", out("x0") time, options(nostack, nomem, preserves_flags)) }
    time
}

/// prints the program's registers as they were at the svc
pub fn dump_registers() {
    unsafe { asm!("svc #3", options(nostack, nomem, preserves_flags)) }
}

/// returns the raw levels of the button gpio pins, bit n is the level of pin n
pub fn read_buttons() -> u64 {
    let levels: u64;
    unsafe { asm!("svc #4", out("x0") levels, options(nostack, nomem, preserves_flags)) }
    levels
}

unsafe extern "C" {
    static mut __bss_start: u8;
    static mut __bss_end: u8;
}

unsafe extern "Rust" {
    fn main();
}

/// has to be naked_asm:
/// running a rust function as the first instructions of a program might work, but has
/// multiple soundess pitfalls, including but not limited to:
/// - executing rust code without estabilishing invariants of the rust abstract machine
/// - pointer provenance issues when referring to addresses of linker symbols
///
/// this function gets linked as the entry symbol to userspace programs compiled for the OS,
/// it zeroes the bss segment and jumps to an `extern "Rust"` function called `main`.
/// If main ever returns, this function calls `svc #0` which is a syscall for exiting the program and
/// returning control back to the OS
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text._start")]
#[unsafe(naked)]
extern "C" fn _start() -> ! {
    naked_asm!(
        "ldr x0, __bss_start",
        "ldr x1, __bss_end",
        "2:",
        "cmp x0, x1",
        "beq 3f",
        "stp xzr, xzr, [x0], #16",
        "b 2b",
        "3:",
        "bl main",
        "svc #0"
    );
}
