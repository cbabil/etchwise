//! Ports of upstream `disk_formatter_test.cpp` plus byte-level checks of
//! the layout functions.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use super::layout::{
    DeviceLayout, Fat32Config, Fat32Layout, VOLUME_ID, sectors_per_cluster_for, sectors_per_fat,
};
use super::sectors::{MBR_ENTRY_OFFSET, chs, fat_head, fs_info, padded_label};
use super::{format_device, plan};
use crate::pipeline::Progress;

pub(super) const MIB: u64 = 1 << 20;

/// A sparse temp file of `size` bytes, formatted.
pub(super) fn formatted(size: u64) -> File {
    let mut file = tempfile::tempfile().unwrap();
    file.set_len(size).unwrap();
    format_device(&mut file, &Progress::default()).unwrap();
    file
}

pub(super) fn read_at(file: &mut File, offset: u64, len: usize) -> Vec<u8> {
    let mut buf = vec![0; len];
    file.seek(SeekFrom::Start(offset)).unwrap();
    file.read_exact(&mut buf).unwrap();
    buf
}

fn le16(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn le32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

/// Upstream `TestBasicFormatting`: formatting keeps the 64 MB size.
#[test]
fn basic_formatting() {
    let file = formatted(64 * MIB);
    assert_eq!(file.metadata().unwrap().len(), 64 * MIB);
}

/// Upstream `TestMbrStructure`, plus the exact entry bytes.
#[test]
fn mbr_structure() {
    let mut file = formatted(64 * MIB);
    let mbr = read_at(&mut file, 0, 512);
    assert_eq!(&mbr[510..], &[0x55, 0xAA]);
    let entry = &mbr[MBR_ENTRY_OFFSET..MBR_ENTRY_OFFSET + 16];
    assert_eq!(entry[0], 0x80, "bootable");
    assert_eq!(entry[4], 0x0C, "FAT32 LBA");
    assert_eq!(le32(entry, 8), 8192);
    assert_eq!(le32(entry, 12), 131_072 - 8192);
    assert_eq!(&entry[1..4], &[130, 3, 0]);
    assert_eq!(&entry[5..8], &[40, 32, 8]);
    assert!(mbr[..MBR_ENTRY_OFFSET].iter().all(|&b| b == 0));
    assert!(mbr[MBR_ENTRY_OFFSET + 16..510].iter().all(|&b| b == 0));
}

/// Upstream `TestFat32Structure`, plus every BPB field.
#[test]
fn fat32_structure() {
    let mut file = formatted(64 * MIB);
    let bs = read_at(&mut file, 8192 * 512, 512);
    assert_eq!(le16(&bs, 510), 0xAA55);
    assert_eq!(le16(&bs, 11), 512);
    assert_eq!(&bs[82..90], b"FAT32   ");
    assert_eq!(&bs[..3], &[0xEB, 0x58, 0x90]);
    assert_eq!(&bs[3..11], b"mkfs.fat");
    assert_eq!((bs[13], le16(&bs, 14), bs[16]), (2, 32, 2));
    assert_eq!((le16(&bs, 17), le16(&bs, 19), bs[21]), (0, 0, 0xF8));
    assert_eq!((le16(&bs, 22), le16(&bs, 24), le16(&bs, 26)), (0, 63, 255));
    assert_eq!(
        (le32(&bs, 28), le32(&bs, 32), le32(&bs, 36)),
        (8192, 122_880, 477)
    );
    assert_eq!((le16(&bs, 40), le16(&bs, 42), le32(&bs, 44)), (0, 0, 2));
    assert_eq!((le16(&bs, 48), le16(&bs, 50)), (1, 6));
    assert_eq!((bs[64], bs[65], bs[66]), (0x80, 0, 0x29));
    assert_eq!(le32(&bs, 67), VOLUME_ID);
    assert_eq!(&bs[71..82], b"BOOT       ");
    assert!(bs[90..510].iter().all(|&b| b == 0));
}

#[test]
fn backup_boot_sector_records_its_own_sector_as_hidden() {
    let mut file = formatted(64 * MIB);
    let primary = read_at(&mut file, 8192 * 512, 512);
    let mut backup = read_at(&mut file, 8198 * 512, 512);
    assert_eq!(le32(&backup, 28), 8198);
    backup[28..32].copy_from_slice(&8192_u32.to_le_bytes());
    assert_eq!(primary, backup);
}

#[test]
fn fsinfo_and_fats_on_disk() {
    let mut file = formatted(64 * MIB);
    let info = read_at(&mut file, 8193 * 512, 512);
    assert_eq!(le32(&info, 0), 0x4161_5252);
    assert_eq!(le32(&info, 484), 0x6141_7272);
    // (122880 - 32 - 954) / 2 clusters, minus the root's.
    assert_eq!((le32(&info, 488), le32(&info, 492)), (60_946, 3));
    assert_eq!(le32(&info, 508), 0xAA55_0000);
    for start in [8192 + 32, 8192 + 32 + 477] {
        let fat = read_at(&mut file, start * 512, 477 * 512);
        assert_eq!(le32(&fat, 0), 0x0FFF_FFF8);
        assert_eq!((le32(&fat, 4), le32(&fat, 8)), (0x0FFF_FFFF, 0x0FFF_FFFF));
        assert!(fat[12..].iter().all(|&b| b == 0));
    }
}

#[test]
fn cluster_size_thresholds() {
    let cases = [
        (0, 1),
        (66_599, 1),
        (66_600, 2),
        (532_479, 2),
        (532_480, 8),
        (16_777_215, 8),
        (16_777_216, 16),
        (33_554_431, 16),
        (33_554_432, 32),
        (u32::MAX, 32),
    ];
    for (sectors, spc) in cases {
        assert_eq!(sectors_per_cluster_for(sectors), spc, "{sectors}");
    }
}

#[test]
fn sectors_per_fat_matches_upstream_arithmetic() {
    let spf = |s| sectors_per_fat(&Fat32Config::for_partition(s));
    assert_eq!(spf(122_880), Some(477));
    assert_eq!(spf(4_194_304 - 8192), Some(4080));
    assert_eq!(spf(31), None);
    let layout = Fat32Layout::new(4_194_304 - 8192).unwrap();
    assert_eq!(
        (layout.total_clusters, layout.free_clusters()),
        (522_240, 522_239)
    );
    assert_eq!(layout.root_start(), 32 + 2 * 4080);
}

#[test]
fn device_layout_beyond_two_tib_matches_upstream_truncation() {
    let layout = DeviceLayout::for_device(3 << 40).unwrap();
    assert_eq!(layout.mbr_partition_sectors, u32::MAX - 8192);
    assert_eq!(layout.fat.config.total_sectors, (1 << 31) - 8192);
    assert_eq!(layout.fat.config.sectors_per_cluster, 32);
    assert!(DeviceLayout::for_device(4 * MIB + 16 * 512).is_none());
    assert!(DeviceLayout::for_device(0).is_none());
}

#[test]
fn small_pieces() {
    assert_eq!(chs(8192), [130, 3, 0]);
    // Cylinder 1024 clamps to 1023: high bits into the sector byte.
    assert_eq!(chs(16_065 * 1024 + 5), [0, 0xC6, 0xFF]);
    assert_eq!(&padded_label("BOOT"), b"BOOT       ");
    assert_eq!(&padded_label("ABCDEFGHIJKLM"), b"ABCDEFGHIJK");
    assert_eq!(
        &fat_head()[..12],
        &[
            0xF8, 0xFF, 0xFF, 0x0F, 0xFF, 0xFF, 0xFF, 0x0F, 0xFF, 0xFF, 0xFF, 0x0F
        ]
    );
    let layout = Fat32Layout::new(122_880).unwrap();
    assert_eq!(fs_info(&layout)[488..492], 60_946_u32.to_le_bytes());
}

#[test]
fn plan_order_and_sizes() {
    let regions = plan(64 * MIB).unwrap();
    let spans: Vec<_> = regions
        .iter()
        .map(|r| (r.offset / 512, r.len() / 512))
        .collect();
    let fat2 = 8224 + 477;
    let expected = [
        (0, 1),
        (8192, 1),
        (8193, 1),
        (8198, 1),
        (8224, 477),
        (fat2, 477),
        (fat2 + 477, 2),
    ];
    assert_eq!(spans, expected);
    assert!(plan(4 * MIB).is_none());
}
