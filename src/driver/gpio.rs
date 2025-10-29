use aarch64_cpu::asm;

use crate::driver::{read_mmio, write_mmio};

pub enum PinMode {
    Input,
    Output,
    Alt0,
    Alt1,
    Alt2,
    Alt3,
    Alt4,
    Alt5,
}

impl PinMode {
    pub fn bit_pattern(&self) -> u8 {
        match self {
            PinMode::Input => 0b000,
            PinMode::Output => 0b001,
            PinMode::Alt0 => 0b100,
            PinMode::Alt1 => 0b101,
            PinMode::Alt2 => 0b110,
            PinMode::Alt3 => 0b111,
            PinMode::Alt4 => 0b011,
            PinMode::Alt5 => 0b010,
        }
    }
}

pub enum PullMode {
    NoPull,
    PullUp,
    PullDown,
}

impl PullMode {
    pub fn bit_pattern(&self) -> u8 {
        match self {
            PullMode::NoPull => 0b00,
            PullMode::PullUp => 0b01,
            PullMode::PullDown => 0b10,
        }
    }
}

mod reg {
    pub const GPIO_REGISTER_BASE: u64 = 0xfe20_0000;
    pub const GPFSEL0: u64 = GPIO_REGISTER_BASE + 0x00;
    pub const GPFSEL1: u64 = GPIO_REGISTER_BASE + 0x04;
    pub const GPFSEL2: u64 = GPIO_REGISTER_BASE + 0x08;
    pub const GPFSEL3: u64 = GPIO_REGISTER_BASE + 0x0c;
    pub const GPFSEL4: u64 = GPIO_REGISTER_BASE + 0x10;
    pub const GPFSEL5: u64 = GPIO_REGISTER_BASE + 0x14;
    pub const GPSET0: u64 = GPIO_REGISTER_BASE + 0x1c;
    pub const GPSET1: u64 = GPIO_REGISTER_BASE + 0x20;
    pub const GPCLR0: u64 = GPIO_REGISTER_BASE + 0x28;
    pub const GPCLR1: u64 = GPIO_REGISTER_BASE + 0x2c;
    pub const GPLEV0: u64 = GPIO_REGISTER_BASE + 0x34;
    pub const GPLEV1: u64 = GPIO_REGISTER_BASE + 0x38;
    pub const GPEDS0: u64 = GPIO_REGISTER_BASE + 0x40;
    pub const GPEDS1: u64 = GPIO_REGISTER_BASE + 0x44;
    pub const GPREN0: u64 = GPIO_REGISTER_BASE + 0x4c;
    pub const GPREN1: u64 = GPIO_REGISTER_BASE + 0x50;
    pub const GPFEN0: u64 = GPIO_REGISTER_BASE + 0x58;
    pub const GPFEN1: u64 = GPIO_REGISTER_BASE + 0x5c;
    pub const GPHEN0: u64 = GPIO_REGISTER_BASE + 0x64;
    pub const GPHEN1: u64 = GPIO_REGISTER_BASE + 0x68;
    pub const GPLEN0: u64 = GPIO_REGISTER_BASE + 0x70;
    pub const GPLEN1: u64 = GPIO_REGISTER_BASE + 0x74;
    pub const GPAREN0: u64 = GPIO_REGISTER_BASE + 0x7c;
    pub const GPAREN1: u64 = GPIO_REGISTER_BASE + 0x80;
    pub const GPAFEN0: u64 = GPIO_REGISTER_BASE + 0x88;
    pub const GPAFEN1: u64 = GPIO_REGISTER_BASE + 0x8c;
    pub const GPIO_PUP_PDN_CNTRL_REG0: u64 = GPIO_REGISTER_BASE + 0xe4;
    pub const GPIO_PUP_PDN_CNTRL_REG1: u64 = GPIO_REGISTER_BASE + 0xe8;
    pub const GPIO_PUP_PDN_CNTRL_REG2: u64 = GPIO_REGISTER_BASE + 0xec;
    pub const GPIO_PUP_PDN_CNTRL_REG3: u64 = GPIO_REGISTER_BASE + 0xf0;
}

/// Sets the mode in which the GPIO pin should operate.
pub fn set_pin_mode(pin_idx: usize, mode: PinMode) {
    let reg_addr: u64 = match pin_idx {
        0..=9 => reg::GPFSEL0,
        10..=19 => reg::GPFSEL1,
        20..=29 => reg::GPFSEL2,
        30..=39 => reg::GPFSEL3,
        40..=49 => reg::GPFSEL4,
        50..=57 => reg::GPFSEL5,
        _ => {
            panic!("GPIO pin index out of range");
        }
    };

    let reg_state = unsafe { read_mmio(reg_addr) };
    asm::barrier::dmb(asm::barrier::SY);

    let bit_offset = (pin_idx % 10) * 3;

    let zeroing_mask: u32 = !(0b111 << bit_offset);
    let reg_state = reg_state & zeroing_mask;
    let bits_to_set = (mode.bit_pattern() as u32) << bit_offset;
    let reg_state = reg_state | bits_to_set;

    unsafe {
        asm::barrier::dmb(asm::barrier::ST);
        write_mmio(reg_addr, reg_state);
    }
}

/// Configures the internal pull-up/down resistors for specified GPIO pin.
pub fn set_pull_mode(pin_idx: usize, mode: PullMode) {
    let reg_addr: u64 = match pin_idx {
        0..=15 => reg::GPIO_PUP_PDN_CNTRL_REG0,
        16..=31 => reg::GPIO_PUP_PDN_CNTRL_REG1,
        32..=47 => reg::GPIO_PUP_PDN_CNTRL_REG2,
        48..=57 => reg::GPIO_PUP_PDN_CNTRL_REG3,
        _ => {
            panic!("GPIO pin index out of range");
        }
    };

    let reg_state = unsafe { read_mmio(reg_addr) };
    asm::barrier::dmb(asm::barrier::SY);

    let bit_offset = (pin_idx % 16) * 2;

    let zeroing_mask: u32 = !(0b11 << bit_offset);
    let reg_state = reg_state & zeroing_mask;
    let bits_to_set = (mode.bit_pattern() as u32) << bit_offset;
    let reg_state = reg_state | bits_to_set;

    unsafe {
        asm::barrier::dmb(asm::barrier::ST);
        write_mmio(reg_addr, reg_state);
    }
}
