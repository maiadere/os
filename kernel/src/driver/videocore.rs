use crate::mmio::{read_mmio, write_mmio};

use crate::driver::PERIPHERAL_BASE;

const MAILBOX_BASE: u64 = PERIPHERAL_BASE + 0xb880;
const MAILBOX_READ: u64 = MAILBOX_BASE + 0x00;
const MAILBOX_PEEK: u64 = MAILBOX_BASE + 0x10;
const MAILBOX_SENDER: u64 = MAILBOX_BASE + 0x14;
const MAILBOX_STATUS: u64 = MAILBOX_BASE + 0x18;
const MAILBOX_CONFIG: u64 = MAILBOX_BASE + 0x1c;
const MAILBOX_WRITE: u64 = MAILBOX_BASE + 0x20;

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
    PropertyFromArm,
    PropertyFromVC,
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
            MailboxChannel::PropertyFromArm => 8,
            MailboxChannel::PropertyFromVC => 9,
        }
    }
}

/// TODO: add memory barriers
pub fn mailbox_send(channel: MailboxChannel, data: u32) {
    unsafe {
        const MAILBOX_FULL: u32 = 0x8000_0000;
        while read_mmio(MAILBOX_STATUS) & MAILBOX_FULL != 0 {}
        write_mmio(MAILBOX_WRITE, (data << 4) | channel.bits() as u32);
    }
}

/// TODO: add memory barriers
pub fn mailbox_receive(channel: MailboxChannel) -> u32 {
    loop {
        let data = unsafe {
            const MAILBOX_EMPTY: u32 = 0x4000_0000;
            while read_mmio(MAILBOX_STATUS) & MAILBOX_EMPTY != 0 {}
            read_mmio(MAILBOX_READ)
        };

        if (data & 0xf) as u8 == channel.bits() {
            return data >> 4;
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C, align(16))]
pub struct PropertyMessage<T> {
    /// Message size
    pub size: u32,
    /// Request/Response code
    pub code: u32,
    /// Concatenated list of tags
    pub tags: T,
    /// End tag (must be 0)
    pub end: u32,
}

impl<T> PropertyMessage<T> {
    pub fn new(tags: T) -> Self {
        Self {
            size: core::mem::size_of::<Self>() as u32,
            code: 0,
            tags,
            end: 0,
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C, align(4))]
pub struct PropertyTag<T> {
    /// Tag identity
    pub id: u32,
    /// Value buffer size
    pub size: u32,
    /// Request/Response code
    pub code: u32,
    /// Value buffer
    pub value: T,
}

impl<T> PropertyTag<T> {
    pub fn new(id: u32, value: T) -> Self {
        Self {
            id,
            size: core::mem::size_of::<T>() as u32,
            code: 0,
            value,
        }
    }
}

pub fn mailbox_property_send<Req, Res>(tags: Req) -> Option<Res> {
    let msg = PropertyMessage::new(tags);
    let data = (&msg as *const _ as u32) >> 4;

    mailbox_send(MailboxChannel::PropertyFromArm, data);

    if mailbox_receive(MailboxChannel::PropertyFromArm) != data {
        return None;
    }

    const REQUEST_SUCCESSFUL: u32 = 0x8000_0000;

    if msg.code != REQUEST_SUCCESSFUL {
        return None;
    }

    assert!(core::mem::size_of::<Req>() >= core::mem::size_of::<Res>());
    let response = unsafe { (&msg as *const _ as *const PropertyMessage<Res>).read_volatile() };
    Some(response.tags)
}
