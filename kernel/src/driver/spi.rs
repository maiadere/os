use crate::{
    driver::{PERIPHERAL_BASE, gpio},
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

fn sync_spi_write(bytes: &[u32]) {
    for byte in bytes {
        loop {
            if unsafe { can_tx_fifo_accept_data() } {
                break;
            }
        }
        // log!(100;"writing {:x} to sdspi\n", *byte);
        unsafe { write_mmio(SPI0_FIFO, *byte) };
        loop {
            if unsafe { rx_fifo_contains_data() } {
                break;
            }
        }
        let res = unsafe { read_mmio(SPI0_FIFO) };
        if res != 0 {
            log!(100;"got non zero return");
        }
        // log!(100; "sdspi returned {}\n", res);
    }
}

pub unsafe fn initialize_spi0() {
    // it seems that cpol and cpha should be set to 0 which is the default
    // cspol should also be set to 0 which means active low
    //
    // i think the core clock is at 500 Mhz and so the divisor to get 400 khz would be 1250, gonna
    // set it to 1300 to be safe
    log!(100;"clock div");
    unsafe { write_mmio(SPI0_CLK_DIVIDER, 1300) }

    gpio::set_pin_mode(7, gpio::PinMode::Alt0); //spi0 chip select 1
    gpio::set_pin_mode(8, gpio::PinMode::Alt0); //spi0 chip select 0
    gpio::set_pin_mode(9, gpio::PinMode::Alt0); //spi0 miso
    gpio::set_pin_mode(10, gpio::PinMode::Alt0); //spi0 mosi
    gpio::set_pin_mode(11, gpio::PinMode::Alt0); //spi0 clock

    let control_reg = unsafe { read_mmio(SPI0_CONTROL_AND_STATUS) };
    let control_reg = control_reg | 1 << 7; //ta enable
    unsafe { write_mmio(SPI0_CONTROL_AND_STATUS, control_reg) };

    let control_reg = unsafe { read_mmio(SPI0_CONTROL_AND_STATUS) };
    log!(100; "control reg contents: {:32b}\n", control_reg);

    let necessary_bytes = [0xffffffff; 20];
    sync_spi_write(&necessary_bytes);

    //enable and disable necessary
    let control_reg = unsafe { read_mmio(SPI0_CONTROL_AND_STATUS) };
    let control_reg = control_reg & !(1 << 7); //ta disable
    unsafe { write_mmio(SPI0_CONTROL_AND_STATUS, control_reg) };
    let control_reg = unsafe { read_mmio(SPI0_CONTROL_AND_STATUS) };
    let control_reg = control_reg | 1 << 7; //ta enable
    unsafe { write_mmio(SPI0_CONTROL_AND_STATUS, control_reg) };

    // binary cmd0: 0b01_000000_0000000000000000000000000000_1001010_1
    let cmd0 = [0x40000000, 0x0095ffff];
    for _ in 0..100 {
        sync_spi_write(&cmd0);
        sync_spi_write(&necessary_bytes); // wait for response
    }
    let cmd8 = [0x48000001, 0xaa87ffff];
    for _ in 0..100 {
        sync_spi_write(&cmd8);
        sync_spi_write(&necessary_bytes); // wait for response
    }
}
