//! Upstream `rpiboot_protocol_test.cpp`: bootcode loader, chip-generation
//! helper and boot-message layout cases.

use std::sync::atomic::AtomicBool;
use std::time::Duration;

use super::bootcode_loader::{BootcodeLoader, bootcode_filename};
use super::test_support::{bulk_bytes, control_outs, temp_dir, write_file};
use crate::transport::{MockTransport, UsbCall};
use crate::types::{ChipGeneration, VENDOR_REQUEST, VENDOR_REQUEST_TYPE, boot_message};

fn loader() -> BootcodeLoader {
    BootcodeLoader {
        return_wait: Duration::ZERO,
    }
}

#[test]
fn selects_correct_bootcode_filename() {
    assert_eq!(bootcode_filename(ChipGeneration::Bcm2836_7), "bootcode.bin");
    assert_eq!(bootcode_filename(ChipGeneration::Bcm2711), "bootcode4.bin");
    assert_eq!(bootcode_filename(ChipGeneration::Bcm2712), "bootcode5.bin");
}

#[test]
fn sends_boot_message_header_and_payload() {
    let fw = temp_dir();
    write_file(fw.path(), "bootcode4.bin", &[0xAB; 256]);
    let mut mock = MockTransport::new();
    let cancel = AtomicBool::new(false);
    let sent = loader().send(&mut mock, ChipGeneration::Bcm2711, fw.path(), &cancel);
    assert_eq!(sent, Ok(()));
    let outs: Vec<_> = mock
        .calls
        .iter()
        .filter_map(|c| match c {
            UsbCall::ControlOut {
                request_type,
                request,
                value,
                index,
                data,
            } => Some((*request_type, *request, *value, *index, data.is_empty())),
            _ => None,
        })
        .collect();
    let (rt, rq) = (VENDOR_REQUEST_TYPE, VENDOR_REQUEST);
    assert_eq!(outs, vec![(rt, rq, 24, 0, true), (rt, rq, 256, 0, true)]);
    assert_eq!(bulk_bytes(&mock), 24 + 256);
}

#[test]
fn fails_when_file_is_missing() {
    let fw = temp_dir();
    let mut mock = MockTransport::new();
    let cancel = AtomicBool::new(false);
    let err = loader().send(&mut mock, ChipGeneration::Bcm2711, fw.path(), &cancel);
    assert!(err.expect_err("missing").contains("Cannot open"));
}

#[test]
fn respects_cancellation() {
    let fw = temp_dir();
    write_file(fw.path(), "bootcode.bin", &[0xCC; 1024]);
    let mut mock = MockTransport::new();
    let cancel = AtomicBool::new(true);
    let sent = loader().send(&mut mock, ChipGeneration::Bcm2836_7, fw.path(), &cancel);
    assert!(sent.is_err());
    assert!(mock.calls.is_empty());
}

#[test]
fn chip_generation_from_pid() {
    assert_eq!(ChipGeneration::from_pid(0x2763), None);
    assert_eq!(
        ChipGeneration::from_pid(0x2764),
        Some(ChipGeneration::Bcm2836_7)
    );
    assert_eq!(
        ChipGeneration::from_pid(0x2711),
        Some(ChipGeneration::Bcm2711)
    );
    assert_eq!(
        ChipGeneration::from_pid(0x2712),
        Some(ChipGeneration::Bcm2712)
    );
    assert_eq!(ChipGeneration::from_pid(0x1234), None);
}

#[test]
fn chip_generation_names() {
    assert_eq!(ChipGeneration::Bcm2711.name(), "BCM2711");
    assert_eq!(ChipGeneration::Bcm2712.name(), "BCM2712");
}

#[test]
fn boot_message_is_24_bytes_with_correct_layout() {
    let message = boot_message(0x1234_5678, [0; 20]);
    assert_eq!(message.len(), 24);
    assert_eq!(
        i32::from_le_bytes([message[0], message[1], message[2], message[3]]),
        0x1234_5678
    );
    assert!(message[4..].iter().all(|b| *b == 0));
}

#[test]
fn fails_when_control_transfer_fails() {
    let fw = temp_dir();
    write_file(fw.path(), "bootcode4.bin", &[0xAB; 256]);
    let mut mock = MockTransport::new();
    mock.control_out_ok = false;
    let cancel = AtomicBool::new(false);
    let err = loader().send(&mut mock, ChipGeneration::Bcm2711, fw.path(), &cancel);
    assert!(
        err.expect_err("control")
            .contains("Control transfer failed")
    );
}

#[test]
fn fails_when_bulk_write_fails() {
    let fw = temp_dir();
    write_file(fw.path(), "bootcode4.bin", &[0xAB; 256]);
    let mut mock = MockTransport::new();
    mock.fail_bulk_write_after = Some(0);
    let cancel = AtomicBool::new(false);
    let err = loader().send(&mut mock, ChipGeneration::Bcm2711, fw.path(), &cancel);
    let err = err.expect_err("bulk");
    assert!(err.contains("Bulk write stalled"), "{err}");
    assert!(err.contains("(libusb error -1)"), "{err}");
}

#[test]
fn fails_for_empty_bootcode_file() {
    let fw = temp_dir();
    write_file(fw.path(), "bootcode4.bin", &[]);
    let mut mock = MockTransport::new();
    let cancel = AtomicBool::new(false);
    let err = loader().send(&mut mock, ChipGeneration::Bcm2711, fw.path(), &cancel);
    assert!(err.expect_err("empty").contains("empty"));
    assert!(control_outs(&mock).is_empty());
}

#[test]
fn chunks_large_payloads_and_reads_return_code() {
    let fw = temp_dir();
    write_file(fw.path(), "bootcode5.bin", &vec![1u8; 70_000]);
    let mut mock = MockTransport::new();
    let cancel = AtomicBool::new(false);
    assert!(
        loader()
            .send(&mut mock, ChipGeneration::Bcm2712, fw.path(), &cancel)
            .is_ok()
    );
    let outs = control_outs(&mock);
    assert_eq!((outs[1].0, outs[1].1), ((70_000 & 0xFFFF) as u16, 1));
    let writes = mock
        .calls
        .iter()
        .filter(|c| matches!(c, UsbCall::BulkWrite { .. }))
        .count();
    assert_eq!(writes, 1 + 5, "header plus ceil(70000 / 16384) chunks");
    assert!(matches!(
        mock.calls.last(),
        Some(UsbCall::ControlIn {
            request_type: 0xC0,
            len: 4,
            ..
        })
    ));
}
