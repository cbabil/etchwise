//! OS-specific services for Etchwise: drive listing, raw disk
//! access, unmount/eject, sleep inhibition, Wi-Fi credentials and the
//! system beep. macOS is implemented (upstream `src/mac/`); other platforms
//! report `Unsupported`.

#[cfg(target_os = "macos")]
pub mod macos;

/// Outcome of an unmount or eject (upstream `PlatformQuirks::DiskResult`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskResult {
    Success,
    InvalidDrive,
    AccessDenied,
    Busy,
    Error,
}

/// Name of the platform this build targets.
pub fn platform_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else {
        "unsupported"
    }
}
