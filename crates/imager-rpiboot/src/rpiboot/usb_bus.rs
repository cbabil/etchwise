//! USB device discovery as the sideload flow needs it (upstream
//! `UsbDeviceInfo` and the scanning half of `LibusbContext`), behind a
//! trait so tests can script devices appearing and disappearing.

use crate::transport::UsbTransport;
use crate::types::ChipGeneration;

/// A device found during USB scanning (upstream `UsbDeviceInfo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbDeviceInfo {
    /// USB bus number.
    pub bus_number: u8,
    /// Device address on the bus.
    pub device_address: u8,
    /// `idVendor`.
    pub vendor_id: u16,
    /// `idProduct`.
    pub product_id: u16,
    /// USB port numbers, for tracking a board across re-enumeration.
    pub port_path: Vec<u8>,
    /// Chip generation (fastboot devices default to BCM2711, as upstream).
    pub chip_generation: ChipGeneration,
    /// `iSerialNumber` (0 = boot ROM, 3 = first stage, >3 = second stage).
    pub serial_number_index: u8,
}

impl UsbDeviceInfo {
    /// The device still needs the bootcode upload: upstream rpiboot
    /// dispatches `iSerialNumber` 0 or 3 to `second_stage_boot()`.
    pub fn needs_bootcode(&self) -> bool {
        self.serial_number_index == 0 || self.serial_number_index == 3
    }
}

/// Scans and opens USB devices (upstream `LibusbContext`).
pub trait UsbBus: Send + Sync {
    /// Broadcom devices in USB boot mode with a supported product id.
    fn scan_boot_devices(&self) -> Vec<UsbDeviceInfo>;
    /// Devices in fastboot mode (VID 0x18d1, PID 0x4e40).
    fn scan_fastboot_devices(&self) -> Vec<UsbDeviceInfo>;
    /// Opens the device at `info`'s bus and address; `None` on failure.
    fn open(&self, info: &UsbDeviceInfo) -> Option<Box<dyn UsbTransport>>;
}
