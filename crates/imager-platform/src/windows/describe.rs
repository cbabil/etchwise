//! What a Windows disk is (upstream `drivelist_windows.cpp`): the
//! `STORAGE_DEVICE_DESCRIPTOR` an `IOCTL_STORAGE_QUERY_PROPERTY` returns,
//! and the rules that turn it into a [`DeviceDescriptor`]. Pure, tested on
//! every host.

use imager_core::drives::{DeviceDescriptor, sanitize_for_display};

use super::physical::{device_path, mount_point};

/// `STORAGE_BUS_TYPE` values.
mod bus {
    pub const SCSI: u32 = 0x1;
    pub const ATAPI: u32 = 0x2;
    pub const ATA: u32 = 0x3;
    pub const USB: u32 = 0x7;
    pub const RAID: u32 = 0x8;
    pub const SAS: u32 = 0xA;
    pub const SATA: u32 = 0xB;
    pub const SD: u32 = 0xC;
    pub const MMC: u32 = 0xD;
    pub const VIRTUAL: u32 = 0xE;
    pub const FILE_BACKED_VIRTUAL: u32 = 0xF;
    pub const SPACES: u32 = 0x10;
    pub const NVME: u32 = 0x11;
    pub const SCM: u32 = 0x12;
    pub const UFS: u32 = 0x13;
}

/// The fields of a `STORAGE_DEVICE_DESCRIPTOR` that matter here.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StorageDescriptor {
    pub removable_media: bool,
    pub bus_type: u32,
    pub vendor: String,
    pub product: String,
}

/// Reads a `STORAGE_DEVICE_DESCRIPTOR` (fixed header, then NUL-terminated
/// strings at the offsets it gives).
pub fn parse_storage_descriptor(bytes: &[u8]) -> Option<StorageDescriptor> {
    let u32_at = |at: usize| {
        bytes
            .get(at..at + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    let string_at = |field: usize| {
        let offset = u32_at(field)? as usize;
        let tail = bytes.get(offset..).filter(|_| offset != 0)?;
        let end = tail.iter().position(|b| *b == 0).unwrap_or(tail.len());
        Some(String::from_utf8_lossy(&tail[..end]).trim().to_owned())
    };
    Some(StorageDescriptor {
        removable_media: *bytes.get(10)? != 0,
        bus_type: u32_at(28)?,
        vendor: string_at(12).unwrap_or_default(),
        product: string_at(16).unwrap_or_default(),
    })
}

/// Disk size and sector size from `IOCTL_DISK_GET_DRIVE_GEOMETRY_EX`
/// output: `DISK_GEOMETRY` (24 bytes, `BytesPerSector` at 20), then the
/// 64-bit `DiskSize`.
pub fn parse_geometry(bytes: &[u8]) -> Option<(u64, u32)> {
    let sector = u32::from_le_bytes(bytes.get(20..24)?.try_into().ok()?);
    let size = i64::from_le_bytes(bytes.get(24..32)?.try_into().ok()?);
    Some((u64::try_from(size).unwrap_or(0), sector))
}

/// Everything known about one physical disk.
#[derive(Debug, Default, Clone)]
pub struct DiskFacts {
    pub number: u32,
    pub storage: StorageDescriptor,
    pub size: u64,
    pub sector_size: u32,
    pub writable: bool,
    /// Drive letters of the volumes on this disk.
    pub letters: Vec<char>,
    /// Holds the Windows directory.
    pub holds_windows: bool,
}

/// Upstream's bus type names.
pub fn bus_name(bus_type: u32) -> &'static str {
    match bus_type {
        bus::SCSI => "SCSI",
        bus::ATAPI => "ATAPI",
        bus::ATA => "ATA",
        bus::USB => "USB",
        bus::RAID => "RAID",
        bus::SAS => "SAS",
        bus::SATA => "SATA",
        bus::SD => "SD",
        bus::MMC => "MMC",
        bus::VIRTUAL => "VIRTUAL",
        bus::FILE_BACKED_VIRTUAL => "FILEBACKEDVIRTUAL",
        bus::SPACES => "SPACES",
        bus::NVME => "NVME",
        bus::SCM => "SCM",
        bus::UFS => "UFS",
        _ => "UNKNOWN",
    }
}

/// Cards and USB media are never system drives; fixed disks are, and so
/// is whatever holds Windows.
pub fn describe(facts: &DiskFacts) -> DeviceDescriptor {
    let b = facts.storage.bus_type;
    let is_card = matches!(b, bus::SD | bus::MMC | bus::UFS);
    let is_usb = b == bus::USB;
    let is_virtual = matches!(b, bus::VIRTUAL | bus::FILE_BACKED_VIRTUAL | bus::SPACES);
    let is_removable = facts.storage.removable_media || is_card || is_usb;
    let fixed = !is_removable && !is_virtual;
    let name = [&facts.storage.vendor, &facts.storage.product]
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let path = device_path(facts.number);
    DeviceDescriptor {
        device: path.clone(),
        raw: path,
        description: sanitize_for_display(if name.is_empty() {
            "Unknown disk"
        } else {
            &name
        }),
        size: facts.size,
        block_size: if facts.sector_size == 0 {
            512
        } else {
            facts.sector_size
        },
        bus_type: bus_name(b).into(),
        is_usb,
        is_card,
        is_virtual,
        is_scsi: matches!(
            b,
            bus::SCSI | bus::ATAPI | bus::ATA | bus::RAID | bus::SAS | bus::SATA | bus::NVME
        ),
        is_read_only: !facts.writable,
        is_removable,
        is_system: facts.holds_windows || b == bus::SCM || (fixed && !is_card && !is_usb),
        mountpoints: facts.letters.iter().map(|l| mount_point(*l)).collect(),
        mountpoint_labels: vec![String::new(); facts.letters.len()],
        ..DeviceDescriptor::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(removable: bool, bus_type: u32, vendor: &str, product: &str) -> Vec<u8> {
        let mut bytes = vec![0u8; 40];
        bytes[10] = u8::from(removable);
        bytes[28..32].copy_from_slice(&bus_type.to_le_bytes());
        for (field, text) in [(12, vendor), (16, product)] {
            if text.is_empty() {
                continue;
            }
            let offset = bytes.len() as u32;
            bytes[field..field + 4].copy_from_slice(&offset.to_le_bytes());
            bytes.extend_from_slice(text.as_bytes());
            bytes.push(0);
        }
        bytes
    }

    fn facts(bus_type: u32, removable: bool) -> DiskFacts {
        DiskFacts {
            number: 2,
            storage: StorageDescriptor {
                removable_media: removable,
                bus_type,
                vendor: "Generic".into(),
                product: "STORAGE DEVICE".into(),
            },
            size: 31_914_983_424,
            sector_size: 512,
            writable: true,
            letters: vec!['E', 'F'],
            holds_windows: false,
        }
    }

    #[test]
    fn reads_a_storage_device_descriptor() {
        let parsed = parse_storage_descriptor(&descriptor(true, 7, "SanDisk ", "Ultra  ")).unwrap();
        assert_eq!(
            parsed,
            StorageDescriptor {
                removable_media: true,
                bus_type: 7,
                vendor: "SanDisk".into(),
                product: "Ultra".into()
            }
        );
        let bare = parse_storage_descriptor(&descriptor(false, 0x11, "", "")).unwrap();
        assert_eq!((bare.vendor.as_str(), bare.product.as_str()), ("", ""));
        assert_eq!(parse_storage_descriptor(&[0u8; 20]), None);
        let mut bad_offset = descriptor(false, 7, "", "");
        bad_offset[12..16].copy_from_slice(&999u32.to_le_bytes());
        assert_eq!(parse_storage_descriptor(&bad_offset).unwrap().vendor, "");
    }

    #[test]
    fn reads_disk_geometry() {
        let mut bytes = vec![0u8; 40];
        bytes[20..24].copy_from_slice(&4096u32.to_le_bytes());
        bytes[24..32].copy_from_slice(&64_000_000_000i64.to_le_bytes());
        assert_eq!(parse_geometry(&bytes), Some((64_000_000_000, 4096)));
        bytes[24..32].copy_from_slice(&(-1i64).to_le_bytes());
        assert_eq!(parse_geometry(&bytes), Some((0, 4096)));
        assert_eq!(parse_geometry(&bytes[..30]), None);
    }

    #[test]
    fn a_usb_stick_is_removable_with_its_letters() {
        let d = describe(&facts(7, true));
        assert_eq!(d.device, "\\\\.\\PhysicalDrive2");
        assert_eq!(d.description, "Generic STORAGE DEVICE");
        assert!(d.is_usb && d.is_removable && !d.is_system && !d.is_scsi && !d.is_read_only);
        assert_eq!(d.mountpoints, ["E:\\", "F:\\"]);
        assert_eq!(d.bus_type, "USB");
    }

    #[test]
    fn a_card_reader_card_is_never_a_system_drive() {
        for bus_type in [0xC, 0xD, 0x13] {
            let d = describe(&facts(bus_type, false));
            assert!(d.is_card && d.is_removable && !d.is_system, "{bus_type}");
        }
    }

    #[test]
    fn fixed_disks_and_the_windows_disk_are_system_drives() {
        let nvme = describe(&facts(0x11, false));
        assert!(nvme.is_system && nvme.is_scsi && !nvme.is_removable);
        let windows_on_usb = describe(&DiskFacts {
            holds_windows: true,
            ..facts(7, true)
        });
        assert!(windows_on_usb.is_system);
        assert!(describe(&facts(0x12, true)).is_system);
    }

    #[test]
    fn virtual_disks_are_virtual_and_read_only_is_kept() {
        let vhd = describe(&facts(0xF, false));
        assert!(vhd.is_virtual && !vhd.is_system);
        let locked = describe(&DiskFacts {
            writable: false,
            sector_size: 0,
            ..facts(7, true)
        });
        assert!(locked.is_read_only);
        assert_eq!(locked.block_size, 512);
        assert_eq!(bus_name(0x99), "UNKNOWN");
    }

    #[test]
    fn a_nameless_disk_still_gets_a_name() {
        let mut nameless = facts(7, true);
        nameless.storage.vendor.clear();
        nameless.storage.product.clear();
        assert_eq!(describe(&nameless).description, "Unknown disk");
    }
}
