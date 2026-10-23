//! Which firmware files to fetch and how to check the cache (upstream
//! `FirmwareManager::buildManifest`, `validateCacheForDevice`,
//! `extractBootcodeFromBootfiles`).

use std::path::Path;

use super::bootcode_loader::bootcode_filename;
use super::bootfiles::Bootfiles;
use crate::types::{ChipGeneration, SideloadMode};

/// usbboot repository raw-file base URL.
pub const USBBOOT_RAW_BASE: &str = "https://github.com/raspberrypi/usbboot/raw/refs/heads/master/";
/// rpi-sb-provisioner repository raw-file base URL.
pub const PROVISIONER_RAW_BASE: &str =
    "https://github.com/raspberrypi/rpi-sb-provisioner/raw/refs/heads/main/";

/// One file to download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    /// Source URL.
    pub url: String,
    /// Destination relative to the version directory.
    pub local_path: String,
}

fn entry(base: &str, path: &str, local_path: &str) -> ManifestEntry {
    ManifestEntry {
        url: format!("{base}{path}"),
        local_path: local_path.to_owned(),
    }
}

/// The files for `mode` and `chip`. BCM2836/7 takes `bootcode.bin` from
/// usbboot's `msd/` and a self-contained 2710 bootfiles bundle; BCM2711 and
/// BCM2712 take the gadget kernel and config separately, and their bootcode
/// is extracted from `bootfiles.bin` at cache time.
pub fn build_manifest(mode: SideloadMode, chip: ChipGeneration) -> Vec<ManifestEntry> {
    match (mode, chip) {
        (SideloadMode::Fastboot, ChipGeneration::Bcm2836_7) => vec![
            entry(USBBOOT_RAW_BASE, "msd/bootcode.bin", "bootcode.bin"),
            entry(
                PROVISIONER_RAW_BASE,
                "host-support/fastboot-gadget.2710-bootfiles-bin",
                "fastboot/bootfiles.bin",
            ),
        ],
        (SideloadMode::Fastboot, ChipGeneration::Bcm2711 | ChipGeneration::Bcm2712) => vec![
            entry(
                PROVISIONER_RAW_BASE,
                "host-support/fastboot-gadget.img",
                "fastboot/boot.img",
            ),
            entry(
                USBBOOT_RAW_BASE,
                "mass-storage-gadget64/config.txt",
                "fastboot/config.txt",
            ),
            entry(
                USBBOOT_RAW_BASE,
                "firmware/bootfiles.bin",
                "fastboot/bootfiles.bin",
            ),
        ],
    }
}

/// BCM2711/BCM2712 Fastboot bundles the bootcode inside `bootfiles.bin`.
pub fn needs_bootcode_extraction(mode: SideloadMode, chip: ChipGeneration) -> bool {
    mode == SideloadMode::Fastboot && chip != ChipGeneration::Bcm2836_7
}

/// The version directory holds a non-empty bootcode file for the chip and
/// the mode's subdirectory.
pub fn validate_cache_for_device(dir: &Path, mode: SideloadMode, chip: ChipGeneration) -> bool {
    if !dir.exists() {
        return false;
    }
    let bootcode_ok = std::fs::metadata(dir.join(bootcode_filename(chip)))
        .map(|m| m.len() > 0)
        .unwrap_or(false);
    bootcode_ok
        && match mode {
            SideloadMode::Fastboot => dir.join("fastboot").exists(),
        }
}

/// Writes `<dir>/<bootcode file>` from `fastboot/bootfiles.bin`, trying
/// `<chip prefix>/<file>` then the bare file name.
///
/// # Errors
/// Upstream's messages for an unreadable archive, a missing entry, or a
/// failed write.
pub fn extract_bootcode_from_bootfiles(dir: &Path, chip: ChipGeneration) -> Result<(), String> {
    let mut bootfiles = Bootfiles::default();
    bootfiles
        .extract_from_file(&dir.join("fastboot").join("bootfiles.bin"))
        .map_err(|e| format!("Cannot read fastboot/bootfiles.bin: {e}"))?;
    let name = bootcode_filename(chip);
    let prefixed = format!("{}/{name}", chip.directory_prefix());
    let data = [prefixed.as_str(), name]
        .into_iter()
        .find_map(|key| bootfiles.find(key, "").filter(|d| !d.is_empty()))
        .ok_or_else(|| {
            format!("{name} not found inside fastboot/bootfiles.bin (tried {prefixed} and {name})")
        })?;
    let dest = dir.join(name);
    std::fs::write(&dest, data).map_err(|_| format!("Cannot write {name}: {}", dest.display()))
}
