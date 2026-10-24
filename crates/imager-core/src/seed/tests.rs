//! The seed partition on a scaled-down copy of the Ubuntu Server ISO layout:
//! protective MBR, GPT with 248 entries at LBA 2, ISO / ESP / gap partitions,
//! backup GPT at the image end (not the device end).

use std::io::Cursor;

use super::*;
use crate::le::{put_u32, put_u64};
use crate::testutil::{read_text, seed_volume_at};

const MIB: u64 = 1 << 20;
const DEVICE: u64 = 64 * MIB;
const IMAGE_SECTORS: u64 = 20_000;
const ENTRIES: u32 = 248;
const FILES: &[(&str, &[u8])] = &[("user-data", b"#cloud-config\n"), ("meta-data", b"id: x\n")];

fn entry(b: &mut [u8], at: usize, type_byte: u8, first: u64, last: u64) {
    b[at] = type_byte;
    b[at + 16] = type_byte ^ 0x55;
    put_u64(b, at + 32, first);
    put_u64(b, at + 40, last);
}

/// Header at `lba` describing entries at `entries_lba`, CRCs filled in.
fn header(b: &mut [u8], lba: u64, alt: u64, entries_lba: u64) {
    let at = (lba * 512) as usize;
    let entries = &b[(entries_lba * 512) as usize..][..(ENTRIES * 128) as usize];
    let entries_crc = crc32fast::hash(entries);
    let h = &mut b[at..at + 512];
    h[..8].copy_from_slice(b"EFI PART");
    put_u32(h, 8, 0x0001_0000);
    put_u32(h, 12, 92);
    put_u64(h, 24, lba);
    put_u64(h, 32, alt);
    put_u64(h, 40, 64);
    put_u64(h, 48, IMAGE_SECTORS - 64);
    put_u64(h, 72, entries_lba);
    put_u32(h, 80, ENTRIES);
    put_u32(h, 84, 128);
    put_u32(h, 88, entries_crc);
    let crc = crc32fast::hash(&h[..92]);
    put_u32(h, 16, crc);
}

fn ubuntu_like_device() -> Vec<u8> {
    let mut b = vec![0u8; DEVICE as usize];
    b[446 + 4] = 0xEE;
    put_u32(&mut b, 446 + 8, 1);
    put_u32(&mut b, 446 + 12, (IMAGE_SECTORS - 1) as u32);
    b[510..512].copy_from_slice(&[0x55, 0xAA]);
    entry(&mut b, 1024, 0xA2, 64, 18_000);
    entry(&mut b, 1024 + 128, 0x28, 18_001, 19_000);
    entry(&mut b, 1024 + 256, 0xA2, 19_001, 19_900);
    let table: Vec<u8> = b[1024..1024 + (ENTRIES * 128) as usize].to_vec();
    let backup_entries = IMAGE_SECTORS - 1 - 62;
    b[(backup_entries * 512) as usize..][..table.len()].copy_from_slice(&table);
    header(&mut b, 1, IMAGE_SECTORS - 1, 2);
    header(&mut b, IMAGE_SECTORS - 1, 1, backup_entries);
    b
}

/// Checks the header at `lba` and its entries with its own CRCs.
fn read_header(b: &[u8], lba: u64) -> (u64, u64, u64, Vec<u8>) {
    let h = &b[(lba * 512) as usize..][..92];
    assert_eq!(&h[..8], b"EFI PART", "header at {lba}");
    let mut zeroed = h.to_vec();
    zeroed[16..20].fill(0);
    let u64_at = |at: usize| u64::from_le_bytes(h[at..at + 8].try_into().unwrap());
    let crc = u32::from_le_bytes(h[16..20].try_into().unwrap());
    assert_eq!(crc32fast::hash(&zeroed), crc, "header CRC at {lba}");
    let entries = b[(u64_at(72) * 512) as usize..][..(ENTRIES * 128) as usize].to_vec();
    let entries_crc = u32::from_le_bytes(h[88..92].try_into().unwrap());
    assert_eq!(
        crc32fast::hash(&entries),
        entries_crc,
        "entries CRC at {lba}"
    );
    (u64_at(24), u64_at(32), u64_at(48), entries)
}

#[test]
fn adds_a_cidata_partition_and_moves_the_backup_gpt() {
    let before = ubuntu_like_device();
    let mut device = Cursor::new(before.clone());
    add_seed_partition(&mut device, DEVICE, IMAGE_SECTORS * 512, FILES).unwrap();
    let b = device.into_inner();
    let last = DEVICE / 512 - 1;
    let (my, alt, last_usable, entries) = read_header(&b, 1);
    assert_eq!((my, alt), (1, last));
    let (bmy, balt, _, backup_entries) = read_header(&b, last);
    assert_eq!((bmy, balt), (last, 1));
    assert_eq!(entries, backup_entries);
    assert_eq!(entries[..384], before[1024..1024 + 384], "p1-p3 unchanged");
    let p4 = &entries[384..512];
    assert_eq!(p4[..16], BASIC_DATA_TYPE);
    let first = u64::from_le_bytes(p4[32..40].try_into().unwrap());
    let end = u64::from_le_bytes(p4[40..48].try_into().unwrap());
    assert_eq!(first % 2048, 0);
    assert!(first >= IMAGE_SECTORS && end <= last_usable);
    assert_eq!((end - first + 1) * 512, SEED_SIZE);
    let name: Vec<u16> = p4[56..68]
        .chunks(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    assert_eq!(String::from_utf16_lossy(&name), "CIDATA");
    assert_eq!(
        u32::from_le_bytes(b[458..462].try_into().unwrap()),
        last as u32
    );
    assert_eq!(
        b[(IMAGE_SECTORS * 512) as usize - 512..][..8],
        [0; 8],
        "old backup cleared"
    );
    check_volume(&b, first * 512);
}

fn check_volume(b: &[u8], start: u64) {
    let fs = seed_volume_at(b, start as usize);
    for (name, contents) in FILES {
        assert_eq!(read_text(&fs, name).as_bytes(), *contents, "{name}");
    }
}

#[test]
fn adds_an_mbr_partition_to_an_mbr_image() {
    let mut b = vec![0u8; DEVICE as usize];
    b[446 + 4] = 0x0C;
    put_u32(&mut b, 446 + 8, 2048);
    put_u32(&mut b, 446 + 12, 18_000);
    b[510..512].copy_from_slice(&[0x55, 0xAA]);
    let mut device = Cursor::new(b);
    add_seed_partition(&mut device, DEVICE, IMAGE_SECTORS * 512, FILES).unwrap();
    let b = device.into_inner();
    let slot = &b[446 + 16..446 + 32];
    assert_eq!(slot[4], 0x0C);
    let start = u32::from_le_bytes(slot[8..12].try_into().unwrap());
    assert_eq!(start % 2048, 0);
    assert_eq!(
        u64::from(u32::from_le_bytes(slot[12..16].try_into().unwrap())) * 512,
        SEED_SIZE
    );
    check_volume(&b, u64::from(start) * 512);
}

#[test]
fn refuses_without_room_or_with_a_damaged_gpt() {
    let small = (IMAGE_SECTORS * 512).next_multiple_of(MIB) + MIB;
    let mut b = ubuntu_like_device();
    b.truncate(small as usize);
    let err = add_seed_partition(&mut Cursor::new(b), small, IMAGE_SECTORS * 512, FILES);
    assert!(err.unwrap_err().to_string().contains("room"));
    let mut b = ubuntu_like_device();
    b[1024 + 40] ^= 1;
    let err = add_seed_partition(&mut Cursor::new(b), DEVICE, IMAGE_SECTORS * 512, FILES);
    assert!(err.unwrap_err().to_string().contains("CRC"));
}
