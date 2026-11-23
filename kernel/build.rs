// builds the fake sd card image for the qemu feature from the files in ../sd
// layout: 64 MiB disk, MBR with one FAT32 partition (type 0x0C) starting at sector 2048
use std::{
    env, fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

const SECTOR: usize = 512;
const DISK_SECTORS: usize = 64 * 1024 * 1024 / SECTOR;
const PART_START: usize = 2048;
const PART_SECTORS: usize = DISK_SECTORS - PART_START;

fn main() {
    if env::var_os("CARGO_FEATURE_QEMU").is_none() {
        return;
    }
    let src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../sd");
    println!("cargo:rerun-if-changed={}", src.display());

    let mut part = Cursor::new(vec![0u8; PART_SECTORS * SECTOR]);
    fatfs::format_volume(
        &mut part,
        fatfs::FormatVolumeOptions::new()
            .fat_type(fatfs::FatType::Fat32)
            .bytes_per_sector(SECTOR as u16)
            .bytes_per_cluster(SECTOR as u32)
            .total_sectors(PART_SECTORS as u32),
    )
    .unwrap();
    {
        let fs = fatfs::FileSystem::new(&mut part, fatfs::FsOptions::new()).unwrap();
        copy_dir(&src, &fs.root_dir());
        fs.unmount().unwrap();
    }

    let mut img = vec![0u8; PART_START * SECTOR];
    // MBR partition entry 0: lba start + sector count, type 0x0C (FAT32 LBA)
    let entry = &mut img[446..462];
    entry[4] = 0x0C;
    entry[8..12].copy_from_slice(&(PART_START as u32).to_le_bytes());
    entry[12..16].copy_from_slice(&(PART_SECTORS as u32).to_le_bytes());
    img[510] = 0x55;
    img[511] = 0xAA;
    img.extend_from_slice(&part.into_inner());

    // the kernel treats everything past the end of the image as zeros, so drop trailing zeros
    let len = img.iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
    img.truncate(len.next_multiple_of(SECTOR));

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("sd.img");
    fs::write(out, img).unwrap();
}

fn copy_dir<T: fatfs::ReadWriteSeek>(src: &Path, dst: &fatfs::Dir<T>) {
    let mut entries: Vec<_> = fs::read_dir(src).unwrap().map(|e| e.unwrap()).collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let name = entry.file_name().into_string().unwrap();
        if name == ".gitkeep" {
            continue;
        }
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &dst.create_dir(&name).unwrap());
        } else {
            let mut file = dst.create_file(&name).unwrap();
            file.truncate().unwrap();
            file.write_all(&fs::read(entry.path()).unwrap()).unwrap();
        }
    }
}
