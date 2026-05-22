use core::ptr;

use crate::{
    driver::PERIPHERAL_BASE,
    mmio::{read_mmio, write_mmio},
};

pub const SDHCI_REGISTER_BASE: u64 = PERIPHERAL_BASE + 0x30_0000;
pub const PRESENT_STATE_REGISTER_BASE: u64 = SDHCI_REGISTER_BASE + 0x24;
pub const CLOCK_CONTROL_REGISTER_BASE: u64 = SDHCI_REGISTER_BASE + 0x2c;
pub const CAPABILITIES_REGISTER_BASE: u64 = SDHCI_REGISTER_BASE + 0x40;

pub fn test_capabilities() -> u64 {
    return unsafe {
        ptr::with_exposed_provenance::<u64>(CAPABILITIES_REGISTER_BASE as usize).read_volatile()
    };
}
pub fn test_presence() -> u32 {
    return unsafe {
        ptr::with_exposed_provenance::<u32>(PRESENT_STATE_REGISTER_BASE as usize).read_volatile()
    };
}

pub fn muxing_state() -> u32 {
    unsafe {
        write_mmio(0x20_00d0 + PERIPHERAL_BASE, 1);
    }
    return unsafe { read_mmio(0x20_00d0 + PERIPHERAL_BASE) };
}
