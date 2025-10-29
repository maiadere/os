use core::arch::global_asm;

global_asm!(include_str!(
    "./boot/boot.s"),
    CORE_ID_BITMASK = const 0b11,
    BOOT_CORE_ID = const 0b0
);

/// Jumps into the kernel entrypoint.
///
/// This function is called from the `boot.s` file.
#[unsafe(no_mangle)]
pub unsafe fn _start_rust() -> ! {
    crate::kernel_main()
}
