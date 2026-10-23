//! Upstream `fastboot_protocol_test.cpp`: device file transfer
//! (`writeDeviceFile` / `readDeviceFile`), error clearing and round-trip.

use super::protocol::{read_device_file, write_device_file};
use super::tests_support::{never, queue_read_file, queue_resp, queue_write_file, written_strings};
use crate::transport::MockTransport;

#[test]
fn write_device_file_stages_then_sends_oem_download_file() {
    let mut mock = MockTransport::new();
    queue_write_file(&mut mock, 5);
    assert!(write_device_file(&mut mock, "/boot/firmware/test.txt", b"hello", &never).is_ok());
    let cmd = "oem download-file /boot/firmware/test.txt";
    assert!(written_strings(&mock).iter().any(|s| s == cmd));
}

#[test]
fn write_device_file_fails_when_stage_fails() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "FAIL", "buffer full");
    assert!(write_device_file(&mut mock, "/boot/firmware/test.txt", &[1, 2, 3], &never).is_err());
}

#[test]
fn read_device_file_sends_oem_upload_file_then_uploads() {
    let mut mock = MockTransport::new();
    queue_read_file(&mut mock, b"world");
    let result = read_device_file(&mut mock, "/boot/firmware/cmdline.txt", &never).unwrap();
    assert_eq!(result, b"world");
    let cmd = "oem upload-file /boot/firmware/cmdline.txt";
    assert!(written_strings(&mock).iter().any(|s| s == cmd));
}

#[test]
fn read_device_file_fails_when_oem_upload_file_fails() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "FAIL", "file not found");
    let err = read_device_file(&mut mock, "/boot/firmware/missing.txt", &never).unwrap_err();
    assert!(err.0.contains("oem upload-file"));
    assert!(err.0.ends_with(": file not found"));
}

#[test]
fn read_device_file_succeeds_after_a_failure() {
    // Upstream: "readDeviceFile clears lastError on success".
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "FAIL", "first call failed");
    let first = read_device_file(&mut mock, "/boot/firmware/missing.txt", &never);
    assert!(first.is_err_and(|e| !e.0.is_empty()));
    queue_read_file(&mut mock, b"OK");
    let second = read_device_file(&mut mock, "/boot/firmware/config.txt", &never);
    assert_eq!(second.unwrap(), b"OK");
}

#[test]
fn write_device_file_succeeds_after_a_failure() {
    // Upstream: "writeDeviceFile clears lastError on success".
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "FAIL", "buffer full");
    let data = [1, 2, 3];
    let first = write_device_file(&mut mock, "/tmp/test", &data, &never);
    assert!(first.is_err_and(|e| !e.0.is_empty()));
    queue_write_file(&mut mock, data.len());
    assert!(write_device_file(&mut mock, "/tmp/test", &data, &never).is_ok());
}

#[test]
fn write_then_read_device_file_round_trip() {
    let mut mock = MockTransport::new();
    let original = b"hello\n";
    queue_write_file(&mut mock, original.len());
    assert!(write_device_file(&mut mock, "/boot/firmware/config.txt", original, &never).is_ok());
    queue_read_file(&mut mock, original);
    let back = read_device_file(&mut mock, "/boot/firmware/config.txt", &never).unwrap();
    assert_eq!(back, original);
    let commands: Vec<String> = written_strings(&mock)
        .into_iter()
        .filter(|s| s.starts_with("stage:") || s.starts_with("oem ") || s == "upload")
        .collect();
    assert_eq!(commands.len(), 4);
    assert!(commands[0].starts_with("stage:"));
    assert_eq!(commands[1], "oem download-file /boot/firmware/config.txt");
    assert_eq!(commands[2], "oem upload-file /boot/firmware/config.txt");
    assert_eq!(commands[3], "upload");
}

#[test]
fn parses_leading_digits_like_stoul() {
    use crate::types::parse_leading;
    assert_eq!(parse_leading("  0x1F00 rest", 16), Some(0x1f00));
    assert_eq!(parse_leading("c03111", 16), Some(0xc0_3111));
    assert_eq!(parse_leading("0xg", 16), Some(0));
    assert_eq!(parse_leading("4096 bytes", 10), Some(4096));
    assert_eq!(parse_leading("none", 10), None);
}
