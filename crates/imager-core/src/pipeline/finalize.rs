//! After the stream: read-back verification (`DownloadThread::_verify`),
//! customisation of the boot partition (`_customizeImage`), then the held
//! first block and a final sync (`_writeComplete`).

use std::io::SeekFrom;

use sha2::{Digest, Sha256};

use super::buffers::verify_buffer_size;
use super::device::Device;
use super::error::WriteError;
use super::progress::{Progress, WriteState};
use super::writer::Written;
use crate::customise::plan::{CustomisationPlan, apply_plan};
use crate::fat::BootPartition;

/// Re-reads everything written (the first block from memory) and compares
/// hashes. Skipping or cancelling counts as success, as upstream.
///
/// # Errors
/// Fails when the device cannot be read or the contents differ.
pub fn verify<D: Device>(
    device: &mut D,
    written: &Written,
    progress: &Progress,
) -> Result<(), WriteError> {
    progress.set_state(WriteState::Verifying);
    progress
        .verify_total
        .store(written.length, std::sync::atomic::Ordering::Relaxed);
    let mut hasher = Sha256::new();
    hasher.update(&written.first_block);
    let mut done = written.first_block.len() as u64;
    progress
        .verify_now
        .store(done, std::sync::atomic::Ordering::Relaxed);
    device
        .seek(SeekFrom::Start(done))
        .map_err(|_| WriteError::VerifyRead)?;
    let mut buffer = vec![0u8; verify_buffer_size(written.length)];
    while done < written.length && !progress.verification_skipped() && !progress.is_cancelled() {
        let want = buffer.len().min((written.length - done) as usize);
        let got = device
            .read(&mut buffer[..want])
            .map_err(|_| WriteError::VerifyRead)?;
        if got == 0 {
            return Err(WriteError::VerifyRead);
        }
        hasher.update(&buffer[..got]);
        done += got as u64;
        Progress::add(&progress.verify_now, got as u64);
    }
    let matches = <[u8; 32]>::from(hasher.finalize()) == written.sha256;
    if matches || progress.verification_skipped() || progress.is_cancelled() {
        Ok(())
    } else {
        Err(WriteError::VerifyMismatch)
    }
}

/// Customises the boot partition when the plan has content for it, else
/// writes the first block; then syncs. Customisation runs, as upstream,
/// only when config, cmdline, firstrun or user-data is non-empty.
///
/// # Errors
/// Fails when customising or the final writes fail.
pub fn finalize<D: Device>(
    device: &mut D,
    written: Written,
    plan: Option<&CustomisationPlan>,
    instance_id: &str,
    progress: &Progress,
) -> Result<(), WriteError> {
    progress.set_state(WriteState::Finalizing);
    let customise = plan.filter(|p| {
        !p.init_format.is_empty()
            && !(p.config.is_empty()
                && p.cmdline.is_empty()
                && p.firstrun.is_empty()
                && p.cloudinit.is_empty())
    });
    match customise {
        Some(plan) => {
            progress.set_status("Customising OS...");
            let mut boot = BootPartition::open_with_prefix(&mut *device, &written.first_block)
                .map_err(WriteError::Customise)?;
            apply_plan(&mut boot, plan, instance_id).map_err(WriteError::Customise)?;
            boot.finish().map_err(WriteError::Customise)?;
        }
        None => {
            device
                .seek(SeekFrom::Start(0))
                .map_err(WriteError::io("writing partition table"))?;
            device
                .write_all(&written.first_block)
                .and_then(|()| device.flush())
                .map_err(WriteError::io("writing partition table"))?;
        }
    }
    device.force_sync().map_err(WriteError::io("final sync"))
}

#[cfg(test)]
mod tests;
