//! OS-specific services for Etchwise: drive listing, raw disk
//! access, unmount/eject, sleep inhibition, Wi-Fi credentials and the
//! system beep. macOS, Linux and Windows are implemented (upstream
//! `src/mac/`, `src/linux/`, `src/windows/`, `src/drivelist/`); any other
//! target gets [`unsupported`], which refuses every disk operation.
//!
//! The app goes through [`os`], the module for the build target. Each one
//! offers the same names: `drivelist::list_drives`, `diskops::eject_disk`,
//! `open_for_write`, `power::{SleepInhibitor, beep}` and
//! `wifi::{current_ssid, saved_password}`.

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos as os;

#[cfg(any(target_os = "linux", test))]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux as os;

#[cfg(any(target_os = "windows", test))]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows as os;

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub use unsupported as os;

#[cfg(any(
    not(any(target_os = "macos", target_os = "linux", target_os = "windows")),
    test
))]
pub mod unsupported;

/// Outcome of an unmount or eject (upstream `PlatformQuirks::DiskResult`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskResult {
    Success,
    InvalidDrive,
    AccessDenied,
    Busy,
    Error,
}

impl DiskResult {
    /// Why an unmount or eject failed, for the error message.
    pub fn reason(self) -> String {
        match self {
            Self::Success => "it succeeded".into(),
            Self::InvalidDrive => "the drive was not found".into(),
            Self::AccessDenied => format!("{} did not allow it", os_label()),
            Self::Busy => "the drive is in use by another app".into(),
            Self::Error => format!("{} reported an error", os_label()),
        }
    }
}

/// Name of the platform this build targets.
pub fn platform_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unsupported"
    }
}

/// Opens an image file (not a device) as the write target.
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
fn open_plain_file(path: &str) -> Result<std::fs::File, String> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|e| format!("Cannot open storage device '{path}' ({e})."))
}

/// The message for an unmount that did not succeed.
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
fn unmount_failed(device: &str, result: DiskResult) -> String {
    format!(
        "Failed to unmount disk '{device}' ({}). Please close any applications using the disk and try again.",
        result.reason()
    )
}

/// The system's name as people say it, for messages.
fn os_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else {
        "The system"
    }
}

/// The names the app calls, with their types, for whichever [`os`] this
/// build targets; a platform missing one fails to compile here.
#[cfg(test)]
#[allow(dead_code)]
fn os_offers_what_the_app_calls(progress: &imager_core::pipeline::Progress) {
    fn device(_: &mut impl imager_core::pipeline::Device) {}
    let _: std::io::Result<Vec<imager_core::drives::DeviceDescriptor>> =
        os::drivelist::list_drives();
    let opened: Result<os::RawDevice, String> = os::open_for_write("/dev/x", progress);
    if let Ok(mut target) = opened {
        device(&mut target);
    }
    let _: DiskResult = os::diskops::eject_disk("/dev/x");
    let _awake = os::power::SleepInhibitor::new();
    os::power::beep();
    let _: Option<String> = os::wifi::current_ssid();
    let _: Option<String> = os::wifi::saved_password("ssid");
}

#[cfg(test)]
mod disk_result_tests {
    use super::DiskResult;

    #[test]
    fn says_why_a_disk_operation_failed() {
        assert_eq!(
            DiskResult::Busy.reason(),
            "the drive is in use by another app"
        );
        assert_eq!(
            DiskResult::AccessDenied.reason(),
            format!("{} did not allow it", super::os_label())
        );
        assert_eq!(DiskResult::InvalidDrive.reason(), "the drive was not found");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn says_why_an_unmount_failed() {
        assert_eq!(
            super::unmount_failed("/dev/disk4", DiskResult::Busy),
            "Failed to unmount disk '/dev/disk4' (the drive is in use by another app). Please close any applications using the disk and try again."
        );
        let missing = super::open_plain_file("/nonexistent/image.img").unwrap_err();
        assert!(missing.starts_with("Cannot open storage device '/nonexistent/image.img' ("));
    }
}
