use crate::driver::{
    gpio::{self, GPIODriver, GPIOPinMode, GPIOPinPullMode},
    read_mmio, write_mmio,
};

pub struct MiniUARTDriver {}

impl MiniUARTDriver {
    const AUX_REGISTER_BASE: u64 = 0x7e21_5000;
    const AUX_IRQ: u64 = 0x00;
    const AUX_ENABLES: u64 = 0x04;
    const AUX_MU_IO_REG: u64 = 0x40;
    const AUX_MU_IER_REG: u64 = 0x44;
    const AUX_MU_IIR_REG: u64 = 0x48;
    const AUX_MU_LCR_REG: u64 = 0x4c;
    const AUX_MU_MCR_REG: u64 = 0x50;
    const AUX_MU_LSR_REG: u64 = 0x54;
    const AUX_MU_MSR_REG: u64 = 0x58;
    const AUX_MU_SCRATCH: u64 = 0x5c;
    const AUX_MU_CNTL_REG: u64 = 0x60;
    const AUX_MU_STAT_REG: u64 = 0x64;
    const AUX_MU_BAUD_REG: u64 = 0x68;
    const AUX_SPI1_CNTL0_REG: u64 = 0x80;
    const AUX_SPI1_CNTL1_REG: u64 = 0x84;
    const AUX_SPI1_STAT_REG: u64 = 0x88;
    const AUX_SPI1_PEEK_REG: u64 = 0x8c;
    const AUX_SPI1_IO_REGa: u64 = 0xa0;
    const AUX_SPI1_IO_REGb: u64 = 0xa4;
    const AUX_SPI1_IO_REGc: u64 = 0xa8;
    const AUX_SPI1_IO_REGd: u64 = 0xac;
    const AUX_SPI1_TXHOLD_REGa: u64 = 0xb0;
    const AUX_SPI1_TXHOLD_REGb: u64 = 0xb4;
    const AUX_SPI1_TXHOLD_REGc: u64 = 0xb8;
    const AUX_SPI1_TXHOLD_REGd: u64 = 0xbc;
    const AUX_SPI2_CNTL0_REG: u64 = 0xc0;
    const AUX_SPI2_CNTL1_REG: u64 = 0xc4;
    const AUX_SPI2_STAT_REG: u64 = 0xc8;
    const AUX_SPI2_PEEK_REG: u64 = 0xcc;
    const AUX_SPI2_IO_REGa: u64 = 0xe0;
    const AUX_SPI2_IO_REGb: u64 = 0xe4;
    const AUX_SPI2_IO_REGc: u64 = 0xe8;
    const AUX_SPI2_IO_REGd: u64 = 0xec;
    const AUX_SPI2_TXHOLD_REGa: u64 = 0xf0;
    const AUX_SPI2_TXHOLD_REGb: u64 = 0xf4;
    const AUX_SPI2_TXHOLD_REGc: u64 = 0xf8;
    const AUX_SPI2_TXHOLD_REGd: u64 = 0xfc;

    pub unsafe fn initialize_mini_uart() -> () {
        unsafe {
            GPIODriver::set_pin_mode(14, GPIOPinMode::Alt5);
            GPIODriver::set_pin_mode(15, GPIOPinMode::Alt5);
            GPIODriver::set_pin_mode(16, GPIOPinMode::Alt5);
            GPIODriver::set_pin_mode(17, GPIOPinMode::Alt5);

            GPIODriver::set_pin_pull_resistor(14, GPIOPinPullMode::NoPull);
            GPIODriver::set_pin_pull_resistor(15, GPIOPinPullMode::NoPull);

            Self::enable_mini_uart()
        }
    }

    unsafe fn enable_mini_uart() -> () {
        let addr = Self::AUX_REGISTER_BASE + Self::AUX_ENABLES;
        let register_value = unsafe { read_mmio(addr) };
        let register_value = register_value | 1; //set bit 0
        unsafe {
            write_mmio(addr, register_value);
        }
    }
}
