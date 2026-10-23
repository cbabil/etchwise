//! Fastboot data phases (upstream `FastbootProtocol::sendData`,
//! `download`, `stage`, `upload`): `download:`/`stage:` with the payload
//! in bulk chunks, and `upload` reading a DATA-announced payload back.
//!
//! As upstream, these write the command with a raw bulk write, so INFO/TEXT
//! before DATA is not handled (the Pi gadget never sends any there).

use super::protocol::{Cancel, FastbootError};
use super::protocol::{EP_IN, EP_OUT, ResponseType, read_response};
use crate::transport::UsbTransport;
use crate::types::{BULK_CHUNK_SIZE, ProgressCallback};

type Outcome<T> = Result<T, FastbootError>;

fn fail<T>(message: String) -> Outcome<T> {
    Err(FastbootError(message))
}

/// `<prefix>:<size-hex>`, DATA, the payload in bulk chunks, then OKAY.
fn send_data<T: UsbTransport + ?Sized>(
    t: &mut T,
    prefix: &str,
    data: &[u8],
    mut progress: Option<ProgressCallback<'_>>,
    cancelled: Cancel<'_>,
) -> Outcome<()> {
    let cmd = format!("{prefix}:{:08x}", data.len());
    if t.bulk_write(EP_OUT, cmd.as_bytes(), 3000).is_none() {
        return fail(format!("Failed to send {prefix} command"));
    }
    let resp = read_response(t, 10_000);
    if resp.kind != ResponseType::Data {
        return fail(format!("Expected DATA response, got: {}", resp.message));
    }
    let mut offset = 0;
    while offset < data.len() {
        if cancelled() {
            return fail("Cancelled".into());
        }
        let end = (offset + BULK_CHUNK_SIZE).min(data.len());
        let Some(sent) = t.bulk_write(EP_OUT, &data[offset..end], 5000) else {
            return fail(format!(
                "Bulk write failed during {prefix} at offset {offset}"
            ));
        };
        offset += sent;
        if let Some(cb) = progress.as_deref_mut() {
            cb(
                offset as u64,
                data.len() as u64,
                "Transferring to device...",
            );
        }
    }
    let done = read_response(t, 30_000);
    if done.kind != ResponseType::Okay {
        return fail(format!("{prefix} failed: {}", done.message));
    }
    Ok(())
}

/// Downloads `data` for a following `flash:`.
///
/// # Errors
/// Transport failures, a non-DATA reply, cancellation or a final FAIL.
pub fn download<T: UsbTransport + ?Sized>(
    t: &mut T,
    data: &[u8],
    progress: Option<ProgressCallback<'_>>,
    cancelled: Cancel<'_>,
) -> Outcome<()> {
    send_data(t, "download", data, progress, cancelled)
}

/// Stages `data` for an `oem` command (same wire format, `stage:` prefix).
///
/// # Errors
/// As [`download`].
pub fn stage<T: UsbTransport + ?Sized>(
    t: &mut T,
    data: &[u8],
    progress: Option<ProgressCallback<'_>>,
    cancelled: Cancel<'_>,
) -> Outcome<()> {
    send_data(t, "stage", data, progress, cancelled)
}

/// Reads previously prepared data back from the device (`upload`).
///
/// # Errors
/// Transport failures, a non-DATA reply, cancellation or a final FAIL.
pub fn upload<T: UsbTransport + ?Sized>(
    t: &mut T,
    mut progress: Option<ProgressCallback<'_>>,
    cancelled: Cancel<'_>,
) -> Outcome<Vec<u8>> {
    if t.bulk_write(EP_OUT, b"upload", 3000).is_none() {
        return fail("Failed to send upload command".into());
    }
    let resp = read_response(t, 10_000);
    if resp.kind != ResponseType::Data {
        let msg = resp.message;
        return fail(format!("Expected DATA response for upload, got: {msg}"));
    }
    let total = resp.data_size as usize;
    let mut result = Vec::with_capacity(total);
    let mut buf = vec![0u8; BULK_CHUNK_SIZE];
    while result.len() < total {
        if cancelled() {
            return fail("Cancelled".into());
        }
        let want = (total - result.len()).min(BULK_CHUNK_SIZE);
        match t.bulk_read(EP_IN, &mut buf[..want], 5000) {
            Some(n) if n > 0 => result.extend_from_slice(&buf[..n]),
            _ => {
                let at = result.len();
                return fail(format!("Bulk read failed during upload at offset {at}"));
            }
        }
        if let Some(cb) = progress.as_deref_mut() {
            cb(
                result.len() as u64,
                total as u64,
                "Uploading from device...",
            );
        }
    }
    let done = read_response(t, 10_000);
    if done.kind != ResponseType::Okay {
        return fail(format!("Upload failed: {}", done.message));
    }
    Ok(result)
}
