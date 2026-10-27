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

impl DiskResult {
    /// Why an unmount or eject failed, for the error message.
    pub fn reason(self) -> &'static str {
        match self {
            Self::Success => "it succeeded",
            Self::InvalidDrive => "the drive was not found",
            Self::AccessDenied => "macOS did not allow it",
            Self::Busy => "the drive is in use by another app",
            Self::Error => "macOS reported an error",
        }
    }
}

/// Name of the platform this build targets.
pub fn platform_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else {
        "unsupported"
    }
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
        assert_eq!(DiskResult::AccessDenied.reason(), "macOS did not allow it");
        assert_eq!(DiskResult::InvalidDrive.reason(), "the drive was not found");
    }
}
