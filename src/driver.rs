pub mod gpio;

pub unsafe fn write_mmio(addr: u64, value: u32) -> () {
    unsafe { (addr as *mut u32).write_volatile(value) }
}
pub unsafe fn read_mmio(addr: u64) -> u32 {
    unsafe { (addr as *mut u32).read_volatile() }
}
