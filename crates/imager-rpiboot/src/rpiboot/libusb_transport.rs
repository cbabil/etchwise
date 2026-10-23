//! An opened libusb device with its interface claimed (upstream
//! `LibusbTransport` in `libusb_transport.cpp`), via `rusb`.

use std::time::Duration;

use rusb::{Context, Device, DeviceHandle};

use crate::transport::UsbTransport;

/// An opened device (upstream `LibusbTransport`).
pub struct LibusbTransport {
    handle: DeviceHandle<Context>,
    interface: u8,
    interface_claimed: bool,
    out_ep: u8,
    last_error: Option<i32>,
}

impl LibusbTransport {
    /// Picks the interface and OUT endpoint from the active configuration
    /// (one interface: 0, EP 0x01; otherwise 1, EP 0x03), sets
    /// configuration 1 on macOS, and claims the interface.
    pub fn new(device: Device<Context>, handle: DeviceHandle<Context>) -> Self {
        let mut t = Self {
            handle,
            interface: 0,
            interface_claimed: false,
            out_ep: 0x01,
            last_error: None,
        };
        if device
            .active_config_descriptor()
            .is_ok_and(|config| config.num_interfaces() != 1)
        {
            (t.interface, t.out_ep) = (1, 0x03);
        }
        // Auto-detach is unsupported (and unnecessary) on macOS.
        let _ = t.handle.set_auto_detach_kernel_driver(true);
        // macOS IOKit only enables bulk transfers beyond the first 64-byte
        // packet after a user-space `USBSetConfiguration()`; Linux would
        // take an avoidable bus reset, so this is macOS only.
        if cfg!(target_os = "macos") {
            let _ = t.handle.set_active_configuration(1);
        }
        t.interface_claimed = t.handle.claim_interface(t.interface).is_ok();
        t
    }

    fn record<T>(&mut self, result: rusb::Result<T>) -> Option<T> {
        result
            .map_err(|e| self.last_error = Some(libusb_code(e)))
            .ok()
    }

    /// Bulk results: a timeout with nothing transferred counts as 0 bytes,
    /// as upstream returns `transferred` on `LIBUSB_ERROR_TIMEOUT`.
    fn record_bulk(&mut self, result: rusb::Result<usize>) -> Option<usize> {
        match result {
            Err(rusb::Error::Timeout | rusb::Error::Interrupted) => Some(0),
            other => self.record(other),
        }
    }
}

impl Drop for LibusbTransport {
    fn drop(&mut self) {
        if self.interface_claimed {
            let _ = self.handle.release_interface(self.interface);
        }
    }
}

fn timeout(ms: u32) -> Duration {
    Duration::from_millis(u64::from(ms))
}

impl UsbTransport for LibusbTransport {
    fn control_out(
        &mut self,
        rt: u8,
        req: u8,
        value: u16,
        index: u16,
        data: &[u8],
        ms: u32,
    ) -> bool {
        let r = self
            .handle
            .write_control(rt, req, value, index, data, timeout(ms));
        self.record(r).is_some()
    }

    fn control_in(
        &mut self,
        rt: u8,
        req: u8,
        value: u16,
        index: u16,
        buffer: &mut [u8],
        ms: u32,
    ) -> Option<usize> {
        let r = self
            .handle
            .read_control(rt, req, value, index, buffer, timeout(ms));
        self.record(r)
    }

    fn bulk_write(&mut self, endpoint: u8, data: &[u8], ms: u32) -> Option<usize> {
        let r = self.handle.write_bulk(endpoint, data, timeout(ms));
        self.record_bulk(r)
    }

    fn bulk_read(&mut self, endpoint: u8, buffer: &mut [u8], ms: u32) -> Option<usize> {
        let r = self.handle.read_bulk(endpoint | 0x80, buffer, timeout(ms));
        self.record_bulk(r)
    }

    fn is_open(&self) -> bool {
        true
    }

    fn out_endpoint(&self) -> u8 {
        self.out_ep
    }

    fn last_error_code(&self) -> Option<i32> {
        self.last_error
    }
}

/// The libusb error code behind a `rusb` error.
pub fn libusb_code(error: rusb::Error) -> i32 {
    use rusb::Error as E;
    match error {
        E::Io => -1,
        E::InvalidParam => -2,
        E::Access => -3,
        E::NoDevice => -4,
        E::NotFound => -5,
        E::Busy => -6,
        E::Timeout => -7,
        E::Overflow => -8,
        E::Pipe => -9,
        E::Interrupted => -10,
        E::NoMem => -11,
        E::NotSupported => -12,
        E::BadDescriptor | E::Other => -99,
    }
}
