//! Helpers shared by the fastboot tests (upstream `makeResponse`,
//! `queueReadFile`, `queueWriteFile`, `capturedBulkWrites`).

use crate::transport::{MockTransport, UsbCall};

/// `makeResponse(prefix, message)`.
pub fn response(prefix: &str, message: &str) -> Vec<u8> {
    format!("{prefix}{message}").into_bytes()
}

/// `%08x` of a length.
pub fn hex_size(len: usize) -> String {
    format!("{len:08x}")
}

/// `queueBulkReadResponse`.
pub fn queue(mock: &mut MockTransport, reply: Vec<u8>) {
    mock.bulk_read_replies.push_back(Some(reply));
}

/// `queueBulkReadResponse(makeResponse(prefix, message))`.
pub fn queue_resp(mock: &mut MockTransport, prefix: &str, message: &str) {
    queue(mock, response(prefix, message));
}

/// Replies for `read_device_file`: oem OKAY, DATA, payload, OKAY.
pub fn queue_read_file(mock: &mut MockTransport, content: &[u8]) {
    queue_resp(mock, "OKAY", "");
    queue_resp(mock, "DATA", &hex_size(content.len()));
    queue(mock, content.to_vec());
    queue_resp(mock, "OKAY", "");
}

/// Replies for `write_device_file`: stage DATA + OKAY, oem OKAY.
pub fn queue_write_file(mock: &mut MockTransport, len: usize) {
    queue_resp(mock, "DATA", &hex_size(len));
    queue_resp(mock, "OKAY", "");
    queue_resp(mock, "OKAY", "");
}

/// `failNextBulkWrites(1)`: the very next bulk write fails.
pub fn fail_next_write(mock: &mut MockTransport) {
    let done = bulk_writes(mock).len();
    mock.fail_bulk_write_after = Some(done);
}

/// `capturedBulkWrites()`.
pub fn bulk_writes(mock: &MockTransport) -> Vec<Vec<u8>> {
    mock.calls
        .iter()
        .filter_map(|c| match c {
            UsbCall::BulkWrite { data, .. } => Some(data.clone()),
            _ => None,
        })
        .collect()
}

/// Bulk writes as (lossy) strings.
pub fn written_strings(mock: &MockTransport) -> Vec<String> {
    bulk_writes(mock)
        .iter()
        .map(|w| String::from_utf8_lossy(w).into_owned())
        .collect()
}

/// A cancel flag that never fires.
pub fn never() -> bool {
    false
}

/// A cancel flag that is already set.
pub fn always() -> bool {
    true
}
