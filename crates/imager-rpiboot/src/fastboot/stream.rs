//! The streaming half of the flash flow (upstream's download and
//! decompress threads feeding the flash loop through ring buffers): a
//! scoped thread decodes the source into 16 MiB chunks over a bounded
//! channel; this thread hashes them, sparse-encodes them and sends each
//! finished segment with `download:` + `flash:mmcblk0`.

use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use imager_core::pipeline::Progress;
use imager_core::pipeline::decode::decoder;
use sha2::{Digest, Sha256};

use super::encoder::SparseEncoder;
use super::flash::{FLASH_PARTITION, FlashError};
use super::protocol::{download, flash};
use crate::transport::UsbTransport;

/// Decompressed bytes per channel message (upstream's 16 MB write slot).
const CHUNK: u64 = 16 * 1024 * 1024;
/// Chunks buffered between the decoder and the USB writer.
const QUEUE_DEPTH: usize = 2;

type Chunk = Result<Vec<u8>, FlashError>;

/// Remembers whether the underlying source failed, to tell a download
/// error from a decompression error.
struct Tracked<'a, R> {
    inner: R,
    failed: &'a AtomicBool,
}

impl<R: Read> Read for Tracked<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).inspect_err(|_| {
            self.failed.store(true, Ordering::Relaxed);
        })
    }
}

fn decode_into(
    source: impl Read + Send,
    tx: &SyncSender<Chunk>,
    progress: &Progress,
    failed: &AtomicBool,
) -> io::Result<()> {
    let mut reader = decoder(Tracked {
        inner: source,
        failed,
    })?;
    while !progress.is_cancelled() {
        let mut chunk = Vec::with_capacity(CHUNK as usize);
        reader.by_ref().take(CHUNK).read_to_end(&mut chunk)?;
        if chunk.is_empty() || tx.send(Ok(chunk)).is_err() {
            break;
        }
    }
    Ok(())
}

/// Decoder thread body (upstream `downloadProducer` and
/// `decompressConsumerProducer`).
fn produce(source: impl Read + Send, tx: SyncSender<Chunk>, progress: &Progress) {
    let failed = AtomicBool::new(false);
    if let Err(e) = decode_into(source, &tx, progress, &failed) {
        let message = e.to_string();
        let _ = tx.send(Err(if failed.load(Ordering::Relaxed) {
            FlashError::Fetch(message)
        } else {
            FlashError::Decompress(message)
        }));
    }
}

/// Sparse encoder, hash and transport for the consumer side.
pub(crate) struct Flasher<'t, T: ?Sized> {
    transport: &'t mut T,
    encoder: SparseEncoder,
    hash: Option<Sha256>,
    extract_len: u64,
    total_fed: u64,
}

impl<'t, T: UsbTransport + ?Sized> Flasher<'t, T> {
    /// `hash`: compute the SHA-256 of the decompressed stream.
    pub(crate) fn new(
        transport: &'t mut T,
        encoder: SparseEncoder,
        extract_len: u64,
        hash: bool,
    ) -> Self {
        Self {
            transport,
            encoder,
            hash: hash.then(Sha256::new),
            extract_len,
            total_fed: 0,
        }
    }

    /// Sends the encoder's ready segment, if any (upstream `sendSegment`).
    fn send_ready(&mut self, progress: &Progress) -> Result<bool, FlashError> {
        let Some(segment) = self.encoder.take_segment() else {
            return Ok(false);
        };
        let cancel = || progress.is_cancelled();
        let cancelled = |e: String, wrap: fn(String) -> FlashError| {
            if progress.is_cancelled() {
                FlashError::Cancelled
            } else {
                wrap(e)
            }
        };
        download(self.transport, segment.data, None, &cancel)
            .map_err(|e| cancelled(e.0, FlashError::Download))?;
        flash(self.transport, FLASH_PARTITION, 120_000)
            .map_err(|e| cancelled(e.0, FlashError::Flash))?;
        Progress::set(&progress.bytes_written, self.total_fed);
        let total = if self.extract_len > 0 {
            self.extract_len
        } else {
            self.total_fed
        };
        Progress::set(&progress.write_total, total);
        Ok(true)
    }

    fn feed(&mut self, mut data: &[u8], progress: &Progress) -> Result<(), FlashError> {
        if let Some(hash) = self.hash.as_mut() {
            hash.update(data);
        }
        self.total_fed += data.len() as u64;
        while !data.is_empty() {
            let used = self.encoder.feed(data);
            data = &data[used..];
            self.send_ready(progress)?;
        }
        Ok(())
    }

    /// Drains the encoder; `finish` may need several rounds.
    fn finish(&mut self, progress: &Progress) -> Result<(), FlashError> {
        loop {
            self.encoder.finish();
            if !self.send_ready(progress)? {
                return Ok(());
            }
        }
    }
}

fn consume<T: UsbTransport + ?Sized>(
    rx: &Receiver<Chunk>,
    flasher: &mut Flasher<'_, T>,
    progress: &Progress,
) -> Result<(), FlashError> {
    for chunk in rx {
        if progress.is_cancelled() {
            return Err(FlashError::Cancelled);
        }
        flasher.feed(&chunk?, progress)?;
    }
    if progress.is_cancelled() {
        return Err(FlashError::Cancelled);
    }
    flasher.finish(progress)
}

/// Runs the pipeline; returns the transport and the lowercase hex SHA-256
/// of the decompressed data (when hashing was requested).
pub(crate) fn stream_image<'t, T: UsbTransport + ?Sized>(
    mut flasher: Flasher<'t, T>,
    source: impl Read + Send,
    progress: &Progress,
) -> Result<(&'t mut T, Option<String>), FlashError> {
    let (tx, rx) = sync_channel(QUEUE_DEPTH);
    std::thread::scope(|scope| {
        scope.spawn(move || produce(source, tx, progress));
        let result = consume(&rx, &mut flasher, progress);
        drop(rx); // unblocks the decoder after an early error
        result
    })?;
    let digest = flasher.hash.map(|h| hex::encode(h.finalize()));
    Ok((flasher.transport, digest))
}
