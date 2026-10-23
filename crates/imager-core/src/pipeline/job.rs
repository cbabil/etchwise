//! One write, end to end (upstream `ImageWriter::startWrite` checks plus
//! the extract thread): source → (cache tee) → decoder on its own thread →
//! write loop → verify → customise → MBR, returning the new cache record.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::cache::{CACHE_FILE_NAME, CacheRecord};
use super::device::Device;
use super::error::WriteError;
use super::feed::spawn_decoder;
use super::finalize::{finalize, verify};
use super::progress::{Progress, WriteState};
use super::writer::{WriteOptions, prepare, write_chunks};
use crate::customise::plan::CustomisationPlan;

/// Where the image bytes come from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Remote(String),
    Local(PathBuf),
}

/// Everything one write needs.
#[derive(Debug, Clone, Default)]
pub struct Job {
    pub source: Option<Source>,
    /// Uncompressed size from the OS list, 0 when unknown.
    pub extract_size: u64,
    pub write: WriteOptions,
    pub verify: bool,
    pub plan: Option<CustomisationPlan>,
    /// Directory for `lastdownload.cache` when caching this download.
    pub cache_dir: Option<PathBuf>,
    /// A zip of files to extract onto a fresh FAT32 card
    /// (`contains_multiple_files`) rather than a disk image.
    pub multiple_files: bool,
}

impl Job {
    /// The cache file for this download: only remote images with an
    /// expected hash are cached (a record needs the hash).
    pub fn cache_file(&self) -> Option<PathBuf> {
        let remote = matches!(self.source, Some(Source::Remote(_)));
        (remote && !self.multiple_files && self.write.expected_sha256.is_some())
            .then(|| self.cache_dir.as_ref().map(|d| d.join(CACHE_FILE_NAME)))
            .flatten()
    }
}

/// Upstream's pre-write checks against the device size.
///
/// # Errors
/// Fails when the image cannot fit or is not whole sectors.
pub fn check_sizes(extract_size: u64, device_size: u64) -> Result<(), WriteError> {
    if device_size != 0 && extract_size > device_size {
        return Err(WriteError::TooSmall {
            needed: extract_size,
        });
    }
    if !extract_size.is_multiple_of(512) {
        return Err(WriteError::NotSectorMultiple { size: extract_size });
    }
    Ok(())
}

/// Runs the job; on success returns the cache record to remember, if any.
///
/// # Errors
/// Any [`WriteError`]; progress ends in `Failed` or `Cancelled`.
pub fn run<D: Device>(
    device: &mut D,
    job: &Job,
    progress: &Arc<Progress>,
) -> Result<Option<CacheRecord>, WriteError> {
    let result = run_inner(device, job, progress);
    progress.set_state(match &result {
        Ok(_) => WriteState::Succeeded,
        Err(WriteError::Cancelled) => WriteState::Cancelled,
        Err(_) => WriteState::Failed,
    });
    result
}

fn run_inner<D: Device>(
    device: &mut D,
    job: &Job,
    progress: &Arc<Progress>,
) -> Result<Option<CacheRecord>, WriteError> {
    progress.set_state(WriteState::Preparing);
    if job.multiple_files {
        let source = super::feed::open_source(job, progress, None)
            .map_err(|e| WriteError::Decompress(e.to_string()))?;
        super::multifile::extract_multi_file(
            device,
            source,
            job.write.expected_sha256.as_deref(),
            progress,
        )?;
        return Ok(None);
    }
    let size = device
        .size()
        .map_err(WriteError::io("getting device size"))?;
    check_sizes(job.extract_size, size)?;
    progress
        .write_total
        .store(job.extract_size, std::sync::atomic::Ordering::Relaxed);
    prepare(device, &job.write, progress)?;
    let (chunks, producer) = spawn_decoder(job, Arc::clone(progress));
    let written = write_chunks(device, chunks, &job.write, progress);
    let cache_hash = producer.join().unwrap_or(None);
    let written = written.inspect_err(|_| {
        if let Some(path) = job.cache_file() {
            let _ = std::fs::remove_file(path);
        }
    })?;
    if job.verify {
        verify(device, &written, progress)?;
    }
    finalize(device, written, job.plan.as_ref(), &instance_id(), progress)?;
    Ok(cache_hash
        .zip(job.cache_dir.as_ref())
        .zip(job.write.expected_sha256.as_ref())
        .map(|((file_sha256, dir), image)| CacheRecord {
            file: dir.join(CACHE_FILE_NAME),
            image_sha256: image.clone(),
            file_sha256,
        }))
}

/// `rpi-imager-<ms since epoch>`, the cloud-init instance id.
pub fn instance_id() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("rpi-imager-{ms}")
}

#[cfg(test)]
mod tests;
