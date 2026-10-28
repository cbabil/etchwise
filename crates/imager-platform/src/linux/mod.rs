//! Linux implementation: drives from `lsblk`, unmount, raw open and eject
//! through udisks2 (polkit asks for the password, the app itself stays
//! unprivileged; as root the syscalls are used directly), logind sleep
//! inhibition and NetworkManager Wi-Fi. The parsers are plain code,
//! compiled and tested on every host.

pub mod lsblk;
pub mod mounts;
pub mod ssid;
pub mod udisks_error;

#[cfg(target_os = "linux")]
pub mod diskops;
#[cfg(target_os = "linux")]
pub mod drivelist;
#[cfg(target_os = "linux")]
pub mod power;
#[cfg(target_os = "linux")]
pub mod wifi;

#[cfg(target_os = "linux")]
pub use open::{RawDevice, open_for_write};

#[cfg(target_os = "linux")]
mod open;

/// Whether this process runs as root (a `sudo` CLI run).
#[cfg(target_os = "linux")]
fn is_root() -> bool {
    // SAFETY: geteuid has no preconditions.
    unsafe { libc::geteuid() == 0 }
}

/// The udisks2 service.
#[cfg(target_os = "linux")]
const UDISKS: &str = "org.freedesktop.UDisks2";
