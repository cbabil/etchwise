//! Failure paths of the fastboot flash flow: hash mismatch, device
//! FAIL, source vs decoder errors, cancellation.

use std::io::{self, Cursor, Read};

use imager_core::pipeline::Progress;

use super::flash::{FlashError, FlashOptions};
use super::flash_over_fastboot;
use super::sparse::BLK;
use super::tests_flash::{commands, options, scripted};
use super::tests_sparse::random_bytes;
use super::tests_support::queue_resp;
use crate::transport::MockTransport;

/// A stored (uncompressed) gzip member, built by hand.
pub(super) fn stored_gzip(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x1F, 0x8B, 8, 0, 0, 0, 0, 0, 0, 0xFF];
    for (i, block) in data.chunks(65535).enumerate() {
        let last = u8::from((i + 1) * 65535 >= data.len());
        let len = block.len() as u16;
        out.push(last);
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(block);
    }
    out.extend_from_slice(&crc32(data).to_le_bytes());
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in data {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

#[test]
fn hash_mismatch_stops_before_customisation_and_reboot() {
    let image = random_bytes(BLK, 3);
    let mut mock = scripted("0x100000");
    let mut opts = options(&image);
    opts.expected_sha256 = Some("00".repeat(32));
    let err = flash_over_fastboot(
        &mut mock,
        Cursor::new(image),
        &opts,
        None,
        &Progress::default(),
    )
    .unwrap_err();
    assert!(
        err.to_string()
            .starts_with("Image hash mismatch. Expected: 0000")
    );
    assert!(!commands(&mock).iter().any(|c| c == "reboot"));
}

#[test]
fn flash_failure_is_reported_with_device_message() {
    let image = random_bytes(BLK, 4);
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "FAIL", "unknown");
    queue_resp(&mut mock, "DATA", "00000000");
    queue_resp(&mut mock, "OKAY", "");
    queue_resp(&mut mock, "FAIL", "bad sector");
    let err = flash_over_fastboot(
        &mut mock,
        Cursor::new(image.clone()),
        &options(&image),
        None,
        &Progress::default(),
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Fastboot flash failed: Flash failed: bad sector"
    );
}

struct Broken;

impl Read for Broken {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("connection reset"))
    }
}

#[test]
fn source_and_decoder_errors_are_told_apart() {
    let progress = Progress::default();
    let opts = FlashOptions::default();
    let mut mock = scripted("0x100000");
    let err = flash_over_fastboot(&mut mock, Broken, &opts, None, &progress).unwrap_err();
    assert_eq!(err, FlashError::Fetch("connection reset".into()));
    let mut corrupt = stored_gzip(&[7; 100]);
    corrupt.truncate(30);
    let mut mock = scripted("0x100000");
    let err =
        flash_over_fastboot(&mut mock, Cursor::new(corrupt), &opts, None, &progress).unwrap_err();
    assert!(matches!(err, FlashError::Decompress(_)), "{err:?}");
}

#[test]
fn cancellation_stops_the_flash() {
    let progress = Progress::default();
    progress.cancel();
    let mut mock = scripted("0x100000");
    let image = random_bytes(BLK, 5);
    let err = flash_over_fastboot(
        &mut mock,
        Cursor::new(image),
        &FlashOptions::default(),
        None,
        &progress,
    )
    .unwrap_err();
    assert_eq!(err, FlashError::Cancelled);
}
