#![no_std]

use core::arch::{asm, naked_asm};

pub const NAME_CAP: usize = 255;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct DirEntryRecord {
    pub name: [u8; NAME_CAP],
    pub name_len: u8,
    pub is_dir: u8,
    pub file_size: u32,
}

impl DirEntryRecord {
    pub const EMPTY: Self = Self {
        name: [0; NAME_CAP],
        name_len: 0,
        is_dir: 0,
        file_size: 0,
    };

    pub fn name(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len as usize]).unwrap_or("?")
    }

    pub fn is_dir(&self) -> bool {
        self.is_dir != 0
    }
}

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

/// returns which buttons are held, bit n is set if the button on gpio pin n is pressed
/// the kernel normalises the pin polarity, so a set bit always means pressed
pub fn read_buttons() -> u64 {
    let levels: u64;
    unsafe { asm!("svc #4", out("x0") levels, options(nostack, nomem, preserves_flags)) }
    levels
}

/// loads a program from the given path
pub fn load_program_from_path(path: &str) -> Result<(), ()> {
    let res: u64;
    unsafe {
        asm!(
            "svc #5",
            in("x0") path.as_ptr(),
            in("x1") path.len(),
            lateout("x0") res,
            options(nostack, readonly, preserves_flags),
        )
    }
    if res == 0 { Ok(()) } else { Err(()) }
}

/// lists the entries of the directory at 'path' into 'out' while skipping "." and ".."
///
/// returns the directory's total entry count, which can be larger than out.len() if the
/// buffer is too small in which case only the first out.len() entries are written
pub fn list_dir(path: &str, out: &mut [DirEntryRecord]) -> Result<usize, ()> {
    let total: u64;
    unsafe {
        asm!(
            "svc #6",
            in("x0") path.as_ptr(),
            in("x1") path.len(),
            in("x2") out.as_mut_ptr(),
            in("x3") out.len(),
            lateout("x0") total,
            options(nostack, preserves_flags),
        )
    }
    if total == u64::MAX {
        Err(())
    } else {
        Ok(total as usize)
    }
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
