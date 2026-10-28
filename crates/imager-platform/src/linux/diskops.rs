//! Unmounting and ejecting whole disks (upstream `unmountDisk` /
//! `ejectDisk`): every filesystem in `/proc/mounts` on the disk is
//! unmounted through udisks2, or with `umount2` when running as root;
//! eject then powers the drive off through udisks2.

use std::collections::HashMap;
use std::ffi::CString;

use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::{OwnedObjectPath, Value};

use super::mounts::{mounts_on_device, udisks_block_path};
use super::udisks_error::classify;
use super::{UDISKS, is_root};
use crate::DiskResult;

/// Unmounts every filesystem on `device` (`/dev/sdX`, `/dev/mmcblkN`).
pub fn unmount_disk(device: &str) -> DiskResult {
    match std::fs::metadata(device) {
        Ok(m) if !m.is_dir() => {}
        _ => return DiskResult::InvalidDrive,
    }
    let Ok(proc_mounts) = std::fs::read_to_string("/proc/mounts") else {
        return DiskResult::Error;
    };
    let bus = Connection::system().ok();
    mounts_on_device(&proc_mounts, device)
        .iter()
        .map(|(source, dir)| unmount_one(bus.as_ref(), source, dir))
        .find(|r| *r != DiskResult::Success)
        .unwrap_or(DiskResult::Success)
}

/// Unmounts, then powers the drive off (USB sticks and card readers).
pub fn eject_disk(device: &str) -> DiskResult {
    let unmounted = unmount_disk(device);
    if let (DiskResult::Success, Ok(bus)) = (unmounted, Connection::system()) {
        let _ = power_off(&bus, device);
    }
    unmounted
}

fn unmount_one(bus: Option<&Connection>, source: &str, dir: &str) -> DiskResult {
    let via_udisks = bus.map_or(DiskResult::Error, |bus| match udisks_unmount(bus, source) {
        Ok(()) => DiskResult::Success,
        Err(zbus::Error::MethodError(name, _, _)) => classify(name.as_str()),
        Err(_) => DiskResult::Error,
    });
    if via_udisks == DiskResult::Success || !is_root() {
        return via_udisks;
    }
    syscall_unmount(dir)
}

fn no_options() -> HashMap<&'static str, Value<'static>> {
    HashMap::new()
}

fn udisks_unmount(bus: &Connection, source: &str) -> zbus::Result<()> {
    Proxy::new(
        bus,
        UDISKS,
        udisks_block_path(source),
        "org.freedesktop.UDisks2.Filesystem",
    )?
    .call("Unmount", &(no_options(),))
}

fn power_off(bus: &Connection, device: &str) -> zbus::Result<()> {
    let block = Proxy::new(
        bus,
        UDISKS,
        udisks_block_path(device),
        "org.freedesktop.UDisks2.Block",
    )?;
    let drive: OwnedObjectPath = block.get_property("Drive")?;
    if drive.as_str() == "/" {
        return Ok(());
    }
    Proxy::new(bus, UDISKS, drive, "org.freedesktop.UDisks2.Drive")?
        .call("PowerOff", &(no_options(),))
}

/// Upstream's order: plain, then lazy (safe here: the disk is about to
/// be overwritten).
fn syscall_unmount(dir: &str) -> DiskResult {
    let Ok(path) = CString::new(dir) else {
        return DiskResult::InvalidDrive;
    };
    for flags in [0, libc::MNT_DETACH] {
        // SAFETY: valid NUL-terminated path.
        if unsafe { libc::umount2(path.as_ptr(), flags) } == 0 {
            return DiskResult::Success;
        }
    }
    DiskResult::Busy
}
