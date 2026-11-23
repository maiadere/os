#![no_std]
#![no_main]

#[unsafe(no_mangle)]
fn main() {
    gcapi::print("hello from the shell\n");
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    gcapi::print("shell panicked\n");
    gcapi::exit()
}
