use std::io::{Cursor, Write};

use fatfs::FatType;

use super::*;
use crate::testutil::{MIB, mbr_image};

fn seed(image: Vec<u8>, files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut boot = BootPartition::open(Cursor::new(image)).unwrap();
    for (name, data) in files {
        boot.write_file(name, data).unwrap();
    }
    boot.finish().unwrap().into_inner()
}

#[test]
fn writes_reads_and_replaces_files_on_fat32() {
    let image = seed(
        mbr_image(FatType::Fat32, 64),
        &[("cmdline.txt", b"console=serial0 root=PARTUUID=1 quiet")],
    );
    let mut boot = BootPartition::open(Cursor::new(image)).unwrap();
    assert_eq!(
        boot.extent(),
        Extent {
            start: MIB as u64,
            len: 64 * MIB as u64
        }
    );
    assert_eq!(
        boot.read_file("CMDLINE.TXT").unwrap().unwrap(),
        b"console=serial0 root=PARTUUID=1 quiet"
    );
    boot.write_file("cmdline.txt", b"short").unwrap();
    assert_eq!(boot.read_file("cmdline.txt").unwrap().unwrap(), b"short");
    assert!(boot.read_file("missing.txt").unwrap().is_none());
}

#[test]
fn long_names_subdirs_listing_and_delete_on_fat16() {
    let image = seed(
        mbr_image(FatType::Fat16, 16),
        &[
            ("network-config", b"net"),
            ("user-data", b"#cloud-config\n"),
        ],
    );
    let mut boot = BootPartition::open(Cursor::new(image)).unwrap();
    boot.with_fs(|fs| fs.root_dir().create_dir("overlays").map(|_| ()))
        .unwrap();
    boot.write_file("overlays/README", b"r").unwrap();
    let mut files = boot.list_files().unwrap();
    files.sort();
    assert_eq!(files, ["network-config", "overlays/README", "user-data"]);
    assert!(boot.exists("overlays").unwrap() && boot.exists("user-data").unwrap());
    assert!(boot.delete_file("user-data").unwrap());
    assert!(!boot.delete_file("user-data").unwrap());
    assert!(!boot.exists("user-data").unwrap());
    assert!(boot.write_file("nodir/x", b"x").is_err());
}

#[test]
fn changes_persist_only_after_finish_and_mbr_is_written_last() {
    let image = mbr_image(FatType::Fat16, 16);
    let mut device = RecordingDevice {
        data: Cursor::new(image),
        writes: Vec::new(),
    };
    {
        let mut boot = BootPartition::open(&mut device).unwrap();
        boot.write_file("firstrun.sh", b"#!/bin/sh\n").unwrap();
        boot.cache.pwrite(&[0x55, 0xAA], 510).unwrap();
        assert!(boot.cache.is_dirty());
        boot.finish().unwrap();
    }
    assert!(device.writes.len() > 1);
    assert_eq!(device.writes.last(), Some(&0));
    assert!(device.writes.iter().all(|offset| offset % 4096 == 0));
    let mut reopened = BootPartition::open(device.data).unwrap();
    assert_eq!(
        reopened.read_file("firstrun.sh").unwrap().unwrap(),
        b"#!/bin/sh\n"
    );
}

#[test]
fn finds_partition_through_gpt() {
    let mut image = mbr_image(FatType::Fat16, 16);
    image[512..520].copy_from_slice(b"EFI PART");
    image[512 + 24..512 + 32].copy_from_slice(&1u64.to_le_bytes());
    image[512 + 72..512 + 80].copy_from_slice(&2u64.to_le_bytes());
    image[512 + 80..512 + 84].copy_from_slice(&128u32.to_le_bytes());
    image[512 + 84..512 + 88].copy_from_slice(&128u32.to_le_bytes());
    let first_lba = (MIB / 512) as u64;
    image[1024 + 32..1024 + 40].copy_from_slice(&first_lba.to_le_bytes());
    image[1024 + 40..1024 + 48]
        .copy_from_slice(&(first_lba + (16 * MIB / 512) as u64 - 1).to_le_bytes());
    image[446 + 8..446 + 16].fill(0);
    let mut boot = BootPartition::open(Cursor::new(image)).unwrap();
    assert_eq!(
        boot.extent(),
        Extent {
            start: MIB as u64,
            len: 16 * MIB as u64
        }
    );
    boot.write_file("meta-data", b"instance-id: x\n").unwrap();
}

#[test]
fn rejects_missing_tables_and_partitions() {
    assert!(BootPartition::open(Cursor::new(vec![0u8; 64 * 1024])).is_err());
    let mut image = mbr_image(FatType::Fat16, 16);
    image[446 + 8..446 + 16].fill(0);
    assert!(BootPartition::open(Cursor::new(image.clone())).is_err());
    let mut cache = BlockCache::new(Cursor::new(image));
    assert!(locate(&mut cache, 5).is_err());
}

#[test]
fn window_refuses_access_beyond_the_partition() {
    let mut cache = BlockCache::new(Cursor::new(vec![0u8; 16384]));
    let mut window = Window::new(
        &mut cache,
        Extent {
            start: 4096,
            len: 4096,
        },
    );
    assert!(window.seek(io::SeekFrom::Start(4097)).is_err());
    window.seek(io::SeekFrom::End(-2)).unwrap();
    assert!(window.write_all(&[1, 2, 3]).is_err());
    window.seek(io::SeekFrom::Current(-2)).unwrap();
    assert!(window.seek(io::SeekFrom::Current(-5000)).is_err());
}

#[test]
fn short_device_reads_fail() {
    let mut cache = BlockCache::new(Cursor::new(vec![0u8; 1000]));
    assert!(cache.pread(&mut [0u8; 4], 0).is_err());
}

/// Records the offset of every write that reaches the device.
struct RecordingDevice {
    data: Cursor<Vec<u8>>,
    writes: Vec<u64>,
}

impl io::Read for RecordingDevice {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.data.read(buf)
    }
}

impl io::Write for RecordingDevice {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writes.push(self.data.position());
        self.data.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Seek for RecordingDevice {
    fn seek(&mut self, to: io::SeekFrom) -> io::Result<u64> {
        self.data.seek(to)
    }
}
