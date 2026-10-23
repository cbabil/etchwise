//! The device side of a write: unmount, authorise, open, run, eject.

use std::sync::Arc;

use imager_core::pipeline::Progress;
use imager_core::pipeline::cache::CacheRecord;
use imager_core::pipeline::job::Job;

pub(crate) type Outcome = Result<Option<CacheRecord>, String>;

#[cfg(target_os = "macos")]
pub(crate) fn run(device: &str, job: &Job, progress: &Arc<Progress>) -> Outcome {
    let _awake = imager_platform::macos::power::SleepInhibitor::new();
    let mut target = open_target(device, progress)?;
    imager_core::pipeline::job::run(&mut target, job, progress).map_err(|e| e.to_string())
}

/// Upstream opens `/dev/…` targets after unmounting them, through
/// authopen on the raw node; any other path is a plain file.
#[cfg(target_os = "macos")]
fn open_target(
    device: &str,
    progress: &Progress,
) -> Result<imager_platform::macos::rawdevice::RawDevice, String> {
    use imager_platform::DiskResult;
    use imager_platform::macos::{authopen, diskops, rawdevice::RawDevice};
    if !device.starts_with("/dev/") {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(device)
            .map_err(|e| format!("Cannot open storage device '{device}' ({e})."))?;
        return Ok(RawDevice::new(file));
    }
    progress.set_status("Unmounting drive...");
    if diskops::unmount_disk(device) != DiskResult::Success {
        return Err(format!(
            "Failed to unmount disk '{device}'. Please close any applications using the disk and try again."
        ));
    }
    progress.set_status("Opening drive...");
    let raw = device.replacen("/dev/disk", "/dev/rdisk", 1);
    let file = authopen::auth_open(&raw, authopen::PROMPT).map_err(|e| {
        imager_platform::macos::open_removable_volume_privacy();
        format!("Error opening disk device '{device}'<br>Please verify if 'Raspberry Pi Imager' is allowed access to 'removable volumes' in privacy settings ({e}).")
    })?;
    Ok(RawDevice::new(file))
}

/// `internal://format` (upstream `DriveFormatThread`): unmount, open,
/// write MBR + FAT32.
#[cfg(target_os = "macos")]
pub(super) fn run_erase(device: &str, progress: &Arc<Progress>) -> Outcome {
    let _awake = imager_platform::macos::power::SleepInhibitor::new();
    let mut target = open_target(device, progress)?;
    let result =
        imager_core::erase::format_device(&mut target, progress).map_err(|e| e.to_string());
    progress.set_state(if result.is_ok() {
        imager_core::pipeline::WriteState::Succeeded
    } else {
        imager_core::pipeline::WriteState::Failed
    });
    result.map(|()| None)
}

#[cfg(not(target_os = "macos"))]
pub(super) fn run_erase(_device: &str, _progress: &Arc<Progress>) -> Outcome {
    Err("Erasing is only implemented on macOS".into())
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn run(_device: &str, _job: &Job, _progress: &Arc<Progress>) -> Outcome {
    Err("Writing is only implemented on macOS".into())
}

#[cfg(target_os = "macos")]
pub(crate) fn after_success(device: &str, eject: bool, beep: bool) {
    std::thread::sleep(std::time::Duration::from_secs(1));
    if eject {
        let _ = imager_platform::macos::diskops::eject_disk(device);
    }
    if beep {
        imager_platform::macos::power::beep();
    }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn after_success(_device: &str, _eject: bool, _beep: bool) {}
