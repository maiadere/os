use aarch64_cpu::asm;

use crate::driver::gpio::{self, PinMode, PullMode};
use crate::mmio::{read_mmio, write_mmio};

mod reg {
    use crate::driver::REGISTER_BASE_OFFSET;

    pub const UART0_REGISTER_BASE: u64 = 0xfe20_1000 + REGISTER_BASE_OFFSET;
    pub const DR: u64 = UART0_REGISTER_BASE + 0x00;
    pub const RSRECR: u64 = UART0_REGISTER_BASE + 0x04;
    pub const FR: u64 = UART0_REGISTER_BASE + 0x18;
    pub const ILPR: u64 = UART0_REGISTER_BASE + 0x20;
    pub const IBRD: u64 = UART0_REGISTER_BASE + 0x24;
    pub const FBRD: u64 = UART0_REGISTER_BASE + 0x28;
    pub const LCRH: u64 = UART0_REGISTER_BASE + 0x2c;
    pub const CR: u64 = UART0_REGISTER_BASE + 0x30;
    pub const IFLS: u64 = UART0_REGISTER_BASE + 0x34;
    pub const IMSC: u64 = UART0_REGISTER_BASE + 0x38;
    pub const RIS: u64 = UART0_REGISTER_BASE + 0x3c;
    pub const MIS: u64 = UART0_REGISTER_BASE + 0x40;
    pub const ICR: u64 = UART0_REGISTER_BASE + 0x44;
    pub const DMACR: u64 = UART0_REGISTER_BASE + 0x48;
    pub const ITCR: u64 = UART0_REGISTER_BASE + 0x80;
    pub const ITIP: u64 = UART0_REGISTER_BASE + 0x84;
    pub const ITOP: u64 = UART0_REGISTER_BASE + 0x88;
    pub const TDR: u64 = UART0_REGISTER_BASE + 0x8c;
}

/// Initializes UART0.
pub fn init() {
    // Disable UART0
    unsafe { write_mmio(reg::CR, 0) };

    unsafe { wait_for_tx() };

    gpio::set_pin_mode(14, PinMode::Alt0);
    gpio::set_pin_mode(15, PinMode::Alt0);
    gpio::set_pin_mode(16, PinMode::Alt3);
    gpio::set_pin_mode(17, PinMode::Alt3);

    gpio::set_pull_mode(14, PullMode::NoPull);
    gpio::set_pull_mode(15, PullMode::PullUp);
    gpio::set_pull_mode(16, PullMode::PullDown);
    gpio::set_pull_mode(17, PullMode::NoPull);

    // Set baud rate divisor
    // BAUDDIV = (FUARTCLK/(16 * Baud rate))
    //         = (48000000/(16 * 115200))
    //         = 26.0416666667
    asm::barrier::dmb(asm::barrier::ST);
    unsafe { write_mmio(reg::IBRD, 26) };
    unsafe { write_mmio(reg::FBRD, 3) };

    // Set word length to 8 bits and flush the transmit FIFO
    unsafe { write_mmio(reg::LCRH, 0b11 << 5) };

    // Enable UART0
    unsafe { write_mmio(reg::CR, 0b1100000001) };
}

/// Writes a string slice with UART0.
///
/// Before this function is called, UART0 must be initialized with [`init`].
pub fn write_str(s: &str) {
    asm::barrier::dmb(asm::barrier::ST);
    unsafe {
        for &byte in s.as_bytes() {
            write_u8(byte);
        }
    }
    asm::barrier::dmb(asm::barrier::SY);
}

unsafe fn wait_for_tx() {
    unsafe { while (read_mmio(reg::FR) >> 3) & 1 != 0 {} }
}

unsafe fn write_u8(value: u8) {
    unsafe {
        wait_for_tx();
        write_mmio(reg::DR, value as u32);
    }
}
