//! Upstream `rpiboot_protocol_test.cpp`: file-server transport failures,
//! plus the disconnect and garbage paths upstream does not test.

use std::sync::atomic::AtomicBool;

use super::test_support::{
    DONE, GET_FILE_SIZE, READ_FILE, file_message, mock_with, quiet, temp_dir, write_file,
};
use super::tests_file_server::server;
use crate::transport::{MockTransport, UsbTransport};

/// A mock that reports a libusb error code for its failures.
struct CodedMock {
    inner: MockTransport,
    code: i32,
}

impl UsbTransport for CodedMock {
    fn control_out(&mut self, rt: u8, rq: u8, v: u16, i: u16, d: &[u8], t: u32) -> bool {
        self.inner.control_out(rt, rq, v, i, d, t)
    }
    fn control_in(
        &mut self,
        rt: u8,
        rq: u8,
        v: u16,
        i: u16,
        b: &mut [u8],
        t: u32,
    ) -> Option<usize> {
        self.inner.control_in(rt, rq, v, i, b, t)
    }
    fn bulk_write(&mut self, ep: u8, d: &[u8], t: u32) -> Option<usize> {
        self.inner.bulk_write(ep, d, t)
    }
    fn bulk_read(&mut self, ep: u8, b: &mut [u8], t: u32) -> Option<usize> {
        self.inner.bulk_read(ep, b, t)
    }
    fn is_open(&self) -> bool {
        self.inner.is_open()
    }
    fn last_error_code(&self) -> Option<i32> {
        Some(self.code)
    }
}

fn run(transport: &mut dyn UsbTransport, dir: &std::path::Path) -> Result<(), String> {
    server().run(transport, dir, &mut quiet, &AtomicBool::new(false), None)
}

#[test]
fn fails_when_control_transfer_fails_on_get_file_size() {
    let fw = temp_dir();
    write_file(fw.path(), "test.bin", b"data");
    let mut mock = mock_with(vec![file_message(GET_FILE_SIZE, "test.bin")]);
    mock.control_out_ok = false;
    let err = run(&mut mock, fw.path()).expect_err("must fail");
    assert!(err.contains("Failed to send file size"), "{err}");
}

#[test]
fn fails_when_bulk_write_fails_on_read_file() {
    let fw = temp_dir();
    write_file(fw.path(), "test.bin", b"some data to send");
    let mut mock = mock_with(vec![file_message(READ_FILE, "test.bin")]);
    mock.fail_bulk_write_after = Some(0);
    let err = run(&mut mock, fw.path()).expect_err("must fail");
    assert!(err.contains("Bulk write failed"), "{err}");
}

#[test]
fn returns_false_on_short_read() {
    // Upstream's mock returns -1 here, which its server treats as
    // LIBUSB_ERROR_IO ("Device disconnected"); an unknown failure code is
    // retried instead, giving the message upstream's test expects.
    let mut mock = MockTransport::new();
    let err = run(&mut mock, &std::env::temp_dir()).expect_err("must fail");
    assert!(err.contains("Failed to read FileMessage"), "{err}");
    assert!(err.contains("after 21 retries"), "{err}");
}

#[test]
fn fails_when_control_transfer_fails_on_read_file_size_header() {
    let fw = temp_dir();
    write_file(fw.path(), "test.bin", b"content");
    let mut mock = mock_with(vec![file_message(READ_FILE, "test.bin")]);
    mock.control_out_ok = false;
    let err = run(&mut mock, fw.path()).expect_err("must fail");
    assert!(err.contains("Failed to send ReadFile size header"), "{err}");
}

#[test]
fn no_device_before_serving_is_an_error() {
    let mut coded = CodedMock {
        inner: MockTransport::new(),
        code: -4,
    };
    let err = run(&mut coded, &std::env::temp_dir()).expect_err("must fail");
    assert_eq!(err, "Device disconnected (libusb error -4)");
}

#[test]
fn disconnect_after_serving_counts_as_reboot() {
    let fw = temp_dir();
    write_file(fw.path(), "start.elf", b"elf");
    let mut coded = CodedMock {
        inner: mock_with(vec![file_message(READ_FILE, "start.elf")]),
        code: -1,
    };
    let mut seen = Vec::new();
    let mut record = |c: u64, t: u64, s: &str| seen.push((c, t, s.to_owned()));
    let cancel = AtomicBool::new(false);
    assert!(
        server()
            .run(&mut coded, fw.path(), &mut record, &cancel, None)
            .is_ok()
    );
    assert_eq!(
        seen.last(),
        Some(&(1, 1, "Device rebooted into next stage".to_owned()))
    );
}

#[test]
fn garbage_messages_are_retried_then_rejected() {
    let mut replies = vec![file_message(9, "x"); 21];
    replies.push(file_message(DONE, ""));
    let mut mock = mock_with(replies);
    let err = run(&mut mock, &std::env::temp_dir()).expect_err("must fail");
    assert_eq!(err, "Unknown file command 9");
}

#[test]
fn garbage_names_do_not_reset_the_retry_count() {
    let mut replies = vec![file_message(READ_FILE, "bad\u{1}name"); 3];
    replies.push(file_message(DONE, ""));
    let mut mock = mock_with(replies);
    assert_eq!(run(&mut mock, &std::env::temp_dir()), Ok(()));
}

#[test]
fn never_serves_files_outside_the_firmware_directory() {
    use super::file_requests::read_file_from_disk;
    let root = temp_dir();
    write_file(root.path(), "secret.txt", b"private");
    let fw = root.path().join("fw");
    std::fs::create_dir(&fw).unwrap();
    write_file(&fw, "start4.elf", b"elf");
    std::fs::create_dir(fw.join("sub")).unwrap();
    write_file(&fw.join("sub"), "a.dtb", b"dtb");
    let outside = root.path().join("secret.txt");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, fw.join("link.txt")).unwrap();

    assert_eq!(read_file_from_disk(&fw, "start4.elf"), b"elf");
    assert_eq!(read_file_from_disk(&fw, "sub/a.dtb"), b"dtb");
    for name in [
        "../secret.txt",
        "sub/../../secret.txt",
        outside.to_str().unwrap(),
        "link.txt",
        "",
    ] {
        assert!(read_file_from_disk(&fw, name).is_empty(), "{name}");
    }
}
