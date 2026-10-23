//! The Android fastboot USB protocol (upstream `fastboot_protocol.{h,cpp}`):
//! ASCII commands on bulk OUT, `OKAY`/`FAIL`/`DATA`/`INFO`/`TEXT` replies
//! on bulk IN, data phases for `download:`/`stage:`/`upload`, and the Pi
//! gadget's `oem download-file`/`oem upload-file` file transfer.
//!
//! Upstream returns `bool` plus `lastError()`; here failures carry the same
//! message in [`FastbootError`].

use crate::transport::UsbTransport;
use crate::types::parse_leading;

/// Fastboot bulk OUT endpoint (upstream `EP_OUT`).
pub const EP_OUT: u8 = 0x01;
/// Fastboot bulk IN endpoint (upstream `EP_IN`).
pub const EP_IN: u8 = 0x81;
/// Largest response packet.
const RESPONSE_MAX: usize = 256;

/// Reply kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseType {
    Okay,
    Fail,
    Data,
    Info,
    Text,
}

/// One parsed reply; `data_size` is only meaningful for `Data`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub kind: ResponseType,
    pub message: String,
    pub data_size: u32,
}

impl Response {
    fn fail(message: impl Into<String>) -> Self {
        Self {
            kind: ResponseType::Fail,
            message: message.into(),
            data_size: 0,
        }
    }
}

/// A failed fastboot operation, with upstream's `lastError()` text.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct FastbootError(pub String);

/// Polled between chunks; true aborts with "Cancelled".
pub type Cancel<'a> = &'a dyn Fn() -> bool;

/// Reads and classifies one reply packet (`readResponse`).
pub fn read_response<T: UsbTransport + ?Sized>(t: &mut T, timeout_ms: u32) -> Response {
    let mut buf = [0u8; RESPONSE_MAX];
    let n = match t.bulk_read(EP_IN, &mut buf, timeout_ms) {
        Some(n) if n >= 4 => n,
        _ => return Response::fail("Short or failed read from device"),
    };
    let (prefix, rest) = buf[..n].split_at(4);
    let message = String::from_utf8_lossy(rest).into_owned();
    let kind = match prefix {
        b"OKAY" => ResponseType::Okay,
        b"FAIL" => ResponseType::Fail,
        b"DATA" => ResponseType::Data,
        b"INFO" => ResponseType::Info,
        b"TEXT" => ResponseType::Text,
        _ => {
            let prefix = String::from_utf8_lossy(prefix);
            return Response::fail(format!("Unknown response prefix: {prefix}"));
        }
    };
    // Upstream's stoul would throw on a non-hex size; treat that as 0.
    let data_size = match (kind, rest.get(..8)) {
        (ResponseType::Data, Some(hex)) => {
            parse_leading(&String::from_utf8_lossy(hex), 16).map_or(0, |v| v as u32)
        }
        _ => 0,
    };
    Response {
        kind,
        message,
        data_size,
    }
}

/// Sends `command` and reads replies until one that is not INFO/TEXT.
pub fn send_command<T: UsbTransport + ?Sized>(
    t: &mut T,
    command: &str,
    timeout_ms: u32,
) -> Response {
    if t.bulk_write(EP_OUT, command.as_bytes(), timeout_ms)
        .is_none()
    {
        return Response::fail("Failed to send command");
    }
    loop {
        let resp = read_response(t, timeout_ms);
        if !matches!(resp.kind, ResponseType::Info | ResponseType::Text) {
            return resp;
        }
    }
}

pub use super::commands::{flash, flash_image, get_var, read_device_file, write_device_file};
pub use super::transfer::{download, stage, upload};
