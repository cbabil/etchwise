//! Stage-2 bootcode upload (upstream `bootcode_loader.cpp`): pick the
//! bootcode binary for the chip, then send the boot message and payload
//! with the rpiboot `ep_write` protocol (a zero-data vendor control
//! transfer announcing the length, then bulk OUT chunks).

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::transport::UsbTransport;
use crate::types::{
    BULK_CHUNK_SIZE, ChipGeneration, DEFAULT_TIMEOUT_MS, VENDOR_REQUEST, VENDOR_REQUEST_TYPE,
    boot_message,
};

/// Uploads the second-stage bootloader.
#[derive(Debug, Clone)]
pub struct BootcodeLoader {
    /// Pause before reading the device's return code (upstream `sleep(1)`).
    pub return_wait: Duration,
}

impl Default for BootcodeLoader {
    fn default() -> Self {
        Self {
            return_wait: Duration::from_secs(1),
        }
    }
}

/// The bootcode file name for a chip (`bootcodeFilename`).
pub fn bootcode_filename(chip: ChipGeneration) -> &'static str {
    match chip {
        ChipGeneration::Bcm2711 => "bootcode4.bin",
        ChipGeneration::Bcm2712 => "bootcode5.bin",
        ChipGeneration::Bcm2836_7 => "bootcode.bin",
    }
}

/// Splits a length into `(wValue, wIndex)`: low and high 16 bits.
pub fn split_length(len: u32) -> (u16, u16) {
    ((len & 0xFFFF) as u16, (len >> 16) as u16)
}

impl BootcodeLoader {
    /// Sends `<firmware_dir>/<bootcode file>` to the device.
    ///
    /// # Errors
    /// Upstream's `lastError()` text: missing or empty file, transport
    /// failures, or `Cancelled`.
    pub fn send(
        &self,
        transport: &mut dyn UsbTransport,
        chip: ChipGeneration,
        firmware_dir: &Path,
        cancel: &AtomicBool,
    ) -> Result<(), String> {
        let path = firmware_dir.join(bootcode_filename(chip));
        let bootcode = std::fs::read(&path)
            .map_err(|_| format!("Cannot open bootcode file: {}", path.display()))?;
        if bootcode.is_empty() {
            return Err(format!("Bootcode file is empty: {}", path.display()));
        }
        // Signature left zeroed: BCM2711/BCM2712 sign inside the bootcode.
        let message = boot_message(bootcode.len() as i32, [0; 20]);
        ep_write(transport, &message, cancel)
            .map_err(|e| format!("Failed to send BootMessage: {e}"))?;
        if cancel.load(Ordering::SeqCst) {
            return Err("Cancelled".into());
        }
        ep_write(transport, &bootcode, cancel)
            .map_err(|e| format!("Failed to send bootcode payload: {e}"))?;
        // Upstream second_stage_boot(): sleep(1) then ep_read() the return
        // code. The device may already have reset, so failure is ignored.
        std::thread::sleep(self.return_wait);
        let mut retcode = [0u8; 4];
        let _ = transport.control_in(
            VENDOR_REQUEST_TYPE | 0x80,
            VENDOR_REQUEST,
            retcode.len() as u16,
            0,
            &mut retcode,
            20_000,
        );
        Ok(())
    }
}

/// The `ep_write` protocol: announce the length, then bulk the data.
fn ep_write(
    transport: &mut dyn UsbTransport,
    data: &[u8],
    cancel: &AtomicBool,
) -> Result<(), String> {
    if cancel.load(Ordering::SeqCst) {
        return Err("Cancelled".into());
    }
    let (value, index) = split_length(data.len() as u32);
    if !transport.control_out(
        VENDOR_REQUEST_TYPE,
        VENDOR_REQUEST,
        value,
        index,
        &[],
        DEFAULT_TIMEOUT_MS,
    ) {
        return Err("Control transfer failed".into());
    }
    let endpoint = transport.out_endpoint();
    let mut offset = 0;
    while offset < data.len() {
        if cancel.load(Ordering::SeqCst) {
            return Err("Cancelled".into());
        }
        let end = data.len().min(offset + BULK_CHUNK_SIZE);
        match transport.bulk_write(endpoint, &data[offset..end], DEFAULT_TIMEOUT_MS) {
            Some(n) if n > 0 => offset += n,
            sent => {
                let code = sent.map_or_else(|| error_code(transport), |n| n as i32);
                return Err(format!(
                    "Bulk write stalled at offset {offset} of {} (libusb error {code})",
                    data.len()
                ));
            }
        }
    }
    Ok(())
}

/// The libusb code of the last failure, `-1` when the backend cannot say
/// (upstream's mock returned `-1` for every failure).
pub fn error_code(transport: &dyn UsbTransport) -> i32 {
    transport.last_error_code().unwrap_or(-1)
}
