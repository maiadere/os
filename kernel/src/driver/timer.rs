mod reg {
    use crate::{driver::PERIPHERAL_BASE, mmio::read_mmio};

    pub const BASE: u64 = PERIPHERAL_BASE + 0x3000;
    pub const CS: u64 = BASE + 0x00;
    pub const CLO: u64 = BASE + 0x04;
    pub const CHI: u64 = BASE + 0x08;
    pub const C0: u64 = BASE + 0x0c;
    pub const C1: u64 = BASE + 0x10;
    pub const C2: u64 = BASE + 0x14;
    pub const C3: u64 = BASE + 0x18;

    pub fn clo() -> u32 {
        unsafe { read_mmio(CLO) }
    }

    pub fn chi() -> u32 {
        unsafe { read_mmio(CHI) }
    }
}

pub fn get_system_time() -> u64 {
    let mut high = reg::chi();
    let mut low = reg::clo();

    if high != reg::chi() {
        high = reg::chi();
        low = reg::clo();
    }

    ((high as u64) << 32) | low as u64
}

pub fn delay_us(us: u64) {
    let start = get_system_time();
    while get_system_time() < start + us {}
}

pub fn delay_ms(ms: u32) {
    delay_us(ms as u64 * 1000);
}
