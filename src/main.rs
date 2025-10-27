#![no_main]
#![no_std]

mod boot;
mod cpu;
mod driver;

use core::panic::PanicInfo;

use crate::driver::uart::UARTDriver;

fn kernel_main() -> ! {
    unsafe {
        UARTDriver::initialize();
        UARTDriver::write_str("hi :3");
    }
    panic!();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    cpu::spin_forever()
}
