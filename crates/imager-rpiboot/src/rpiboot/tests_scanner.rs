//! Drive-list identity of boot-mode devices (upstream
//! `rpiboot_scanner.cpp`, `DeviceDescriptor::uniqueKey`, and
//! `ImageWriter`'s device-id parsing).

use super::scanner::{RpibootDevice, parse_device_id, port_path_to_string};
use crate::types::ChipGeneration;

fn cm5() -> RpibootDevice {
    RpibootDevice {
        bus_number: 2,
        device_address: 17,
        port_path: vec![1, 4, 2],
        product_id: 0x2712,
        chip_generation: ChipGeneration::Bcm2712,
    }
}

#[test]
fn formats_port_paths() {
    assert_eq!(port_path_to_string(&[1, 2, 3]), "1.2.3");
    assert_eq!(port_path_to_string(&[7]), "7");
    assert_eq!(port_path_to_string(&[]), "");
}

#[test]
fn device_id_uses_decimal_pid() {
    assert_eq!(cm5().device_id(), "rpiboot://2:17:1.4.2:10002");
}

#[test]
fn unique_key_names_chip_and_port() {
    assert_eq!(cm5().unique_key(), "rpiboot:BCM2712:1.4.2");
    assert_eq!(cm5().description(), "Compute Module 5 (USB Boot)");
    assert_eq!(cm5().chip_name(), "BCM2712");
}

#[test]
fn parses_device_ids_back() {
    let parsed = parse_device_id(&cm5().device_id());
    assert_eq!(
        parsed,
        RpibootDevice {
            product_id: 0,
            ..cm5()
        }
    );
}

#[test]
fn parsing_defaults_like_image_writer() {
    let parsed = parse_device_id("rpiboot://3:4");
    assert_eq!((parsed.bus_number, parsed.device_address), (3, 4));
    assert!(parsed.port_path.is_empty());
    assert_eq!(parsed.chip_generation, ChipGeneration::Bcm2711);
    let unknown = parse_device_id("rpiboot://1:2:3..4:10083");
    assert_eq!(unknown.port_path, vec![3, 4]);
    assert_eq!(unknown.chip_generation, ChipGeneration::Bcm2711);
    let cm3 = parse_device_id("rpiboot://1:2::10084");
    assert_eq!(cm3.chip_generation, ChipGeneration::Bcm2836_7);
}
