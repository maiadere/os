use core::ops::ControlFlow;

use crate::driver::sd_spi;

const SECTOR_SIZE: usize = 512;

const FIRST_DATA_CLUSTER: u32 = 2;

const MIN_FAT32_CLUSTERS: u32 = 65525;

const FAT_ENTRY_SIZE: usize = 4;
const FAT_ENTRY_MASK: u32 = 0x0FFF_FFFF;
const END_OF_CHAIN: u32 = 0x0FFF_FFF8;

const DIR_ENTRY_SIZE: usize = 32;
const END_OF_DIRECTORY: u8 = 0x00;
const DELETED_ENTRY: u8 = 0xE5;
const ATTR_DIRECTORY: u8 = 0x10;
const ATTR_VOLUME_LABEL: u8 = 0x08;

const ATTR_LONG_NAME: u8 = 0x0F;
const LONG_NAME_ORDER_MASK: u8 = 0x1F;
const LONG_NAME_CHAR_OFFSETS: [usize; 13] = [1, 3, 5, 7, 9, 14, 16, 18, 20, 22, 24, 28, 30];
pub const MAX_NAME_LEN: usize = 255;

#[derive(Debug)]
pub enum Fat32Error {
    MissingMbrSignature,
    NotFat32Partition,
    MissingBootSignature,
    UnsupportedSectorSize,
    InvalidSectorsPerCluster,
    InvalidLayout,
    NotFat32,
    InvalidCluster,
    NotFound,
    NotADirectory,
    NotAFile,
    SdError(&'static str),
}

fn has_boot_signature(sector: &[u8; SECTOR_SIZE]) -> bool {
    sector[510] == 0x55 && sector[511] == 0xAA
}

fn is_fat32_partition_type(partition_type: u8) -> bool {
    partition_type == 0x0B || partition_type == 0x0C
}

pub fn find_partition_start() -> Result<u32, Fat32Error> {
    let mbr = sd_spi::read_sd_block(0).map_err(|e| Fat32Error::SdError(e))?;
    let partition_type = mbr[450];
    let partition_start = u32::from_le_bytes([mbr[454], mbr[455], mbr[456], mbr[457]]);

    if !has_boot_signature(&mbr) {
        return Err(Fat32Error::MissingMbrSignature);
    }
    if !is_fat32_partition_type(partition_type) {
        return Err(Fat32Error::NotFat32Partition);
    }
    Ok(partition_start)
}

#[derive(Clone, Copy)]
pub struct Name {
    bytes: [u8; MAX_NAME_LEN],
    len: usize,
}

impl Name {
    const EMPTY: Name = Name {
        bytes: [0; MAX_NAME_LEN],
        len: 0,
    };

    pub fn as_str(&self) -> &str {
        let bytes = self.bytes.get(..self.len).unwrap_or_default();
        core::str::from_utf8(bytes).unwrap_or("?")
    }

    fn push(&mut self, position: usize, char: u8) {
        if let Some(slot) = self.bytes.get_mut(position) {
            *slot = char;
            self.len = self.len.max(position + 1);
        }
    }

    fn from_short(entry: &[u8; DIR_ENTRY_SIZE]) -> Name {
        let base = entry[0..8].trim_ascii_end();
        let extension = entry[8..11].trim_ascii_end();
        let dot: &[u8] = if extension.is_empty() { b"" } else { b"." };

        let mut name = Name::EMPTY;
        for (position, char) in base.iter().chain(dot).chain(extension).enumerate() {
            name.push(position, *char);
        }
        name
    }

    fn add_long_piece(&mut self, entry: &[u8; DIR_ENTRY_SIZE]) {
        let piece_number = (entry[0] & LONG_NAME_ORDER_MASK) as usize;
        let start = piece_number.saturating_sub(1) * LONG_NAME_CHAR_OFFSETS.len();

        for (position, offset) in (start..).zip(LONG_NAME_CHAR_OFFSETS) {
            if let Some(char) = long_name_char(entry, offset) {
                self.push(position, char);
            }
        }
    }
}

fn long_name_char(entry: &[u8; DIR_ENTRY_SIZE], offset: usize) -> Option<u8> {
    let code = u16::from_le_bytes([*entry.get(offset)?, *entry.get(offset + 1)?]);
    match code {
        0x0000 | 0xFFFF => None,
        0x0001..=0x007F => Some(code as u8),
        _ => Some(b'?'),
    }
}

enum EntryKind {
    EndOfDirectory,
    Unused,
    LongNamePiece,
    FileOrDirectory,
}

fn entry_kind(entry: &[u8; DIR_ENTRY_SIZE]) -> EntryKind {
    match (entry[0], entry[11]) {
        (END_OF_DIRECTORY, _) => EntryKind::EndOfDirectory,
        (DELETED_ENTRY, _) => EntryKind::Unused,
        (_, ATTR_LONG_NAME) => EntryKind::LongNamePiece,
        (_, attributes) if attributes & ATTR_VOLUME_LABEL != 0 => EntryKind::Unused,
        _ => EntryKind::FileOrDirectory,
    }
}

#[derive(Clone, Copy)]
pub struct DirEntry {
    name: Name,
    pub first_cluster: u32,
    pub size: u32,
    pub is_dir: bool,
}

impl DirEntry {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

#[derive(Debug)]
pub struct Fat32 {
    sectors_per_cluster: u32,
    root_cluster: u32,
    cluster_count: u32,
    fat_start: u32,
    data_start: u32,
}

impl Fat32 {
    pub fn mount(partition_start: u32) -> Result<Self, Fat32Error> {
        let boot = sd_spi::read_sd_block(partition_start).map_err(|e| Fat32Error::SdError(e))?;
        let bytes_per_sector = u16::from_le_bytes([boot[11], boot[12]]) as usize;
        let sectors_per_cluster = boot[13] as u32;
        let reserved_sectors = u16::from_le_bytes([boot[14], boot[15]]) as u32;
        let fat_count = boot[16] as u32;
        let root_entry_count = u16::from_le_bytes([boot[17], boot[18]]);
        let sectors_per_fat_16 = u16::from_le_bytes([boot[22], boot[23]]);
        let total_sectors = u32::from_le_bytes([boot[32], boot[33], boot[34], boot[35]]);
        let sectors_per_fat = u32::from_le_bytes([boot[36], boot[37], boot[38], boot[39]]);
        let root_cluster = u32::from_le_bytes([boot[44], boot[45], boot[46], boot[47]]);

        if !has_boot_signature(&boot) {
            return Err(Fat32Error::MissingBootSignature);
        }

        if bytes_per_sector != SECTOR_SIZE {
            return Err(Fat32Error::UnsupportedSectorSize);
        }

        if !sectors_per_cluster.is_power_of_two() {
            return Err(Fat32Error::InvalidSectorsPerCluster);
        }

        if root_entry_count != 0 || sectors_per_fat_16 != 0 {
            return Err(Fat32Error::NotFat32);
        }

        let fat_start = partition_start
            .checked_add(reserved_sectors)
            .ok_or(Fat32Error::InvalidLayout)?;
        let fat_sectors = fat_count
            .checked_mul(sectors_per_fat)
            .ok_or(Fat32Error::InvalidLayout)?;
        let data_start = fat_start
            .checked_add(fat_sectors)
            .ok_or(Fat32Error::InvalidLayout)?;
        let data_sectors = total_sectors
            .checked_sub(data_start - partition_start)
            .ok_or(Fat32Error::InvalidLayout)?;
        let cluster_count = data_sectors / sectors_per_cluster;

        if reserved_sectors == 0 || fat_sectors == 0 {
            return Err(Fat32Error::InvalidLayout);
        }
        if cluster_count < MIN_FAT32_CLUSTERS {
            return Err(Fat32Error::NotFat32);
        }

        let fs = Fat32 {
            sectors_per_cluster,
            root_cluster,
            cluster_count,
            fat_start,
            data_start,
        };
        fs.cluster_to_lba(root_cluster)?;
        Ok(fs)
    }

    fn cluster_to_lba(&self, cluster: u32) -> Result<u32, Fat32Error> {
        let cluster_index = cluster
            .checked_sub(FIRST_DATA_CLUSTER)
            .ok_or(Fat32Error::InvalidCluster)?;
        if cluster_index >= self.cluster_count {
            return Err(Fat32Error::InvalidCluster);
        }
        Ok(self.data_start + cluster_index * self.sectors_per_cluster)
    }

    fn next_cluster(&self, cluster: u32) -> Result<Option<u32>, Fat32Error> {
        let entries_per_sector = (SECTOR_SIZE / FAT_ENTRY_SIZE) as u32;
        let fat_sector = sd_spi::read_sd_block(self.fat_start + cluster / entries_per_sector)
            .map_err(|e| Fat32Error::SdError(e))?;
        let (entries, _) = fat_sector.as_chunks::<FAT_ENTRY_SIZE>();
        let entry = entries
            .get((cluster % entries_per_sector) as usize)
            .ok_or(Fat32Error::InvalidCluster)?;

        let next = u32::from_le_bytes(*entry) & FAT_ENTRY_MASK;
        Ok((next < END_OF_CHAIN).then_some(next))
    }

    fn for_each_sector<T>(
        &self,
        first_cluster: u32,
        mut visit: impl FnMut(&[u8; SECTOR_SIZE]) -> ControlFlow<T>,
    ) -> Result<Option<T>, Fat32Error> {
        let mut cluster = first_cluster;
        loop {
            let first_sector = self.cluster_to_lba(cluster)?;
            for lba in first_sector..first_sector + self.sectors_per_cluster {
                if let ControlFlow::Break(value) =
                    visit(&sd_spi::read_sd_block(lba).map_err(|e| Fat32Error::SdError(e))?)
                {
                    return Ok(Some(value));
                }
            }
            match self.next_cluster(cluster)? {
                Some(next) => cluster = next,
                None => return Ok(None),
            }
        }
    }

    pub fn root(&self) -> DirEntry {
        DirEntry {
            name: Name::EMPTY,
            first_cluster: self.root_cluster,
            size: 0,
            is_dir: true,
        }
    }

    fn parse_entry(&self, entry: &[u8; DIR_ENTRY_SIZE], long_name: &Name) -> DirEntry {
        let is_dir = entry[11] & ATTR_DIRECTORY != 0;
        let cluster_high = u16::from_le_bytes([entry[20], entry[21]]) as u32;
        let cluster_low = u16::from_le_bytes([entry[26], entry[27]]) as u32;
        let first_cluster = cluster_high << 16 | cluster_low;
        let size = u32::from_le_bytes([entry[28], entry[29], entry[30], entry[31]]);

        let first_cluster = if is_dir && first_cluster == 0 {
            self.root_cluster
        } else {
            first_cluster
        };
        let name = if long_name.len > 0 {
            *long_name
        } else {
            Name::from_short(entry)
        };
        DirEntry {
            name,
            first_cluster,
            size,
            is_dir,
        }
    }

    pub fn read_dir<T>(
        &self,
        dir: &DirEntry,
        mut visit: impl FnMut(&DirEntry) -> ControlFlow<T>,
    ) -> Result<Option<T>, Fat32Error> {
        if !dir.is_dir {
            return Err(Fat32Error::NotADirectory);
        }

        let mut long_name = Name::EMPTY;
        let result = self.for_each_sector(dir.first_cluster, |sector| {
            let (entries, _) = sector.as_chunks::<DIR_ENTRY_SIZE>();
            for entry in entries {
                match entry_kind(entry) {
                    EntryKind::EndOfDirectory => return ControlFlow::Break(None),
                    EntryKind::Unused => long_name = Name::EMPTY,
                    EntryKind::LongNamePiece => long_name.add_long_piece(entry),
                    EntryKind::FileOrDirectory => {
                        let found = self.parse_entry(entry, &long_name);
                        long_name = Name::EMPTY;
                        if let ControlFlow::Break(value) = visit(&found) {
                            return ControlFlow::Break(Some(value));
                        }
                    }
                }
            }
            ControlFlow::Continue(())
        })?;
        Ok(result.flatten())
    }

    pub fn find(&self, dir: &DirEntry, name: &str) -> Result<DirEntry, Fat32Error> {
        let found = self.read_dir(dir, |entry| match entry.name().eq_ignore_ascii_case(name) {
            true => ControlFlow::Break(*entry),
            false => ControlFlow::Continue(()),
        })?;
        found.ok_or(Fat32Error::NotFound)
    }

    pub fn open(&self, path: &str) -> Result<DirEntry, Fat32Error> {
        path.split('/')
            .filter(|part| !part.is_empty())
            .try_fold(self.root(), |dir, part| self.find(&dir, part))
    }

    pub fn read_file(&self, file: &DirEntry, buffer: &mut [u8]) -> Result<usize, Fat32Error> {
        if file.is_dir {
            return Err(Fat32Error::NotAFile);
        }

        if file.size == 0 {
            return Ok(0);
        }

        let read_len = buffer.len().min(file.size as usize);
        let mut remaining = buffer.iter_mut().take(read_len);
        self.for_each_sector(file.first_cluster, |sector| {
            sector
                .iter()
                .zip(remaining.by_ref())
                .for_each(|(byte, slot)| *slot = *byte);
            match remaining.len() {
                0 => ControlFlow::Break(()),
                _ => ControlFlow::Continue(()),
            }
        })?;
        Ok(read_len - remaining.len())
    }
}
