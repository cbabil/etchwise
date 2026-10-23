//! Unmounting and ejecting whole disks through DiskArbitration (upstream
//! `PlatformQuirks::unmountDisk` / `ejectDisk`): forced whole-disk unmount,
//! run loop pumped in 0.05 s steps for up to 60 s.

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

/// Unmounts every volume of the disk behind `device` (`/dev/[r]diskN`).
pub fn unmount_disk(device: &str) -> DiskResult {
    run(device, unmount_done)
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
        DADiskUnmount(
            disk.as_raw(),
            kDADiskUnmountOptionWhole | kDADiskUnmountOptionForce,
            Some(callback),
            (&raw mut context).cast(),
        );
        DASessionScheduleWithRunLoop(
            session.raw(),
            run_loop.as_concrete_TypeRef(),
            kCFRunLoopDefaultMode,
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
    fn unknown_disks_are_invalid() {
        assert_eq!(unmount_disk("/dev/disk999"), DiskResult::InvalidDrive);
    }
}
