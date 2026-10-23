//! Shared helpers for the rpiboot tests (upstream `TempFirmwareDir`,
//! `makeFileMessage`, `createTarInMemory`).

use std::path::Path;

use crate::transport::{MockTransport, UsbCall};
use crate::types::FileMessage;

/// Writes `data` to `<dir>/<name>`, creating parent directories.
pub fn write_file(dir: &Path, name: &str, data: &[u8]) {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create dirs");
    }
    std::fs::write(path, data).expect("write file");
}

/// A fresh temporary firmware directory.
pub fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("temp dir")
}

/// Wire bytes of a `FileMessage`.
pub fn file_message(command: i32, name: &str) -> Vec<u8> {
    let mut bytes = vec![0u8; FileMessage::SIZE];
    bytes[..4].copy_from_slice(&command.to_le_bytes());
    let n = name.len().min(255);
    bytes[4..4 + n].copy_from_slice(&name.as_bytes()[..n]);
    bytes
}

/// `FileCommand` wire values.
pub const GET_FILE_SIZE: i32 = 0;
pub const READ_FILE: i32 = 1;
pub const DONE: i32 = 2;

/// A mock whose control IN reads return `replies` in order.
pub fn mock_with(replies: Vec<Vec<u8>>) -> MockTransport {
    let mut mock = MockTransport::new();
    mock.control_in_replies = replies.into_iter().map(Some).collect();
    mock
}

/// Recorded control OUT transfers as `(value, index, data)`.
pub fn control_outs(mock: &MockTransport) -> Vec<(u16, u16, Vec<u8>)> {
    mock.calls
        .iter()
        .filter_map(|c| match c {
            UsbCall::ControlOut {
                value, index, data, ..
            } => Some((*value, *index, data.clone())),
            _ => None,
        })
        .collect()
}

/// Total bytes of recorded bulk writes.
pub fn bulk_bytes(mock: &MockTransport) -> usize {
    mock.calls
        .iter()
        .map(|c| match c {
            UsbCall::BulkWrite { data, .. } => data.len(),
            _ => 0,
        })
        .sum()
}

/// A progress callback that ignores everything.
pub fn quiet(_: u64, _: u64, _: &str) {}

/// An in-memory ustar archive of regular files, names kept verbatim.
pub fn tar_of(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    for (name, data) in files {
        let mut header = [0u8; 512];
        header[..name.len()].copy_from_slice(name.as_bytes());
        header[100..108].copy_from_slice(b"0000644\0");
        header[108..116].copy_from_slice(b"0000000\0");
        header[116..124].copy_from_slice(b"0000000\0");
        header[124..136].copy_from_slice(format!("{:011o}\0", data.len()).as_bytes());
        header[136..148].copy_from_slice(b"00000000000\0");
        header[156] = b'0';
        header[257..263].copy_from_slice(b"ustar\0");
        header[263..265].copy_from_slice(b"00");
        header[148..156].copy_from_slice(b"        ");
        let sum: u32 = header.iter().map(|b| u32::from(*b)).sum();
        header[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
        out.extend_from_slice(&header);
        out.extend_from_slice(data);
        out.resize(out.len().div_ceil(512) * 512, 0);
    }
    out.resize(out.len() + 1024, 0);
    out
}
