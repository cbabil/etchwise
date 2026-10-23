//! The waiting halves of the sideload flow (upstream `RpibootThread::
//! waitForBootDeviceReEnum`, `pollForFastbootDevice`, and the file-server
//! phase that runs while the fastboot scanner polls in the background).

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use super::RpibootError;
use super::protocol::RpibootProtocol;
use super::scanner::RpibootDevice;
use super::sideload::{Sideloader, check_cancel, protocol_error};
use super::usb_bus::UsbDeviceInfo;
use crate::types::{ProgressCallback, SideloadMode};

/// Disconnect polls (3 s at 500 ms).
const DISCONNECT_POLLS: u32 = 6;
/// Reconnect polls (15 s at 500 ms).
const RECONNECT_POLLS: u32 = 30;
/// Fastboot polls (60 s: the gadget may take 20 s+ to boot).
const FASTBOOT_POLLS: u32 = 120;

/// Same physical port; bus and address without a port path; anything when
/// neither is known.
fn matches_port(device: &RpibootDevice, candidate: &UsbDeviceInfo) -> bool {
    if !device.port_path.is_empty() {
        candidate.port_path == device.port_path
    } else if device.bus_number != 0 {
        candidate.bus_number == device.bus_number
            && candidate.device_address == device.device_address
    } else {
        true
    }
}

impl Sideloader<'_> {
    /// Waits for the boot ROM device to leave, then for the second stage
    /// (`iSerialNumber` > 3) on the same port.
    pub(super) fn wait_for_reenum(
        &self,
        device: &RpibootDevice,
        progress: ProgressCallback<'_>,
        cancel: &AtomicBool,
    ) -> Result<UsbDeviceInfo, RpibootError> {
        for i in 1..=DISCONNECT_POLLS {
            check_cancel(cancel)?;
            let status = format!("Waiting for device to disconnect ({i}/{DISCONNECT_POLLS})...");
            progress(0, 0, &status);
            std::thread::sleep(self.timing.poll_interval);
            let still_present = self
                .bus
                .scan_boot_devices()
                .iter()
                .any(|d| matches_port(device, d) && d.needs_bootcode());
            if !still_present {
                progress(0, 0, "Device disconnected, waiting for reconnect...");
                break;
            }
        }
        check_cancel(cancel)?;
        for i in 1..=RECONNECT_POLLS {
            check_cancel(cancel)?;
            let status = format!(
                "Waiting for device to reconnect ({}/{}s)...",
                i / 2,
                RECONNECT_POLLS / 2
            );
            progress(0, 0, &status);
            let found = self
                .bus
                .scan_boot_devices()
                .into_iter()
                .find(|d| matches_port(device, d) && !d.needs_bootcode());
            if let Some(found) = found {
                return Ok(found);
            }
            std::thread::sleep(self.timing.poll_interval);
        }
        Err(RpibootError::ReenumTimeout(RECONNECT_POLLS / 2))
    }

    /// Serves files while a background scanner polls for the gadget, which
    /// reboots into fastboot without sending Done: once it appears, the
    /// file server is stopped at its next progress report.
    pub(super) fn serve_and_wait(
        &self,
        protocol: &mut RpibootProtocol,
        device: &RpibootDevice,
        target: &UsbDeviceInfo,
        fw_dir: &Path,
        progress: ProgressCallback<'_>,
        cancel: &AtomicBool,
    ) -> Result<(u8, u8), RpibootError> {
        let found = AtomicBool::new(false);
        let stop = AtomicBool::new(false);
        std::thread::scope(|scope| {
            let scanner = scope.spawn(|| self.poll_for_fastboot(device, &found, &stop, cancel));
            // Serve as the requested chip: the second stage's PID may differ.
            let target = UsbDeviceInfo {
                chip_generation: device.chip_generation,
                ..target.clone()
            };
            let served = self.serve(protocol, &target, fw_dir, &found, &mut *progress, cancel);
            let served = served.and_then(|()| check_cancel(cancel));
            if let Err(e) = served {
                stop.store(true, Ordering::SeqCst);
                return Err(e);
            }
            if !found.load(Ordering::SeqCst) {
                progress(0, 0, "Waiting for fastboot device...");
            }
            match scanner.join().ok().flatten() {
                Some(id) => Ok(id),
                None if cancel.load(Ordering::SeqCst) => Err(RpibootError::Cancelled),
                None => Err(RpibootError::FastbootTimeout),
            }
        })
    }

    fn serve(
        &self,
        protocol: &mut RpibootProtocol,
        target: &UsbDeviceInfo,
        fw_dir: &Path,
        found: &AtomicBool,
        progress: ProgressCallback<'_>,
        cancel: &AtomicBool,
    ) -> Result<(), RpibootError> {
        let mut transport = match self.bus.open(target) {
            Some(t) if t.is_open() => t,
            _ => return Err(RpibootError::OpenAfterReenumFailed),
        };
        let combined = AtomicBool::new(false);
        let mut wrapped = |current: u64, total: u64, status: &str| {
            if found.load(Ordering::SeqCst) || cancel.load(Ordering::SeqCst) {
                combined.store(true, Ordering::SeqCst);
            }
            progress(current, total, status);
        };
        let result = protocol.serve_files(
            transport.as_mut(),
            target.chip_generation,
            SideloadMode::Fastboot,
            fw_dir,
            &mut wrapped,
            &combined,
        );
        if found.load(Ordering::SeqCst) {
            return Ok(());
        }
        result.map_err(|e| protocol_error(e, cancel))
    }

    /// Polls for the fastboot gadget on the same port (or the only one
    /// present when the port path is unknown); returns its `(bus, address)`.
    fn poll_for_fastboot(
        &self,
        device: &RpibootDevice,
        found: &AtomicBool,
        stop: &AtomicBool,
        cancel: &AtomicBool,
    ) -> Option<(u8, u8)> {
        for _ in 0..FASTBOOT_POLLS {
            if cancel.load(Ordering::SeqCst) || stop.load(Ordering::SeqCst) {
                return None;
            }
            std::thread::sleep(self.timing.poll_interval);
            let devices = self.bus.scan_fastboot_devices();
            let hit = if device.port_path.is_empty() {
                devices.first().filter(|_| devices.len() == 1)
            } else {
                devices.iter().find(|d| d.port_path == device.port_path)
            };
            if let Some(dev) = hit {
                found.store(true, Ordering::SeqCst);
                return Some((dev.bus_number, dev.device_address));
            }
        }
        None
    }
}
