use crate::{
    driver::{PERIPHERAL_BASE, gpio, timer},
    log,
    mmio::{read_mmio, write_mmio},
};

const SPI0_REGISTER_BASE: u64 = PERIPHERAL_BASE + 0x204000;
const SPI0_CONTROL_AND_STATUS: u64 = SPI0_REGISTER_BASE + 0x00;
const SPI0_FIFO: u64 = SPI0_REGISTER_BASE + 0x04;
const SPI0_CLK_DIVIDER: u64 = SPI0_REGISTER_BASE + 0x08;
const SPI0_DATA_LENGTH: u64 = SPI0_REGISTER_BASE + 0x0c;
const SPI0_LOSSI_MODE_TOH: u64 = SPI0_REGISTER_BASE + 0x10;
const SPI0_DMA_DREQ_CONTROL: u64 = SPI0_REGISTER_BASE + 0x14;

unsafe fn can_tx_fifo_accept_data() -> bool {
    let control_reg = unsafe { read_mmio(SPI0_CONTROL_AND_STATUS) };
    return control_reg & (1 << 18) != 0;
}
unsafe fn rx_fifo_contains_data() -> bool {
    let control_reg = unsafe { read_mmio(SPI0_CONTROL_AND_STATUS) };
    return control_reg & (1 << 17) != 0;
}

pub fn spi_write_bytes_sync(bytes: &[u8]) {
    for byte in bytes {
        // log!(100;"writing {:x} to sdspi\n", *byte);
        let _res = spi_send_byte_sync(*byte);
        // log!(100; "sdspi returned {:x}\n", res);
    }
}

pub fn spi_send_byte_sync(byte: u8) -> u8 {
    loop {
        if unsafe { can_tx_fifo_accept_data() } {
            break;
        }
    }
    unsafe { write_mmio(SPI0_FIFO, byte as u32) };
    loop {
        if unsafe { rx_fifo_contains_data() } {
            break;
        }
    }
    // mmio reads from SPI in single byte mode will always fit in u8
    unsafe { read_mmio(SPI0_FIFO) as u8 }
}

pub unsafe fn spi0_set_ta(enable: bool) {
    if enable {
        let control_reg = unsafe { read_mmio(SPI0_CONTROL_AND_STATUS) };
        let control_reg = control_reg | 1 << 7; //ta enable
        unsafe { write_mmio(SPI0_CONTROL_AND_STATUS, control_reg) };
    } else {
        let control_reg = unsafe { read_mmio(SPI0_CONTROL_AND_STATUS) };
        let control_reg = control_reg & !(1 << 7); //ta disable
        unsafe { write_mmio(SPI0_CONTROL_AND_STATUS, control_reg) };
    }
}

/// initializes the spi0 into a mode which allows communication with an sd card
pub unsafe fn initialize_spi0() {
    // it seems that cpol and cpha should be set to 0 which is the default
    // cspol should also be set to 0 which means active low
    //
    // i think the core clock is at 500 Mhz and so the divisor to get 400 khz would be 1250, gonna
    // set it to 1500 to be safe
    unsafe { write_mmio(SPI0_CLK_DIVIDER, 1500) }

    gpio::set_pin_mode(7, gpio::PinMode::Alt0); //spi0 chip select 1
    gpio::set_pin_mode(8, gpio::PinMode::Alt0); //spi0 chip select 0
    gpio::set_pin_mode(9, gpio::PinMode::Alt0); //spi0 miso
    gpio::set_pin_mode(10, gpio::PinMode::Alt0); //spi0 mosi
    gpio::set_pin_mode(11, gpio::PinMode::Alt0); //spi0 clock

    unsafe { spi0_set_ta(true) };

    // let control_reg = unsafe { read_mmio(SPI0_CONTROL_AND_STATUS) };
    // log!(100; "control reg contents: {:32b}\n", control_reg);
}
