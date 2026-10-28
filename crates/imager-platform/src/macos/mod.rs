//! macOS implementation.

pub mod authopen;
mod cf;
pub mod diskops;
pub mod drivelist;
mod ffi;
mod iokit;
pub mod location;
pub mod power;
pub mod rawdevice;
mod session;
pub mod wifi;

use imager_core::APP_NAME;
use imager_core::pipeline::Progress;

use crate::DiskResult;
pub use rawdevice::RawDevice;

/// Upstream opens `/dev/…` targets after unmounting them, through
/// authopen on the raw node; any other path is a plain file.
///
/// # Errors
/// The message for the user when the disk cannot be unmounted or opened.
pub fn open_for_write(device: &str, progress: &Progress) -> Result<RawDevice, String> {
    if !device.starts_with("/dev/") {
        return crate::open_plain_file(device).map(RawDevice::new);
    }
    progress.set_status("Unmounting drive...");
    let unmounted = diskops::unmount_disk(device);
    if unmounted != DiskResult::Success {
        return Err(crate::unmount_failed(device, unmounted));
    }
    progress.set_status("Opening drive...");
    let raw = device.replacen("/dev/disk", "/dev/rdisk", 1);
    let file = authopen::auth_open(&raw, &authopen::prompt()).map_err(|e| {
        open_removable_volume_privacy();
        format!("Error opening disk device '{device}'<br>Please verify if '{APP_NAME}' is allowed access to 'removable volumes' in privacy settings ({e}).")
    })?;
    Ok(RawDevice::new(file))
}

/// Opens System Settings at Privacy > Removable Volumes, as upstream does
/// when the disk cannot be opened.
pub fn open_removable_volume_privacy() {
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_RemovableVolume")
        .spawn();
}
