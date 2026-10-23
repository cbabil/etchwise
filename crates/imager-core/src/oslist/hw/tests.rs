use serde_json::json;

use super::*;

fn doc() -> serde_json::Value {
    json!({"imager": {"devices": [
        {"name": "Raspberry Pi 5", "tags": ["pi5-64bit", "pi5-32bit"], "icon": "https://h/RPi_5.png",
         "description": "Pi 5", "matching_type": "exclusive", "capabilities": ["usb_otg"], "architecture": "arm64"},
        {"name": "No filtering", "tags": [], "icon": "icons/x.png", "matching_type": "inclusive", "default": true}
    ]}})
}

#[test]
fn parses_devices_and_default() {
    let (devices, default) = parse_devices(&doc()).unwrap();
    assert_eq!(devices.len(), 2);
    assert_eq!(default, Some(1));
    assert_eq!(devices[0].tags, ["pi5-64bit", "pi5-32bit"]);
    assert_eq!(devices[0].icon, "https://h/RPi_5.png");
    assert_eq!(devices[1].icon, "icons/x.png");
    assert!(devices[1].is_inclusive());
    assert!(!devices[0].is_inclusive());
}

#[test]
fn non_bool_default_is_ignored() {
    let d = json!({"imager": {"devices": [{"name": "a", "default": "yes"}]}});
    assert_eq!(parse_devices(&d).unwrap().1, None);
}

#[test]
fn missing_imager_or_devices_is_none() {
    assert!(parse_devices(&json!({})).is_none());
    assert!(parse_devices(&json!({"imager": 1})).is_none());
    assert!(parse_devices(&json!({"imager": {}})).is_none());
}

#[test]
fn matches_rpiboot_chips_to_tags() {
    let pi5 = vec!["pi5-64bit".to_owned()];
    let pi3 = vec!["pi3-64bit".to_owned()];
    let pi1 = vec!["pi1-32bit".to_owned()];
    assert!(tags_match_chip(&pi5, "BCM2712"));
    assert!(!tags_match_chip(&pi5, "BCM2711"));
    assert!(tags_match_chip(&["pi4-64bit".to_owned()], "BCM2711"));
    assert!(tags_match_chip(&pi3, "BCM2836/7"));
    // Upstream only matches the BCM2836 prefix; a bare BCM2837 is unknown.
    assert!(!tags_match_chip(&pi3, "BCM2837"));
    assert!(tags_match_chip(&pi3, "BCM2836"));
    assert!(tags_match_chip(&pi1, "BCM2835"));
    assert!(!tags_match_chip(&pi1, "BCM9999"));
}

#[test]
fn capability_checks_are_case_and_space_insensitive() {
    let hw = vec!["USB_OTG".to_owned(), "i2c".to_owned()];
    let sw = vec!["usb_otg".to_owned()];
    assert!(has_capability(&hw, " usb_otg "));
    assert!(!has_capability(&hw, "spi"));
    let caps = Capabilities {
        hardware: hw,
        software: sw,
    };
    assert!(caps.hw_and_sw("usb_otg", None));
    assert!(!caps.hw_and_sw("i2c", None));
    assert!(caps.hw_and_sw("i2c", Some("usb_otg")));
}

#[test]
fn software_caps_parse_json_or_csv() {
    assert_eq!(parse_sw_capabilities(r#"["a","B"]"#), ["a", "B"]);
    assert_eq!(parse_sw_capabilities("A, b,,c"), ["a", "b", "c"]);
    assert_eq!(parse_sw_capabilities("a, ,b"), ["a", "", "b"]);
}
