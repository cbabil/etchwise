//! Higher-level fastboot commands built on the wire protocol (upstream
//! `FastbootProtocol::flash`, `getVar`, `flashImage`, `writeDeviceFile`,
//! `readDeviceFile`).

use super::protocol::{Cancel, FastbootError, ResponseType, download, send_command, stage, upload};
use crate::transport::UsbTransport;
use crate::types::ProgressCallback;

/// Writes the downloaded data to `partition` (`flash:<partition>`).
///
/// # Errors
/// "Flash failed: <device message>".
pub fn flash<T: UsbTransport + ?Sized>(
    t: &mut T,
    partition: &str,
    timeout_ms: u32,
) -> Result<(), FastbootError> {
    let resp = send_command(t, &format!("flash:{partition}"), timeout_ms);
    if resp.kind != ResponseType::Okay {
        return Err(FastbootError(format!("Flash failed: {}", resp.message)));
    }
    Ok(())
}

/// `getvar:<name>`; `None` unless the device answers OKAY.
pub fn get_var<T: UsbTransport + ?Sized>(t: &mut T, name: &str) -> Option<String> {
    let resp = send_command(t, &format!("getvar:{name}"), 3000);
    (resp.kind == ResponseType::Okay).then_some(resp.message)
}

/// Download then flash in one call.
///
/// # Errors
/// The failing step's message.
pub fn flash_image<T: UsbTransport + ?Sized>(
    t: &mut T,
    partition: &str,
    data: &[u8],
    progress: Option<ProgressCallback<'_>>,
    cancelled: Cancel<'_>,
) -> Result<(), FastbootError> {
    download(t, data, progress, cancelled)?;
    flash(t, partition, 60_000)
}

/// Writes a file on the device: stage, then `oem download-file <path>`.
///
/// # Errors
/// The stage failure, or "oem download-file failed for <path>: <msg>".
pub fn write_device_file<T: UsbTransport + ?Sized>(
    t: &mut T,
    device_path: &str,
    data: &[u8],
    cancelled: Cancel<'_>,
) -> Result<(), FastbootError> {
    stage(t, data, None, cancelled)?;
    let resp = send_command(t, &format!("oem download-file {device_path}"), 30_000);
    if resp.kind != ResponseType::Okay {
        let msg = resp.message;
        return Err(FastbootError(format!(
            "oem download-file failed for {device_path}: {msg}"
        )));
    }
    Ok(())
}

/// Reads a file from the device: `oem upload-file <path>`, then upload.
///
/// # Errors
/// "oem upload-file failed for <path>: <msg>", or the upload failure.
pub fn read_device_file<T: UsbTransport + ?Sized>(
    t: &mut T,
    device_path: &str,
    cancelled: Cancel<'_>,
) -> Result<Vec<u8>, FastbootError> {
    let resp = send_command(t, &format!("oem upload-file {device_path}"), 30_000);
    if resp.kind != ResponseType::Okay {
        let msg = resp.message;
        return Err(FastbootError(format!(
            "oem upload-file failed for {device_path}: {msg}"
        )));
    }
    upload(t, None, cancelled)
}
