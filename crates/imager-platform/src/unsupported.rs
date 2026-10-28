//! Targets without a platform layer: no drives are listed and every disk
//! operation is refused, so the rest of the app builds and runs unchanged.

use std::io;

use imager_core::pipeline::Progress;

/// What every refused operation says.
pub const UNSUPPORTED: &str = "Writing drives is not supported on this system";

pub mod drivelist {
    use imager_core::drives::DeviceDescriptor;

    /// # Errors
    /// Always: there is no drive listing on this target.
    pub fn list_drives() -> std::io::Result<Vec<DeviceDescriptor>> {
        Err(super::refused())
    }
}

pub mod diskops {
    use crate::DiskResult;

    /// Nothing to eject without a platform layer.
    pub fn eject_disk(_device: &str) -> DiskResult {
        DiskResult::Error
    }
}

pub mod power {
    /// Holds nothing.
    pub struct SleepInhibitor;

    impl SleepInhibitor {
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self
        }
    }

    /// Silent.
    pub fn beep() {}
}

pub mod wifi {
    pub fn current_ssid() -> Option<String> {
        None
    }

    pub fn saved_password(_ssid: &str) -> Option<String> {
        None
    }
}

/// An opened target; never constructed here.
pub type RawDevice = std::fs::File;

/// # Errors
/// Always.
pub fn open_for_write(_device: &str, _progress: &Progress) -> Result<RawDevice, String> {
    Err(UNSUPPORTED.into())
}

fn refused() -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, UNSUPPORTED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_every_disk_operation() {
        assert_eq!(
            drivelist::list_drives().unwrap_err().to_string(),
            UNSUPPORTED
        );
        assert_eq!(
            open_for_write("/dev/sda", &Progress::default()).unwrap_err(),
            UNSUPPORTED
        );
        assert_eq!(diskops::eject_disk("/dev/sda"), crate::DiskResult::Error);
        assert_eq!(wifi::current_ssid(), None);
        let _awake = power::SleepInhibitor::new();
        power::beep();
    }
}
