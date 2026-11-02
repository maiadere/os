use crate::driver::timer;

use crate::driver::spi::{spi_send_byte_sync, spi_write_bytes_sync, spi0_set_ta};

pub trait SDCommand {
    type ResponseType: SDResponse;
    fn bytes(&self) -> [u8; 6];
}

pub trait SDResponse: Sized {
    const LENGTH: usize;
    fn from_bytes(bytes: [u8; 5]) -> Self;
}

#[derive(Debug)]
pub struct RES1(u8);

impl SDResponse for RES1 {
    const LENGTH: usize = 1;

    fn from_bytes(bytes: [u8; 5]) -> Self {
        Self(bytes[0])
    }
}

#[derive(Debug)]
pub struct RES3([u8; 5]);

impl SDResponse for RES3 {
    const LENGTH: usize = 5;

    fn from_bytes(bytes: [u8; 5]) -> Self {
        Self(bytes)
    }
}

#[derive(Debug)]
pub struct RES7([u8; 5]);

impl SDResponse for RES7 {
    const LENGTH: usize = 5;
    fn from_bytes(bytes: [u8; 5]) -> Self {
        Self(bytes)
    }
}

pub struct CMD0 {}

impl SDCommand for CMD0 {
    type ResponseType = RES1;
    fn bytes(&self) -> [u8; 6] {
        [0x40, 0x00, 0x00, 0x00, 0x00, 0x95]
    }
}

pub struct CMD8 {}

impl SDCommand for CMD8 {
    type ResponseType = RES7;
    fn bytes(&self) -> [u8; 6] {
        [0x48, 0x00, 0x00, 0x01, 0xaa, 0x87]
    }
}

pub struct CMD55 {}

impl SDCommand for CMD55 {
    type ResponseType = RES1;
    fn bytes(&self) -> [u8; 6] {
        [0x40 | 55, 0x00, 0x00, 0x00, 0x00, 0x01]
    }
}

pub struct CMD41 {}

impl SDCommand for CMD41 {
    type ResponseType = RES1;
    fn bytes(&self) -> [u8; 6] {
        [0x40 | 41, 0x40, 0x00, 0x00, 0x00, 0x01]
    }
}

pub struct CMD58 {}

impl SDCommand for CMD58 {
    fn bytes(&self) -> [u8; 6] {
        [0x40 | 58, 0x00, 0x00, 0x00, 0x00, 0x01]
    }
    type ResponseType = RES3;
}

pub struct CMD17 {
    pub addr: u32,
}

impl SDCommand for CMD17 {
    fn bytes(&self) -> [u8; 6] {
        let [addr0, addr1, addr2, addr3] = self.addr.to_be_bytes();
        [0x40 | 17, addr0, addr1, addr2, addr3, 0x01]
    }
    type ResponseType = RES1;
}

#[derive(Debug)]
pub struct PartitionDescriptor {
    lba_addr: u32,
    sector_count: u32,
    partition_type: u8,
}
pub fn parse_mbr(mbr: &[u8; 512]) -> [Option<PartitionDescriptor>; 4] {
    const ENTRY_OFFSETS: [usize; 4] = [0x1be, 0x1ce, 0x1de, 0x1ee];
    const STATUS_OFFSET: usize = 0x0;
    const TYPE_OFFSET: usize = 0x4;
    const LBA_OFFSET: usize = 0x8;
    const SECTOR_COUNT_OFFSET: usize = 0xc;

    const STATUS_ACTIVE: u8 = 0x80;
    const STATUS_INACTIVE: u8 = 0x00;

    return ENTRY_OFFSETS.map(|entry_offset| {
        let status = mbr[entry_offset + STATUS_OFFSET];
        if status != STATUS_ACTIVE && status != STATUS_INACTIVE {
            return None;
        }

        // bounds are constant, unwrap is safe
        let lba_bytes: [u8; 4] = (&mbr[entry_offset + LBA_OFFSET..][..4]).try_into().unwrap();
        let lba_addr = u32::from_le_bytes(lba_bytes);

        let sector_count_bytes: [u8; 4] = (&mbr[entry_offset + SECTOR_COUNT_OFFSET..][..4])
            .try_into()
            .unwrap();
        let sector_count = u32::from_le_bytes(sector_count_bytes);

        let partition_type = mbr[entry_offset + TYPE_OFFSET];

        return Some(PartitionDescriptor {
            lba_addr,
            sector_count,
            partition_type,
        });
    });
}

pub fn send_sd_command<T: SDCommand>(command: T) -> Option<T::ResponseType> {
    for byte in command.bytes() {
        spi_send_byte_sync(byte);
    }
    let mut timeout_counter = 0;
    let mut response = [0xffu8; 5];
    const TIMEOUT: i32 = 8;
    loop {
        if timeout_counter >= TIMEOUT {
            return None;
        }

        let res_byte = spi_send_byte_sync(0xff); // dummy byte to get response

        if res_byte == 0xff {
            timeout_counter += 1;
            continue;
        }

        //response started
        response[0] = res_byte;
        for i in 1..T::ResponseType::LENGTH {
            response[i] = spi_send_byte_sync(0xff);
        }
        break;
    }
    return Some(T::ResponseType::from_bytes(response));
}

pub fn receive_sd_data_packet() -> Result<[u8; 512], &'static str> {
    // TODO: add some kind of timeout (around 100ms?)
    loop {
        let res_byte = spi_send_byte_sync(0xff); // dummy byte to get response
        if res_byte == 0xff {
            continue;
        }
        // data token for cmd17/18/24
        if res_byte != 0xfe {
            return Err("got error token instead of data token");
        }
        let mut res = [0u8; 512];
        for i in 0..512 {
            res[i] = spi_send_byte_sync(0xff);
        }
        let crc0 = spi_send_byte_sync(0xff);
        let crc1 = spi_send_byte_sync(0xff);
        let _crc = u16::from_be_bytes([crc0, crc1]);
        return Ok(res);
    }
}

pub fn read_sd_block(addr: u32) -> Result<[u8; 512], &'static str> {
    let res = send_sd_command(CMD17 { addr }).ok_or("timed out waiting for SD response ")?;
    if res.0 != 0x0 {
        return Err("recieved error response from SD");
    }

    return receive_sd_data_packet();
}

pub unsafe fn initalize_sd_spi() -> Result<(), &'static str> {
    // send dummy clock cycles to the sd as per documentation
    let padding_bytes = [0xff; 160];
    spi_write_bytes_sync(&padding_bytes);

    //disable and enable necessary
    unsafe { spi0_set_ta(false) };
    unsafe { spi0_set_ta(true) };

    //TODO handle error responses

    for _ in 0..5 {
        let _res = send_sd_command(CMD0 {}).ok_or("SD response timeout")?;
    }

    for _ in 0..2 {
        let _res = send_sd_command(CMD8 {}).ok_or("SD response timeout")?;
    }

    let start_time = timer::get_system_time();
    for i in 0..10000000 {
        if i % 1000 == 0 {
            if timer::get_system_time() - start_time > 2_000_000 {
                return Err("timeout while initalizing sd via AMCD41");
            }
        }
        let res = send_sd_command(CMD55 {}).ok_or("SD response timeout")?;
        if res.0 != 1 {
            return Err("CMD55 returned non 0x1 response");
        }
        let res = send_sd_command(CMD41 {}).ok_or("SD response timeout")?;
        if res.0 == 0 {
            let res = send_sd_command(CMD58 {}).unwrap();
            if res.0[0] != 0 {
                return Err("error response to CMD58");
            }
            if res.0[1] != 192 {
                return Err("card unitialized or not high capacity which is unsupported");
            }
            break;
        }
    }
    return Ok(());
}
