/// Specifies the ID of the boot core needed for the arch specific code
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text._start_arguments")]
pub static BOOT_CORE_ID: u64 = 0;
