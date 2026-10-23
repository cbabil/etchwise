//! Downloads and caches rpiboot firmware (upstream `firmware_manager.cpp`):
//! files come individually from the usbboot and rpi-sb-provisioner
//! repositories' default branches and are cached under `<root>/master/`.
//! The cache root is chosen by the caller (upstream: the app's cache
//! location plus `rpiboot-firmware`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use super::fetcher::Fetcher;
use super::manifest::{
    ManifestEntry, build_manifest, extract_bootcode_from_bootfiles, needs_bootcode_extraction,
    validate_cache_for_device,
};
use crate::types::{ChipGeneration, ProgressCallback, SideloadMode};

const DOWNLOADING: &str = "Downloading rpiboot firmware...";

/// Firmware cache under a root directory.
pub struct FirmwareManager<'a> {
    root: PathBuf,
    fetcher: &'a dyn Fetcher,
}

fn tmp_path(dest: &Path) -> PathBuf {
    let mut tmp = dest.as_os_str().to_owned();
    tmp.push(".tmp");
    PathBuf::from(tmp)
}

impl<'a> FirmwareManager<'a> {
    /// A cache rooted at `root`, downloading with `fetcher`.
    pub fn new(root: &Path, fetcher: &'a dyn Fetcher) -> Self {
        Self {
            root: root.to_path_buf(),
            fetcher,
        }
    }

    /// Removes all cached firmware.
    #[cfg(test)]
    pub(crate) fn clear_cache(&self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }

    /// `<root>/master` when it validates for `mode` and `chip`.
    #[cfg(test)]
    pub(crate) fn find_cached_version(
        &self,
        mode: SideloadMode,
        chip: ChipGeneration,
    ) -> Option<PathBuf> {
        let dir = self.root.join("master");
        validate_cache_for_device(&dir, mode, chip).then_some(dir)
    }

    /// Downloads whatever is missing and returns the firmware directory.
    /// BCM2711/BCM2712 re-extract their bootcode on every run so a stale
    /// binary is never used.
    ///
    /// # Errors
    /// Upstream's `lastError()` text.
    pub fn ensure_available(
        &self,
        mode: SideloadMode,
        chip: ChipGeneration,
        progress: ProgressCallback<'_>,
        cancel: &AtomicBool,
    ) -> Result<PathBuf, String> {
        let dir = self.root.join("master");
        let manifest = build_manifest(mode, chip);
        if manifest.is_empty() {
            return Err("No firmware files defined for this device".into());
        }
        let extract = needs_bootcode_extraction(mode, chip);
        if self.all_cached(&dir, &manifest) && !extract {
            return Ok(dir);
        }
        progress(0, 100, DOWNLOADING);
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Cannot create firmware cache directory: {e}"))?;
        self.download_missing(&dir, &manifest, &mut *progress, cancel)?;
        if extract {
            extract_bootcode_from_bootfiles(&dir, chip)?;
        }
        if !validate_cache_for_device(&dir, mode, chip) {
            let _ = std::fs::remove_dir_all(&dir);
            return Err("Downloaded firmware does not contain required files".into());
        }
        progress(100, 100, "Firmware ready");
        Ok(dir)
    }

    /// Every manifest file exists; the first missing one also loses any
    /// leftover `.tmp` from an interrupted download.
    fn all_cached(&self, dir: &Path, manifest: &[ManifestEntry]) -> bool {
        match manifest
            .iter()
            .map(|e| dir.join(&e.local_path))
            .find(|p| !p.exists())
        {
            Some(missing) => {
                let _ = std::fs::remove_file(tmp_path(&missing));
                false
            }
            None => true,
        }
    }

    fn download_missing(
        &self,
        dir: &Path,
        manifest: &[ManifestEntry],
        progress: ProgressCallback<'_>,
        cancel: &AtomicBool,
    ) -> Result<(), String> {
        let total = manifest.len() as u64;
        for (i, entry) in (0u64..).zip(manifest) {
            if cancel.load(Ordering::SeqCst) {
                return Err("Cancelled".into());
            }
            let dest = dir.join(&entry.local_path);
            if dest.exists() {
                continue;
            }
            self.download_file(&entry.url, &dest, cancel)?;
            // Whole-file fetches: overall progress advances per file.
            progress((i * 100 + 100) / total, 100, DOWNLOADING);
        }
        Ok(())
    }

    /// Fetches into `<dest>.tmp` and renames, so a partial download never
    /// poisons the cache.
    fn download_file(&self, url: &str, dest: &Path, cancel: &AtomicBool) -> Result<(), String> {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create directory: {e}"))?;
        }
        let tmp = tmp_path(dest);
        let fetched = self.fetcher.fetch(url);
        if cancel.load(Ordering::SeqCst) {
            return Err("Download cancelled".into());
        }
        let body = fetched.map_err(|e| format!("Download failed: {e}"))?;
        std::fs::write(&tmp, body).map_err(|_| format!("Cannot create file: {}", tmp.display()))?;
        std::fs::rename(&tmp, dest).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            format!("Failed to rename downloaded file: {e}")
        })
    }
}
