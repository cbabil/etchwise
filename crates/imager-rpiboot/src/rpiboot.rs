//! Broadcom USB boot protocol (upstream `src/rpiboot/` and the Fastboot
//! path of `rpibootthread.cpp`): find boards in USB boot mode, upload the
//! second-stage bootloader, serve the fastboot gadget's files, and wait for
//! the gadget to enumerate.
//!
//! Upstream's `SecureBootRecovery` mode (OTP key programming) is
//! unreachable in its GUI and is deliberately not ported.

pub(crate) mod bootcode_loader;
pub(crate) mod bootfiles;
pub mod fetcher;
pub(crate) mod file_requests;
pub(crate) mod file_server;
mod file_session;
pub mod firmware_manager;
pub mod libusb_context;
pub mod libusb_transport;
pub(crate) mod manifest;
pub(crate) mod protocol;
pub mod scanner;
pub mod sideload;
mod sideload_wait;
pub mod usb_bus;

#[cfg(test)]
mod test_bus;
#[cfg(test)]
mod test_fetcher;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests_bootcode;
#[cfg(test)]
mod tests_bootfiles;
#[cfg(test)]
mod tests_file_server;
#[cfg(test)]
mod tests_file_server_errors;
#[cfg(test)]
mod tests_firmware;
#[cfg(test)]
mod tests_integration;
#[cfg(test)]
mod tests_protocol;
#[cfg(test)]
mod tests_scanner;
#[cfg(test)]
mod tests_sideload;

pub use libusb_context::LibusbContext;
pub use scanner::{RpibootDevice, parse_device_id, port_path_to_string, scan_rpiboot_devices};
pub use sideload::{SideloadTiming, Sideloader, sideload_fastboot};

/// Failures of the sideload flow, with upstream's user-facing messages
/// (`RpibootThread::error`).
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RpibootError {
    /// The firmware cache could not be filled.
    #[error("Failed to obtain rpiboot firmware: {0}")]
    Firmware(String),
    /// The boot-mode device could not be opened for the bootcode upload.
    #[error("Failed to open USB device")]
    OpenFailed,
    /// The second-stage device could not be opened for the file server.
    #[error("Failed to open USB device after re-enumeration")]
    OpenAfterReenumFailed,
    /// Bootcode upload or file server failed.
    #[error("rpiboot protocol failed: {0}")]
    Protocol(String),
    /// libusb could not be initialised.
    #[error("USB error: {0}")]
    Usb(String),
    /// The second stage never appeared (seconds waited).
    #[error("Timed out waiting for device to re-enumerate after bootcode upload (waited {0}s).")]
    ReenumTimeout(u32),
    /// The fastboot gadget never appeared.
    #[error("Timed out waiting for fastboot device to appear.")]
    FastbootTimeout,
    /// The caller cancelled (upstream ends silently).
    #[error("Cancelled")]
    Cancelled,
}
