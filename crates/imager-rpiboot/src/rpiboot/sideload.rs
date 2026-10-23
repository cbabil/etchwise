//! The Fastboot sideload flow of upstream `RpibootThread::run()`: fetch
//! firmware, upload bootcode, wait for the second stage, serve files while
//! polling for the fastboot gadget, and return the gadget's bus number and address.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use super::RpibootError;
use super::fetcher::{Fetcher, HttpFetcher};
use super::firmware_manager::FirmwareManager;
use super::libusb_context::LibusbContext;
use super::protocol::RpibootProtocol;
use super::scanner::RpibootDevice;
use super::usb_bus::{UsbBus, UsbDeviceInfo};
use crate::types::{BROADCOM_VID, ProgressCallback, SideloadMode};

/// Delays of the flow (upstream values by default; tests shrink them).
#[derive(Debug, Clone, Copy)]
pub struct SideloadTiming {
    /// USB polling interval (upstream 500 ms).
    pub poll_interval: Duration,
    /// Pause before reading the bootcode return code (upstream 1 s).
    pub bootcode_return_wait: Duration,
    /// Pause between failed file-server reads (upstream 1 s).
    pub file_server_retry_delay: Duration,
}

impl Default for SideloadTiming {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(500),
            bootcode_return_wait: Duration::from_secs(1),
            file_server_retry_delay: Duration::from_secs(1),
        }
    }
}

/// Runs the flow against injectable USB and network access.
pub struct Sideloader<'a> {
    pub(super) bus: &'a dyn UsbBus,
    fetcher: &'a dyn Fetcher,
    pub(super) timing: SideloadTiming,
}

/// Sideloads the fastboot gadget onto `device` with the real USB stack and
/// network, caching firmware under `firmware_root`; returns the fastboot
/// device's `(bus, address)`. Status-only updates arrive as `(0, 0, status)`.
///
/// # Errors
/// [`RpibootError`] with upstream's messages; `Cancelled` when `cancel`
/// was set.
pub fn sideload_fastboot(
    device: &RpibootDevice,
    firmware_root: &Path,
    progress: ProgressCallback<'_>,
    cancel: &AtomicBool,
) -> Result<(u8, u8), RpibootError> {
    let fetcher = HttpFetcher::new().map_err(|e| RpibootError::Firmware(e.to_string()))?;
    let bus = LibusbContext::new().map_err(RpibootError::Usb)?;
    Sideloader::new(&bus, &fetcher).run(device, firmware_root, progress, cancel)
}

pub(super) fn check_cancel(cancel: &AtomicBool) -> Result<(), RpibootError> {
    if cancel.load(Ordering::SeqCst) {
        Err(RpibootError::Cancelled)
    } else {
        Ok(())
    }
}

impl<'a> Sideloader<'a> {
    /// A sideloader with upstream's timing.
    pub fn new(bus: &'a dyn UsbBus, fetcher: &'a dyn Fetcher) -> Self {
        Self {
            bus,
            fetcher,
            timing: SideloadTiming::default(),
        }
    }

    /// Overrides the delays.
    pub fn with_timing(mut self, timing: SideloadTiming) -> Self {
        self.timing = timing;
        self
    }

    /// The whole flow; see [`sideload_fastboot`].
    ///
    /// # Errors
    /// As [`sideload_fastboot`].
    pub fn run(
        &self,
        device: &RpibootDevice,
        firmware_root: &Path,
        progress: ProgressCallback<'_>,
        cancel: &AtomicBool,
    ) -> Result<(u8, u8), RpibootError> {
        let chip = device.chip_generation;
        progress(0, 0, "Downloading firmware...");
        let fw_dir = FirmwareManager::new(firmware_root, self.fetcher)
            .ensure_available(SideloadMode::Fastboot, chip, &mut *progress, cancel)
            .map_err(RpibootError::Firmware)?;
        check_cancel(cancel)?;
        progress(0, 0, "Connecting to device...");
        let mut protocol = self.protocol();
        let mut target = self.locate(device);
        if target.needs_bootcode() {
            let mut transport = self.bus.open(&target).ok_or(RpibootError::OpenFailed)?;
            if !transport.is_open() {
                return Err(RpibootError::OpenFailed);
            }
            let uploaded =
                protocol.upload_bootcode(transport.as_mut(), chip, &fw_dir, &mut *progress, cancel);
            drop(transport);
            uploaded.map_err(|e| protocol_error(e, cancel))?;
            check_cancel(cancel)?;
            progress(0, 0, "Waiting for device to restart...");
            target = self.wait_for_reenum(device, &mut *progress, cancel)?;
        }
        check_cancel(cancel)?;
        self.serve_and_wait(&mut protocol, device, &target, &fw_dir, progress, cancel)
    }

    fn protocol(&self) -> RpibootProtocol {
        let mut protocol = RpibootProtocol::default();
        protocol.bootcode_loader.return_wait = self.timing.bootcode_return_wait;
        protocol.file_server.retry_delay = self.timing.file_server_retry_delay;
        protocol
    }

    /// The device's current state: matched by port path (or bus and
    /// address), keeping the requested chip; the stored fields otherwise.
    fn locate(&self, device: &RpibootDevice) -> UsbDeviceInfo {
        let fallback = UsbDeviceInfo {
            bus_number: device.bus_number,
            device_address: device.device_address,
            vendor_id: BROADCOM_VID,
            product_id: device.product_id,
            port_path: device.port_path.clone(),
            chip_generation: device.chip_generation,
            serial_number_index: 0,
        };
        let matches = |d: &UsbDeviceInfo| {
            if device.port_path.is_empty() {
                d.bus_number == device.bus_number && d.device_address == device.device_address
            } else {
                d.port_path == device.port_path
            }
        };
        self.bus
            .scan_boot_devices()
            .into_iter()
            .find(matches)
            .map_or(fallback, |d| UsbDeviceInfo {
                chip_generation: device.chip_generation,
                ..d
            })
    }
}

/// Upstream reports protocol failures unless the user cancelled.
pub(super) fn protocol_error(error: String, cancel: &AtomicBool) -> RpibootError {
    if cancel.load(Ordering::SeqCst) {
        RpibootError::Cancelled
    } else {
        RpibootError::Protocol(error)
    }
}
