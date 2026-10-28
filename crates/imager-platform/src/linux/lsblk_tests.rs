//! Upstream `drivelist_test.cpp` "Linux lsblk parsing" cases.

use serde_json::{Value, json};

use super::*;

fn disk(fields: Value) -> Value {
    let mut base = json!({
        "kname": "/dev/sda", "type": "disk", "subsystems": "block:scsi:usb:pci",
        "ro": false, "rm": true, "hotplug": true, "size": "32010928128",
        "phy-sec": 512, "log-sec": 512, "label": "", "vendor": "", "model": "",
        "mountpoint": null
    });
    if let (Some(base), Some(fields)) = (base.as_object_mut(), fields.as_object()) {
        base.extend(fields.clone());
    }
    base
}

fn parse_one(fields: Value) -> Vec<DeviceDescriptor> {
    parse_with(fields, "")
}

fn parse_with(fields: Value, mmc: &'static str) -> Vec<DeviceDescriptor> {
    let doc = json!({ "blockdevices": [disk(fields)] }).to_string();
    parse(&doc, |_| mmc.to_owned()).unwrap()
}

#[test]
fn parses_a_usb_drive() {
    let d = &parse_one(json!({"vendor": "SanDisk ", "model": "Cruzer Blade    "}))[0];
    assert_eq!(d.device, "/dev/sda");
    assert_eq!(d.raw, "/dev/sda");
    assert_eq!(d.size, 32_010_928_128);
    assert!(d.is_usb && d.is_removable && !d.is_system && !d.is_virtual && !d.is_scsi);
    assert_eq!(d.block_size, 512);
    assert_eq!(d.description, "SanDisk Cruzer Blade");
}

#[test]
fn an_sd_card_in_an_internal_reader_is_removable() {
    let d = &parse_with(
        json!({"kname": "/dev/mmcblk0", "subsystems": "block:mmc:mmc_host:pci",
               "rm": false, "hotplug": false}),
        "SD",
    )[0];
    assert!(d.is_card && d.is_removable && !d.is_system && !d.is_virtual);
    assert_eq!(d.description, "Internal SD card reader");
}

#[test]
fn names_emmc_and_unknown_mmc_without_claiming_sd() {
    let fields = json!({"kname": "/dev/mmcblk1", "subsystems": "block:mmc:mmc_host"});
    assert_eq!(
        parse_with(fields.clone(), "MMC")[0].description,
        "Internal eMMC storage"
    );
    assert_eq!(
        parse_with(fields, "")[0].description,
        "Internal MMC storage"
    );
}

#[test]
fn nvme_carrying_root_is_a_system_drive() {
    let d = &parse_one(
        json!({"kname": "/dev/nvme0n1", "subsystems": "block:nvme:pci",
        "rm": false, "hotplug": false, "model": "Samsung SSD 970 EVO Plus", "mountpoint": "/"}),
    )[0];
    assert!(d.is_system);
    assert_eq!(d.mountpoints, ["/"]);
}

#[test]
fn skips_optical_ram_zram_and_emmc_boot_areas() {
    for kname in [
        "/dev/sr0",
        "/dev/ram0",
        "/dev/zram0",
        "/dev/mmcblk0boot0",
        "",
    ] {
        assert!(parse_one(json!({ "kname": kname })).is_empty(), "{kname}");
    }
}

#[test]
fn loop_devices_are_virtual_with_a_fallback_name() {
    for subsystems in ["block", ""] {
        let d = &parse_one(json!({"kname": "/dev/loop3", "subsystems": subsystems,
            "rm": false, "hotplug": false}))[0];
        assert!(d.is_virtual && !d.is_system, "{subsystems:?}");
        assert_eq!(d.description, "Loopback device (loop3)");
    }
}

#[test]
fn takes_sizes_and_flags_in_every_shape_lsblk_writes() {
    assert_eq!(parse_one(json!({"size": 1_000_000}))[0].size, 1_000_000);
    assert_eq!(parse_one(json!({"size": "not a number"}))[0].size, 0);
    assert_eq!(parse_one(json!({"size": -5}))[0].size, 0);
    let old = &parse_one(json!({"ro": "1", "rm": "0", "hotplug": "0",
        "subsystems": "block:scsi:pci"}))[0];
    assert!(old.is_read_only && !old.is_removable && old.is_system && old.is_scsi);
    assert_eq!(parse_one(json!({"phy-sec": 0}))[0].block_size, 512);
    assert_eq!(parse_one(json!({"phy-sec": 4096}))[0].block_size, 4096);
}

#[test]
fn a_removable_disk_with_a_system_mount_is_a_system_drive() {
    for mount in ["/", "/home", "/boot/firmware", "/snap/core/1"] {
        let d = &parse_one(json!({"children": [{"kname": "/dev/sda1", "mountpoint": mount}]}))[0];
        assert!(d.is_system, "{mount}");
    }
    let data =
        &parse_one(json!({"children": [{"kname": "/dev/sda1", "mountpoint": "/media/me/DATA"}]}))
            [0];
    assert!(!data.is_system);
}

#[test]
fn collects_mount_points_and_labels_from_nested_children() {
    let d = &parse_one(json!({"vendor": "Kingston", "children": [
        {"kname": "/dev/sda1", "label": "bootfs", "mountpoint": "/media/me/bootfs"},
        {"kname": "/dev/sda2", "label": "rootfs", "mountpoint": null, "children": [
            {"kname": "/dev/dm-0", "label": "", "mountpoint": "/mnt/inner"}
        ]}
    ]}))[0];
    assert_eq!(d.mountpoints, ["/media/me/bootfs", "/mnt/inner"]);
    assert_eq!(d.mountpoint_labels, ["bootfs", ""]);
    assert_eq!(d.description, "Kingston (bootfs, rootfs)");
}

#[test]
fn strips_direction_overrides_from_names() {
    let d = &parse_one(json!({"model": "Evil\u{202e}gnp.exe"}))[0];
    assert_eq!(d.description, "Evilgnp.exe");
}

#[test]
fn empty_output_offers_nothing_and_rubbish_is_an_error() {
    assert!(
        parse(r#"{"blockdevices": []}"#, |_| String::new())
            .unwrap()
            .is_empty()
    );
    let err = parse("not json", |_| String::new()).unwrap_err();
    assert!(err.to_string().starts_with("Failed to enumerate drives"));
}
