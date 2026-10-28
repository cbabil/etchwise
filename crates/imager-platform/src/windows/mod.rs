//! Windows implementation: physical disks probed with storage IOCTLs,
//! their volumes locked and dismounted for the write, raw
//! `\\.\PhysicalDriveN` access in whole sectors, and the WLAN API for
//! Wi-Fi. Writing needs an administrator, which the app's manifest asks
//! for at launch (as upstream). The parsers are plain code, compiled and
//! tested on every host.

pub mod aligned;
pub mod describe;
pub mod physical;
pub mod wlan_xml;

#[cfg(target_os = "windows")]
pub mod diskops;
#[cfg(target_os = "windows")]
pub mod drivelist;
#[cfg(target_os = "windows")]
mod ioctl;
#[cfg(target_os = "windows")]
mod open;
#[cfg(target_os = "windows")]
pub mod power;
#[cfg(target_os = "windows")]
pub mod wifi;

#[cfg(target_os = "windows")]
pub use open::{RawDevice, open_for_write};
