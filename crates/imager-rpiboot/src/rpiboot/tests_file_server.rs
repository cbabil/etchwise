//! Upstream `rpiboot_protocol_test.cpp`: file-server request handling
//! and metadata cases.

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use super::file_server::FileServer;
use super::test_support::{
    DONE, GET_FILE_SIZE, READ_FILE, bulk_bytes, control_outs, file_message, mock_with, quiet,
    temp_dir, write_file,
};
use crate::transport::MockTransport;

pub fn server() -> FileServer {
    let mut server = FileServer::default();
    server.retry_delay = Duration::ZERO;
    server
}

fn run(server: &mut FileServer, mock: &mut MockTransport, dir: &Path) -> Result<(), String> {
    server.run(mock, dir, &mut quiet, &AtomicBool::new(false), None)
}

#[test]
fn handles_get_file_size_request() {
    let fw = temp_dir();
    write_file(fw.path(), "config.txt", b"enable_uart=1\n");
    let mut mock = mock_with(vec![
        file_message(GET_FILE_SIZE, "config.txt"),
        file_message(DONE, ""),
    ]);
    assert_eq!(run(&mut server(), &mut mock, fw.path()), Ok(()));
    // Upstream's test expects one transfer, but its server also
    // acknowledges the empty-name Done message; the size comes first.
    let outs = control_outs(&mock);
    assert_eq!(outs, vec![(14, 0, vec![]), (0, 0, vec![])]);
}

#[test]
fn handles_read_file_request() {
    let fw = temp_dir();
    let content = b"test file data for read";
    write_file(fw.path(), "test.bin", content);
    let mut mock = mock_with(vec![
        file_message(READ_FILE, "test.bin"),
        file_message(DONE, ""),
    ]);
    assert_eq!(run(&mut server(), &mut mock, fw.path()), Ok(()));
    assert!(!control_outs(&mock).is_empty());
    assert_eq!(bulk_bytes(&mock), content.len());
}

#[test]
fn handles_done_command_cleanly() {
    let mut mock = mock_with(vec![file_message(DONE, "")]);
    assert_eq!(run(&mut server(), &mut mock, &std::env::temp_dir()), Ok(()));
}

#[test]
fn done_with_a_name_ends_without_acknowledgement() {
    let mut mock = mock_with(vec![file_message(DONE, "x")]);
    assert_eq!(run(&mut server(), &mut mock, &std::env::temp_dir()), Ok(()));
    assert!(control_outs(&mock).is_empty());
}

#[test]
fn parses_star_prefixed_metadata() {
    let mut mock = mock_with(vec![
        file_message(READ_FILE, "*serial=ABC123"),
        file_message(READ_FILE, "*mac=B8:27:EB:AA:BB:CC"),
        file_message(DONE, ""),
    ]);
    let mut server = server();
    assert_eq!(run(&mut server, &mut mock, &std::env::temp_dir()), Ok(()));
    assert_eq!(server.metadata().serial_number.as_deref(), Some("ABC123"));
    assert_eq!(
        server.metadata().mac_address.as_deref(),
        Some("B8:27:EB:AA:BB:CC")
    );
}

#[test]
fn respects_cancellation() {
    let mut mock = mock_with(vec![file_message(READ_FILE, "big.bin")]);
    let dir = std::env::temp_dir();
    let result = server().run(&mut mock, &dir, &mut quiet, &AtomicBool::new(true), None);
    assert_eq!(result, Err("Cancelled".to_owned()));
}

#[test]
fn uses_custom_resolver() {
    let content = b"resolved-content";
    let mut mock = mock_with(vec![
        file_message(READ_FILE, "virtual.txt"),
        file_message(DONE, ""),
    ]);
    let resolver = |name: &str| {
        if name == "virtual.txt" {
            content.to_vec()
        } else {
            Vec::new()
        }
    };
    let dir = std::env::temp_dir();
    let cancel = AtomicBool::new(false);
    let result = server().run(&mut mock, &dir, &mut quiet, &cancel, Some(&resolver));
    assert_eq!(result, Ok(()));
    assert_eq!(bulk_bytes(&mock), content.len());
}

#[test]
fn handles_missing_file_gracefully() {
    let fw = temp_dir();
    let mut mock = mock_with(vec![
        file_message(READ_FILE, "nonexistent.bin"),
        file_message(DONE, ""),
    ]);
    assert_eq!(run(&mut server(), &mut mock, fw.path()), Ok(()));
    assert_eq!(control_outs(&mock).first(), Some(&(0, 0, vec![])));
}

#[test]
fn parses_board_rev_metadata_as_hex() {
    let mut mock = mock_with(vec![
        file_message(READ_FILE, "*board-rev=c03111"),
        file_message(DONE, ""),
    ]);
    let mut server = server();
    assert_eq!(run(&mut server, &mut mock, &std::env::temp_dir()), Ok(()));
    assert_eq!(server.metadata().board_revision, Some(0xc03111));
}

#[test]
fn ignores_unknown_metadata_keys() {
    let mut mock = mock_with(vec![
        file_message(READ_FILE, "*unknownKey=someValue"),
        file_message(DONE, ""),
    ]);
    let mut server = server();
    assert_eq!(run(&mut server, &mut mock, &std::env::temp_dir()), Ok(()));
    assert_eq!(*server.metadata(), crate::types::DeviceMetadata::default());
}

#[test]
fn reports_progress_per_request() {
    let fw = temp_dir();
    write_file(fw.path(), "a.bin", b"a");
    let mut mock = mock_with(vec![
        file_message(GET_FILE_SIZE, "a.bin"),
        file_message(READ_FILE, "a.bin"),
        file_message(DONE, ""),
    ]);
    let mut seen = Vec::new();
    let mut record = |c: u64, t: u64, s: &str| seen.push((c, t, s.to_owned()));
    let cancel = AtomicBool::new(false);
    assert!(
        server()
            .run(&mut mock, fw.path(), &mut record, &cancel, None)
            .is_ok()
    );
    let statuses: Vec<_> = seen.iter().map(|(c, t, s)| (*c, *t, s.as_str())).collect();
    assert_eq!(
        statuses,
        vec![
            (0, 0, "Waiting for device file requests..."),
            (0, 0, "Querying: a.bin"),
            (1, 0, "Sending: a.bin"),
            (1, 1, "Device boot complete"),
        ]
    );
}
