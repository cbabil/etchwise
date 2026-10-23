//! Integration test against a real attached disk image (`hdiutil`), the
//! way the write path sees an SD card: listed, unmounted, ejected.
#![cfg(target_os = "macos")]

use std::process::Command;

use imager_platform::DiskResult;
use imager_platform::macos::diskops::{eject_disk, unmount_disk};
use imager_platform::macos::drivelist::list_drives;

fn hdiutil(args: &[&str]) -> String {
    let out = Command::new("hdiutil")
        .args(args)
        .output()
        .expect("hdiutil runs");
    assert!(
        out.status.success(),
        "hdiutil {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn lists_unmounts_and_ejects_an_attached_image() {
    let dir = tempfile::tempdir().unwrap();
    let image = dir.path().join("card.dmg");
    let image = image.to_str().unwrap();
    hdiutil(&[
        "create",
        "-size",
        "64m",
        "-fs",
        "MS-DOS FAT32",
        "-volname",
        "PITEST",
        "-layout",
        "MBRSPUD",
        image,
    ]);
    let attached = hdiutil(&["attach", image]);
    // The first line of `hdiutil attach` is the whole disk.
    let whole = attached.split_whitespace().next().unwrap().to_owned();

    let drives = list_drives().unwrap();
    let card = drives
        .iter()
        .find(|d| d.device == whole)
        .unwrap_or_else(|| panic!("{whole} in {drives:#?}"));
    assert!(
        card.is_virtual && card.size >= 64 * 1024 * 1024 - 512,
        "{card:?}"
    );
    assert!(
        card.mountpoints.iter().any(|m| m.contains("PITEST")),
        "{card:?}"
    );

    assert_eq!(
        unmount_disk(&whole.replace("/dev/disk", "/dev/rdisk")),
        DiskResult::Success
    );
    let after = list_drives().unwrap();
    assert!(
        after
            .iter()
            .find(|d| d.device == whole)
            .is_some_and(|d| d.mountpoints.is_empty())
    );

    assert_eq!(eject_disk(&whole), DiskResult::Success);
    assert!(list_drives().unwrap().iter().all(|d| d.device != whole));
}
