//! Unmounting and ejecting whole disks through DiskArbitration (upstream
//! `PlatformQuirks::unmountDisk` / `ejectDisk`): forced whole-disk unmount,
//! run loop pumped in 0.05 s steps for up to 60 s, a busy unmount retried.

use std::ffi::c_void;

use core_foundation::base::TCFType;
use core_foundation::runloop::{CFRunLoop, kCFRunLoopDefaultMode};
use core_foundation_sys::runloop::{
    CFRunLoopRunInMode, kCFRunLoopRunFinished, kCFRunLoopRunStopped,
};

use super::ffi::*;
use super::session::Session;
use crate::DiskResult;

const DA_SUCCESS: DAReturn = 0;
const DA_BUSY: DAReturn = 0xF8DA_0002_u32 as i32;
const DA_BAD_ARGUMENT: DAReturn = 0xF8DA_0003_u32 as i32;
const DA_EXCLUSIVE_ACCESS: DAReturn = 0xF8DA_0004_u32 as i32;
const DA_NOT_FOUND: DAReturn = 0xF8DA_0006_u32 as i32;
const DA_NOT_PERMITTED: DAReturn = 0xF8DA_0008_u32 as i32;
const DA_NOT_PRIVILEGED: DAReturn = 0xF8DA_0009_u32 as i32;

/// Steps of 0.05 s: 12 × 100 × 0.05 s = 60 s.
const MAX_STEPS: u32 = 1200;

/// Attempts at an unmount another app is holding up (Spotlight indexing a
/// freshly inserted drive, a Finder window), one second apart.
const ATTEMPTS: usize = 3;

/// Unmounts every volume of the disk behind `device` (`/dev/[r]diskN`),
/// retrying while the drive is busy.
pub fn unmount_disk(device: &str) -> DiskResult {
    retried(
        || run(device, unmount_done),
        || std::thread::sleep(std::time::Duration::from_secs(1)),
    )
}

/// `attempt` until it gives a final answer (success, missing drive, access
/// denied) or ATTEMPTS run out, with `pause` between attempts.
fn retried(mut attempt: impl FnMut() -> DiskResult, mut pause: impl FnMut()) -> DiskResult {
    let mut result = attempt();
    for _ in 1..ATTEMPTS {
        if !matches!(result, DiskResult::Busy | DiskResult::Error) {
            break;
        }
        pause();
        result = attempt();
    }
    result
}

/// Unmounts, then ejects.
pub fn eject_disk(device: &str) -> DiskResult {
    run(device, unmount_then_eject)
}

/// `/dev/rdisk4` → `disk4`.
fn bsd_name(device: &str) -> &str {
    let name = device.strip_prefix("/dev/").unwrap_or(device);
    if name.starts_with("rdisk") {
        &name[1..]
    } else {
        name
    }
}

fn run(device: &str, callback: DADiskCallback) -> DiskResult {
    let Ok(session) = Session::new() else {
        return DiskResult::Error;
    };
    let Some(disk) = session.disk(bsd_name(device)) else {
        return DiskResult::InvalidDrive;
    };
    let mut context: Option<DiskResult> = None;
    let run_loop = CFRunLoop::get_current();
    // SAFETY: session, disk and context outlive the run loop pumping below;
    // the callback only runs on this thread inside CFRunLoopRunInMode.
    unsafe {
        // Scheduled before the request, as DiskArbitration requires.
        DASessionScheduleWithRunLoop(
            session.raw(),
            run_loop.as_concrete_TypeRef(),
            kCFRunLoopDefaultMode,
        );
        DADiskUnmount(
            disk.as_raw(),
            kDADiskUnmountOptionWhole | kDADiskUnmountOptionForce,
            Some(callback),
            (&raw mut context).cast(),
        );
        for _ in 0..MAX_STEPS {
            if context.is_some() {
                break;
            }
            let status = CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.05, 0);
            if status == kCFRunLoopRunStopped || status == kCFRunLoopRunFinished {
                break;
            }
        }
        DASessionUnscheduleFromRunLoop(
            session.raw(),
            run_loop.as_concrete_TypeRef(),
            kCFRunLoopDefaultMode,
        );
    }
    context.unwrap_or(DiskResult::Busy)
}

/// `translateDissenter`.
fn translate(dissenter: DADissenterRef) -> DiskResult {
    if dissenter.is_null() {
        return DiskResult::Success;
    }
    // SAFETY: non-null dissenter from DiskArbitration.
    match unsafe { DADissenterGetStatus(dissenter) } {
        DA_SUCCESS => DiskResult::Success,
        DA_BAD_ARGUMENT | DA_NOT_FOUND => DiskResult::InvalidDrive,
        DA_NOT_PERMITTED | DA_NOT_PRIVILEGED => DiskResult::AccessDenied,
        DA_BUSY | DA_EXCLUSIVE_ACCESS => DiskResult::Busy,
        _ => DiskResult::Error,
    }
}

fn finish(context: *mut c_void, result: DiskResult) {
    // SAFETY: `context` is the `Option<DiskResult>` passed to
    // DADiskUnmount/DADiskEject, alive while the run loop is pumped.
    unsafe { *context.cast::<Option<DiskResult>>() = Some(result) };
    CFRunLoop::get_current().stop();
}

extern "C" fn unmount_done(_disk: DADiskRef, dissenter: DADissenterRef, context: *mut c_void) {
    finish(context, translate(dissenter));
}

extern "C" fn unmount_then_eject(disk: DADiskRef, dissenter: DADissenterRef, context: *mut c_void) {
    if dissenter.is_null() {
        // SAFETY: valid disk handed to the callback; same context.
        unsafe { DADiskEject(disk, kDADiskEjectOptionDefault, Some(unmount_done), context) };
    } else {
        finish(context, translate(dissenter));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_device_paths_to_bsd_names() {
        assert_eq!(bsd_name("/dev/rdisk4"), "disk4");
        assert_eq!(bsd_name("/dev/disk4"), "disk4");
        assert_eq!(bsd_name("disk12"), "disk12");
        assert_eq!(translate(std::ptr::null()), DiskResult::Success);
    }

    #[test]
    fn retries_a_busy_disk_then_gives_its_last_answer() {
        let mut answers = vec![DiskResult::Success, DiskResult::Busy, DiskResult::Busy];
        let mut pauses = 0;
        let result = retried(|| answers.pop().unwrap(), || pauses += 1);
        assert_eq!((result, pauses), (DiskResult::Success, 2));

        let mut calls = 0;
        let result = retried(
            || {
                calls += 1;
                DiskResult::Busy
            },
            || {},
        );
        assert_eq!((result, calls), (DiskResult::Busy, ATTEMPTS));
    }

    #[test]
    fn does_not_retry_a_final_answer() {
        for answer in [DiskResult::InvalidDrive, DiskResult::AccessDenied] {
            let mut calls = 0;
            let result = retried(
                || {
                    calls += 1;
                    answer
                },
                || {},
            );
            assert_eq!((result, calls), (answer, 1));
        }
    }

    #[test]
    fn unknown_disks_are_invalid() {
        assert_eq!(unmount_disk("/dev/disk999"), DiskResult::InvalidDrive);
    }
}
