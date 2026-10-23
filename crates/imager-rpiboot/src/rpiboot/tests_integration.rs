//! Upstream `rpiboot_integration_test.cpp`: hardware tests, ignored by
//! default. Run with a board in USB boot mode:
//! `cargo test -p imager-rpiboot rpiboot::tests_integration -- --ignored`.

use std::sync::atomic::AtomicBool;

use super::fetcher::HttpFetcher;
use super::firmware_manager::FirmwareManager;
use super::libusb_context::LibusbContext;
use super::protocol::RpibootProtocol;
use super::scanner::scan_rpiboot_devices;
use super::test_support::{quiet, temp_dir};
use super::usb_bus::UsbBus;
use crate::types::SideloadMode;

#[test]
#[ignore = "needs a Raspberry Pi in USB boot mode"]
fn scan_for_rpiboot_devices() {
    for device in scan_rpiboot_devices() {
        eprintln!("{} ({})", device.device_id(), device.chip_name());
    }
}

#[test]
#[ignore = "needs a Raspberry Pi in USB boot mode and network access"]
fn fastboot_sideload_to_connected_cm() {
    let ctx = LibusbContext::new().expect("libusb");
    let devices = ctx.scan_boot_devices();
    let device = devices.first().expect("a device in USB boot mode");
    let mut transport = ctx.open(device).expect("open");
    let fetcher = HttpFetcher::new().expect("http");
    let cache = temp_dir();
    let cancel = AtomicBool::new(false);
    let chip = device.chip_generation;
    let manager = FirmwareManager::new(cache.path(), &fetcher);
    let fw_dir = manager
        .ensure_available(SideloadMode::Fastboot, chip, &mut quiet, &cancel)
        .expect("firmware");
    let mut protocol = RpibootProtocol::default();
    let result = protocol.execute(
        transport.as_mut(),
        chip,
        &fw_dir,
        &mut quiet,
        &cancel,
        std::time::Duration::from_millis(200),
    );
    assert_eq!(result, Ok(()));
    eprintln!("metadata: {:?}", protocol.metadata());
}
