//! Opening a disk for writing: unmount, then udisks2 `OpenDevice`, which
//! hands back a read-write descriptor once polkit has asked for the
//! administrator password (the Linux counterpart of macOS authopen). As
//! root the device is opened directly. Opened with `O_EXCL` so nothing
//! mounts it again during the write.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::os::fd::OwnedFd;
use std::os::unix::fs::OpenOptionsExt;

use imager_core::pipeline::Progress;
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::{self, Value};

use super::mounts::udisks_block_path;
use super::{UDISKS, diskops, is_root};
use crate::DiskResult;

/// A block device or image file; `fsync` and the seek-to-end size of
/// `File` are right for Linux block devices.
pub type RawDevice = File;

/// Unmounts and opens `device`; any path outside `/dev/` is a plain file.
///
/// # Errors
/// The message for the user when the disk cannot be unmounted or opened.
pub fn open_for_write(device: &str, progress: &Progress) -> Result<RawDevice, String> {
    if !device.starts_with("/dev/") {
        return crate::open_plain_file(device);
    }
    progress.set_status("Unmounting drive...");
    let unmounted = diskops::unmount_disk(device);
    if unmounted != DiskResult::Success {
        return Err(crate::unmount_failed(device, unmounted));
    }
    progress.set_status("Opening drive...");
    if is_root() {
        return OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_EXCL)
            .open(device)
            .map_err(|e| format!("Error opening disk device '{device}' ({e})."));
    }
    udisks_open(device).map(File::from).map_err(|e| {
        format!(
            "Error opening disk device '{device}'<br>Writing drives needs udisks2 and an administrator password ({e})."
        )
    })
}

fn udisks_open(device: &str) -> zbus::Result<OwnedFd> {
    let bus = Connection::system()?;
    let block = Proxy::new(
        &bus,
        UDISKS,
        udisks_block_path(device),
        "org.freedesktop.UDisks2.Block",
    )?;
    let options = HashMap::from([("flags", Value::from(libc::O_EXCL))]);
    let fd: zvariant::OwnedFd = block.call("OpenDevice", &("rw", options))?;
    Ok(fd.into())
}
