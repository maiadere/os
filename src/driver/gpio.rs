use crate::driver::{self, read_mmio, write_mmio};

struct GPIODriver {}

pub enum GPIOPinMode {
    Input,
    Output,
    Alt0,
    Alt1,
    Alt2,
    Alt3,
    Alt4,
    Alt5,
}
impl GPIOPinMode {
    pub fn bit_pattern(&self) -> u8 {
        match self {
            GPIOPinMode::Input => 0b000,
            GPIOPinMode::Output => 0b001,
            GPIOPinMode::Alt0 => 0b100,
            GPIOPinMode::Alt1 => 0b101,
            GPIOPinMode::Alt2 => 0b110,
            GPIOPinMode::Alt3 => 0b111,
            GPIOPinMode::Alt4 => 0b011,
            GPIOPinMode::Alt5 => 0b010,
        }
    }
}

pub enum GPIOPinPullMode {
    NoPull,
    PullUp,
    PullDown,
}

impl GPIOPinPullMode {
    pub fn bit_pattern(&self) -> u8 {
        match self {
            GPIOPinPullMode::NoPull => 0b00,
            GPIOPinPullMode::PullUp => 0b01,
            GPIOPinPullMode::PullDown => 0b10,
        }
    }
}

//TODO: MAKE THE ACCESS TO THIS DRIVER REQUIRE HOLDING A MUTEX
impl GPIODriver {
    // address of the memory mapped gpio registers using the full 35 bit addressing
    const GPIO_REGISTER_BASE: u64 = 0x7e20_0000;
    const GPFSEL0: u64 = 0x00;
    const GPFSEL1: u64 = 0x04;
    const GPFSEL2: u64 = 0x08;
    const GPFSEL3: u64 = 0x0c;
    const GPFSEL4: u64 = 0x10;
    const GPFSEL5: u64 = 0x14;
    const GPSET0: u64 = 0x1c;
    const GPSET1: u64 = 0x20;
    const GPCLR0: u64 = 0x28;
    const GPCLR1: u64 = 0x2c;
    const GPLEV0: u64 = 0x34;
    const GPLEV1: u64 = 0x38;
    const GPEDS0: u64 = 0x40;
    const GPEDS1: u64 = 0x44;
    const GPREN0: u64 = 0x4c;
    const GPREN1: u64 = 0x50;
    const GPFEN0: u64 = 0x58;
    const GPFEN1: u64 = 0x5c;
    const GPHEN0: u64 = 0x64;
    const GPHEN1: u64 = 0x68;
    const GPLEN0: u64 = 0x70;
    const GPLEN1: u64 = 0x74;
    const GPAREN0: u64 = 0x7c;
    const GPAREN1: u64 = 0x80;
    const GPAFEN0: u64 = 0x88;
    const GPAFEN1: u64 = 0x8c;
    const GPIO_PUP_PDN_CNTRL_REG0: u64 = 0xe4;
    const GPIO_PUP_PDN_CNTRL_REG1: u64 = 0xe8;
    const GPIO_PUP_PDN_CNTRL_REG2: u64 = 0xec;
    const GPIO_PUP_PDN_CNTRL_REG3: u64 = 0xf0;

    pub unsafe fn set_pin_mode(pin_idx: usize, mode: GPIOPinMode) -> () {
        let register_offset: u64 = match pin_idx {
            0..=9 => Self::GPFSEL0,
            10..=19 => Self::GPFSEL1,
            20..=29 => Self::GPFSEL2,
            30..=39 => Self::GPFSEL3,
            40..=49 => Self::GPFSEL4,
            50..=57 => Self::GPFSEL5,
            _ => {
                panic!("GPIO pin index out of range");
            }
        };
        let register_addr = Self::GPIO_REGISTER_BASE + register_offset;
        let register_state = unsafe { read_mmio(register_addr) };

        let bit_offset = (pin_idx % 10) * 3;

        let zeroing_mask: u32 = !(0b111 << bit_offset);
        let register_state = register_state & zeroing_mask;
        let bits_to_set = (mode.bit_pattern() as u32) << bit_offset;
        let register_state = register_state | bits_to_set;

        unsafe {
            write_mmio(register_addr, register_state);
        }
    }

    pub unsafe fn set_pin_pull_resistor(pin_idx: usize, mode: GPIOPinPullMode) -> () {
        let register_offset: u64 = match pin_idx {
            0..=15 => Self::GPIO_PUP_PDN_CNTRL_REG0,
            16..=31 => Self::GPIO_PUP_PDN_CNTRL_REG1,
            32..=47 => Self::GPIO_PUP_PDN_CNTRL_REG2,
            48..=57 => Self::GPIO_PUP_PDN_CNTRL_REG3,
            _ => {
                panic!("GPIO pin index out of range");
            }
        };
        let register_addr = Self::GPIO_REGISTER_BASE + register_offset;

        let register_state = unsafe { read_mmio(register_addr) };

        let bit_offset = (pin_idx % 16) * 2;

        let zeroing_mask: u32 = !(0b11 << bit_offset);
        let register_state = register_state & zeroing_mask;
        let bits_to_set = (mode.bit_pattern() as u32) << bit_offset;
        let register_state = register_state | bits_to_set;

        unsafe {
            write_mmio(register_addr, register_state);
        }
    }
}

