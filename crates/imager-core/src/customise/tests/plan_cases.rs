//! Payload assembly (`applyCustomisationFromSettings`) and applying it to a
//! boot partition (`_customizeImage`).

use std::io::Cursor;

use fatfs::FatType;
use serde_json::json;

use super::map;
use crate::customise::Settings;
use crate::customise::plan::{
    CustomisationPlan, KURA, KURA_SETUP_FILE, PlanInputs, apply_plan, merge_config,
    plan_from_settings,
};
use crate::fat::BootPartition;
use crate::testutil::mbr_image;

fn plan(settings: serde_json::Value, init_format: &str) -> CustomisationPlan {
    let m = map(settings);
    plan_from_settings(
        Settings(&m),
        PlanInputs {
            init_format,
            current_user: "me",
            ..PlanInputs::default()
        },
    )
}

#[test]
fn no_init_format_stages_nothing() {
    let p = plan(
        json!({"hostname": "pi", "recommendedWifiCountry": "GB"}),
        "",
    );
    assert_eq!(p, CustomisationPlan::default());
}

#[test]
fn systemd_always_stages_firstrun_and_regdom() {
    let p = plan(
        json!({"recommendedWifiCountry": "GB", "secureBootEnabled": true}),
        "systemd",
    );
    assert!(p.firstrun.starts_with("#!/bin/sh\n"));
    assert_eq!(p.cmdline, " cfg80211.ieee80211_regdom=GB");
    assert!(p.secure_boot_key_path.is_empty(), "no RSA key configured");
}

#[test]
fn cloudinit_without_content_skips_cmdline() {
    let p = plan(json!({"recommendedWifiCountry": "GB"}), "cloudinit");
    // Country alone still produces the rfkill runcmd, so there is content.
    assert!(p.cloudinit.contains("rfkill"));
    let empty = plan(json!({}), "cloudinit");
    assert!(empty.cloudinit.is_empty() && empty.network.is_empty() && empty.cmdline.is_empty());
    // Upstream quirk: a missing `enableSerial` is not "Disabled", so
    // cloudinit-rpi images get the serial interface even with no settings.
    assert!(
        plan(json!({}), "cloudinit-rpi")
            .cloudinit
            .contains("    serial: true")
    );
}

#[test]
fn secure_boot_needs_setting_and_key() {
    let m = map(json!({"hostname": "pi", "secureBootEnabled": true}));
    let inputs = PlanInputs {
        init_format: "cloudinit-rpi",
        secure_boot_key_path: "/keys/private.pem",
        ..PlanInputs::default()
    };
    assert_eq!(
        plan_from_settings(Settings(&m), inputs).secure_boot_key_path,
        "/keys/private.pem"
    );
}

#[test]
fn merges_config_like_upstream() {
    assert_eq!(
        merge_config(b"a=1\n#dtparam=i2c_arm=on\n", "dtparam=i2c_arm=on"),
        b"a=1\ndtparam=i2c_arm=on\n"
    );
    assert_eq!(merge_config(b"a=1\nb=2", "b=2\nc=3\n"), b"a=1\nb=2\nc=3\n");
    assert_eq!(merge_config(b"", "x=1"), b"\nx=1\n");
    assert_eq!(
        merge_config(b"x=1\n", "x=1"),
        b"x=1\nx=1\n",
        "first line has no preceding newline, as upstream"
    );
}

#[test]
fn applies_cloudinit_plan_to_boot_partition() {
    let mut p = plan(
        json!({"hostname": "pi", "wifiSSID": "Home", "wifiPasswordCrypt": "abc", "recommendedWifiCountry": "GB"}),
        "cloudinit-rpi",
    );
    p.config = "dtoverlay=dwc2".into();
    let image = seeded(&[
        ("cmdline.txt", b"console=tty1 root=PARTUUID=x rootwait \n"),
        ("config.txt", b"[all]\n"),
    ]);
    let mut boot = BootPartition::open(Cursor::new(image)).unwrap();
    apply_plan(&mut boot, &p, "rpi-imager-1700000000000").unwrap();
    let read = |boot: &mut BootPartition<_>, f| {
        String::from_utf8(boot.read_file(f).unwrap().unwrap()).unwrap()
    };
    assert_eq!(
        read(&mut boot, "meta-data"),
        "instance-id: rpi-imager-1700000000000\n"
    );
    assert!(read(&mut boot, "user-data").starts_with("#cloud-config\nmanage_resolv_conf: false\n"));
    assert!(read(&mut boot, "network-config").contains("\"Home\":"));
    assert_eq!(read(&mut boot, "config.txt"), "[all]\ndtoverlay=dwc2\n");
    assert_eq!(
        read(&mut boot, "cmdline.txt"),
        "console=tty1 root=PARTUUID=x rootwait cfg80211.ieee80211_regdom=GB ds=nocloud;i=rpi-imager-1700000000000"
    );
    assert!(!boot.exists("firstrun.sh").unwrap());
}

#[test]
fn applies_systemd_plan_to_boot_partition() {
    let p = plan(json!({"hostname": "pi"}), "systemd");
    let mut boot = BootPartition::open(Cursor::new(seeded(&[]))).unwrap();
    apply_plan(&mut boot, &p, "unused").unwrap();
    assert!(boot.exists("firstrun.sh").unwrap() && !boot.exists("meta-data").unwrap());
    let cmdline = boot.read_file("cmdline.txt").unwrap().unwrap();
    assert_eq!(cmdline, b" systemd.run=/boot/firstrun.sh systemd.run_success_action=reboot systemd.unit=kernel-command-line.target");
}

#[test]
fn empty_plan_touches_nothing() {
    let mut boot = BootPartition::open(Cursor::new(seeded(&[]))).unwrap();
    apply_plan(&mut boot, &CustomisationPlan::default(), "x").unwrap();
    assert!(boot.list_files().unwrap().is_empty());
}

fn seeded(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut boot = BootPartition::open(Cursor::new(mbr_image(FatType::Fat16, 16))).unwrap();
    for (name, data) in files {
        boot.write_file(name, data).unwrap();
    }
    boot.finish().unwrap().into_inner()
}

#[test]
fn kura_stages_its_setup_file_and_leaves_cmdline_alone() {
    let p = plan(
        json!({"hostname": "kura-01", "recommendedWifiCountry": "FR"}),
        KURA,
    );
    assert!(p.cloudinit.contains("\"hostname\":\"kura-01\""));
    assert!(p.cmdline.is_empty() && p.firstrun.is_empty() && p.network.is_empty());
    assert!(!p.needs_seed());
}

#[test]
fn applies_kura_plan_to_boot_partition() {
    let p = plan(json!({"hostname": "kura-01"}), KURA);
    let image = seeded(&[(
        "cmdline.txt",
        b"console=tty1 root=/dev/mmcblk0p2 rootwait\n",
    )]);
    let mut boot = BootPartition::open(Cursor::new(image)).unwrap();
    apply_plan(&mut boot, &p, "unused").unwrap();
    let setup = boot.read_file(KURA_SETUP_FILE).unwrap().unwrap();
    assert_eq!(setup, p.cloudinit.as_bytes());
    assert_eq!(
        boot.read_file("cmdline.txt").unwrap().unwrap(),
        b"console=tty1 root=/dev/mmcblk0p2 rootwait\n"
    );
    assert!(!boot.exists("user-data").unwrap() && !boot.exists("firstrun.sh").unwrap());
}
