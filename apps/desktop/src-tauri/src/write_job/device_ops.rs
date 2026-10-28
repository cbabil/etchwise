//! The device side of a write: unmount, authorise, open, run, eject.

use std::sync::Arc;

use imager_core::pipeline::Progress;
use imager_core::pipeline::cache::CacheRecord;
use imager_core::pipeline::job::Job;
use imager_platform::os;

pub(crate) type Outcome = Result<Option<CacheRecord>, String>;

pub(crate) fn run(device: &str, job: &Job, progress: &Arc<Progress>) -> Outcome {
    let _awake = os::power::SleepInhibitor::new();
    let mut target = os::open_for_write(device, progress)?;
    imager_core::pipeline::job::run(&mut target, job, progress).map_err(|e| e.to_string())
}

/// `internal://format` (upstream `DriveFormatThread`): unmount, open,
/// write MBR + FAT32.
pub(super) fn run_erase(device: &str, progress: &Arc<Progress>) -> Outcome {
    let _awake = os::power::SleepInhibitor::new();
    let mut target = os::open_for_write(device, progress)?;
    let result =
        imager_core::erase::format_device(&mut target, progress).map_err(|e| e.to_string());
    progress.set_state(if result.is_ok() {
        imager_core::pipeline::WriteState::Succeeded
    } else {
        imager_core::pipeline::WriteState::Failed
    });
    result.map(|()| None)
}

pub(crate) fn after_success(device: &str, eject: bool, beep: bool) {
    std::thread::sleep(std::time::Duration::from_secs(1));
    if eject {
        let _ = os::diskops::eject_disk(device);
    }
    if beep {
        os::power::beep();
    }
}
