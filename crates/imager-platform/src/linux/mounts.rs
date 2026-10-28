//! Which mounted filesystems live on a disk (upstream `mountIsOnDevice`),
//! and the udisks2 object for a block device. Pure, tested on every host.

/// Whether a `/proc/mounts` source belongs to `device`: the whole disk or
/// one of its partitions (`/dev/sda1`, `/dev/mmcblk0p2`), never a
/// neighbour such as `/dev/sda_backup`, `/dev/mmcblk0boot0` or `/dev/loop11`.
pub fn mount_is_on_device(device: &str, source: &str) -> bool {
    let Some(suffix) = source.strip_prefix(device).filter(|_| !device.is_empty()) else {
        return false;
    };
    if suffix.is_empty() {
        return true;
    }
    let number = if device.ends_with(|c: char| c.is_ascii_digit()) {
        match suffix.strip_prefix('p') {
            Some(rest) => rest,
            None => return false,
        }
    } else {
        suffix
    };
    !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit())
}

/// `(source, mount point)` of every `/proc/mounts` entry on `device`.
pub fn mounts_on_device(proc_mounts: &str, device: &str) -> Vec<(String, String)> {
    proc_mounts
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            Some((unescape(fields.next()?), unescape(fields.next()?)))
        })
        .filter(|(source, _)| mount_is_on_device(device, source))
        .collect()
}

/// `/proc/mounts` writes space, tab, newline and backslash as `\ooo`.
fn unescape(field: &str) -> String {
    let bytes = field.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let octal = bytes
            .get(i + 1..i + 4)
            .filter(|d| bytes[i] == b'\\' && d.iter().all(|b| (b'0'..=b'7').contains(b)));
        match octal {
            Some(d) => {
                out.push(d.iter().fold(0u8, |n, b| n.wrapping_mul(8) + (b - b'0')));
                i += 4;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// udisks2's object path for a block device: `/dev/sda1` →
/// `/org/freedesktop/UDisks2/block_devices/sda1`, with every character
/// other than ASCII letters and digits written as `_xx` (hex).
pub fn udisks_block_path(device: &str) -> String {
    let name = device.rsplit('/').next().unwrap_or(device);
    let escaped: String = name
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() {
                char::from(b).to_string()
            } else {
                format!("_{b:02x}")
            }
        })
        .collect();
    format!("/org/freedesktop/UDisks2/block_devices/{escaped}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_disk_and_its_partitions_only() {
        assert!(mount_is_on_device("/dev/sda", "/dev/sda"));
        assert!(mount_is_on_device("/dev/sda", "/dev/sda1"));
        assert!(mount_is_on_device("/dev/sda", "/dev/sda12"));
        assert!(mount_is_on_device("/dev/mmcblk0", "/dev/mmcblk0p2"));
        assert!(mount_is_on_device("/dev/nvme0n1", "/dev/nvme0n1p1"));
        for other in ["/dev/sdb1", "/dev/sda_backup", "/dev/sdap1", "tmpfs"] {
            assert!(!mount_is_on_device("/dev/sda", other), "{other}");
        }
        assert!(!mount_is_on_device("/dev/mmcblk0", "/dev/mmcblk0boot0"));
        assert!(!mount_is_on_device("/dev/mmcblk0", "/dev/mmcblk01"));
        assert!(!mount_is_on_device("/dev/loop1", "/dev/loop11"));
        assert!(!mount_is_on_device("/dev/mmcblk0", "/dev/mmcblk0p"));
        assert!(!mount_is_on_device("", "/dev/sda"));
    }

    #[test]
    fn reads_proc_mounts_with_escaped_names() {
        let mounts = "/dev/sda1 /media/me/BOOT\\040FS vfat rw 0 0\n\
                      /dev/sdb1 /media/me/other ext4 rw 0 0\n\
                      /dev/sda2 /media/me/root ext4 rw 0 0\n\
                      proc /proc proc rw 0 0\n\
                      broken\n";
        assert_eq!(
            mounts_on_device(mounts, "/dev/sda"),
            [
                ("/dev/sda1".into(), "/media/me/BOOT FS".into()),
                ("/dev/sda2".into(), "/media/me/root".into())
            ]
        );
        assert_eq!(unescape("a\\134b\\011c\\0"), "a\\b\tc\\0");
    }

    #[test]
    fn names_udisks_objects_like_udisks_does() {
        assert_eq!(
            udisks_block_path("/dev/sda1"),
            "/org/freedesktop/UDisks2/block_devices/sda1"
        );
        assert_eq!(
            udisks_block_path("/dev/dm-0"),
            "/org/freedesktop/UDisks2/block_devices/dm_2d0"
        );
        assert_eq!(
            udisks_block_path("/dev/mmcblk0p1"),
            "/org/freedesktop/UDisks2/block_devices/mmcblk0p1"
        );
    }
}
