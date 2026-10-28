//! What a udisks2 D-Bus error means for an unmount. Pure, tested on every
//! host.

use crate::DiskResult;

/// Maps a udisks2 error name (`org.freedesktop.UDisks2.Error.*`).
pub fn classify(error_name: &str) -> DiskResult {
    let kind = error_name.rsplit('.').next().unwrap_or_default();
    match kind {
        "NotMounted" => DiskResult::Success,
        "DeviceBusy" => DiskResult::Busy,
        k if k.starts_with("NotAuthorized") || k == "Cancelled" => DiskResult::AccessDenied,
        "UnknownObject" | "UnknownMethod" => DiskResult::InvalidDrive,
        _ => DiskResult::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_udisks_errors_to_disk_results() {
        let name = |k| format!("org.freedesktop.UDisks2.Error.{k}");
        assert_eq!(classify(&name("NotMounted")), DiskResult::Success);
        assert_eq!(classify(&name("DeviceBusy")), DiskResult::Busy);
        assert_eq!(
            classify(&name("NotAuthorizedCanObtain")),
            DiskResult::AccessDenied
        );
        assert_eq!(
            classify(&name("NotAuthorizedDismissed")),
            DiskResult::AccessDenied
        );
        assert_eq!(classify(&name("Cancelled")), DiskResult::AccessDenied);
        assert_eq!(
            classify("org.freedesktop.DBus.Error.UnknownObject"),
            DiskResult::InvalidDrive
        );
        assert_eq!(classify(&name("Failed")), DiskResult::Error);
    }
}
