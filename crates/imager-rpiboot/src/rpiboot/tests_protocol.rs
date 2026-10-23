//! Upstream `rpiboot_protocol_test.cpp`: sideload-directory resolution
//! and `execute()` error paths. The `secure-boot-recovery5/` case is not
//! ported with the mode it tests.

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use super::protocol::{RpibootProtocol, resolve_sideload_dir};
use super::test_support::{
    DONE, GET_FILE_SIZE, READ_FILE, bulk_bytes, control_outs, file_message, mock_with, quiet,
    tar_of, temp_dir, write_file,
};
use crate::transport::MockTransport;
use crate::types::{ChipGeneration, SideloadMode};

fn protocol() -> RpibootProtocol {
    let mut p = RpibootProtocol::default();
    p.bootcode_loader.return_wait = Duration::ZERO;
    p.file_server.retry_delay = Duration::ZERO;
    p
}

fn execute(
    mock: &mut MockTransport,
    chip: ChipGeneration,
    dir: &Path,
    cancel: bool,
) -> Result<(), String> {
    let cancel = AtomicBool::new(cancel);
    protocol().execute(mock, chip, dir, &mut quiet, &cancel, Duration::ZERO)
}

/// Bootcode return value, then the given file-server messages.
fn scripted(messages: Vec<Vec<u8>>) -> MockTransport {
    let mut replies = vec![vec![0, 0, 0, 0]];
    replies.extend(messages);
    mock_with(replies)
}

fn announced(mock: &MockTransport, size: u16) -> bool {
    control_outs(mock)
        .iter()
        .any(|(v, _, d)| d.is_empty() && *v == size)
}

#[test]
fn resolve_sideload_dir_returns_fastboot_subdir() {
    let fw = temp_dir();
    write_file(fw.path(), "fastboot/gadget.bin", b"fastboot-payload");
    write_file(fw.path(), "bootcode4.bin", &[0xAA; 64]);
    let mut mock = scripted(vec![
        file_message(GET_FILE_SIZE, "gadget.bin"),
        file_message(DONE, ""),
    ]);
    assert_eq!(
        execute(&mut mock, ChipGeneration::Bcm2711, fw.path(), false),
        Ok(())
    );
    assert!(announced(&mock, 16), "fastboot-payload is 16 bytes");
    assert_eq!(
        resolve_sideload_dir(fw.path(), SideloadMode::Fastboot),
        fw.path().join("fastboot")
    );
}

#[test]
fn resolve_sideload_dir_falls_back_to_base_dir() {
    let fw = temp_dir();
    write_file(fw.path(), "bootcode4.bin", &[0xCC; 64]);
    write_file(fw.path(), "fallback.bin", b"in-base-dir");
    let mut mock = scripted(vec![
        file_message(GET_FILE_SIZE, "fallback.bin"),
        file_message(DONE, ""),
    ]);
    assert_eq!(
        execute(&mut mock, ChipGeneration::Bcm2711, fw.path(), false),
        Ok(())
    );
    assert!(announced(&mock, 11), "in-base-dir is 11 bytes");
}

#[test]
fn execute_fails_when_bootcode_upload_fails() {
    let fw = temp_dir();
    let mut mock = MockTransport::new();
    let err = execute(&mut mock, ChipGeneration::Bcm2711, fw.path(), false).expect_err("fails");
    assert!(err.contains("Bootcode upload failed"), "{err}");
}

#[test]
fn execute_fails_when_transport_closes_after_bootcode() {
    let fw = temp_dir();
    write_file(fw.path(), "bootcode4.bin", &[0xDD; 64]);
    let mut mock = MockTransport::new();
    mock.closed = true;
    let err = execute(&mut mock, ChipGeneration::Bcm2711, fw.path(), false).expect_err("fails");
    assert!(err.contains("re-enumerate"), "{err}");
}

#[test]
fn execute_respects_cancellation() {
    let fw = temp_dir();
    write_file(fw.path(), "bootcode4.bin", &[0xEE; 64]);
    let mut mock = MockTransport::new();
    assert!(execute(&mut mock, ChipGeneration::Bcm2711, fw.path(), true).is_err());
}

#[test]
fn serves_chip_prefixed_files_from_bootfiles_bin() {
    let fw = temp_dir();
    write_file(fw.path(), "bootcode5.bin", &[0x55; 64]);
    let tar = tar_of(&[("2712/mcb.bin", b"memory-config"), ("2711/mcb.bin", b"x")]);
    write_file(fw.path(), "fastboot/bootfiles.bin", &tar);
    write_file(fw.path(), "fastboot/boot.img", b"kernel");
    let mut mock = scripted(vec![
        file_message(READ_FILE, "mcb.bin"),
        file_message(READ_FILE, "boot.img"),
        file_message(DONE, ""),
    ]);
    assert_eq!(
        execute(&mut mock, ChipGeneration::Bcm2712, fw.path(), false),
        Ok(())
    );
    assert!(announced(&mock, 13), "2712/mcb.bin from the archive");
    assert_eq!(
        bulk_bytes(&mock),
        24 + 64 + 13 + 6,
        "header, bootcode, mcb, disk boot.img"
    );
}
