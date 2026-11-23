#![no_std]

use core::arch::asm;

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

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text._start")]
extern "C" fn _start() -> ! {
    unsafe {
        let start = &raw mut __bss_start;
        let end = &raw mut __bss_end;
        start.write_bytes(0, end.addr() - start.addr());
        main();
    }
    exit()
}
