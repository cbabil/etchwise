//! Sizes of a local image file (upstream `ImageWriter::setSrc`,
//! `_parseXZFile`, `_parseGzFile`, `_parseCompressedFile`): the
//! uncompressed size drives the capacity check and progress bar.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// What upstream learns about a local source before writing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourceSize {
    /// File size.
    pub download: u64,
    /// Uncompressed size, 0 when unknown.
    pub extract: u64,
    /// A zip with more than one non-empty entry.
    pub multiple_files: bool,
    /// False for `.gz` without a known size: the progress is indeterminate.
    pub extract_known: bool,
}

const COMPRESSED: [&str; 7] = [".zip", ".xz", ".bz2", ".gz", ".7z", ".zst", ".cache"];

/// Inspects `path` like upstream does for custom images.
pub fn local_source_size(path: &Path) -> SourceSize {
    let download = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let name = path.to_string_lossy().to_lowercase();
    let (extract, multiple_files) = if !COMPRESSED.iter().any(|ext| name.ends_with(ext)) {
        (download, false)
    } else if name.ends_with(".xz") {
        (xz_uncompressed_size(path).unwrap_or(0), false)
    } else if name.ends_with(".gz") {
        (gz_estimate(path).unwrap_or(0), false)
    } else {
        zip_entries(path).unwrap_or((0, false))
    };
    SourceSize {
        download,
        extract,
        multiple_files,
        extract_known: !(extract == 0 && name.ends_with(".gz")),
    }
}

/// Uncompressed size from the last xz stream's index.
fn xz_uncompressed_size(path: &Path) -> Option<u64> {
    let (mut file, len, footer) = read_tail(path, 12, 12)?;
    let crc = u32::from_le_bytes(footer[0..4].try_into().ok()?);
    if &footer[10..12] != b"YZ" || crc32fast::hash(&footer[4..10]) != crc {
        return None;
    }
    let backward = (u64::from(u32::from_le_bytes(footer[4..8].try_into().ok()?)) + 1) * 4;
    if backward >= 1_000_000 || backward >= len - 12 {
        return None;
    }
    let mut index = vec![0u8; backward as usize];
    file.seek(SeekFrom::Start(len - 12 - backward)).ok()?;
    file.read_exact(&mut index).ok()?;
    parse_xz_index(&index)
}

/// The last `n` bytes of a file longer than `min_len`, with the open file
/// and its length.
fn read_tail(path: &Path, n: usize, min_len: u64) -> Option<(File, u64, Vec<u8>)> {
    let mut file = File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    if len <= min_len {
        return None;
    }
    let mut tail = vec![0u8; n];
    file.seek(SeekFrom::Start(len - n as u64)).ok()?;
    file.read_exact(&mut tail).ok()?;
    Some((file, len, tail))
}

/// Sums the uncompressed sizes in an xz index (indicator, count, records).
fn parse_xz_index(index: &[u8]) -> Option<u64> {
    let (&indicator, mut rest) = index.split_first()?;
    if indicator != 0 {
        return None;
    }
    let records = varint(&mut rest)?;
    let mut total = 0u64;
    for _ in 0..records {
        varint(&mut rest)?;
        total = total.checked_add(varint(&mut rest)?)?;
    }
    Some(total)
}

/// xz multibyte integer (7 bits per byte, little-endian, max 9 bytes).
fn varint(bytes: &mut &[u8]) -> Option<u64> {
    let mut value = 0u64;
    for i in 0..9 {
        let (&b, rest) = bytes.split_first()?;
        *bytes = rest;
        value |= u64::from(b & 0x7F) << (7 * i);
        if b & 0x80 == 0 {
            return Some(value);
        }
    }
    None
}

/// ISIZE (size mod 2^32), raised by 4 GiB until it exceeds the file size.
fn gz_estimate(path: &Path) -> Option<u64> {
    let (_, len, isize) = read_tail(path, 4, 8)?;
    let mut size = u64::from(u32::from_le_bytes(isize[..].try_into().ok()?));
    while size < len {
        size += 1 << 32;
    }
    Some(size)
}

/// Total of non-empty zip entries and whether there are several.
fn zip_entries(path: &Path) -> Option<(u64, bool)> {
    let mut archive = zip::ZipArchive::new(File::open(path).ok()?).ok()?;
    let mut total = 0;
    let mut count = 0;
    for i in 0..archive.len() {
        let size = archive.by_index_raw(i).ok()?.size();
        if size > 0 {
            total += size;
            count += 1;
        }
    }
    Some((total, count > 1))
}

#[cfg(test)]
mod tests;
