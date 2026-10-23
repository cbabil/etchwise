//! Upstream `fastboot_protocol_test.cpp`: download, flashImage, stage and
//! upload data phases, cancellation and progress callbacks.

use super::protocol::{download, flash_image, stage, upload};
use super::tests_support::{
    always, fail_next_write, hex_size, never, queue, queue_resp, written_strings,
};
use crate::transport::MockTransport;

fn data_then_okay(mock: &mut MockTransport, len: usize) {
    queue_resp(mock, "DATA", &hex_size(len));
    queue_resp(mock, "OKAY", "");
}

#[test]
fn download_sends_data_after_data_response() {
    let mut mock = MockTransport::new();
    let payload = vec![0xBB; 4096];
    data_then_okay(&mut mock, payload.len());
    assert!(download(&mut mock, &payload, None, &never).is_ok());
    let writes = super::tests_support::bulk_writes(&mock);
    let found = writes.iter().any(|w| w.starts_with(b"download:"));
    let payload_bytes: usize = writes
        .iter()
        .filter(|w| !w.starts_with(b"download:"))
        .map(Vec::len)
        .sum();
    assert!(found);
    assert_eq!(payload_bytes, payload.len());
}

#[test]
fn download_fails_when_initial_write_fails() {
    let mut mock = MockTransport::new();
    fail_next_write(&mut mock);
    let err = download(&mut mock, &[0xAA; 256], None, &never).unwrap_err();
    assert!(err.0.contains("Failed to send download command"));
}

#[test]
fn download_fails_when_device_returns_fail_instead_of_data() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "FAIL", "no space");
    let err = download(&mut mock, &[0xAA; 256], None, &never).unwrap_err();
    assert!(err.0.contains("Expected DATA"));
}

#[test]
fn download_aborts_when_cancelled() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "DATA", &hex_size(4096));
    let err = download(&mut mock, &[0xBB; 4096], None, &always).unwrap_err();
    assert!(err.0.contains("Cancelled"));
}

#[test]
fn download_fails_when_final_reply_is_fail() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "DATA", &hex_size(4096));
    queue_resp(&mut mock, "FAIL", "write error");
    let err = download(&mut mock, &[0xCC; 4096], None, &never).unwrap_err();
    assert!(err.0.contains("download failed"));
}

#[test]
fn download_fails_on_bulk_write_error_during_transfer() {
    // Upstream's mock could not fail a later write; ours can.
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "DATA", &hex_size(4096));
    mock.fail_bulk_write_after = Some(1);
    let err = download(&mut mock, &[0xCC; 4096], None, &never).unwrap_err();
    assert_eq!(err.0, "Bulk write failed during download at offset 0");
}

#[test]
fn flash_image_performs_download_then_flash() {
    let mut mock = MockTransport::new();
    data_then_okay(&mut mock, 512);
    queue_resp(&mut mock, "OKAY", "");
    assert!(flash_image(&mut mock, "boot", &[0xDD; 512], None, &never).is_ok());
    assert!(written_strings(&mock).iter().any(|s| s == "flash:boot"));
}

#[test]
fn flash_image_fails_if_download_fails() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "FAIL", "no space");
    assert!(flash_image(&mut mock, "boot", &[0xEE; 256], None, &never).is_err());
}

#[test]
fn download_invokes_progress_callback() {
    let mut mock = MockTransport::new();
    data_then_okay(&mut mock, 4096);
    let (mut calls, mut last) = (0, 0);
    let mut cb = |current: u64, total: u64, _: &str| {
        calls += 1;
        last = current;
        assert_eq!(total, 4096);
    };
    assert!(download(&mut mock, &[0xFF; 4096], Some(&mut cb), &never).is_ok());
    assert!(calls > 0);
    assert_eq!(last, 4096);
}

#[test]
fn stage_sends_data_with_stage_prefix() {
    let mut mock = MockTransport::new();
    data_then_okay(&mut mock, 256);
    assert!(stage(&mut mock, &[0xAA; 256], None, &never).is_ok());
    assert_eq!(written_strings(&mock)[0], "stage:00000100");
}

#[test]
fn stage_fails_when_device_returns_fail() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "FAIL", "out of memory");
    let err = stage(&mut mock, &[0xBB; 64], None, &never).unwrap_err();
    assert!(err.0.contains("DATA"));
}

#[test]
fn upload_reads_data_from_device() {
    let mut mock = MockTransport::new();
    let payload = vec![1, 2, 3, 4, 5];
    queue_resp(&mut mock, "DATA", &hex_size(payload.len()));
    queue(&mut mock, payload.clone());
    queue_resp(&mut mock, "OKAY", "");
    assert_eq!(upload(&mut mock, None, &never).unwrap(), payload);
    assert_eq!(written_strings(&mock)[0], "upload");
}

#[test]
fn upload_fails_on_fail_response() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "FAIL", "no staged data");
    let err = upload(&mut mock, None, &never).unwrap_err();
    assert!(err.0.contains("DATA"));
}

#[test]
fn upload_aborts_when_cancelled() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "DATA", "00100000");
    queue(&mut mock, vec![0xFF; 1024]);
    assert!(upload(&mut mock, None, &always).is_err());
}

#[test]
fn upload_invokes_progress_callback() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "DATA", &hex_size(4096));
    queue(&mut mock, vec![0xCC; 4096]);
    queue_resp(&mut mock, "OKAY", "");
    let (mut calls, mut last) = (0, 0);
    let mut cb = |current: u64, total: u64, _: &str| {
        calls += 1;
        last = current;
        assert_eq!(total, 4096);
    };
    assert_eq!(
        upload(&mut mock, Some(&mut cb), &never).unwrap().len(),
        4096
    );
    assert!(calls > 0);
    assert_eq!(last, 4096);
}

#[test]
fn stage_invokes_progress_callback() {
    let mut mock = MockTransport::new();
    data_then_okay(&mut mock, 2048);
    let mut calls = 0;
    let mut cb = |_: u64, total: u64, _: &str| {
        calls += 1;
        assert_eq!(total, 2048);
    };
    assert!(stage(&mut mock, &[0xDD; 2048], Some(&mut cb), &never).is_ok());
    assert!(calls > 0);
}
