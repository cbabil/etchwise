//! libusb device discovery (upstream `LibusbContext` in
//! `libusb_transport.cpp`): boot-mode and fastboot scans, and opening a
//! device by bus and address.

use rusb::{Context, UsbContext};

use super::libusb_transport::LibusbTransport;
use super::usb_bus::{UsbBus, UsbDeviceInfo};
use crate::transport::UsbTransport;
use crate::types::{BROADCOM_VID, ChipGeneration, FASTBOOT_PID, FASTBOOT_VID};

/// A libusb context (upstream `LibusbContext`).
pub struct LibusbContext {
    context: Context,
}

impl LibusbContext {
    /// Initialises libusb.
    ///
    /// # Errors
    /// `libusb_init failed: <reason>`, as upstream's exception text.
    pub fn new() -> Result<Self, String> {
        Context::new()
            .map(|context| Self { context })
            .map_err(|e| format!("libusb_init failed: {e}"))
    }

    /// Opens the device at `bus:address` and claims its interface.
    pub fn open_at(&self, bus: u8, address: u8) -> Option<LibusbTransport> {
        let list = self.context.devices().ok()?;
        let device = list
            .iter()
            .find(|d| d.bus_number() == bus && d.address() == address)?;
        let handle = device.open().ok()?;
        Some(LibusbTransport::new(device, handle))
    }

    fn scan(&self, pick: impl Fn(u16, u16) -> Option<ChipGeneration>) -> Vec<UsbDeviceInfo> {
        let Ok(list) = self.context.devices() else {
            return Vec::new();
        };
        list.iter()
            .filter_map(|device| {
                let desc = device.device_descriptor().ok()?;
                let chip = pick(desc.vendor_id(), desc.product_id())?;
                Some(UsbDeviceInfo {
                    bus_number: device.bus_number(),
                    device_address: device.address(),
                    vendor_id: desc.vendor_id(),
                    product_id: desc.product_id(),
                    port_path: device.port_numbers().unwrap_or_default(),
                    chip_generation: chip,
                    serial_number_index: desc.serial_number_string_index().unwrap_or(0),
                })
            })
            .collect()
    }
}

impl UsbBus for LibusbContext {
    fn scan_boot_devices(&self) -> Vec<UsbDeviceInfo> {
        self.scan(|vid, pid| {
            (vid == BROADCOM_VID)
                .then(|| ChipGeneration::from_pid(pid))
                .flatten()
        })
    }

    fn scan_fastboot_devices(&self) -> Vec<UsbDeviceInfo> {
        // Chip generation is not meaningful for fastboot; upstream's default.
        self.scan(|vid, pid| {
            (vid == FASTBOOT_VID && pid == FASTBOOT_PID).then_some(ChipGeneration::Bcm2711)
        })
    }

    fn open(&self, info: &UsbDeviceInfo) -> Option<Box<dyn UsbTransport>> {
        let transport = self.open_at(info.bus_number, info.device_address)?;
        Some(Box::new(transport))
    }
}
