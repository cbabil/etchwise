//! `generateCloudInitNetworkConfig` cases: WiFi variants and SSID escaping.

use serde_json::json;

use super::helpers::{ETH0_DHCP, has, lacks, network};

#[test]
fn customisation_generator_generates_cloud_init_network_config_with_wifi() {
    let n = network(&json!({
        "wifiConfigured": true,
        "wifiSSID": "TestNetwork",
        "wifiPasswordCrypt": "fakecryptedhash123",
        "recommendedWifiCountry": "DE",
    }));
    has(
        &n,
        &[
            "network:",
            "version: 2",
            "ethernets:",
            "eth0:",
            "dhcp6: true",
            "wifis:",
            "wlan0:",
            "dhcp4: true",
            "regulatory-domain: \"DE\"",
            "access-points:",
            "\"TestNetwork\":",
            "password: \"fakecryptedhash123\"",
            "optional: true",
        ],
    );
}

#[test]
fn customisation_generator_generates_cloud_init_network_config_with_hidden_wifi() {
    let n = network(&json!({
        "wifiConfigured": true,
        "wifiSSID": "HiddenNetwork",
        "wifiPasswordCrypt": "fakecryptedhash123",
        "wifiHidden": true,
    }));
    has(&n, ETH0_DHCP);
    has(&n, &["\"HiddenNetwork\":", "hidden: true", "password:"]);
}

#[test]
fn customisation_generator_generates_cloud_init_network_config_for_open_wifi_no_password() {
    let n = network(&json!({
        "wifiConfigured": true,
        "wifiSSID": "OpenNetwork",
        "wifiPasswordCrypt": "",
        "recommendedWifiCountry": "US",
    }));
    has(&n, ETH0_DHCP);
    has(
        &n,
        &[
            "wifis:",
            "wlan0:",
            "\"OpenNetwork\":",
            "regulatory-domain: \"US\"",
            "auth:",
            "key-management: none",
        ],
    );
    lacks(&n, &["password:"]);
}

#[test]
fn customisation_generator_generates_cloud_init_network_config_with_special_characters_in_ssid() {
    let n = network(&json!({
        "wifiConfigured": true,
        "wifiSSID": "Test \"Network\" (5GHz)",
        "wifiPasswordCrypt": "fakecryptedhash123",
    }));
    has(&n, &["ethernets:", "eth0:"]);
    has(&n, &["Test \\\"Network\\\" (5GHz)", "password:"]);
}

#[test]
fn customisation_generator_cloud_init_network_config_escapes_backslashes_in_ssid() {
    let n = network(&json!({
        "wifiConfigured": true,
        "wifiSSID": "Network\\With\\Backslashes",
        "wifiPasswordCrypt": "fakecryptedhash123",
    }));
    has(&n, &["Network\\\\With\\\\Backslashes"]);
}

#[test]
fn customisation_generator_cloud_init_network_config_escapes_control_characters_in_ssid() {
    let n = network(&json!({
        "wifiConfigured": true,
        "wifiSSID": "Net\twork\nWith\rControl",
        "wifiPasswordCrypt": "fakecryptedhash123",
    }));
    has(&n, &["Net\\twork\\nWith\\rControl"]);
}

#[test]
fn customisation_generator_cloud_init_network_config_escapes_mixed_special_characters_in_ssid() {
    let n = network(&json!({
        "wifiConfigured": true,
        "wifiSSID": "Test\\\"Net\twork\"",
        "wifiPasswordCrypt": "fakecryptedhash123",
    }));
    has(&n, &["Test\\\\\\\"Net\\twork\\\""]);
}
