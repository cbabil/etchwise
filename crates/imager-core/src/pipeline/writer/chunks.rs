//! The chunk loop: hold back the first chunk, write the rest, hash
//! alongside the writes, and upstream's zero-skip writer.

use std::io::{self, SeekFrom};
use std::sync::Arc;
use std::sync::mpsc::SyncSender;
use std::thread::ScopedJoinHandle;

use sha2::{Digest, Sha256};

use super::{Written, ZERO_BLOCK};
use crate::pipeline::device::Device;
use crate::pipeline::error::WriteError;
use crate::pipeline::progress::Progress;

/// Where written bytes get hashed.
pub(super) enum ChunkHasher<'scope> {
    Threaded(SyncSender<Arc<Vec<u8>>>, ScopedJoinHandle<'scope, Sha256>),
    Inline(Sha256),
}

impl ChunkHasher<'_> {
    pub(super) fn add(&mut self, chunk: &Arc<Vec<u8>>) {
        match self {
            Self::Threaded(tx, _) => {
                let _ = tx.send(Arc::clone(chunk));
            }
            Self::Inline(h) => h.update(&chunk[..]),
        }
    }

    pub(super) fn finish(self) -> [u8; 32] {
        match self {
            Self::Threaded(tx, thread) => {
                drop(tx);
                thread.join().unwrap_or_default().finalize().into()
            }
            Self::Inline(h) => h.finalize().into(),
        }
    }
}

pub(super) fn write_all_chunks<D: Device>(
    device: &mut D,
    chunks: impl IntoIterator<Item = io::Result<Vec<u8>>>,
    hasher: &mut ChunkHasher<'_>,
    progress: &Progress,
) -> Result<Written, WriteError> {
    let mut first_block = None;
    let mut position = 0u64;
    for chunk in chunks {
        if progress.is_cancelled() {
            return Err(WriteError::Cancelled);
        }
        let chunk = Arc::new(chunk.map_err(|e| WriteError::Decompress(e.to_string()))?);
        let len = chunk.len() as u64;
        match (first_block.is_none(), &mut *hasher) {
            (true, h) => {
                h.add(&chunk);
                first_block = Some(chunk.to_vec());
                device
                    .seek(SeekFrom::Start(len))
                    .map_err(WriteError::io("writing"))?;
            }
            (false, ChunkHasher::Inline(h)) => write_skipping_zeros(device, &chunk, h)?,
            (false, h) => {
                h.add(&chunk);
                device
                    .write_all(&chunk)
                    .map_err(WriteError::io("writing"))?;
            }
        }
        position += len;
        Progress::add(&progress.bytes_written, len);
    }
    if progress.is_cancelled() {
        return Err(WriteError::Cancelled);
    }
    Ok(Written {
        first_block: first_block.unwrap_or_default(),
        length: position,
        sha256: [0; 32],
    })
}

/// Writes non-zero runs and seeks over 4 KiB-aligned zero blocks; only the
/// written bytes are hashed (upstream `_writeFileZeroSkip`). Blocks are
/// aligned to the chunk start, as upstream.
pub(super) fn write_skipping_zeros<D: Device>(
    device: &mut D,
    chunk: &[u8],
    hasher: &mut Sha256,
) -> Result<(), WriteError> {
    let (mut start, mut at) = (0, 0);
    while at + ZERO_BLOCK <= chunk.len() {
        if chunk[at..at + ZERO_BLOCK].iter().all(|b| *b == 0) {
            let zeros_from = at;
            while at + ZERO_BLOCK <= chunk.len()
                && chunk[at..at + ZERO_BLOCK].iter().all(|b| *b == 0)
            {
                at += ZERO_BLOCK;
            }
            hasher.update(&chunk[start..zeros_from]);
            device
                .write_all(&chunk[start..zeros_from])
                .map_err(WriteError::io("writing"))?;
            device
                .seek(SeekFrom::Current((at - zeros_from) as i64))
                .map_err(WriteError::io("writing"))?;
            start = at;
        } else {
            at += ZERO_BLOCK;
        }
    }
    hasher.update(&chunk[start..]);
    device
        .write_all(&chunk[start..])
        .map_err(WriteError::io("writing"))
}
