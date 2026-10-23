//! Writing to a board in USB boot mode (upstream `RpibootThread` then
//! `FastbootFlashThread`): sideload the fastboot gadget, open it, then
//! stream the image as sparse chunks and apply customisation over fastboot.

use std::path::Path;
use std::sync::Arc;

use imager_core::pipeline::job::Job;
use imager_core::pipeline::{Progress, WriteState, open_source};
use imager_rpiboot::fastboot::{FlashError, FlashOptions, flash_over_fastboot};
use imager_rpiboot::rpiboot::{LibusbContext, parse_device_id, sideload_fastboot};

use super::device_ops::Outcome;

/// Flashes `job` onto the board `device_id` (`rpiboot://…`).
pub(crate) fn run_usb_boot(
    device_id: &str,
    job: &Job,
    bmap_url: &str,
    extract_len: u64,
    cache_dir: &Path,
    progress: &Arc<Progress>,
) -> Outcome {
    progress.set_state(WriteState::Preparing);
    progress.set_status("Preparing USB boot...");
    let board = parse_device_id(device_id);
    let mut status = |_: u64, _: u64, text: &str| progress.set_status(text);
    let (bus, address) = sideload_fastboot(
        &board,
        &cache_dir.join("rpiboot"),
        &mut status,
        progress.cancel_flag(),
    )
    .map_err(|e| e.to_string())?;
    let mut usb = LibusbContext::new()?
        .open_at(bus, address)
        .ok_or_else(|| FlashError::Open(format!("{bus}:{address}")).to_string())?;
    let options = FlashOptions {
        expected_sha256: job.write.expected_sha256.clone(),
        extract_len,
        bmap: fetch_bmap(bmap_url),
    };
    let source = open_source(job, progress, None).map_err(|e| e.to_string())?;
    flash_over_fastboot(&mut usb, source, &options, job.plan.as_ref(), progress)
        .map_err(|e| e.to_string())?;
    progress.set_state(WriteState::Succeeded);
    Ok(None)
}

/// The OS's `.bmap`; upstream flashes without one when it cannot be fetched.
fn fetch_bmap(url: &str) -> Option<Vec<u8>> {
    if url.is_empty() {
        return None;
    }
    let client = imager_core::pipeline::http::client().ok()?;
    imager_core::pipeline::http::fetch(&client, url)
        .ok()
        .map(|(bytes, _)| bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_bmap_without_url() {
        assert_eq!(fetch_bmap(""), None);
    }
}
