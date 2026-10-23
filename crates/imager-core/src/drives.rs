//! Storage devices as the platform reports them (upstream
//! `Drivelist::DeviceDescriptor`), the display sanitiser, and the list
//! the Storage step shows (`DriveListModel::processDriveList`).

use serde::Serialize;

/// One whole disk.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDescriptor {
    /// `/dev/diskN`.
    pub device: String,
    /// `/dev/rdiskN`.
    pub raw: String,
    pub description: String,
    pub size: u64,
    pub block_size: u32,
    pub bus_type: String,
    pub is_usb: bool,
    pub is_card: bool,
    pub is_virtual: bool,
    pub is_scsi: bool,
    pub is_read_only: bool,
    pub is_removable: bool,
    pub is_system: bool,
    pub mountpoints: Vec<String>,
    pub mountpoint_labels: Vec<String>,
    pub child_devices: Vec<String>,
    /// APFS container's physical disk (macOS).
    pub parent_device: String,
}

impl DeviceDescriptor {
    /// `uniqueKey()`: device + size, plus `ro` when read-only.
    pub fn unique_key(&self) -> String {
        format!(
            "{}:{}{}",
            self.device,
            self.size,
            if self.is_read_only { "ro" } else { "" }
        )
    }
}

/// A row of the Storage step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveInfo {
    pub key: String,
    pub device: String,
    pub description: String,
    pub size: u64,
    pub is_usb: bool,
    /// SCSI or NVMe: shows the internal-storage icon.
    pub is_scsi: bool,
    pub is_read_only: bool,
    /// System drives (and virtual disks) need typed confirmation.
    pub is_system: bool,
    pub mountpoints: Vec<String>,
    pub child_devices: Vec<String>,
    /// A board in USB boot mode (`rpiboot://…` id), written over fastboot.
    pub is_rpiboot: bool,
}

/// Drops the boot disk, empty media and unusable virtual disks; virtual
/// disks that remain are treated as system drives.
pub fn visible_drives(devices: &[DeviceDescriptor]) -> Vec<DriveInfo> {
    devices
        .iter()
        .filter(|d| !d.mountpoints.iter().any(|m| m == "/" || m == "C://"))
        .filter(|d| d.size != 0)
        .filter(|d| !d.is_virtual || (d.is_removable && !d.is_read_only && !d.is_system))
        .map(|d| DriveInfo {
            key: d.unique_key(),
            device: d.device.clone(),
            description: d.description.clone(),
            size: d.size,
            is_usb: d.is_usb,
            is_scsi: d.is_scsi
                || d.bus_type.eq_ignore_ascii_case("NVME")
                || d.device.starts_with("/dev/nvme"),
            is_read_only: d.is_read_only,
            is_system: d.is_system || d.is_virtual,
            mountpoints: d.mountpoints.clone(),
            child_devices: d.child_devices.clone(),
            is_rpiboot: false,
        })
        .collect()
}

/// Upstream `sanitizeForDisplay`: control characters become spaces and
/// invisible or direction-changing code points are removed.
pub fn sanitize_for_display(input: &str) -> String {
    input
        .chars()
        .filter_map(|c| match c {
            c if (c as u32) < 0x20 || c == '\u{7f}' => Some(' '),
            c if is_hidden(c as u32) => None,
            c => Some(c),
        })
        .collect()
}

fn is_hidden(cp: u32) -> bool {
    matches!(cp,
        0x202A..=0x202E | 0x2066..=0x2069 | 0x200B..=0x200F | 0xFEFF
        | 0x00AD | 0x034F | 0x061C | 0x115F..=0x1160 | 0x17B4..=0x17B5 | 0x180B..=0x180E | 0x2060..=0x2064
        | 0xE0000..=0xE007F | 0xFE00..=0xFE0F | 0xE0100..=0xE01EF)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disk(device: &str) -> DeviceDescriptor {
        DeviceDescriptor {
            device: device.into(),
            size: 32_000_000_000,
            is_removable: true,
            ..DeviceDescriptor::default()
        }
    }

    #[test]
    fn filters_like_the_drive_list_model() {
        let boot = DeviceDescriptor {
            mountpoints: vec!["/".into()],
            ..disk("/dev/disk0")
        };
        let empty = DeviceDescriptor {
            size: 0,
            ..disk("/dev/disk1")
        };
        let virt_ro = DeviceDescriptor {
            is_virtual: true,
            is_read_only: true,
            ..disk("/dev/disk2")
        };
        let virt_fixed = DeviceDescriptor {
            is_virtual: true,
            is_removable: false,
            ..disk("/dev/disk3")
        };
        let virt_ok = DeviceDescriptor {
            is_virtual: true,
            ..disk("/dev/disk4")
        };
        let nvme = DeviceDescriptor {
            bus_type: "NVMe".into(),
            ..disk("/dev/disk5")
        };
        let sd = DeviceDescriptor {
            is_read_only: true,
            is_card: true,
            ..disk("/dev/disk6")
        };
        let shown = visible_drives(&[boot, empty, virt_ro, virt_fixed, virt_ok, nvme, sd]);
        let devices: Vec<_> = shown.iter().map(|d| d.device.as_str()).collect();
        assert_eq!(devices, ["/dev/disk4", "/dev/disk5", "/dev/disk6"]);
        assert!(shown[0].is_system, "virtual disks need confirmation");
        assert!(shown[1].is_scsi);
        assert_eq!(shown[2].key, "/dev/disk6:32000000000ro");
    }

    #[test]
    fn sanitizes_display_names() {
        assert_eq!(sanitize_for_display("SD\tCard\u{7f}"), "SD Card ");
        assert_eq!(
            sanitize_for_display("evil\u{202E}gpj.exe\u{200B}\u{FE0F}"),
            "evilgpj.exe"
        );
        assert_eq!(
            sanitize_for_display("Café 日本 \u{1F600}"),
            "Café 日本 \u{1F600}"
        );
        assert_eq!(sanitize_for_display("\u{E0041}x\u{AD}"), "x");
    }
}
