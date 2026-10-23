//! A scripted USB bus for the sideload tests.

use std::collections::VecDeque;
use std::sync::Mutex;

use super::usb_bus::{UsbBus, UsbDeviceInfo};
use crate::transport::{MockTransport, UsbTransport};

/// Scan results are consumed in order; the last one repeats.
#[derive(Default)]
pub struct FakeBus {
    pub boot: Mutex<VecDeque<Vec<UsbDeviceInfo>>>,
    pub fastboot: Mutex<VecDeque<Vec<UsbDeviceInfo>>>,
    pub transports: Mutex<VecDeque<MockTransport>>,
    pub opened: Mutex<Vec<u8>>,
}

fn next(queue: &Mutex<VecDeque<Vec<UsbDeviceInfo>>>) -> Vec<UsbDeviceInfo> {
    let mut q = queue.lock().expect("lock");
    if q.len() > 1 {
        q.pop_front().unwrap_or_default()
    } else {
        q.front().cloned().unwrap_or_default()
    }
}

impl UsbBus for FakeBus {
    fn scan_boot_devices(&self) -> Vec<UsbDeviceInfo> {
        next(&self.boot)
    }
    fn scan_fastboot_devices(&self) -> Vec<UsbDeviceInfo> {
        next(&self.fastboot)
    }
    fn open(&self, info: &UsbDeviceInfo) -> Option<Box<dyn UsbTransport>> {
        self.opened
            .lock()
            .expect("lock")
            .push(info.serial_number_index);
        let t = self.transports.lock().expect("lock").pop_front()?;
        Some(Box::new(t))
    }
}
