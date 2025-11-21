use crate::mmio::{read_mmio, write_mmio};

use crate::driver::PERIPHERAL_BASE;

const MAILBOX_BASE: u64 = PERIPHERAL_BASE + 0xb880;
const MAILBOX_READ: u64 = MAILBOX_BASE + 0x00;
const MAILBOX_PEEK: u64 = MAILBOX_BASE + 0x10;
const MAILBOX_SENDER: u64 = MAILBOX_BASE + 0x14;
const MAILBOX_STATUS: u64 = MAILBOX_BASE + 0x18;
const MAILBOX_CONFIG: u64 = MAILBOX_BASE + 0x1c;
const MAILBOX_WRITE: u64 = MAILBOX_BASE + 0x20;

const MAILBOX_FULL: u32 = 0x8000_0000;
const MAILBOX_EMPTY: u32 = 0x4000_0000;

#[derive(Debug, Clone, Copy)]
pub enum MailboxChannel {
    PowerManagement,
    #[deprecated]
    Framebuffer,
    VirtualUART,
    VCHIQ,
    LED,
    Button,
    TouchScreen,
    Property,
}

impl MailboxChannel {
    pub fn bits(&self) -> u8 {
        #[allow(deprecated)]
        match self {
            MailboxChannel::PowerManagement => 0,
            MailboxChannel::Framebuffer => 1,
            MailboxChannel::VirtualUART => 2,
            MailboxChannel::VCHIQ => 3,
            MailboxChannel::LED => 4,
            MailboxChannel::Button => 5,
            MailboxChannel::TouchScreen => 6,
            MailboxChannel::Property => 8,
        }
    }
}

/// TODO: add memory barriers
pub fn mailbox_send(channel: MailboxChannel, data: u32) {
    unsafe {
        while read_mmio(MAILBOX_STATUS) & MAILBOX_FULL != 0 {}
        write_mmio(MAILBOX_WRITE, (data << 4) | channel.bits() as u32);
    }
}

/// TODO: add memory barriers
pub fn mailbox_receive(channel: MailboxChannel) -> u32 {
    loop {
        let data = unsafe {
            while read_mmio(MAILBOX_STATUS) & MAILBOX_EMPTY != 0 {}
            read_mmio(MAILBOX_READ)
        };

        if (data & 0xf) as u8 == channel.bits() {
            return data >> 4;
        }
    }
}
