//! Secure boot signing of the boot partition (upstream `secureboot.cpp` and
//! `DownloadThread::_createSecureBootFiles`).
//!
//! Every non-empty file on the boot FAT partition except the customisation
//! files is packed into a FAT32 `boot.img`, the originals are deleted, and
//! `boot.img` plus its RSA-2048 signature `boot.sig` are written in their
//! place. The customisation files are then written back next to them.

mod image;
mod key;

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Read, Seek, Write};

use crate::fat::BootPartition;

pub use self::image::{boot_img_size, create_boot_img};
pub use self::key::{boot_sig, key_fingerprint};

/// Files that stay on the partition beside `boot.img` (matched
/// case-insensitively against the full path, written back in this casing).
pub const CUSTOMISATION_FILES: [&str; 4] =
    ["firstrun.sh", "user-data", "meta-data", "network-config"];

/// Why signing the boot partition failed.
#[derive(Debug, thiserror::Error)]
pub enum SecureBootError {
    /// The PEM text is not an RSA private key (PKCS#1 or PKCS#8).
    #[error("invalid RSA private key: {0}")]
    InvalidKey(String),
    /// The key is not RSA-2048 (size in bits).
    #[error("RSA key must be 2048 bits, got {0}")]
    KeySize(usize),
    /// The RSA signing operation failed.
    #[error("failed to sign boot image: {0}")]
    Sign(#[from] rsa::Error),
    /// The boot partition holds no readable, non-empty files.
    #[error("failed to extract boot partition files")]
    NoFiles,
    /// Only customisation files were found; nothing to pack.
    #[error("no boot files found to package")]
    NoBootFiles,
    /// The boot partition or the in-memory image could not be accessed.
    #[error("boot partition I/O error: {0}")]
    Io(#[from] io::Error),
}

/// Packs the boot partition into a signed `boot.img`/`boot.sig` pair.
///
/// `timestamp` (Unix seconds) goes into the `ts:` line of `boot.sig`.
///
/// # Errors
/// Fails on an invalid key, an empty partition, or partition I/O errors.
pub fn sign_boot_partition<D: Read + Write + Seek>(
    boot: &mut BootPartition<D>,
    private_key_pem: &str,
    timestamp: u64,
) -> Result<(), SecureBootError> {
    let all_files = extract_files(boot)?;
    if all_files.is_empty() {
        return Err(SecureBootError::NoFiles);
    }
    let (boot_files, custom_files) = split_customisation(&all_files);
    if boot_files.is_empty() {
        return Err(SecureBootError::NoBootFiles);
    }
    let boot_img = create_boot_img(&boot_files)?;
    let sig = boot_sig(&boot_img, private_key_pem, timestamp)?;

    for path in boot_files.keys() {
        // A file that has already gone is not an error (upstream warns).
        boot.delete_file(path)?;
    }
    remove_directories(boot, all_files.keys());

    boot.write_file("boot.img", &boot_img)?;
    boot.write_file("boot.sig", sig.as_bytes())?;
    for (name, data) in &custom_files {
        boot.write_file(name, data)?;
    }
    Ok(())
}

/// Reads every non-empty file; zero-length files are skipped (and so stay on
/// the partition), as upstream's `extractFatPartitionFiles` does.
fn extract_files<D: Read + Write + Seek>(
    boot: &mut BootPartition<D>,
) -> io::Result<BTreeMap<String, Vec<u8>>> {
    let mut files = BTreeMap::new();
    for path in boot.list_files()? {
        if let Some(data) = boot.read_file(&path)?.filter(|d| !d.is_empty()) {
            files.insert(path, data);
        }
    }
    Ok(files)
}

type FileMap = BTreeMap<String, Vec<u8>>;

/// Splits into (boot files, customisation files keyed by canonical name).
fn split_customisation(all: &FileMap) -> (FileMap, FileMap) {
    let mut boot_files = BTreeMap::new();
    let mut custom_files = BTreeMap::new();
    for (path, data) in all {
        let lower = path.to_lowercase();
        match CUSTOMISATION_FILES.iter().find(|name| **name == lower) {
            Some(name) => custom_files.insert((*name).to_owned(), data.clone()),
            None => boot_files.insert(path.clone(), data.clone()),
        };
    }
    (boot_files, custom_files)
}

/// Removes the directories the moved files lived in, deepest first.
/// Failures are ignored like upstream: a directory still holding a skipped
/// zero-length file simply stays.
fn remove_directories<'a, D: Read + Write + Seek>(
    boot: &mut BootPartition<D>,
    paths: impl Iterator<Item = &'a String>,
) {
    let unique: BTreeSet<&str> = paths
        .flat_map(|path| path.match_indices('/').map(|(i, _)| &path[..i]))
        .collect();
    let mut dirs: Vec<&str> = unique.into_iter().collect();
    dirs.sort_by_key(|dir| Reverse(dir.matches('/').count()));
    for dir in dirs {
        // Non-empty or already-removed directories are left alone.
        let _ignored = boot.delete_file(dir);
    }
}

#[cfg(test)]
mod tests;
