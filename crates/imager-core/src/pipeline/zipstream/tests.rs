use std::io::{Cursor, Read, Write};

use flate2::Compression;
use flate2::write::DeflateEncoder;

use super::*;

/// A local header with `method`, sizes and an optional Zip64 extra.
fn entry(name: &str, method: u16, flags: u16, data: &[u8], zip64: bool) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(LOCAL_HEADER.to_le_bytes());
    out.extend(20u16.to_le_bytes());
    out.extend(flags.to_le_bytes());
    out.extend(method.to_le_bytes());
    out.extend([0u8; 8]);
    let size = if zip64 { u32::MAX } else { data.len() as u32 };
    out.extend(size.to_le_bytes());
    out.extend(size.to_le_bytes());
    out.extend((name.len() as u16).to_le_bytes());
    let extra: Vec<u8> = if zip64 {
        let mut e = vec![1, 0, 16, 0];
        e.extend((data.len() as u64).to_le_bytes());
        e.extend((data.len() as u64).to_le_bytes());
        e
    } else {
        Vec::new()
    };
    out.extend((extra.len() as u16).to_le_bytes());
    out.extend(name.as_bytes());
    out.extend(extra);
    out.extend(data);
    out
}

fn read_all(zip: Vec<u8>) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    ZipEntryReader::new(Cursor::new(zip))?.read_to_end(&mut out)?;
    Ok(out)
}

#[test]
fn reads_deflated_first_file_after_directories() {
    let mut enc = DeflateEncoder::new(Vec::new(), Compression::fast());
    enc.write_all(b"disk image bytes").unwrap();
    let mut zip = entry("images/", 0, 0, b"", false);
    zip.extend(entry(
        "images/a.img",
        8,
        0x0008,
        &enc.finish().unwrap(),
        false,
    ));
    zip.extend(entry("other.txt", 0, 0, b"ignored", false));
    assert_eq!(read_all(zip).unwrap(), b"disk image bytes");
}

#[test]
fn reads_stored_files_with_and_without_zip64() {
    assert_eq!(
        read_all(entry("a.img", 0, 0, b"raw", false)).unwrap(),
        b"raw"
    );
    assert_eq!(
        read_all(entry("a.img", 0, 0, b"raw64", true)).unwrap(),
        b"raw64"
    );
}

#[test]
fn rejects_unsupported_entries() {
    assert!(read_all(entry("a.img", 0, 0x0008, b"x", false)).is_err());
    assert!(read_all(entry("a.img", 14, 0, b"x", false)).is_err());
    assert!(read_all(b"PK\x05\x06rest-of-something-else-here".to_vec()).is_err());
    let mut broken = entry("a.img", 0, 0, b"", true);
    broken.truncate(30 + 5);
    broken.extend([0u8; 0]);
    assert!(ZipEntryReader::new(Cursor::new(broken)).is_err());
    assert_eq!(zip64_compressed_size(&[1, 0, 2, 0, 9, 9]), None);
    assert_eq!(zip64_compressed_size(&[9, 0, 40, 0]), None);
}
