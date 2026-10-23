//! Firmware manager against an offline fetcher (upstream has no unit
//! tests for it; these pin its manifest and cache behaviour).

use std::path::Path;
use std::sync::atomic::AtomicBool;

use super::firmware_manager::FirmwareManager;
use super::manifest::{PROVISIONER_RAW_BASE, USBBOOT_RAW_BASE, build_manifest};
use super::test_fetcher::{FakeFetcher, fetcher_for_cm4_cm5};
use super::test_support::{quiet, tar_of, temp_dir, write_file};
use crate::types::{ChipGeneration, SideloadMode};

fn ensure(
    root: &Path,
    fetcher: &FakeFetcher,
    chip: ChipGeneration,
) -> Result<std::path::PathBuf, String> {
    let cancel = AtomicBool::new(false);
    FirmwareManager::new(root, fetcher).ensure_available(
        SideloadMode::Fastboot,
        chip,
        &mut quiet,
        &cancel,
    )
}

#[test]
fn manifests_use_upstream_urls() {
    let urls = |chip| -> Vec<(String, String)> {
        build_manifest(SideloadMode::Fastboot, chip)
            .into_iter()
            .map(|e| (e.url, e.local_path))
            .collect()
    };
    let pair = |u: &str, l: &str| (u.to_owned(), l.to_owned());
    assert_eq!(
        urls(ChipGeneration::Bcm2836_7),
        vec![
            pair(
                "https://github.com/raspberrypi/usbboot/raw/refs/heads/master/msd/bootcode.bin",
                "bootcode.bin"
            ),
            pair(
                "https://github.com/raspberrypi/rpi-sb-provisioner/raw/refs/heads/main/host-support/fastboot-gadget.2710-bootfiles-bin",
                "fastboot/bootfiles.bin"
            ),
        ]
    );
    let cm = vec![
        pair(
            "https://github.com/raspberrypi/rpi-sb-provisioner/raw/refs/heads/main/host-support/fastboot-gadget.img",
            "fastboot/boot.img",
        ),
        pair(
            "https://github.com/raspberrypi/usbboot/raw/refs/heads/master/mass-storage-gadget64/config.txt",
            "fastboot/config.txt",
        ),
        pair(
            "https://github.com/raspberrypi/usbboot/raw/refs/heads/master/firmware/bootfiles.bin",
            "fastboot/bootfiles.bin",
        ),
    ];
    assert_eq!(urls(ChipGeneration::Bcm2711), cm);
    assert_eq!(urls(ChipGeneration::Bcm2712), cm);
}

#[test]
fn downloads_and_extracts_bootcode_for_cm4() {
    let root = temp_dir();
    let fetcher = fetcher_for_cm4_cm5();
    let cancel = AtomicBool::new(false);
    let mut seen = Vec::new();
    let mut record = |c: u64, t: u64, s: &str| seen.push((c, t, s.to_owned()));
    let manager = FirmwareManager::new(root.path(), &fetcher);
    let dir = manager.ensure_available(
        SideloadMode::Fastboot,
        ChipGeneration::Bcm2711,
        &mut record,
        &cancel,
    );
    let dir = dir.expect("firmware");
    assert_eq!(dir, root.path().join("master"));
    assert_eq!(
        std::fs::read(dir.join("bootcode4.bin")).expect("bc"),
        b"bc4"
    );
    assert_eq!(
        std::fs::read(dir.join("fastboot/boot.img")).expect("img"),
        b"img"
    );
    assert_eq!(
        seen.first().map(|s| s.2.as_str()),
        Some("Downloading rpiboot firmware...")
    );
    assert_eq!(seen.last(), Some(&(100, 100, "Firmware ready".to_owned())));
    assert_eq!(
        manager.find_cached_version(SideloadMode::Fastboot, ChipGeneration::Bcm2711),
        Some(dir)
    );
}

#[test]
fn cached_files_are_not_downloaded_again_but_bootcode_is_re_extracted() {
    let root = temp_dir();
    let fetcher = fetcher_for_cm4_cm5();
    assert!(ensure(root.path(), &fetcher, ChipGeneration::Bcm2711).is_ok());
    let dir = ensure(root.path(), &fetcher, ChipGeneration::Bcm2712).expect("cached");
    assert_eq!(fetcher.requested.lock().expect("lock").len(), 3);
    assert_eq!(
        std::fs::read(dir.join("bootcode5.bin")).expect("bc"),
        b"bc5"
    );
}

#[test]
fn cache_hit_for_cm3_needs_no_network() {
    let root = temp_dir();
    write_file(root.path(), "master/bootcode.bin", b"bc");
    write_file(root.path(), "master/fastboot/bootfiles.bin", b"tar");
    let fetcher = FakeFetcher::default();
    assert!(ensure(root.path(), &fetcher, ChipGeneration::Bcm2836_7).is_ok());
    assert!(fetcher.requested.lock().expect("lock").is_empty());
}

#[test]
fn download_failure_reports_and_leaves_no_partial_file() {
    let root = temp_dir();
    write_file(root.path(), "master/bootcode.bin.tmp", b"stale");
    let err = ensure(
        root.path(),
        &FakeFetcher::default(),
        ChipGeneration::Bcm2836_7,
    );
    assert_eq!(err, Err("Download failed: 404 Not Found".to_owned()));
    assert!(!root.path().join("master/bootcode.bin").exists());
    assert!(!root.path().join("master/bootcode.bin.tmp").exists());
}

#[test]
fn missing_bootcode_in_archive_is_reported() {
    let root = temp_dir();
    let mut fetcher = fetcher_for_cm4_cm5();
    let url = format!("{USBBOOT_RAW_BASE}firmware/bootfiles.bin");
    fetcher.bodies.insert(url, tar_of(&[("other.bin", b"x")]));
    let err = ensure(root.path(), &fetcher, ChipGeneration::Bcm2712).expect_err("missing");
    assert_eq!(
        err,
        "bootcode5.bin not found inside fastboot/bootfiles.bin (tried 2712/bootcode5.bin and bootcode5.bin)"
    );
}

#[test]
fn invalid_download_is_removed() {
    let root = temp_dir();
    let mut fetcher = FakeFetcher::default();
    fetcher
        .bodies
        .insert(format!("{USBBOOT_RAW_BASE}msd/bootcode.bin"), Vec::new());
    let url = format!("{PROVISIONER_RAW_BASE}host-support/fastboot-gadget.2710-bootfiles-bin");
    fetcher.bodies.insert(url, b"tar".to_vec());
    let err = ensure(root.path(), &fetcher, ChipGeneration::Bcm2836_7);
    assert_eq!(
        err,
        Err("Downloaded firmware does not contain required files".to_owned())
    );
    assert!(!root.path().join("master").exists());
}

#[test]
fn cancellation_stops_downloads() {
    let root = temp_dir();
    let fetcher = fetcher_for_cm4_cm5();
    let cancel = AtomicBool::new(true);
    let manager = FirmwareManager::new(root.path(), &fetcher);
    let err = manager.ensure_available(
        SideloadMode::Fastboot,
        ChipGeneration::Bcm2711,
        &mut quiet,
        &cancel,
    );
    assert_eq!(err, Err("Cancelled".to_owned()));
    manager.clear_cache();
    assert!(!root.path().exists());
}
