//! The rpiboot orchestrator (upstream `rpiboot_protocol.cpp`): bootcode
//! upload, then the file server fed from `bootfiles.bin` and the sideload
//! directory on disk.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use super::bootcode_loader::BootcodeLoader;
use super::bootfiles::Bootfiles;
use super::file_requests::read_file_from_disk;
use super::file_server::FileServer;
use crate::transport::UsbTransport;
use crate::types::{ChipGeneration, ProgressCallback, SideloadMode};

/// Drives a device from USB boot mode to the sideloaded payload.
#[derive(Debug, Clone, Default)]
pub struct RpibootProtocol {
    /// Stage-2 upload.
    pub bootcode_loader: BootcodeLoader,
    /// File-server phase.
    pub file_server: FileServer,
    bootfiles: Bootfiles,
}

/// The directory holding the sideload payload: `fastboot/` when present,
/// otherwise the firmware directory itself.
pub fn resolve_sideload_dir(firmware_dir: &Path, mode: SideloadMode) -> PathBuf {
    match mode {
        SideloadMode::Fastboot => {
            let dir = firmware_dir.join("fastboot");
            if dir.exists() {
                return dir;
            }
        }
    }
    firmware_dir.to_path_buf()
}

impl RpibootProtocol {
    #[cfg(test)]
    /// Metadata collected by the file server.
    pub fn metadata(&self) -> &crate::types::DeviceMetadata {
        self.file_server.metadata()
    }

    /// Uploads the second-stage bootloader. The device then re-enumerates;
    /// the caller must open a new transport for [`Self::serve_files`].
    ///
    /// # Errors
    /// `Bootcode upload failed: ...`.
    pub fn upload_bootcode(
        &mut self,
        transport: &mut dyn UsbTransport,
        chip: ChipGeneration,
        firmware_dir: &Path,
        progress: ProgressCallback<'_>,
        cancel: &AtomicBool,
    ) -> Result<(), String> {
        progress(0, 3, "Uploading bootloader...");
        self.bootcode_loader
            .send(transport, chip, firmware_dir, cancel)
            .map_err(|e| format!("Bootcode upload failed: {e}"))
    }

    /// Runs the file server on the second-stage device, serving from the
    /// extracted `bootfiles.bin` (chip-prefixed lookup) before disk.
    ///
    /// # Errors
    /// `Failed to extract bootfiles.bin: ...` or `File server failed: ...`.
    pub fn serve_files(
        &mut self,
        transport: &mut dyn UsbTransport,
        chip: ChipGeneration,
        mode: SideloadMode,
        firmware_dir: &Path,
        progress: ProgressCallback<'_>,
        cancel: &AtomicBool,
    ) -> Result<(), String> {
        progress(2, 3, "Loading firmware...");
        let sideload_dir = resolve_sideload_dir(firmware_dir, mode);
        let tar = sideload_dir.join("bootfiles.bin");
        let have_bootfiles = tar.exists();
        if have_bootfiles {
            self.bootfiles
                .extract_from_file(&tar)
                .map_err(|e| format!("Failed to extract bootfiles.bin: {e}"))?;
        }
        let bootfiles = &self.bootfiles;
        let from_tar = |name: &str| match bootfiles.find(name, chip.directory_prefix()) {
            Some(data) => data.to_vec(),
            None => read_file_from_disk(&sideload_dir, name),
        };
        let resolver = have_bootfiles.then_some(&from_tar as &dyn Fn(&str) -> Vec<u8>);
        self.file_server
            .run(transport, &sideload_dir, &mut *progress, cancel, resolver)
            .map_err(|e| format!("File server failed: {e}"))?;
        progress(3, 3, "Device boot complete");
        Ok(())
    }

    #[cfg(test)]
    /// The whole sequence on one transport, checking re-enumeration ten
    /// times every `reenum_poll` (upstream 200 ms). It does not reconnect,
    /// so it only suits transports that stay valid (tests); real Fastboot
    /// sideloading reconnects between the two phases.
    ///
    /// # Errors
    /// Either phase's error, `Cancelled`, or `Device did not re-enumerate
    /// after bootcode upload`.
    pub fn execute(
        &mut self,
        transport: &mut dyn UsbTransport,
        chip: ChipGeneration,
        firmware_dir: &Path,
        progress: ProgressCallback<'_>,
        cancel: &AtomicBool,
        reenum_poll: std::time::Duration,
    ) -> Result<(), String> {
        self.upload_bootcode(transport, chip, firmware_dir, &mut *progress, cancel)?;
        let cancelled = || cancel.load(std::sync::atomic::Ordering::SeqCst);
        if cancelled() {
            return Err("Cancelled".into());
        }
        progress(1, 3, "Waiting for device to restart...");
        for _ in 0..10 {
            if cancelled() {
                break;
            }
            std::thread::sleep(reenum_poll);
            if transport.is_open() {
                break;
            }
        }
        if cancelled() {
            return Err("Cancelled".into());
        }
        if !transport.is_open() {
            return Err("Device did not re-enumerate after bootcode upload".into());
        }
        self.serve_files(
            transport,
            chip,
            SideloadMode::Fastboot,
            firmware_dir,
            progress,
            cancel,
        )
    }
}
