//! Upstream `fastboot_protocol_test.cpp` "Customisation pattern" cases,
//! run through the real [`apply_customisation`] instead of re-implementing
//! its logic inline as upstream's tests must (the method is private there).

use imager_core::customise::plan::{CustomisationPlan, merge_config};

use super::customise::apply_customisation;
use super::flash::FlashError;
use super::tests_support::{
    bulk_writes, never, queue_read_file, queue_resp, queue_write_file, written_strings,
};
use crate::transport::MockTransport;

/// The bytes staged right before `oem download-file <path>`.
fn staged(mock: &MockTransport, path: &str) -> Vec<u8> {
    let marker = format!("oem download-file {path}");
    let mut data = Vec::new();
    for w in bulk_writes(mock) {
        if w.starts_with(b"stage:") {
            data.clear();
        } else if w == marker.as_bytes() {
            return data;
        } else {
            data.extend_from_slice(&w);
        }
    }
    panic!("{path} was not written");
}

fn plan(init_format: &str) -> CustomisationPlan {
    CustomisationPlan {
        init_format: init_format.into(),
        ..CustomisationPlan::default()
    }
}

#[test]
fn config_txt_read_modify_write_with_uncomment() {
    let mut mock = MockTransport::new();
    let original = "# Some comment\n#dtparam=spi=on\ndtparam=audio=on\n";
    let new = merge_config(original.as_bytes(), "dtparam=spi=on\ndtoverlay=vc4-kms-v3d");
    queue_read_file(&mut mock, original.as_bytes());
    queue_write_file(&mut mock, new.len());
    let plan = CustomisationPlan {
        config: "dtparam=spi=on\ndtoverlay=vc4-kms-v3d".into(),
        ..plan("systemd")
    };
    apply_customisation(&mut mock, &plan, "id", &never).unwrap();
    let modified = String::from_utf8(staged(&mock, "/boot/firmware/config.txt")).unwrap();
    assert!(modified.contains("dtparam=spi=on\n"));
    assert!(!modified.contains("#dtparam=spi=on"));
    assert!(modified.contains("dtoverlay=vc4-kms-v3d\n"));
    assert!(modified.contains("dtparam=audio=on\n"));
}

#[test]
fn config_txt_skips_already_present_entries() {
    let original = "dtparam=audio=on\ndtoverlay=vc4-kms-v3d\n";
    let merged = merge_config(original.as_bytes(), "dtoverlay=vc4-kms-v3d");
    assert_eq!(merged, original.as_bytes());
}

#[test]
fn cmdline_txt_append_with_firstrun_params() {
    let mut mock = MockTransport::new();
    let original = "console=serial0,115200 console=tty1 root=PARTUUID=abcd-1234 rootwait\n";
    let firstrun = "#!/bin/bash\n";
    queue_write_file(&mut mock, firstrun.len());
    queue_read_file(&mut mock, original.as_bytes());
    queue_write_file(&mut mock, 200);
    let plan = CustomisationPlan {
        firstrun: firstrun.into(),
        ..plan("systemd")
    };
    apply_customisation(&mut mock, &plan, "id", &never).unwrap();
    assert_eq!(
        staged(&mock, "/boot/firmware/firstrun.sh"),
        firstrun.as_bytes()
    );
    let modified = String::from_utf8(staged(&mock, "/boot/firmware/cmdline.txt")).unwrap();
    assert!(modified.contains("rootwait"));
    assert!(modified.contains("systemd.run=/boot/firstrun.sh"));
    assert!(modified.contains("systemd.unit=kernel-command-line.target"));
    assert!(!modified.contains('\n'));
}

#[test]
fn cloud_init_meta_data_user_data_network_config() {
    let mut mock = MockTransport::new();
    for _ in 0..3 {
        queue_write_file(&mut mock, 64);
    }
    queue_read_file(&mut mock, b"console=tty1 rootwait");
    queue_write_file(&mut mock, 64);
    let plan = CustomisationPlan {
        cloudinit: "hostname: mypi\npassword: secret\n".into(),
        network: "version: 2\nethernets:\n  eth0:\n    dhcp4: true\n".into(),
        ..plan("cloudinit")
    };
    apply_customisation(&mut mock, &plan, "rpi-imager-1234567890", &never).unwrap();
    let oem: Vec<String> = written_strings(&mock)
        .into_iter()
        .filter(|s| s.starts_with("oem download-file"))
        .collect();
    assert_eq!(
        oem[..3],
        [
            "oem download-file /boot/firmware/meta-data",
            "oem download-file /boot/firmware/user-data",
            "oem download-file /boot/firmware/network-config",
        ]
    );
    let meta = staged(&mock, "/boot/firmware/meta-data");
    assert_eq!(meta, b"instance-id: rpi-imager-1234567890\n");
    let user = staged(&mock, "/boot/firmware/user-data");
    assert_eq!(user, b"#cloud-config\nhostname: mypi\npassword: secret\n");
    let cmdline = staged(&mock, "/boot/firmware/cmdline.txt");
    assert_eq!(
        cmdline,
        b"console=tty1 rootwait ds=nocloud;i=rpi-imager-1234567890"
    );
}

#[test]
fn read_device_file_failure_aborts_before_write() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "FAIL", "I/O error");
    let plan = CustomisationPlan {
        config: "dtparam=spi=on".into(),
        ..plan("systemd")
    };
    let err = apply_customisation(&mut mock, &plan, "id", &never).unwrap_err();
    assert!(matches!(err, FlashError::ReadFile("config.txt", _)));
    assert!(
        err.to_string()
            .starts_with("Failed to read config.txt: oem upload-file failed")
    );
    for s in written_strings(&mock) {
        assert!(!s.starts_with("stage:") && !s.starts_with("oem download-file"));
    }
}

#[test]
fn nothing_is_sent_without_customisation_or_init_format() {
    let mut mock = MockTransport::new();
    apply_customisation(&mut mock, &plan("systemd"), "id", &never).unwrap();
    let no_format = CustomisationPlan {
        firstrun: "x".into(),
        ..plan("")
    };
    apply_customisation(&mut mock, &no_format, "id", &never).unwrap();
    assert!(mock.calls.is_empty());
}
