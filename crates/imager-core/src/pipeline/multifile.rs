//! Images shipped as a zip of files rather than a disk image (upstream
//! `contains_multiple_files`, `DownloadExtractThread::extractMultiFileRun`):
//! erase the card to MBR + FAT32, then extract every entry into it. The
//! hash covers the downloaded archive. On failure the extracted files are
//! removed again.

use std::io::{self, BufReader, Read};

use sha2::{Digest, Sha256};

use super::device::Device;
use super::error::WriteError;
use super::progress::{Progress, WriteState};
use crate::fat::BootPartition;

/// Formats `device` and extracts the zip read from `source` onto it.
///
/// # Errors
/// Formatting, extraction, unsafe paths, cancellation or a hash mismatch.
pub fn extract_multi_file<D: Device, R: Read>(
    device: &mut D,
    source: R,
    expected_sha256: Option<&str>,
    progress: &Progress,
) -> Result<(), WriteError> {
    if progress.is_cancelled() {
        return Err(WriteError::Cancelled);
    }
    crate::erase::format_device(device, progress)?;
    let mut boot = BootPartition::open(&mut *device).map_err(WriteError::Customise)?;
    progress.set_state(WriteState::Writing);
    // Upstream formats in a separate thread; extraction progress starts at 0.
    Progress::set(&progress.bytes_written, 0);
    let mut input = HashingReader {
        inner: source,
        hasher: Sha256::new(),
    };
    let mut extracted = Vec::new();
    let outcome = extract_entries(
        &mut boot,
        &mut BufReader::new(&mut input),
        &mut extracted,
        progress,
    )
    .and_then(|()| {
        io::copy(&mut input, &mut io::sink()).map_err(|e| WriteError::Decompress(e.to_string()))?;
        check_hash(expected_sha256, std::mem::take(&mut input.hasher))
    });
    if outcome.is_err() {
        for path in &extracted {
            let _ = boot.delete_file(path);
        }
    }
    boot.finish().map_err(WriteError::io("final sync"))?;
    outcome?;
    device.force_sync().map_err(WriteError::io("final sync"))
}

fn extract_entries<R: Read>(
    boot: &mut BootPartition<&mut impl Device>,
    reader: &mut R,
    extracted: &mut Vec<String>,
    progress: &Progress,
) -> Result<(), WriteError> {
    loop {
        if progress.is_cancelled() {
            return Err(WriteError::Cancelled);
        }
        let Some(mut entry) = zip::read::read_zipfile_from_stream(reader)
            .map_err(|e| WriteError::Decompress(e.to_string()))?
        else {
            return Ok(());
        };
        let path = safe_path(&entry)?;
        let fail = |e: io::Error| WriteError::Decompress(format!("{path}: {e}"));
        if entry.is_dir() {
            boot.create_dir_all(&path).map_err(fail)?;
            continue;
        }
        extracted.push(path.clone());
        let written = boot.write_new_file(&path, &mut entry).map_err(fail)?;
        Progress::add(&progress.bytes_written, written);
    }
}

/// libarchive's `SECURE_NOABSOLUTEPATHS | SECURE_NODOTDOT`.
fn safe_path(entry: &zip::read::ZipFile<'_, impl Read>) -> Result<String, WriteError> {
    let unsafe_path =
        || WriteError::Decompress(format!("refusing unsafe path in archive: {}", entry.name()));
    let path = entry.enclosed_name().ok_or_else(unsafe_path)?;
    let text: Vec<String> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    Ok(text.join("/"))
}

fn check_hash(expected: Option<&str>, hasher: Sha256) -> Result<(), WriteError> {
    let actual = hex::encode(hasher.finalize());
    match expected {
        Some(expected) if !expected.eq_ignore_ascii_case(&actual) => {
            Err(WriteError::HashMismatch {
                expected: expected.to_owned(),
                actual,
            })
        }
        _ => Ok(()),
    }
}

/// Hashes everything read through it.
struct HashingReader<R> {
    inner: R,
    hasher: Sha256,
}

impl<R: Read> Read for HashingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }
}

#[cfg(test)]
mod tests;
