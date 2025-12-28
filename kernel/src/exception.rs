mod low_level_handler;

use core::arch::{asm, global_asm};
global_asm!(include_str!("./exception/el1_exception_vector.s"));

pub unsafe fn set_vbar_el1() {
    unsafe {
        // out parameter with wildcard to get a register from rustc that is safe to clobber
        asm!("adr {0}, el1_vector_table","msr VBAR_EL1, {0}", out(reg) _);
    }
}
