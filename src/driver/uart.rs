use crate::driver::{
    gpio::{GPIODriver, GPIOPinMode, GPIOPinPullMode},
    read_mmio, write_mmio,
};

pub struct UARTDriver;

impl UARTDriver {
    const UART0_REGISTER_BASE: u64 = 0x7e201000;
    const DR: u64 = 0x00;
    const RSRECR: u64 = 0x04;
    const FR: u64 = 0x18;
    const ILPR: u64 = 0x20;
    const IBRD: u64 = 0x24;
    const FBRD: u64 = 0x28;
    const LCRH: u64 = 0x2c;
    const CR: u64 = 0x30;
    const IFLS: u64 = 0x34;
    const IMSC: u64 = 0x38;
    const RIS: u64 = 0x3c;
    const MIS: u64 = 0x40;
    const ICR: u64 = 0x44;
    const DMACR: u64 = 0x48;
    const ITCR: u64 = 0x80;
    const ITIP: u64 = 0x84;
    const ITOP: u64 = 0x88;
    const TDR: u64 = 0x8c;

    pub unsafe fn initialize() {
        unsafe {
            GPIODriver::set_pin_mode(14, GPIOPinMode::Alt0);
            GPIODriver::set_pin_mode(15, GPIOPinMode::Alt0);
            GPIODriver::set_pin_mode(16, GPIOPinMode::Alt3);
            GPIODriver::set_pin_mode(17, GPIOPinMode::Alt3);

            GPIODriver::set_pin_pull_resistor(14, GPIOPinPullMode::NoPull);
            GPIODriver::set_pin_pull_resistor(15, GPIOPinPullMode::PullUp);
            GPIODriver::set_pin_pull_resistor(16, GPIOPinPullMode::PullDown);
            GPIODriver::set_pin_pull_resistor(17, GPIOPinPullMode::NoPull);

            // Baud rate divisor
            // BAUDDIV = (FUARTCLK/(16 * Baud rate))
            //         = (48000000/(16 * 115200))
            //         = 26.0416666667
            write_mmio(Self::UART0_REGISTER_BASE + Self::IBRD, 26);
            write_mmio(Self::UART0_REGISTER_BASE + Self::FBRD, 3);

            // Set word length to 8 bits
            write_mmio(Self::UART0_REGISTER_BASE + Self::LCRH, 0b11 << 5);

            // Enable UART0
            write_mmio(Self::UART0_REGISTER_BASE + Self::CR, 0b1100000001);
        }
    }

    pub unsafe fn wait_for_tx() {
        unsafe { while (read_mmio(Self::UART0_REGISTER_BASE + Self::FR) >> 3) & 1 != 0 {} }
    }

    pub unsafe fn write_u8(value: u8) {
        unsafe {
            Self::wait_for_tx();
            write_mmio(Self::UART0_REGISTER_BASE + Self::DR, value as u32);
        }
    }

    pub unsafe fn write_str(s: &str) {
        unsafe {
            for &byte in s.as_bytes() {
                Self::write_u8(byte);
            }
        }
    }
}
