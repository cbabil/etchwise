//! Whole-disk enumeration from `lsblk` JSON (upstream `drivelist_linux.cpp`):
//! CD, RAM and eMMC boot areas skipped, loop devices virtual, USB and MMC
//! media removable, NVMe and anything carrying a system mount point a
//! system drive. Pure, so the rules are tested on every host.

use std::io;

use imager_core::drives::{DeviceDescriptor, sanitize_for_display};
use serde_json::Value;

/// The `lsblk` arguments upstream uses.
pub const LSBLK_ARGS: [&str; 6] = [
    "--bytes",
    "--json",
    "--paths",
    "--tree",
    "--output",
    "kname,type,subsystems,ro,rm,hotplug,size,phy-sec,log-sec,label,vendor,model,mountpoint",
];

/// Children deeper than this (LVM on LUKS on a partition...) are ignored.
const MAX_CHILD_DEPTH: usize = 10;

/// Parses `lsblk` output. `mmc_type` gives the kernel's card type for an
/// MMC disk (`/sys/block/<name>/device/type`: "SD", "MMC" or empty).
///
/// # Errors
/// Fails when the output is not the JSON `lsblk` writes.
pub fn parse(json: &str, mmc_type: impl Fn(&str) -> String) -> io::Result<Vec<DeviceDescriptor>> {
    let doc: Value = serde_json::from_str(json)
        .map_err(|e| io::Error::other(format!("Failed to enumerate drives: {e}")))?;
    Ok(doc["blockdevices"]
        .as_array()
        .map(|devices| {
            devices
                .iter()
                .filter_map(|d| describe(d, &mmc_type))
                .collect()
        })
        .unwrap_or_default())
}

fn describe(bdev: &Value, mmc_type: &impl Fn(&str) -> String) -> Option<DeviceDescriptor> {
    let name = text(bdev, "kname");
    let subsystems = text(bdev, "subsystems");
    if name.is_empty()
        || ["/dev/sr", "/dev/ram", "/dev/zram"]
            .iter()
            .any(|p| name.starts_with(p))
        || (name.contains("boot") && name.contains("mmcblk"))
    {
        return None;
    }
    let is_card = subsystems.contains("mmc");
    let is_usb = subsystems.contains("usb");
    // lsblk 2.39 sometimes leaves `subsystems` empty for loop devices.
    let is_virtual =
        !is_usb && !is_card && (name.starts_with("/dev/loop") || subsystems == "block");
    let is_removable = flag(bdev, "rm") || flag(bdev, "hotplug") || is_card || is_usb;
    let (mut mountpoints, mut labels, mut child_labels) = (Vec::new(), Vec::new(), Vec::new());
    collect_mounts(bdev, &mut mountpoints, &mut labels);
    if let Some(children) = bdev["children"].as_array() {
        walk_children(
            children,
            &mut mountpoints,
            &mut labels,
            &mut child_labels,
            0,
        );
    }
    let mut device = DeviceDescriptor {
        device: name.clone(),
        raw: name.clone(),
        description: sanitize_for_display(&description(bdev, &name, &child_labels, mmc_type)),
        size: size(&bdev["size"]),
        block_size: match bdev["phy-sec"].as_u64() {
            Some(n) if n > 0 => u32::try_from(n).unwrap_or(512),
            _ => 512,
        },
        bus_type: text(bdev, "busType"),
        is_usb,
        is_card,
        is_virtual,
        is_scsi: subsystems.contains("scsi") && !is_usb,
        is_read_only: flag(bdev, "ro"),
        is_removable,
        is_system: !is_removable && !is_virtual,
        mountpoints,
        mountpoint_labels: labels,
        ..DeviceDescriptor::default()
    };
    if has_system_mountpoint(&device.mountpoints) {
        device.is_system = true;
    }
    Some(device)
}

/// Upstream `hasSystemMountpoint`: a running system lives on this disk.
pub fn has_system_mountpoint(mountpoints: &[String]) -> bool {
    mountpoints.iter().any(|m| {
        ["/", "/usr", "/var", "/home", "/boot", "/boot/firmware"].contains(&m.as_str())
            || m.starts_with("/snap/")
    })
}

fn collect_mounts(node: &Value, mountpoints: &mut Vec<String>, labels: &mut Vec<String>) {
    let mountpoint = text(node, "mountpoint");
    if !mountpoint.is_empty() {
        mountpoints.push(mountpoint);
        labels.push(text(node, "label"));
    }
}

fn walk_children(
    children: &[Value],
    mountpoints: &mut Vec<String>,
    labels: &mut Vec<String>,
    child_labels: &mut Vec<String>,
    depth: usize,
) {
    if depth >= MAX_CHILD_DEPTH {
        return;
    }
    for child in children {
        let label = text(child, "label");
        if !label.is_empty() {
            child_labels.push(label);
        }
        collect_mounts(child, mountpoints, labels);
        if let Some(nested) = child["children"].as_array() {
            walk_children(nested, mountpoints, labels, child_labels, depth + 1);
        }
    }
}

/// Label, vendor and model, a name for bare MMC and loop devices, then
/// the partition labels in brackets.
fn description(
    bdev: &Value,
    name: &str,
    child_labels: &[String],
    mmc_type: &impl Fn(&str) -> String,
) -> String {
    let mut parts: Vec<String> = ["label", "vendor", "model"]
        .iter()
        .map(|k| text(bdev, k).trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() && is_whole_mmc(name) {
        // Only claim a card reader when the kernel says SD: eMMC passed
        // off as one invites overwriting the system drive (upstream #1658).
        let kname = name.rsplit('/').next().unwrap_or(name);
        parts.push(
            match mmc_type(kname).as_str() {
                "MMC" => "Internal eMMC storage",
                "SD" => "Internal SD card reader",
                _ => "Internal MMC storage",
            }
            .into(),
        );
    }
    if let Some(short) = name
        .strip_prefix("/dev/")
        .filter(|n| parts.is_empty() && n.starts_with("loop"))
    {
        parts.push(format!("Loopback device ({short})"));
    }
    if !child_labels.is_empty() {
        parts.push(format!("({})", child_labels.join(", ")));
    }
    parts.join(" ")
}

/// `/dev/mmcblk<digits>`.
fn is_whole_mmc(name: &str) -> bool {
    name.strip_prefix("/dev/mmcblk")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

fn text(node: &Value, key: &str) -> String {
    node[key].as_str().unwrap_or_default().to_owned()
}

/// lsblk writes booleans, or "0"/"1" in older versions.
fn flag(node: &Value, key: &str) -> bool {
    match &node[key] {
        Value::Bool(b) => *b,
        Value::String(s) => s == "1",
        Value::Number(n) => n.as_u64() == Some(1),
        _ => false,
    }
}

/// A number or a numeric string; anything else is an unknown size (0).
fn size(value: &Value) -> u64 {
    match value {
        Value::String(s) => s.parse().unwrap_or(0),
        Value::Number(n) => n.as_u64().unwrap_or(0),
        _ => 0,
    }
}

#[cfg(test)]
#[path = "lsblk_tests.rs"]
mod tests;
