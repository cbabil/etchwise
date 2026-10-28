//! `\\.\PhysicalDriveN` paths (upstream `parseDeviceNumberImpl`). Pure,
//! tested on every host.

const PREFIXES: [&str; 2] = ["\\\\.\\PhysicalDrive", "//./PhysicalDrive"];

/// The disk number in `\\.\PhysicalDriveN` (or `//./PhysicalDriveN`).
pub fn device_number(device: &str) -> Option<u32> {
    let digits = PREFIXES
        .iter()
        .find_map(|p| strip_prefix_ignore_case(device, p))?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// `\\.\PhysicalDriveN`.
pub fn device_path(number: u32) -> String {
    format!("{}{number}", PREFIXES[0])
}

/// `\\.\E:`, the volume behind a drive letter.
pub fn volume_path(letter: char) -> String {
    format!("\\\\.\\{letter}:")
}

/// `E:\`, how a drive letter is shown as a mount point.
pub fn mount_point(letter: char) -> String {
    format!("{letter}:\\")
}

fn strip_prefix_ignore_case<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let head = s.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix)
        .then(|| &s[prefix.len()..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_physical_drive_paths() {
        assert_eq!(device_number("\\\\.\\PhysicalDrive3"), Some(3));
        assert_eq!(device_number("\\\\.\\PHYSICALDRIVE12"), Some(12));
        assert_eq!(device_number("//./PhysicalDrive0"), Some(0));
        for bad in [
            "\\\\.\\PhysicalDrive",
            "\\\\.\\PhysicalDrive1a",
            "/dev/sda",
            "C:",
            "",
        ] {
            assert_eq!(device_number(bad), None, "{bad}");
        }
        assert_eq!(device_path(4), "\\\\.\\PhysicalDrive4");
        assert_eq!(volume_path('E'), "\\\\.\\E:");
        assert_eq!(mount_point('E'), "E:\\");
    }
}
