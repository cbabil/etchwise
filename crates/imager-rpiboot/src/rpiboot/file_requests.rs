//! Answers to single file-server requests (upstream `FileServer::
//! handleGetFileSize`, `handleReadFile`, `parseMetadata`,
//! `readFileFromDisk`). Star-prefixed names carry device metadata.

use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use super::bootcode_loader::{error_code, split_length};
use crate::transport::UsbTransport;
use crate::types::{
    BULK_CHUNK_SIZE, DEFAULT_TIMEOUT_MS, DeviceMetadata, VENDOR_REQUEST, VENDOR_REQUEST_TYPE,
    parse_leading,
};

/// Looks up a requested file's bytes; empty means not found.
pub type FileResolver<'a> = &'a dyn Fn(&str) -> Vec<u8>;

fn is_metadata(name: &str) -> bool {
    name.starts_with('*')
}

/// Sends a zero-data `ep_write` header announcing `len`.
fn announce(transport: &mut dyn UsbTransport, len: u32) -> bool {
    let (value, index) = split_length(len);
    transport.control_out(
        VENDOR_REQUEST_TYPE,
        VENDOR_REQUEST,
        value,
        index,
        &[],
        DEFAULT_TIMEOUT_MS,
    )
}

/// Sends a zero-length response; failures are ignored, as upstream.
pub fn acknowledge(transport: &mut dyn UsbTransport) {
    let _ = announce(transport, 0);
}

/// Replies to `GetFileSize` with the size in wValue/wIndex (0 for metadata
/// and missing files).
///
/// # Errors
/// `Failed to send file size for: <name>`.
pub fn get_file_size(
    transport: &mut dyn UsbTransport,
    name: &str,
    resolver: FileResolver<'_>,
) -> Result<(), String> {
    let size = if is_metadata(name) {
        0
    } else {
        resolver(name).len() as u32
    };
    if announce(transport, size) {
        Ok(())
    } else {
        Err(format!("Failed to send file size for: {name}"))
    }
}

/// Replies to `ReadFile`: metadata and missing files get a zero-length
/// response; otherwise the size header then the data in bulk chunks.
///
/// # Errors
/// Header or bulk transfer failures, or `Cancelled`.
pub fn read_file(
    transport: &mut dyn UsbTransport,
    name: &str,
    resolver: FileResolver<'_>,
    metadata: &mut DeviceMetadata,
    cancel: &AtomicBool,
) -> Result<(), String> {
    if is_metadata(name) {
        parse_metadata(metadata, name, &[]);
        acknowledge(transport);
        return Ok(());
    }
    let data = resolver(name);
    if data.is_empty() {
        acknowledge(transport);
        return Ok(());
    }
    if !announce(transport, data.len() as u32) {
        return Err(format!("Failed to send ReadFile size header for: {name}"));
    }
    let endpoint = transport.out_endpoint();
    let mut offset = 0;
    while offset < data.len() {
        if cancel.load(Ordering::SeqCst) {
            return Err("Cancelled".into());
        }
        let end = data.len().min(offset + BULK_CHUNK_SIZE);
        let Some(sent) = transport.bulk_write(endpoint, &data[offset..end], DEFAULT_TIMEOUT_MS)
        else {
            return Err(format!(
                "Bulk write failed sending file: {name} at offset {offset} of {} (libusb error {})",
                data.len(),
                error_code(transport)
            ));
        };
        offset += sent;
    }
    Ok(())
}

/// Records `*key=value` metadata (`serial`, `mac`, `otp`, `board-rev` in
/// hex, or their camel-case aliases); unknown keys are ignored.
pub fn parse_metadata(metadata: &mut DeviceMetadata, name: &str, data: &[u8]) {
    let key = name.strip_prefix('*').unwrap_or(name);
    let (key, value) = match key.split_once('=') {
        Some((key, value)) => (key, value.to_owned()),
        None => (key, String::from_utf8_lossy(data).into_owned()),
    };
    match key {
        "serial" | "serialNumber" => metadata.serial_number = Some(value),
        "mac" | "macAddress" => metadata.mac_address = Some(value),
        "otp" | "otpState" => metadata.otp_state = Some(value),
        "board-rev" | "boardRevision" => {
            if let Some(rev) = parse_leading(&value, 16).map(|n| n as u32) {
                metadata.board_revision = Some(rev);
            }
        }
        _ => {}
    }
}

/// The default resolver: `<dir>/<name>`, empty for metadata, missing or
/// empty files. The name comes from the USB device, so only relative paths
/// that stay inside `dir` (after resolving symlinks) are served; upstream
/// opens `dir / name` unchecked.
pub fn read_file_from_disk(dir: &Path, name: &str) -> Vec<u8> {
    if is_metadata(name) {
        return Vec::new();
    }
    contained_path(dir, name)
        .and_then(|path| std::fs::read(path).ok())
        .unwrap_or_default()
}

/// `dir/name` when `name` is a non-empty relative path of plain components
/// whose real location is under `dir`.
fn contained_path(dir: &Path, name: &str) -> Option<PathBuf> {
    let relative = Path::new(name);
    let plain = relative
        .components()
        .all(|c| matches!(c, Component::Normal(_)));
    if name.is_empty() || !plain {
        return None;
    }
    let root = dir.canonicalize().ok()?;
    let path = root.join(relative).canonicalize().ok()?;
    path.starts_with(&root).then_some(path)
}
