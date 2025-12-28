use aarch64_cpu::asm;

use crate::driver::uart0;

#[unsafe(no_mangle)]
unsafe extern "C" fn _unsafe_exception() {
    // assumes uart is initialized
    uart0::write_str("exception handled\n");
    //returning from this is UB as it could clobber all registers
    loop {
        asm::wfe();
    }
}
