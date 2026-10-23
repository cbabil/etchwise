use std::fs::File;
use std::io::{Cursor, Write};

use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;

use super::*;
use crate::fat::BootPartition;

const GIB: u64 = 1024 * 1024 * 1024;

/// A sparse 2 GiB device: big enough for upstream's FAT32 layout to be
/// mountable (smaller ones have a FAT16-sized cluster count).
fn device() -> File {
    let file = tempfile::tempfile().unwrap();
    file.set_len(2 * GIB).unwrap();
    file
}

fn zip(entries: &[(&str, Option<&[u8]>)]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in entries {
        match data {
            Some(bytes) => {
                zip.start_file(*name, SimpleFileOptions::default()).unwrap();
                zip.write_all(bytes).unwrap();
            }
            None => zip
                .add_directory(*name, SimpleFileOptions::default())
                .unwrap(),
        }
    }
    zip.finish().unwrap().into_inner()
}

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[test]
fn formats_then_extracts_every_entry() {
    let archive = zip(&[
        ("os/", None),
        ("os/partitions.json", Some(b"{}")),
        ("os/boot/kernel.img", Some(&[7u8; 70_000])),
        ("recovery.cmdline", Some(b"quiet")),
    ]);
    let mut dev = device();
    let progress = Progress::default();
    extract_multi_file(
        &mut dev,
        Cursor::new(archive.clone()),
        Some(&sha(&archive)),
        &progress,
    )
    .unwrap();
    let mut boot = BootPartition::open(&mut dev).unwrap();
    let mut files = boot.list_files().unwrap();
    files.sort();
    assert_eq!(
        files,
        [
            "os/boot/kernel.img",
            "os/partitions.json",
            "recovery.cmdline"
        ]
    );
    assert_eq!(
        boot.read_file("os/boot/kernel.img").unwrap().unwrap(),
        vec![7u8; 70_000]
    );
    assert_eq!(
        progress
            .bytes_written
            .load(std::sync::atomic::Ordering::Relaxed),
        70_000 + 2 + 5
    );
}

#[test]
fn hash_mismatch_removes_what_was_extracted() {
    let archive = zip(&[("a.txt", Some(b"a")), ("dir/b.txt", Some(b"b"))]);
    let mut dev = device();
    let err = extract_multi_file(
        &mut dev,
        Cursor::new(archive),
        Some(&"00".repeat(32)),
        &Progress::default(),
    )
    .unwrap_err();
    assert!(matches!(err, WriteError::HashMismatch { .. }), "{err}");
    let mut boot = BootPartition::open(&mut dev).unwrap();
    assert!(boot.list_files().unwrap().is_empty());
}

#[test]
fn rejects_unsafe_paths() {
    let traversal = zip(&[("../evil.txt", Some(b"x"))]);
    let err = extract_multi_file(
        &mut device(),
        Cursor::new(traversal),
        None,
        &Progress::default(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("evil.txt"), "{err}");
}

#[test]
fn never_overwrites_an_existing_file() {
    let mut dev = device();
    crate::erase::format_device(&mut dev, &Progress::default()).unwrap();
    let mut boot = BootPartition::open(&mut dev).unwrap();
    boot.write_new_file("dir/same.txt", &mut Cursor::new(b"1".to_vec()))
        .unwrap();
    let err = boot
        .write_new_file("dir/same.txt", &mut Cursor::new(b"2".to_vec()))
        .unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(boot.read_file("dir/same.txt").unwrap().unwrap(), b"1");
    let err = boot
        .write_new_file("dir", &mut Cursor::new(b"3".to_vec()))
        .unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
}

#[test]
fn cancellation_stops_extraction() {
    let progress = Progress::default();
    progress.cancel();
    let archive = zip(&[("a.txt", Some(b"a"))]);
    let err = extract_multi_file(&mut device(), Cursor::new(archive), None, &progress).unwrap_err();
    assert!(matches!(err, WriteError::Cancelled));
}
