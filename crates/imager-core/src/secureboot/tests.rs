mod keys;

use std::io::{Cursor, Read, Write};

use fatfs::{FatType, FileSystem, FsOptions};
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use rsa::{Pkcs1v15Sign, RsaPrivateKey};
use sha2::{Digest, Sha256};

use self::keys::{TestRng, pem, test_key};
use super::*;
use crate::testutil::{MIB, mbr_image};

/// Boot partition contents: boot files, a nested directory, customisation
/// files (one upper-case) and a zero-length file.
const FILES: [(&str, &[u8]); 8] = [
    ("config.txt", b"arm_64bit=1\n"),
    ("kernel8.img", &[0xA5; 70_000]),
    ("cmdline.txt", b"console=serial0 root=PARTUUID=1"),
    ("overlays/deep/x.dtbo", b"dtbo"),
    ("firstrun.sh", b"#!/bin/bash\n"),
    ("USER-DATA", b"#cloud-config\n"),
    ("network-config", b"version: 2\n"),
    ("empty.txt", b""),
];

const BOOT_FILES: [&str; 4] = [
    "config.txt",
    "kernel8.img",
    "cmdline.txt",
    "overlays/deep/x.dtbo",
];

fn partition(files: &[(&str, &[u8])]) -> BootPartition<Cursor<Vec<u8>>> {
    let mut image = mbr_image(FatType::Fat16, 64);
    let fs = FileSystem::new(Cursor::new(&mut image[MIB..]), FsOptions::new()).unwrap();
    for (path, data) in files {
        for (end, _) in path.match_indices('/') {
            fs.root_dir().create_dir(&path[..end]).unwrap();
        }
        fs.root_dir()
            .create_file(path)
            .unwrap()
            .write_all(data)
            .unwrap();
    }
    fs.unmount().unwrap();
    BootPartition::open(Cursor::new(image)).unwrap()
}

fn signed() -> BootPartition<Cursor<Vec<u8>>> {
    let mut boot = partition(&FILES);
    sign_boot_partition(&mut boot, &pem(), 1_700_000_000).unwrap();
    boot
}

fn read_img_file(image: &[u8], path: &str) -> Vec<u8> {
    let fs = FileSystem::new(Cursor::new(image.to_vec()), FsOptions::new()).unwrap();
    assert_eq!(fs.fat_type(), FatType::Fat32);
    let mut data = Vec::new();
    fs.root_dir()
        .open_file(path)
        .unwrap()
        .read_to_end(&mut data)
        .unwrap();
    data
}

#[test]
fn moves_boot_files_into_boot_img_and_keeps_customisation() {
    let mut boot = signed();
    let mut listed = boot.list_files().unwrap();
    listed.sort();
    let mut expected = vec![
        "boot.img",
        "boot.sig",
        "empty.txt",
        "firstrun.sh",
        "network-config",
    ];
    // The upper-case original is rewritten in place (FAT names are case-insensitive).
    expected.push(if listed.iter().any(|p| p == "USER-DATA") {
        "USER-DATA"
    } else {
        "user-data"
    });
    expected.sort_unstable();
    assert_eq!(listed, expected);
    assert!(!boot.exists("overlays").unwrap());
    assert_eq!(
        boot.read_file("user-data").unwrap().unwrap(),
        b"#cloud-config\n"
    );
    assert_eq!(
        boot.read_file("firstrun.sh").unwrap().unwrap(),
        b"#!/bin/bash\n"
    );

    let image = boot.read_file("boot.img").unwrap().unwrap();
    assert_eq!(image.len() as u64, 33 * MIB as u64);
    for (path, data) in FILES.iter().filter(|(p, _)| BOOT_FILES.contains(p)) {
        assert_eq!(read_img_file(&image, path), *data, "{path}");
    }
    let fs = FileSystem::new(Cursor::new(image.clone()), FsOptions::new()).unwrap();
    assert!(fs.root_dir().open_file("firstrun.sh").is_err());
    assert!(fs.root_dir().open_file("empty.txt").is_err());
}

#[test]
fn boot_sig_verifies_over_sha256_of_boot_img() {
    let mut boot = signed();
    let image = boot.read_file("boot.img").unwrap().unwrap();
    let sig = String::from_utf8(boot.read_file("boot.sig").unwrap().unwrap()).unwrap();
    let lines: Vec<&str> = sig.split_terminator('\n').collect();
    assert!(sig.ends_with('\n'));
    assert_eq!(lines.len(), 3);
    let digest = Sha256::digest(&image);
    assert_eq!(lines[0], hex::encode(digest));
    assert_eq!(lines[1], "ts: 1700000000");
    let sig_hex = lines[2].strip_prefix("rsa2048: ").unwrap();
    assert_eq!(sig_hex.len(), 512);
    assert_eq!(sig_hex, sig_hex.to_lowercase());
    let signature = hex::decode(sig_hex).unwrap();
    test_key()
        .to_public_key()
        .verify(Pkcs1v15Sign::new::<Sha256>(), &digest, &signature)
        .unwrap();
}

#[test]
fn boot_sig_is_deterministic_and_accepts_pkcs8() {
    let pkcs8 = test_key()
        .to_pkcs8_pem(LineEnding::CRLF)
        .unwrap()
        .to_string();
    let a = boot_sig(b"image", &pem(), 5).unwrap();
    assert_eq!(a, boot_sig(b"image", &pkcs8, 5).unwrap());
    assert!(a.starts_with(&format!(
        "{}\nts: 5\nrsa2048: ",
        hex::encode(Sha256::digest(b"image"))
    )));
}

#[test]
fn fingerprint_matches_bootloader_key_format() {
    let fp = key_fingerprint(&pem()).unwrap();
    assert_eq!(fp.len(), 33);
    assert_eq!(&fp[16..17], ":");
    assert!(
        fp.chars()
            .all(|c| c == ':' || c.is_ascii_digit() || c.is_ascii_uppercase())
    );
    let pkcs8 = test_key().to_pkcs8_pem(LineEnding::LF).unwrap().to_string();
    assert_eq!(fp, key_fingerprint(&pkcs8).unwrap());
}

#[test]
fn rejects_bad_keys() {
    assert!(matches!(
        key_fingerprint("nonsense"),
        Err(SecureBootError::InvalidKey(_))
    ));
    let small = RsaPrivateKey::new(&mut TestRng(7), 1024).unwrap();
    let small_pem = small.to_pkcs1_pem(LineEnding::LF).unwrap().to_string();
    assert!(matches!(
        boot_sig(b"x", &small_pem, 0),
        Err(SecureBootError::KeySize(1024))
    ));
}

#[test]
fn errors_when_nothing_to_pack() {
    let mut boot = partition(&[("user-data", b"x"), ("empty", b"")]);
    let err = sign_boot_partition(&mut boot, &pem(), 0).unwrap_err();
    assert!(matches!(err, SecureBootError::NoBootFiles));
    assert!(boot.exists("user-data").unwrap());
    let mut empty = partition(&[]);
    let err = sign_boot_partition(&mut empty, &pem(), 0).unwrap_err();
    assert!(matches!(err, SecureBootError::NoFiles));
}

#[test]
fn image_size_follows_upstream_rule() {
    let mib = MIB as u64;
    assert_eq!(boot_img_size(0), 33 * mib);
    assert_eq!(boot_img_size(22 * mib), 34 * mib);
    assert_eq!(boot_img_size(100 * mib), 151 * mib);
    assert_eq!(boot_img_size(100 * mib + 1), 151 * mib);
}
