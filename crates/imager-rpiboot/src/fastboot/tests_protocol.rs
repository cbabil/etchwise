//! Upstream `fastboot_protocol_test.cpp`: response parsing, command
//! encoding, flash, getvar and transport failures.

use super::protocol::{ResponseType, flash, get_var, send_command};
use super::tests_support::{bulk_writes, fail_next_write, queue, queue_resp, written_strings};
use crate::transport::MockTransport;

#[test]
fn parses_okay_response() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "OKAY", "done");
    let resp = send_command(&mut mock, "getvar:version", 3000);
    assert_eq!(resp.kind, ResponseType::Okay);
    assert_eq!(resp.message, "done");
}

#[test]
fn parses_fail_response() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "FAIL", "unknown command");
    let resp = send_command(&mut mock, "oem unsupported", 3000);
    assert_eq!(resp.kind, ResponseType::Fail);
    assert_eq!(resp.message, "unknown command");
}

#[test]
fn parses_data_response() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "DATA", "00001000");
    let resp = send_command(&mut mock, "download:00001000", 3000);
    assert_eq!(resp.kind, ResponseType::Data);
    assert_eq!(resp.data_size, 0x1000);
}

#[test]
fn parses_info_response_and_continues() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "INFO", "erasing partition...");
    queue_resp(&mut mock, "OKAY", "");
    let resp = send_command(&mut mock, "erase:boot", 3000);
    assert_eq!(resp.kind, ResponseType::Okay);
}

#[test]
fn sends_command_as_ascii_via_bulk_out() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "OKAY", "");
    send_command(&mut mock, "getvar:version", 3000);
    let writes = written_strings(&mock);
    assert!(!writes.is_empty());
    assert_eq!(writes[0], "getvar:version");
}

#[test]
fn flash_sends_flash_partition_command() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "OKAY", "");
    assert!(flash(&mut mock, "boot", 3000).is_ok());
    assert_eq!(written_strings(&mock)[0], "flash:boot");
}

#[test]
fn get_var_returns_variable_value() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "OKAY", "0x10000");
    assert_eq!(
        get_var(&mut mock, "max-download-size").as_deref(),
        Some("0x10000")
    );
}

#[test]
fn get_var_returns_none_on_fail() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "FAIL", "not found");
    assert_eq!(get_var(&mut mock, "nonexistent"), None);
}

#[test]
fn send_command_returns_fail_when_bulk_write_fails() {
    let mut mock = MockTransport::new();
    fail_next_write(&mut mock);
    let resp = send_command(&mut mock, "getvar:version", 3000);
    assert_eq!(resp.kind, ResponseType::Fail);
    assert!(resp.message.contains("Failed to send"));
}

#[test]
fn send_command_returns_fail_on_short_response() {
    let mut mock = MockTransport::new();
    queue(&mut mock, b"OK".to_vec());
    let resp = send_command(&mut mock, "getvar:version", 3000);
    assert_eq!(resp.kind, ResponseType::Fail);
    assert!(resp.message.contains("Short"));
}

#[test]
fn send_command_returns_fail_on_empty_read_queue() {
    let mut mock = MockTransport::new();
    let resp = send_command(&mut mock, "getvar:version", 3000);
    assert_eq!(resp.kind, ResponseType::Fail);
}

#[test]
fn send_command_handles_unknown_response_prefix() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "XXXX", "garbage");
    let resp = send_command(&mut mock, "getvar:version", 3000);
    assert_eq!(resp.kind, ResponseType::Fail);
    assert!(resp.message.contains("Unknown response prefix"));
}

#[test]
fn send_command_handles_multiple_info_before_okay() {
    let mut mock = MockTransport::new();
    for step in ["step 1...", "step 2...", "step 3..."] {
        queue_resp(&mut mock, "INFO", step);
    }
    queue_resp(&mut mock, "OKAY", "done");
    let resp = send_command(&mut mock, "erase:boot", 3000);
    assert_eq!(resp.kind, ResponseType::Okay);
    assert_eq!(resp.message, "done");
}

#[test]
fn send_command_handles_info_then_fail() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "INFO", "erasing...");
    queue_resp(&mut mock, "FAIL", "erase failed");
    let resp = send_command(&mut mock, "erase:boot", 3000);
    assert_eq!(resp.kind, ResponseType::Fail);
    assert_eq!(resp.message, "erase failed");
}

#[test]
fn flash_returns_error_on_fail_response() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "FAIL", "partition not found");
    let err = flash(&mut mock, "nonexistent", 3000).unwrap_err().0;
    assert!(err.contains("Flash failed"));
    assert!(err.contains("partition not found"));
}

#[test]
fn flash_fails_on_transport_write_error() {
    let mut mock = MockTransport::new();
    fail_next_write(&mut mock);
    assert!(flash(&mut mock, "boot", 3000).is_err());
    assert_eq!(bulk_writes(&mock).len(), 1);
}

#[test]
fn text_replies_are_intermediate_and_data_size_parses_like_stoul() {
    let mut mock = MockTransport::new();
    queue_resp(&mut mock, "TEXT", "hello");
    queue_resp(&mut mock, "DATA", "0000100z");
    let resp = send_command(&mut mock, "x", 3000);
    assert_eq!((resp.kind, resp.data_size), (ResponseType::Data, 0x100));
}
