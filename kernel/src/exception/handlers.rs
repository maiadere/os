use aarch64_cpu::asm;
use heapless::format;

use crate::driver::uart0;
#[derive(Copy, Clone, Debug)]
#[repr(C)]

pub struct SavedRegisters {
    x0: u64,
    x1: u64,
    x2: u64,
    x3: u64,
    x4: u64,
    x5: u64,
    x6: u64,
    x7: u64,
    x8: u64,
    x9: u64,
    x10: u64,
    x11: u64,
    x12: u64,
    x13: u64,
    x14: u64,
    x15: u64,
    x16: u64,
    x17: u64,
    x18: u64,
    fp: u64,
    lr: u64,
    xzr: u64,
    esr: u64,
    far: u64,
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _synchronous_kernel_exception(register_contents: *mut SavedRegisters) {
    // assumes uart is initialized
    uart0::write_str("synchronous kernel exception\n");
    // safe as this should be set from the low level handler code
    let register_contents = unsafe { *register_contents };
    uart0::write_str(format!(512; "{:?}\n", register_contents).unwrap().as_str());
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _unhandled_exception() -> ! {
    // assumes uart is initialized
    panic!("unhandled exception raised, kernel panic");
    //returning from this is UB as it will clobber registers
}
