#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/cpu.rs"]
mod arch_cpu;

//public reexport of architetcture specific code
pub use arch_cpu::spin_forever;

mod boot;
