//! The `RpibootThread` Fastboot flow against a scripted USB bus and an
//! offline fetcher (upstream has no unit tests for the thread).

use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use super::RpibootError;
use super::scanner::RpibootDevice;
use super::sideload::{SideloadTiming, Sideloader};
use super::test_bus::FakeBus;
use super::test_fetcher::{FakeFetcher, fetcher_for_cm4_cm5};
use super::test_support::{READ_FILE, file_message, mock_with, quiet, temp_dir};
use super::usb_bus::UsbDeviceInfo;
use crate::transport::MockTransport;
use crate::types::ChipGeneration;

fn usb(address: u8, serial: u8, port: &[u8]) -> UsbDeviceInfo {
    UsbDeviceInfo {
        bus_number: 1,
        device_address: address,
        vendor_id: 0x0a5c,
        product_id: 0x2711,
        port_path: port.to_vec(),
        chip_generation: ChipGeneration::Bcm2711,
        serial_number_index: serial,
    }
}

fn cm4(port: &[u8]) -> RpibootDevice {
    RpibootDevice {
        bus_number: 1,
        device_address: 5,
        port_path: port.to_vec(),
        product_id: 0x2711,
        chip_generation: ChipGeneration::Bcm2711,
    }
}

fn bus(
    boot: Vec<Vec<UsbDeviceInfo>>,
    fastboot: Vec<Vec<UsbDeviceInfo>>,
    t: Vec<MockTransport>,
) -> FakeBus {
    FakeBus {
        boot: Mutex::new(boot.into()),
        fastboot: Mutex::new(fastboot.into()),
        transports: Mutex::new(t.into()),
        opened: Mutex::default(),
    }
}

fn run(
    bus: &FakeBus,
    fetcher: &FakeFetcher,
    device: &RpibootDevice,
    cancel: bool,
) -> Result<(u8, u8), RpibootError> {
    let root = temp_dir();
    let timing = SideloadTiming {
        poll_interval: Duration::from_millis(1),
        bootcode_return_wait: Duration::ZERO,
        file_server_retry_delay: Duration::ZERO,
    };
    let cancel = AtomicBool::new(cancel);
    Sideloader::new(bus, fetcher)
        .with_timing(timing)
        .run(device, root.path(), &mut quiet, &cancel)
}

fn serving() -> MockTransport {
    mock_with(vec![file_message(READ_FILE, "boot.img")])
}

#[test]
fn uploads_bootcode_waits_for_second_stage_and_finds_fastboot() {
    let port = [1, 2];
    let boot = vec![
        vec![usb(5, 0, &port)],
        vec![usb(5, 0, &port)],
        vec![],
        vec![usb(6, 4, &port)],
    ];
    let fastboot = vec![vec![], vec![usb(9, 0, &[3]), usb(8, 0, &port)]];
    let bus = bus(boot, fastboot, vec![MockTransport::new(), serving()]);
    assert_eq!(
        run(&bus, &fetcher_for_cm4_cm5(), &cm4(&port), false),
        Ok((1, 8))
    );
    assert_eq!(*bus.opened.lock().expect("lock"), vec![0, 4]);
}

#[test]
fn second_stage_device_skips_bootcode_upload() {
    let port = [4];
    let bus = bus(
        vec![vec![usb(5, 5, &port)]],
        vec![vec![usb(7, 0, &port)]],
        vec![serving()],
    );
    assert_eq!(
        run(&bus, &fetcher_for_cm4_cm5(), &cm4(&port), false),
        Ok((1, 7))
    );
    assert_eq!(*bus.opened.lock().expect("lock"), vec![5]);
}

#[test]
fn times_out_when_second_stage_never_appears() {
    let bus = bus(
        vec![vec![usb(5, 0, &[1])]],
        vec![vec![]],
        vec![MockTransport::new()],
    );
    let result = run(&bus, &fetcher_for_cm4_cm5(), &cm4(&[1]), false);
    assert_eq!(result, Err(RpibootError::ReenumTimeout(15)));
    let message = result.expect_err("timeout").to_string();
    assert_eq!(
        message,
        "Timed out waiting for device to re-enumerate after bootcode upload (waited 15s)."
    );
}

#[test]
fn times_out_when_fastboot_never_appears() {
    let done = mock_with(vec![file_message(2, "")]);
    let bus = bus(vec![vec![usb(5, 4, &[1])]], vec![vec![]], vec![done]);
    let result = run(&bus, &fetcher_for_cm4_cm5(), &cm4(&[1]), false);
    assert_eq!(result, Err(RpibootError::FastbootTimeout));
}

#[test]
fn reports_open_failures() {
    let rom = bus(vec![vec![usb(5, 0, &[1])]], vec![vec![]], vec![]);
    assert_eq!(
        run(&rom, &fetcher_for_cm4_cm5(), &cm4(&[1]), false),
        Err(RpibootError::OpenFailed)
    );
    let stage2 = bus(vec![vec![usb(5, 4, &[1])]], vec![vec![]], vec![]);
    let result = run(&stage2, &fetcher_for_cm4_cm5(), &cm4(&[1]), false);
    assert_eq!(result, Err(RpibootError::OpenAfterReenumFailed));
}

#[test]
fn reports_protocol_failures_with_upstream_message() {
    let mut broken = MockTransport::new();
    broken.control_out_ok = false;
    let bus = bus(vec![vec![usb(5, 0, &[1])]], vec![vec![]], vec![broken]);
    let err = run(&bus, &fetcher_for_cm4_cm5(), &cm4(&[1]), false).expect_err("fails");
    assert_eq!(
        err.to_string(),
        "rpiboot protocol failed: Bootcode upload failed: Failed to send BootMessage: Control transfer failed"
    );
}

#[test]
fn reports_firmware_failures_and_cancellation() {
    let bus = bus(vec![vec![usb(5, 4, &[1])]], vec![vec![]], vec![]);
    let err = run(&bus, &FakeFetcher::default(), &cm4(&[1]), false).expect_err("fails");
    assert_eq!(
        err.to_string(),
        "Failed to obtain rpiboot firmware: Download failed: 404 Not Found"
    );
    let result = run(&bus, &fetcher_for_cm4_cm5(), &cm4(&[1]), true);
    assert!(matches!(
        result,
        Err(RpibootError::Firmware(_) | RpibootError::Cancelled)
    ));
}

#[test]
fn without_port_path_only_a_sole_fastboot_device_is_accepted() {
    let device = cm4(&[]);
    let two = vec![vec![usb(8, 0, &[1]), usb(9, 0, &[2])]];
    let stage2 = || vec![vec![usb(5, 4, &[])]];
    let ambiguous = bus(stage2(), two, vec![serving()]);
    let result = run(&ambiguous, &fetcher_for_cm4_cm5(), &device, false);
    assert_eq!(result, Err(RpibootError::FastbootTimeout));
    let sole = bus(stage2(), vec![vec![usb(8, 0, &[1])]], vec![serving()]);
    assert_eq!(
        run(&sole, &fetcher_for_cm4_cm5(), &device, false),
        Ok((1, 8))
    );
}
