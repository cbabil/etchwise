//! Whole-disk enumeration through DiskArbitration (upstream
//! `drivelist_darwin.mm`): description fields, SD-card detection by media
//! icon, APFS containers mapped to their physical disk, and mount points.

use std::ffi::CStr;
use std::io;

use imager_core::drives::{DeviceDescriptor, sanitize_for_display};

use super::cf::key;
use super::ffi::*;
use super::iokit::apfs_parent;
use super::session::Session;

/// Every whole disk with its mount points.
///
/// # Errors
/// Fails when no DiskArbitration session can be created.
pub fn list_drives() -> io::Result<Vec<DeviceDescriptor>> {
    let session = Session::new()?;
    let mut devices: Vec<DeviceDescriptor> = whole_disk_names()?
        .iter()
        .filter_map(|name| describe(&session, name))
        .collect();
    add_mount_points(&mut devices, &session);
    Ok(devices)
}

/// `disk<N>` nodes in `/dev`, sorted case-insensitively like upstream.
fn whole_disk_names() -> io::Result<Vec<String>> {
    let mut names: Vec<String> = std::fs::read_dir("/dev")?
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| {
            n.strip_prefix("disk")
                .is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()))
        })
        .collect();
    // The filter leaves ASCII `disk<digits>` only, so a plain sort matches
    // upstream's case-insensitive compare.
    names.sort();
    Ok(names)
}

fn describe(session: &Session, bsd_name: &str) -> Option<DeviceDescriptor> {
    let d = session.disk(bsd_name)?.description()?;
    // SAFETY: the keys are DiskArbitration's extern static CFStrings.
    let k = |raw| unsafe { key(raw) };
    let protocol = d
        .string(&k(unsafe { kDADiskDescriptionDeviceProtocolKey }))
        .unwrap_or_default();
    let internal = d.bool(&k(unsafe { kDADiskDescriptionDeviceInternalKey }));
    let removable = d.bool(&k(unsafe { kDADiskDescriptionMediaRemovableKey }));
    let is_card = d
        .icon_file(&k(unsafe { kDADiskDescriptionMediaIconKey }))
        .as_deref()
        == Some("SD.icns");
    let device = DeviceDescriptor {
        device: format!("/dev/{bsd_name}"),
        raw: format!("/dev/r{bsd_name}"),
        description: sanitize_for_display(
            &d.string(&k(unsafe { kDADiskDescriptionMediaNameKey }))
                .unwrap_or_default(),
        ),
        size: d
            .u64(&k(unsafe { kDADiskDescriptionMediaSizeKey }))
            .unwrap_or(0),
        block_size: d
            .u64(&k(unsafe { kDADiskDescriptionMediaBlockSizeKey }))
            .map_or(512, |b| b as u32),
        is_usb: protocol == "USB",
        is_card,
        is_virtual: protocol == "Virtual Interface",
        is_scsi: ["SATA", "SCSI", "ATA", "IDE", "PCI", "NVMe"].contains(&protocol.as_str()),
        is_read_only: !d.bool(&k(unsafe { kDADiskDescriptionMediaWritableKey })),
        is_removable: removable || d.bool(&k(unsafe { kDADiskDescriptionMediaEjectableKey })),
        is_system: internal && !removable && !is_card,
        bus_type: protocol,
        ..DeviceDescriptor::default()
    };
    if device.description != "AppleAPFSMedia" {
        return Some(device);
    }
    Some(DeviceDescriptor {
        is_virtual: true,
        parent_device: apfs_parent(bsd_name).unwrap_or_default(),
        ..device
    })
}

/// Credits each mounted volume to its whole disk (APFS volumes to the
/// physical disk, recording the container as a child device).
fn add_mount_points(devices: &mut [DeviceDescriptor], session: &Session) {
    // SAFETY: DiskArbitration's extern static key.
    let volume_name = unsafe { key(kDADiskDescriptionVolumeNameKey) };
    for mount_point in mount_points() {
        let Some(volume) = session.disk_for_volume(&mount_point) else {
            continue;
        };
        let Some(bsd) = volume.bsd_name() else {
            continue;
        };
        let mut disk = whole_disk_of(&bsd).to_owned();
        let mut child = None;
        if let Some(parent) = find(devices, &disk)
            .filter(|d| !d.parent_device.is_empty())
            .map(|d| d.parent_device.clone())
        {
            child = Some(disk);
            disk = parent;
        }
        let label = volume
            .description()
            .and_then(|d| d.string(&volume_name))
            .unwrap_or_default();
        if let Some(device) = devices.iter_mut().find(|d| is_disk(d, &disk)) {
            device.mountpoints.push(mount_point);
            device.mountpoint_labels.push(label);
            if let Some(child) = child {
                device.child_devices.push(format!("/dev/{child}"));
            }
        }
    }
}

fn is_disk(device: &DeviceDescriptor, bsd: &str) -> bool {
    device.device.strip_prefix("/dev/") == Some(bsd)
}

fn find<'a>(devices: &'a [DeviceDescriptor], bsd: &str) -> Option<&'a DeviceDescriptor> {
    devices.iter().find(|d| is_disk(d, bsd))
}

/// `disk3s1` → `disk3` (upstream: first `s` from index 5).
fn whole_disk_of(bsd: &str) -> &str {
    bsd.get(5..)
        .and_then(|rest| rest.find('s'))
        .map_or(bsd, |i| &bsd[..5 + i])
}

/// Mount points of every mounted filesystem, copied into an owned buffer
/// (`getfsstat`; `getmntinfo` would share a process-wide one).
fn mount_points() -> Vec<String> {
    // SAFETY: a null buffer asks only for the count.
    let count = unsafe { libc::getfsstat(std::ptr::null_mut(), 0, libc::MNT_NOWAIT) };
    let Ok(capacity) = usize::try_from(count) else {
        return Vec::new();
    };
    // SAFETY: statfs is plain data; zeroed is a valid value.
    let mut entries: Vec<libc::statfs> = vec![unsafe { std::mem::zeroed() }; capacity];
    let bytes = (capacity * std::mem::size_of::<libc::statfs>()) as libc::c_int;
    // SAFETY: the buffer holds `capacity` entries of the given byte size.
    let filled = unsafe { libc::getfsstat(entries.as_mut_ptr(), bytes, libc::MNT_NOWAIT) };
    entries.truncate(usize::try_from(filled).unwrap_or(0));
    // SAFETY: f_mntonname is a NUL-terminated C string.
    entries
        .iter()
        .map(|m| {
            unsafe { CStr::from_ptr(m.f_mntonname.as_ptr()) }
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_partition_names() {
        assert_eq!(whole_disk_of("disk3s1"), "disk3");
        assert_eq!(whole_disk_of("disk12s3"), "disk12");
        assert_eq!(whole_disk_of("disk4"), "disk4");
    }

    #[test]
    fn lists_this_macs_disks() {
        let drives = list_drives().unwrap();
        assert!(!drives.is_empty());
        let boot = drives
            .iter()
            .find(|d| d.mountpoints.iter().any(|m| m == "/"))
            .expect("boot disk with /");
        assert!(boot.is_system && boot.size > 0, "{boot:?}");
        assert!(
            imager_core::drives::visible_drives(&drives)
                .iter()
                .all(|d| d.device != boot.device)
        );
    }
}
