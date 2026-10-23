//! The write loop (upstream `DownloadThread::_openAndPrepareDevice`,
//! `_writeFile`, `_writeFileZeroSkip`, `_writeComplete` up to verify):
//! zero the first and last MiB, hold back the first chunk (it holds the
//! partition table and is written last), hash everything written and check
//! it against the expected SHA-256.

use std::io::{self, Read, SeekFrom};
use std::sync::Arc;
use std::sync::mpsc::sync_channel;

use sha2::{Digest, Sha256};

use self::chunks::{ChunkHasher, write_all_chunks};
use super::buffers::{DEFAULT_WRITE_BUFFER, MIB};
use super::device::Device;
use super::error::WriteError;
use super::progress::{Progress, WriteState};

/// Upstream `fastboot::SPARSE_BLK_SZ`, the zero-skip granularity.
pub(crate) const ZERO_BLOCK: usize = 4096;

/// How to write one image.
#[derive(Debug, Clone)]
pub struct WriteOptions {
    /// Lower-case hex SHA-256 of the uncompressed image (`extract_sha256`).
    pub expected_sha256: Option<String>,
    /// Debug option: skip zeroing the last MiB (counterfeit cards).
    pub skip_end_of_device: bool,
    /// Seek over all-zero 4 KiB blocks, only without an expected hash.
    /// Upstream has this on its base `DownloadThread` path, which image
    /// writes never take (they all go through the extract threads), so the
    /// app leaves it off; kept for parity and the CLI.
    pub zero_skip: bool,
    /// Write chunk size; the held-back first chunk has this size.
    pub buffer_size: usize,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            expected_sha256: None,
            skip_end_of_device: false,
            zero_skip: false,
            buffer_size: DEFAULT_WRITE_BUFFER,
        }
    }
}

/// What the loop wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    /// The held-back first chunk, still to be written at offset 0.
    pub first_block: Vec<u8>,
    /// Bytes covered on the device, first block included (`Tell()`).
    pub length: u64,
    /// SHA-256 of the bytes written (zero-skipped blocks excluded, as upstream).
    pub sha256: [u8; 32],
}

/// Zeroes the first and (unless skipped) last MiB, then rewinds.
///
/// # Errors
/// Fails when the device cannot be sized or written.
pub fn prepare<D: Device>(
    device: &mut D,
    options: &WriteOptions,
    progress: &Progress,
) -> Result<(), WriteError> {
    progress.set_status("Zero'ing out first and last MB of drive...");
    let size = device
        .size()
        .map_err(WriteError::io("getting device size"))?;
    let zeros = vec![0u8; MIB];
    device
        .seek(SeekFrom::Start(0))
        .map_err(WriteError::io("preparing storage device"))?;
    device
        .write_all(&zeros)
        .and_then(|()| device.flush())
        .map_err(WriteError::io("preparing storage device"))?;
    if !options.skip_end_of_device && size > MIB as u64 {
        progress.set_status("Zero'ing out end of drive...");
        device
            .seek(SeekFrom::Start(size - MIB as u64))
            .and_then(|_| device.write_all(&zeros))
            .and_then(|()| device.force_sync())
            .map_err(|_| WriteError::EndOfDevice)?;
    }
    device
        .seek(SeekFrom::Start(0))
        .map_err(WriteError::io("preparing storage device"))?;
    Ok(())
}

/// Writes `chunks` onto the device after [`prepare`]. The first chunk is
/// kept in memory and the device position skips past it. SHA-256 runs on
/// its own thread, overlapping the writes as upstream's `_hashData` does;
/// zero-skip, which hashes only what it writes, hashes inline.
///
/// # Errors
/// Fails on cancellation, read or write errors, or a hash mismatch.
pub fn write_chunks<D, I>(
    device: &mut D,
    chunks: I,
    options: &WriteOptions,
    progress: &Progress,
) -> Result<Written, WriteError>
where
    D: Device,
    I: IntoIterator<Item = io::Result<Vec<u8>>>,
{
    progress.set_state(WriteState::Writing);
    let zero_skip = options.zero_skip && options.expected_sha256.is_none();
    let written = std::thread::scope(|scope| {
        let mut hasher = if zero_skip {
            ChunkHasher::Inline(Sha256::new())
        } else {
            let (tx, rx) = sync_channel::<Arc<Vec<u8>>>(4);
            let thread = scope.spawn(move || {
                rx.into_iter()
                    .fold(Sha256::new(), |h, chunk| h.chain_update(&chunk[..]))
            });
            ChunkHasher::Threaded(tx, thread)
        };
        let outcome = write_all_chunks(device, chunks, &mut hasher, progress);
        let sha256 = hasher.finish();
        outcome.map(|w| Written { sha256, ..w })
    })?;
    if let Some(expected) = &options.expected_sha256 {
        let actual = hex::encode(written.sha256);
        if !expected.eq_ignore_ascii_case(&actual) {
            return Err(WriteError::HashMismatch {
                expected: expected.clone(),
                actual,
            });
        }
    }
    device.force_sync().map_err(WriteError::io("sync"))?;
    Ok(written)
}

/// [`write_chunks`] over a reader cut into `options.buffer_size` chunks.
///
/// # Errors
/// As [`write_chunks`].
pub fn write_stream<D: Device, R: Read>(
    device: &mut D,
    source: R,
    options: &WriteOptions,
    progress: &Progress,
) -> Result<Written, WriteError> {
    write_chunks(
        device,
        read_chunks(source, options.buffer_size),
        options,
        progress,
    )
}

/// Splits a reader into owned chunks of `size` bytes (the last may be
/// shorter), for callers that have a stream rather than chunks.
pub fn read_chunks<R: Read>(
    mut reader: R,
    size: usize,
) -> impl Iterator<Item = io::Result<Vec<u8>>> {
    std::iter::from_fn(move || {
        let mut chunk = Vec::with_capacity(size);
        match (&mut reader).take(size as u64).read_to_end(&mut chunk) {
            Ok(0) => None,
            Ok(_) => Some(Ok(chunk)),
            Err(e) => Some(Err(e)),
        }
    })
}

mod chunks;
#[cfg(test)]
mod tests;
