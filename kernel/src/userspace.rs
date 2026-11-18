use core::arch;

use aarch64_cpu::registers::{
    self, ReadWriteable,
    SPSR_EL1::{A, D, F, I, IL, M, SS},
};
use registers::Writeable;

use crate::driver::gpio::{self, PinMode, PullMode};

//stack starts at 0x8_0000 and grows downwards to 0x0
const STACK_BASE: usize = 0x8_0000;

// gpio pins with buttons, their levels can be read by the program with the read_buttons syscall
// 0 - up, 1 - left, 2 - down, 3 - right, 25 - restart
pub const BUTTON_PINS: [usize; 5] = [0, 1, 2, 3, 25];

/// loads a slice of bytes containing machine code into userspace memory, sets up the program state
/// then jumps to it in el0
pub unsafe fn load_program(code: &[u8]) -> ! {
    for (offset, byte) in code.iter().enumerate() {
        unsafe {
            ((STACK_BASE + offset) as *mut u8).write_volatile(*byte);
        }
    }
    // TODO: zero out bss

    init_buttons();

    // a bit of a hack to definitely clear the M (not M[3:0]) bit
    // to select AArch64 as the userspace execution state
    registers::SPSR_EL1.set(0x0);

    //mask all exceptions for now
    registers::SPSR_EL1.modify(D::Masked);
    registers::SPSR_EL1.modify(A::Masked);
    registers::SPSR_EL1.modify(I::Masked);
    registers::SPSR_EL1.modify(F::Masked);
    // clear software step, and illegal state
    registers::SPSR_EL1.modify(SS::CLEAR);
    registers::SPSR_EL1.modify(IL::CLEAR);
    // set the return state to EL0 with EL0 stack
    registers::SPSR_EL1.modify(M::EL0t);
    // set simulated exception return address
    registers::ELR_EL1.set(STACK_BASE as u64);
    // set simulated exception stack pointer
    registers::SP_EL0.set(STACK_BASE as u64);
    // perform simulated exception return
    unsafe {
        arch::asm!("eret");
    }
    unreachable!()
}

fn init_buttons() {
    for pin in BUTTON_PINS {
        gpio::set_pin_mode(pin, PinMode::Input);
        gpio::set_pull_mode(pin, PullMode::PullUp);
    }
}

// checks whether the userspace provided pointer is safe to read by the kernel
// this function will require updating as the memory model evolves
// for now it simply checks whether the pointer is within the memory which is statically allocated
// for userspace
pub fn check_userspace_pointer(ptr: *const u8) -> bool {
    let addr = ptr.expose_provenance();
    return addr > 0 && addr < memory::mmu::USER_MEMORY as usize;
}

// test program for the syscalls, it only uses registers and pc relative addressing
// it dumps its registers once a second and whenever the buttons change:
// x0 - buttons, x9 - time at start, x10 - time of the print, x11 - time since the last report,
// x12 - report count, x13 - previous buttons (times are in microseconds)
arch::global_asm!(
    ".pushsection .rodata.syscall_test_program, \"a\"",
    ".balign 4",
    ".global syscall_test_program_start",
    ".global syscall_test_program_end",
    "syscall_test_program_start:",
    "    adr x0, .Lsyscall_test_msg",
    "    adr x1, .Lsyscall_test_msg_end",
    "    sub x1, x1, x0",
    "    svc #1",
    "    svc #2",
    "    mov x9, x0",
    "    mov x10, x0",
    "    mov x12, #0",
    "    svc #4",
    "    mov x13, x0",
    // 1 s in microseconds
    "    movz x14, #0x4240",
    "    movk x14, #0xf, lsl #16",
    ".Lsyscall_test_loop:",
    "    svc #2",
    "    mov x15, x0",
    "    sub x11, x15, x10",
    "    svc #4",
    "    cmp x0, x13",
    "    b.ne .Lsyscall_test_report",
    "    cmp x11, x14",
    "    b.lo .Lsyscall_test_loop",
    ".Lsyscall_test_report:",
    "    mov x13, x0",
    "    mov x10, x15",
    "    add x12, x12, #1",
    "    svc #3",
    "    b .Lsyscall_test_loop",
    ".Lsyscall_test_msg:",
    "    .ascii \"syscall test: x0 = buttons, x10 = time (us), x11 = us since last report, x12 = report count\\n\"",
    ".Lsyscall_test_msg_end:",
    "syscall_test_program_end:",
    ".popsection",
);

unsafe extern "C" {
    static syscall_test_program_start: u8;
    static syscall_test_program_end: u8;
}

/// returns the machine code of the syscall test program, to be passed to [`load_program`]
pub fn syscall_test_program() -> &'static [u8] {
    let start = &raw const syscall_test_program_start;
    let end = &raw const syscall_test_program_end;

    unsafe { core::slice::from_raw_parts(start, end.addr() - start.addr()) }
}
