//! The decoder thread: opens the source (teeing remote downloads into the
//! cache), decompresses it and hands write-buffer-sized chunks to the write
//! loop through a bounded channel, so download, decompression and writing
//! overlap.

use std::fs::File;
use std::io;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, OnceLock};
use std::thread::{self, JoinHandle};

use super::cache::CacheTee;
use super::decode::decoder;
use super::http::{ResumableDownload, client};
use super::job::{Job, Source};
use super::progress::Progress;
use super::writer::read_chunks;

type Chunk = io::Result<Vec<u8>>;

/// The raw (still compressed) source: a local file, or a download teed
/// into the cache when `cache_done` is given and the job caches.
pub fn open_source(
    job: &Job,
    progress: &Arc<Progress>,
    cache_done: Option<&Arc<OnceLock<String>>>,
) -> io::Result<Box<dyn io::Read + Send>> {
    match &job.source {
        Some(Source::Remote(url)) => {
            let download = ResumableDownload::open(
                client().map_err(io::Error::other)?,
                url,
                Arc::clone(progress),
            )?;
            Ok(match (job.cache_file(), cache_done) {
                (Some(path), Some(done)) => {
                    Box::new(CacheTee::new(download, &path, Arc::clone(done)))
                }
                _ => Box::new(download),
            })
        }
        Some(Source::Local(path)) => {
            let file = File::open(path)?;
            Progress::set(&progress.download_total, file.metadata()?.len());
            Ok(Box::new(file))
        }
        None => Err(io::Error::new(io::ErrorKind::NotFound, "no image selected")),
    }
}

/// Starts decoding; the thread returns the cache file hash if it cached.
pub(super) fn spawn_decoder(
    job: &Job,
    progress: Arc<Progress>,
) -> (Receiver<Chunk>, JoinHandle<Option<String>>) {
    let (tx, rx) = sync_channel::<Chunk>(4);
    let job = job.clone();
    let handle = thread::spawn(move || match decode_into(&job, &tx, &progress) {
        Ok(hash) => hash,
        Err(e) => {
            let _ = tx.send(Err(e));
            None
        }
    });
    (rx, handle)
}

fn decode_into(
    job: &Job,
    tx: &SyncSender<Chunk>,
    progress: &Arc<Progress>,
) -> io::Result<Option<String>> {
    let done = Arc::new(OnceLock::new());
    let raw = open_source(job, progress, Some(&done))?;
    for chunk in read_chunks(decoder(raw)?, job.write.buffer_size) {
        if progress.is_cancelled() || tx.send(chunk).is_err() {
            break;
        }
    }
    Ok(done.get().cloned())
}
