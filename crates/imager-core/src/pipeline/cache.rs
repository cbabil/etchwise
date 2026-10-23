//! The last-download cache (upstream `CacheManager`): the compressed
//! download is teed into `lastdownload.cache`, remembered in the `caching`
//! settings group by the image's uncompressed SHA-256, re-hashed before
//! reuse, and skipped when less than 5 GiB would remain free.

use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::buffers::{GIB, MIB};
use crate::settings::{Settings, SettingsError};

/// `IMAGEWRITER_MINIMAL_SPACE_FOR_CACHING`.
pub const MIN_FREE_AFTER_CACHING: u64 = 5 * GIB;
/// Cache file name in the app cache directory.
pub const CACHE_FILE_NAME: &str = "lastdownload.cache";
const GROUP: &str = "caching";
const IMAGE_SHA: &str = "lastDownloadSHA256";
const FILE_SHA: &str = "lastCacheFileHash";
const FILE_NAME: &str = "lastFileName";

/// What the settings remember about the cached download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheRecord {
    pub file: PathBuf,
    /// Uncompressed image hash; matches the OS list's `extract_sha256`.
    pub image_sha256: String,
    /// Hash of the cache file itself (the compressed download).
    pub file_sha256: String,
}

/// `caching/enabled`, default on.
pub fn caching_enabled(settings: &Settings) -> bool {
    settings
        .group_value(GROUP, "enabled")
        .is_none_or(crate::settings::variant_to_bool)
}

/// The remembered cache, if its file still exists and is non-empty.
pub fn load_record(settings: &Settings) -> Option<CacheRecord> {
    let text = |key| {
        settings
            .group_value(GROUP, key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    let (file, image_sha256) = (text(FILE_NAME), text(IMAGE_SHA));
    if file.is_empty() || image_sha256.is_empty() {
        return None;
    }
    let usable = fs::metadata(&file).is_ok_and(|m| m.is_file() && m.len() > 0);
    usable.then(|| CacheRecord {
        file: PathBuf::from(file),
        image_sha256,
        file_sha256: text("lastCacheFileHash"),
    })
}

/// `updateCacheFile`.
///
/// # Errors
/// Fails when the settings cannot be saved.
pub fn save_record(settings: &mut Settings, record: &CacheRecord) -> Result<(), SettingsError> {
    settings.set_group_value(
        GROUP,
        "lastDownloadSHA256",
        Value::String(record.image_sha256.clone()),
    )?;
    settings.set_group_value(
        GROUP,
        "lastCacheFileHash",
        Value::String(record.file_sha256.clone()),
    )?;
    settings.set_group_value(
        GROUP,
        "lastFileName",
        Value::String(record.file.to_string_lossy().into_owned()),
    )
}

/// `invalidateCache`: forgets the record and deletes the file.
///
/// # Errors
/// Fails when the settings cannot be saved.
pub fn invalidate(settings: &mut Settings, file: Option<&Path>) -> Result<(), SettingsError> {
    for key in [IMAGE_SHA, FILE_SHA, FILE_NAME] {
        settings.remove_group_value(GROUP, key)?;
    }
    if let Some(file) = file {
        let _ = fs::remove_file(file);
    }
    Ok(())
}

/// Background verification: the file still hashes to `file_sha256`.
///
/// # Errors
/// Fails when the file cannot be read.
pub fn verify_file(record: &CacheRecord) -> io::Result<bool> {
    let mut hasher = Sha256::new();
    io::copy(
        &mut io::BufReader::with_capacity(4 * MIB, File::open(&record.file)?),
        &mut hasher,
    )?;
    Ok(hex::encode(hasher.finalize()).eq_ignore_ascii_case(&record.file_sha256))
}

/// `isCached`: a verified record for exactly this image.
pub fn is_cached(record: Option<&CacheRecord>, verified: bool, image_sha256: &str) -> bool {
    verified
        && !image_sha256.is_empty()
        && record.is_some_and(|r| r.image_sha256 == image_sha256 && r.file.exists())
}

/// `setupCacheForDownload`'s space rule.
pub fn has_room(available_bytes: u64, download_size: u64) -> bool {
    available_bytes.saturating_sub(download_size) >= MIN_FREE_AFTER_CACHING
}

/// Copies everything read through it into the cache file and hashes it;
/// at end of stream the file hash is published to `done`. A failed cache
/// write only stops caching, never the download; an unfinished file is
/// removed when the tee is dropped.
pub struct CacheTee<R> {
    inner: R,
    path: PathBuf,
    file: Option<BufWriter<File>>,
    hasher: Sha256,
    done: Arc<OnceLock<String>>,
}

impl<R: Read> CacheTee<R> {
    /// Tees into a new file at `path`; caching is off if it cannot be created.
    pub fn new(inner: R, path: &Path, done: Arc<OnceLock<String>>) -> Self {
        let file = File::create(path)
            .ok()
            .map(|f| BufWriter::with_capacity(MIB, f));
        Self {
            inner,
            path: path.to_path_buf(),
            file,
            hasher: Sha256::new(),
            done,
        }
    }

    fn complete(&mut self) {
        if let Some(mut file) = self.file.take()
            && file.flush().is_ok()
        {
            let _ = self
                .done
                .set(hex::encode(std::mem::take(&mut self.hasher).finalize()));
        }
    }
}

impl<R> Drop for CacheTee<R> {
    fn drop(&mut self) {
        if self.done.get().is_none() {
            let _ = fs::remove_file(&self.path);
        }
    }
}

impl<R: Read> Read for CacheTee<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        if n == 0 && !buf.is_empty() {
            self.complete();
        } else if let Some(file) = self.file.as_mut() {
            if file.write_all(&buf[..n]).is_ok() {
                self.hasher.update(&buf[..n]);
            } else {
                self.file = None;
            }
        }
        Ok(n)
    }
}

#[cfg(test)]
mod tests;
