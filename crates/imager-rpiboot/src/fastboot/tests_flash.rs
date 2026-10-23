//! The whole fastboot flash flow over a scripted transport (upstream has
//! no test for `FastbootFlashThread`; these follow `runImpl`).

use std::io::{self, Cursor};

use imager_core::customise::plan::CustomisationPlan;
use imager_core::pipeline::{Progress, WriteState};
use sha2::{Digest, Sha256};

use super::flash::{DEFAULT_MAX_DOWNLOAD_SIZE, FlashError, FlashOptions, max_download_size};
use super::flash_over_fastboot;
use super::sparse::BLK;
use super::tests_flash_errors::stored_gzip;
use super::tests_sparse::random_bytes;
use super::tests_support::{queue_read_file, queue_resp, queue_write_file, written_strings};
use crate::transport::MockTransport;

pub(super) fn options(image: &[u8]) -> FlashOptions {
    FlashOptions {
        expected_sha256: Some(hex::encode(Sha256::digest(image))),
        extract_len: image.len() as u64,
        bmap: None,
    }
}

/// getvar, then one segment's download + flash replies.
pub(super) fn scripted(max: &str) -> MockTransport {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "OKAY", max);
    queue_resp(&mut mock, "DATA", "00000000");
    queue_resp(&mut mock, "OKAY", "");
    queue_resp(&mut mock, "OKAY", "");
    mock
}

pub(super) fn commands(mock: &MockTransport) -> Vec<String> {
    written_strings(mock)
        .into_iter()
        .filter(|s| s.is_ascii() && s.len() < 80 && s.chars().all(|c| !c.is_control()))
        .collect()
}

#[test]
fn flashes_one_segment_verifies_and_reboots() {
    let image = random_bytes(3 * BLK + 100, 1);
    let mut mock = scripted("0x100000");
    queue_resp(&mut mock, "OKAY", "");
    let progress = Progress::default();
    flash_over_fastboot(
        &mut mock,
        Cursor::new(image.clone()),
        &options(&image),
        None,
        &progress,
    )
    .unwrap();
    let cmds = commands(&mock);
    assert_eq!(cmds[0], "getvar:max-download-size");
    assert!(cmds[1].starts_with("download:"));
    assert_eq!(cmds[2..], ["flash:mmcblk0", "reboot"]);
    let snap = progress.snapshot();
    assert_eq!(
        (snap.bytes_written, snap.write_total),
        (image.len() as u64, image.len() as u64)
    );
    assert_eq!(snap.state, WriteState::Finalizing);
}

#[test]
fn decompresses_gzip_sources() {
    let image = random_bytes(2 * BLK, 2);
    let mut mock = scripted("1048576");
    queue_resp(&mut mock, "OKAY", "");
    let progress = Progress::default();
    let source = Cursor::new(stored_gzip(&image));
    flash_over_fastboot(&mut mock, source, &options(&image), None, &progress).unwrap();
}

#[test]
fn customisation_runs_after_flashing_and_before_reboot() {
    let image = random_bytes(BLK, 6);
    let mut mock = scripted("0x100000");
    queue_write_file(&mut mock, 3);
    queue_read_file(&mut mock, b"root=/dev/mmcblk0p2 rootwait\n");
    queue_write_file(&mut mock, 100);
    queue_resp(&mut mock, "OKAY", "");
    let plan = CustomisationPlan {
        firstrun: "#!/bin/sh\n".into(),
        init_format: "systemd".into(),
        ..CustomisationPlan::default()
    };
    let progress = Progress::default();
    flash_over_fastboot(
        &mut mock,
        Cursor::new(image.clone()),
        &options(&image),
        Some(&plan),
        &progress,
    )
    .unwrap();
    let oem: Vec<String> = commands(&mock)
        .into_iter()
        .filter(|c| c.starts_with("oem ") || c == "reboot")
        .collect();
    assert_eq!(
        oem,
        [
            "oem download-file /boot/firmware/firstrun.sh",
            "oem upload-file /boot/firmware/cmdline.txt",
            "oem download-file /boot/firmware/cmdline.txt",
            "reboot",
        ]
    );
}

#[test]
fn max_download_size_parses_hex_decimal_and_falls_back() {
    for (reply, want) in [
        ("0x10000", 0x10000),
        ("0X20", 0x20),
        ("123abc", 123),
        ("junk", DEFAULT_MAX_DOWNLOAD_SIZE),
    ] {
        let mut mock = MockTransport::new();
        queue_resp(&mut mock, "OKAY", reply);
        assert_eq!(max_download_size(&mut mock), want, "{reply}");
    }
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "FAIL", "");
    assert_eq!(max_download_size(&mut mock), DEFAULT_MAX_DOWNLOAD_SIZE);
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "OKAY", "64");
    let err = flash_over_fastboot(
        &mut mock,
        io::empty(),
        &FlashOptions::default(),
        None,
        &Progress::default(),
    );
    assert_eq!(err.unwrap_err(), FlashError::SegmentTooSmall(64));
}
