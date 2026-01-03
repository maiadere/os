use core::arch;

use aarch64_cpu::registers::{
    self, DAIF, ELR_EL1, ReadWriteable,
    SPSR_EL1::{self, A, D, F, I, IL, M, SS},
};
use registers::Writeable;

//stack starts at 0x8_0000 and grows downwards to 0x0
const STACK_BASE: usize = 0x8_0000;
/// loads a slice of bytes containing machine code into userspace memory, sets up the program state
/// then jumps to it in el0
pub fn load_program(code: &[u8]) -> ! {
    for (offset, byte) in code.iter().enumerate() {
        unsafe {
            ((STACK_BASE + offset) as *mut u8).write_volatile(*byte);
        }
    }
    // TODO: zero out bss

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
