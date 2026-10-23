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

/// Opens System Settings at Privacy > Removable Volumes, as upstream does
/// when the disk cannot be opened.
pub fn open_removable_volume_privacy() {
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_RemovableVolume")
        .spawn();
}
