pub mod mailbox {
    use crate::driver::PERIPHERAL_BASE;
    use crate::mmio::{read_mmio, write_mmio};

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

    pub mod property {
        pub const GET_FIRMWARE_REVISION: u32 = 0x00000001;
        pub const GET_BOARD_MODEL: u32 = 0x00010001;
        pub const GET_BOARD_REVISION: u32 = 0x00010002;
        pub const GET_BOARD_MAC_ADDRESS: u32 = 0x00010003;
        pub const GET_BOARD_SERIAL: u32 = 0x00010004;
        pub const GET_ARM_MEMORY: u32 = 0x00010005;
        pub const GET_VC_MEMORY: u32 = 0x00010006;
        pub const GET_CLOCKS: u32 = 0x00010007;
        pub const GET_COMMAND_LINE: u32 = 0x00050001;
        pub const GET_DMA_CHANNELS: u32 = 0x00060001;
        pub const GET_POWER_STATE: u32 = 0x00020001;
        pub const GET_TIMING: u32 = 0x00020002;
        pub const SET_POWER_STATE: u32 = 0x00028001;
        pub const GET_CLOCK_STATE: u32 = 0x00030001;
        pub const SET_CLOCK_STATE: u32 = 0x00038001;
        pub const GET_CLOCK_RATE: u32 = 0x00030002;
        pub const GET_ONBOARD_LED_STATUS: u32 = 0x00030041;
        pub const TEST_ONBOARD_LED_STATUS: u32 = 0x00034041;
        pub const SET_ONBOARD_LED_STATUS: u32 = 0x00038041;
        pub const GET_CLOCK_RATE_MEASURED: u32 = 0x00030047;
        pub const SET_CLOCK_RATE: u32 = 0x00038002;
        pub const GET_MAX_CLOCK_RATE: u32 = 0x00030004;
        pub const GET_MIN_CLOCK_RATE: u32 = 0x00030007;
        pub const GET_TURBO: u32 = 0x00030009;
        pub const SET_TURBO: u32 = 0x00038009;
        pub const GET_VOLTAGE: u32 = 0x00030003;
        pub const SET_VOLTAGE: u32 = 0x00038003;
        pub const GET_MAX_VOLTAGE: u32 = 0x00030005;
        pub const GET_MIN_VOLTAGE: u32 = 0x00030008;
        pub const GET_TEMPERATURE: u32 = 0x00030006;
        pub const GET_MAX_TEMPERATURE: u32 = 0x0003000a;
        pub const ALLOCATE_MEMORY: u32 = 0x0003000c;
        pub const LOCK_MEMORY: u32 = 0x0003000d;
        pub const UNLOCK_MEMORY: u32 = 0x0003000e;
        pub const RELEASE_MEMORY: u32 = 0x0003000f;
        pub const EXECUTE_CODE: u32 = 0x00030010;
        pub const GET_DISPMANX_RESOURCE_MEM_HANDLE: u32 = 0x00030014;
        pub const GET_EDID_BLOCK: u32 = 0x00030020;
        pub const ALLOCATE_BUFFER: u32 = 0x00040001;
        pub const RELEASE_BUFFER: u32 = 0x00048001;
        pub const BLANK_SCREEN: u32 = 0x00040002;
        pub const GET_PHYSICAL_RESOLUTION: u32 = 0x00040003;
        pub const TEST_PHYSICAL_RESOLUTION: u32 = 0x00044003;
        pub const SET_PHYSICAL_RESOLUTION: u32 = 0x00048003;
        pub const GET_VIRTUAL_RESOLUTION: u32 = 0x00040004;
        pub const TEST_VIRTUAL_RESOLUTION: u32 = 0x00044004;
        pub const SET_VIRTUAL_RESOLUTION: u32 = 0x00048004;
        pub const GET_DEPTH: u32 = 0x00040005;
        pub const TEST_DEPTH: u32 = 0x00044005;
        pub const SET_DEPTH: u32 = 0x00048005;
        pub const GET_PIXEL_ORDER: u32 = 0x00040006;
        pub const TEST_PIXEL_ORDER: u32 = 0x00044006;
        pub const SET_PIXEL_ORDER: u32 = 0x00048006;
        pub const GET_ALPHA_MODE: u32 = 0x00040007;
        pub const TEST_ALPHA_MODE: u32 = 0x00044007;
        pub const SET_ALPHA_MODE: u32 = 0x00048007;
        pub const GET_PITCH: u32 = 0x00040008;
        pub const GET_VIRTUAL_OFFSET: u32 = 0x00040009;
        pub const TEST_VIRTUAL_OFFSET: u32 = 0x00044009;
        pub const SET_VIRTUAL_OFFSET: u32 = 0x00048009;
        pub const GET_OVERSCAN: u32 = 0x0004000a;
        pub const TEST_OVERSCAN: u32 = 0x0004400a;
        pub const SET_OVERSCAN: u32 = 0x0004800a;
        pub const GET_PALETTE: u32 = 0x0004000b;
        pub const TEST_PALETTE: u32 = 0x0004400b;
        pub const SET_PALETTE: u32 = 0x0004800b;
        pub const SET_CURSOR_INFO: u32 = 0x00008010;
        pub const SET_CURSOR_STATE: u32 = 0x00008011;
    }

    pub fn mailbox_property_send<T>(tags: T) -> Option<T> {
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

        Some(msg.tags)
    }
}

pub mod framebuffer {
    use crate::driver::{
        UPPER_HALF_OFFSET,
        videocore::mailbox::{
            PropertyTag, mailbox_property_send,
            property::{
                ALLOCATE_BUFFER, GET_PITCH, SET_DEPTH, SET_PHYSICAL_RESOLUTION, SET_PIXEL_ORDER,
                SET_VIRTUAL_OFFSET, SET_VIRTUAL_RESOLUTION,
            },
        },
    };

    #[derive(Debug, Clone, Copy)]
    pub struct Color {
        r: f32,
        g: f32,
        b: f32,
    }

    impl Color {
        pub fn new(r: f32, g: f32, b: f32) -> Self {
            Self { r, g, b }
        }

        pub fn as_rgba32(&self) -> u32 {
            let mut color = 0;
            color |= ((255.0 * self.r.clamp(0.0, 1.0)) as u32) << 0;
            color |= ((255.0 * self.g.clamp(0.0, 1.0)) as u32) << 8;
            color |= ((255.0 * self.b.clamp(0.0, 1.0)) as u32) << 16;
            color
        }

        pub fn as_bgra32(&self) -> u32 {
            let mut color = 0;
            color |= ((255.0 * self.b.clamp(0.0, 1.0)) as u32) << 0;
            color |= ((255.0 * self.g.clamp(0.0, 1.0)) as u32) << 8;
            color |= ((255.0 * self.r.clamp(0.0, 1.0)) as u32) << 16;
            color
        }
    }

    #[derive(Debug, Clone)]
    pub struct Framebuffer {
        pub addr: u32,
        pub size: u32,
        pub pitch: u32,
        pub phys_res: (u32, u32),
        pub virt_res: (u32, u32),
        pub depth: u32,
        pub order: u32,
    }

    impl Framebuffer {
        pub fn init(
            width: u32,
            height: u32,
            depth: u32,
            order: u32,
        ) -> Result<Framebuffer, &'static str> {
            let (phys_res, virt_res, _, depth, order, buffer, pitch) = mailbox_property_send((
                PropertyTag::new(SET_PHYSICAL_RESOLUTION, (width, height)),
                PropertyTag::new(SET_VIRTUAL_RESOLUTION, (width, height)),
                PropertyTag::new(SET_VIRTUAL_OFFSET, (0, 0)),
                PropertyTag::new(SET_DEPTH, depth),
                PropertyTag::new(SET_PIXEL_ORDER, order),
                PropertyTag::new(ALLOCATE_BUFFER, (4096u32, 0u32)),
                PropertyTag::new(GET_PITCH, 0u32),
            ))
            .ok_or("failed to send a mailbox")?;

            let fb = Framebuffer {
                addr: buffer.value.0 & 0x3fff_ffff,
                size: buffer.value.1,
                pitch: pitch.value,
                phys_res: phys_res.value,
                virt_res: virt_res.value,
                depth: depth.value,
                order: order.value,
            };

            if fb.addr == 0 || fb.size == 0 {
                return Err("failed to allocate buffer");
            }

            if fb.depth == 0 {
                return Err("unsupported color depth");
            }

            if fb.phys_res.0 == 0 || fb.phys_res.1 == 0 {
                return Err("unsupported physical resolution");
            }

            if fb.virt_res.0 == 0 || fb.virt_res.1 == 0 {
                return Err("unsupported virtual resolution");
            }

            Ok(fb)
        }

        pub fn set_pixel(&self, x: u32, y: u32, color: Color) -> Result<(), &'static str> {
            let offset = y * self.pitch + x * (self.depth / 8);
            if offset >= self.size {
                return Err("pixel out of range");
            }
            let p = (self.addr + offset) as u64 + UPPER_HALF_OFFSET;
            match self.depth {
                32 => Ok(unsafe {
                    *(p as *mut u32) = match self.order {
                        0 => color.as_bgra32(),
                        1 => color.as_rgba32(),
                        _ => return Err("unsupported pixel order"),
                    };
                }),
                _ => Err("unsupported color depth"),
            }
        }
    }
}
