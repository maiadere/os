use core::arch::global_asm;

global_asm!(include_str!(
    "./boot/boot.s"),
    CORE_ID_BITMASK = const 0b11,
    BOOT_CORE_ID = const 0b0
);

// called from the handwritten assembly then jumps into the kernel entrypoint
#[unsafe(no_mangle)]
pub unsafe fn _start_rust() -> ! {
    unsafe { crate::kernel_init() }
}
