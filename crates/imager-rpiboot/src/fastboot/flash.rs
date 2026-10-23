//! The fastboot flash flow (upstream `FastbootFlashThread::runImpl`):
//! query max-download-size, decompress and sparse-encode the image while
//! streaming each segment with `download:` + `flash:mmcblk0`, check the
//! SHA-256 of the decompressed data, apply OS customisation over the file
//! protocol, then `reboot`.
//!
//! Upstream has no read-back verification on this path (fastboot cannot
//! read the card); the hash of the streamed data is the only check.

use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use imager_core::customise::plan::CustomisationPlan;
use imager_core::pipeline::{Progress, WriteState};

use super::bmap::BlockMap;
use super::customise::apply_customisation;
use super::encoder::{MIN_SEGMENT_SIZE, SparseEncoder};
use super::protocol::{get_var, send_command};
use super::stream::{Flasher, stream_image};
use crate::transport::UsbTransport;
use crate::types::parse_leading;

/// Used when the device does not report max-download-size.
pub const DEFAULT_MAX_DOWNLOAD_SIZE: u32 = 256 * 1024 * 1024;
/// The partition every segment is flashed to.
pub const FLASH_PARTITION: &str = "mmcblk0";

/// What to write and how to check it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlashOptions {
    /// Hex SHA-256 of the decompressed image (upstream `expectedHash`,
    /// compared exactly); `None` or empty skips the check.
    pub expected_sha256: Option<String>,
    /// Uncompressed image size in bytes (upstream `extractLen`); 0 when
    /// unknown, which makes every segment cover only what was seen so far.
    pub extract_len: u64,
    /// The `.bmap` document, if the OS provides one; unparsable maps are
    /// ignored, as upstream.
    pub bmap: Option<Vec<u8>>,
}

/// Why a fastboot flash stopped, with upstream's English messages.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FlashError {
    #[error("Failed to open fastboot device: {0}")]
    Open(String),
    #[error("Fastboot download failed: {0}")]
    Download(String),
    #[error("Fastboot flash failed: {0}")]
    Flash(String),
    /// Reading the (compressed) source failed.
    #[error("Download failed: {0}")]
    Fetch(String),
    #[error("Decompression failed: {0}")]
    Decompress(String),
    #[error("Cancelled")]
    Cancelled,
    #[error("Image hash mismatch. Expected: {expected} Got: {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("Failed to read {0}: {1}")]
    ReadFile(&'static str, String),
    #[error("Failed to write {0}: {1}")]
    WriteFile(&'static str, String),
    /// Not upstream (it asserts): the device's max-download-size cannot
    /// hold one sparse block.
    #[error("Fastboot error: max-download-size {0} is too small")]
    SegmentTooSmall(u32),
}

/// `getvar:max-download-size`, hex with `0x` or decimal, else the default.
pub fn max_download_size<T: UsbTransport + ?Sized>(t: &mut T) -> u32 {
    let Some(value) = get_var(t, "max-download-size") else {
        return DEFAULT_MAX_DOWNLOAD_SIZE;
    };
    let parsed = if value.starts_with("0x") || value.starts_with("0X") {
        parse_leading(&value, 16)
    } else {
        parse_leading(&value, 10)
    };
    parsed.map_or(DEFAULT_MAX_DOWNLOAD_SIZE, |v| v as u32)
}

/// `rpi-imager-<ms since epoch>`.
fn instance_id() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    format!("rpi-imager-{ms}")
}

/// Flashes the (possibly compressed) image from `source` to an opened
/// fastboot device, then customises and reboots it. Progress goes to
/// `bytes_written`/`write_total`; the state is left at `Finalizing` for
/// the caller to settle, as the core write job does.
///
/// # Errors
/// See [`FlashError`]; a failed final `reboot` is not an error.
pub fn flash_over_fastboot<T: UsbTransport + ?Sized>(
    transport: &mut T,
    source: impl Read + Send,
    options: &FlashOptions,
    plan: Option<&CustomisationPlan>,
    progress: &Progress,
) -> Result<(), FlashError> {
    let max = max_download_size(transport);
    if max < MIN_SEGMENT_SIZE {
        return Err(FlashError::SegmentTooSmall(max));
    }
    progress.set_state(WriteState::Writing);
    progress.set_status("Downloading and flashing OS image...");
    Progress::set(&progress.write_total, options.extract_len);
    let mut encoder = SparseEncoder::new(max, options.extract_len);
    if let Some(map) = options
        .bmap
        .as_deref()
        .and_then(|b| BlockMap::parse(b).ok())
    {
        encoder.set_block_map(map);
    }
    let expected = options.expected_sha256.as_deref().filter(|h| !h.is_empty());
    let flasher = Flasher::new(transport, encoder, options.extract_len, expected.is_some());
    let (transport, actual) = stream_image(flasher, source, progress)?;
    if let (Some(expected), Some(actual)) = (expected, actual)
        && actual != expected
    {
        let expected = expected.to_owned();
        return Err(FlashError::HashMismatch { expected, actual });
    }
    let cancel = || progress.is_cancelled();
    if let Some(plan) = plan {
        progress.set_status("Applying OS customisation...");
        apply_customisation(transport, plan, &instance_id(), &cancel)?;
    }
    progress.set_state(WriteState::Finalizing);
    // Non-fatal, as upstream: the flash succeeded even if reboot fails.
    let _ = send_command(transport, "reboot", 10_000);
    Ok(())
}
