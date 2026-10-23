//! Boards in USB boot mode as drive-list entries (upstream
//! `rpiboot_scanner.cpp`, the rpiboot half of `DeviceDescriptor::
//! uniqueKey`, and `ImageWriter`'s parsing of the synthetic device id).

use super::libusb_context::LibusbContext;
use super::usb_bus::UsbBus;
use crate::types::ChipGeneration;

/// A Broadcom device waiting in USB boot mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpibootDevice {
    /// USB bus number.
    pub bus_number: u8,
    /// Device address on the bus.
    pub device_address: u8,
    /// USB port numbers (stable across re-enumeration).
    pub port_path: Vec<u8>,
    /// `idProduct` (0 when parsed from a device id, as upstream).
    pub product_id: u16,
    /// Chip generation selected by the product id.
    pub chip_generation: ChipGeneration,
}

impl RpibootDevice {
    /// `rpiboot://bus:addr:port.path:pid`, the pid in decimal.
    pub fn device_id(&self) -> String {
        format!(
            "rpiboot://{}:{}:{}:{}",
            self.bus_number,
            self.device_address,
            port_path_to_string(&self.port_path),
            self.chip_generation.pid()
        )
    }

    /// `rpiboot:<chip name>:<port.path>` (drive-list tracking key).
    pub fn unique_key(&self) -> String {
        format!(
            "rpiboot:{}:{}",
            self.chip_generation.name(),
            port_path_to_string(&self.port_path)
        )
    }

    /// Drive-list description, e.g. `Compute Module 4 (USB Boot)`.
    pub fn description(&self) -> &'static str {
        self.chip_generation.description()
    }

    /// Chip name, e.g. `BCM2711`.
    pub fn chip_name(&self) -> &'static str {
        self.chip_generation.name()
    }
}

/// Dotted port path, e.g. `1.2.3`.
pub fn port_path_to_string(port_path: &[u8]) -> String {
    port_path
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(".")
}

/// Boot-mode devices on `bus`.
pub fn scan_with(bus: &dyn UsbBus) -> Vec<RpibootDevice> {
    bus.scan_boot_devices()
        .into_iter()
        .map(|d| RpibootDevice {
            bus_number: d.bus_number,
            device_address: d.device_address,
            port_path: d.port_path,
            product_id: d.product_id,
            chip_generation: d.chip_generation,
        })
        .collect()
}

/// Boot-mode devices on the system's USB buses; empty when libusb cannot
/// be initialised.
pub fn scan_rpiboot_devices() -> Vec<RpibootDevice> {
    LibusbContext::new()
        .map(|ctx| scan_with(&ctx))
        .unwrap_or_default()
}

/// Parses a device id as `ImageWriter` does: missing or malformed parts
/// become 0 / empty, an unknown pid falls back to BCM2711.
pub fn parse_device_id(id: &str) -> RpibootDevice {
    let rest = id.strip_prefix("rpiboot://").unwrap_or(id);
    let parts: Vec<&str> = rest.split(':').collect();
    let number = |i: usize| {
        parts
            .get(i)
            .and_then(|p| p.parse::<u32>().ok())
            .unwrap_or(0)
    };
    let (bus_number, device_address) = if parts.len() >= 2 {
        (number(0) as u8, number(1) as u8)
    } else {
        (0, 0)
    };
    let port_path = parts.get(2).map_or_else(Vec::new, |p| {
        p.split('.')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<u32>().unwrap_or(0) as u8)
            .collect()
    });
    let chip_generation =
        ChipGeneration::from_pid(number(3) as u16).unwrap_or(ChipGeneration::Bcm2711);
    RpibootDevice {
        bus_number,
        device_address,
        port_path,
        product_id: 0,
        chip_generation,
    }
}
