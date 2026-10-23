//! `generateSystemdScript` cases: WiFi, Pi Connect, empty settings, quoting.

use serde_json::json;

use super::helpers::{has, lacks, script, script_with_token};
use crate::customise::text::{PI_CONNECT_CONFIG_PATH, PI_CONNECT_DEPLOY_KEY_FILENAME};

#[test]
fn customisation_generator_wifi_configuration() {
    let s = script(&json!({
        "wifiConfigured": true,
        "wifiSSID": "TestNetwork",
        "wifiPasswordCrypt": "hashed_password_here",
        "recommendedWifiCountry": "GB",
        "wifiHidden": true,
    }));
    has(
        &s,
        &[
            "set_wlan",
            "wpa_supplicant.conf",
            "country=GB",
            "ssid=\"TestNetwork\"",
            "scan_ssid=1",
            "key_mgmt=WPA-PSK SAE",
            "psk=hashed_password_here",
            "ieee80211w=1",
        ],
    );
}

#[test]
fn customisation_generator_wifi_configuration_with_empty_psk_open_network() {
    let s = script(&json!({
        "wifiConfigured": true,
        "wifiSSID": "OpenNetwork",
        "wifiPasswordCrypt": "",
        "recommendedWifiCountry": "US",
    }));
    has(
        &s,
        &[
            "wpa_supplicant.conf",
            "ssid=\"OpenNetwork\"",
            "key_mgmt=NONE",
        ],
    );
    lacks(&s, &["WPA-PSK", "\tpsk=", "ieee80211w=1"]);
}

#[test]
fn customisation_generator_wifi_country_only_no_ssid() {
    let s = script(&json!({"recommendedWifiCountry": "GB"}));
    has(
        &s,
        &["rfkill unblock wifi", "/var/lib/systemd/rfkill/*:wlan"],
    );
    lacks(&s, &["wpa_supplicant.conf", "set_wlan"]);
}

#[test]
fn customisation_generator_raspberry_pi_connect() {
    let s = script_with_token(
        &json!({"sshUserName": "testuser", "piConnectEnabled": true}),
        "test-token-12345",
    );
    has(
        &s,
        &[
            PI_CONNECT_CONFIG_PATH,
            PI_CONNECT_DEPLOY_KEY_FILENAME,
            "test-token-12345",
            "SYSTEMD_USER_BASE=",
            "default.target.wants",
            "paths.target.wants",
            "rpi-connect.service",
            "rpi-connect-signin.path",
            "rpi-connect-wayvnc.service",
            "/var/lib/systemd/linger",
            "install -m 0644 /dev/null \"/var/lib/systemd/linger/$TARGET_USER\"",
        ],
    );
}

#[test]
fn customisation_generator_handles_empty_settings_gracefully() {
    let s = script(&json!({}));
    has(
        &s,
        &["#!/bin/sh", "set +e", "rm -f /boot/firstrun.sh", "exit 0"],
    );
}

#[test]
fn customisation_generator_handles_shell_injection_attempts_in_username() {
    let s = script(&json!({
        "sshUserName": "user'; rm -rf /; echo '",
        "sshUserPassword": "$5$salt$hash",
    }));
    has(&s, &["'user'\"'\"'; rm -rf /; echo '\"'\"''"]);
    assert!(s.ends_with("exit 0\n"));
}

#[test]
fn customisation_generator_handles_special_characters_in_hostname() {
    let s = script(&json!({"hostname": "test-host_123"}));
    has(&s, &["test-host_123"]);
}

#[test]
fn customisation_generator_handles_quotes_in_timezone() {
    let s = script(&json!({"timezone": "Europe/London'; rm -rf /; echo 'pwned"}));
    has(
        &s,
        &["set_timezone 'Europe/London'\"'\"'; rm -rf /; echo '\"'\"'pwned'"],
    );
}

#[test]
fn customisation_generator_handles_empty_password_with_username() {
    let s = script(&json!({"sshUserName": "testuser", "sshUserPassword": ""}));
    has(&s, &["userconf-pi/userconf", "'testuser'"]);
}

#[test]
fn customisation_generator_handles_special_characters_in_wifi_ssid() {
    let s = script(&json!({
        "wifiConfigured": true,
        "wifiSSID": "Test Network (5GHz)",
        "wifiPasswordCrypt": "fakehash",
        "recommendedWifiCountry": "US",
    }));
    has(&s, &["wpa_supplicant.conf", "ssid=\"Test Network (5GHz)\""]);
}

#[test]
fn customisation_generator_handles_quotes_in_wifi_ssid() {
    let s = script(&json!({
        "wifiConfigured": true,
        "wifiSSID": "My \"Quoted\" Network",
        "wifiPasswordCrypt": "fakehash",
        "recommendedWifiCountry": "US",
    }));
    has(&s, &["ssid=\"My \\\"Quoted\\\" Network\"", "set_wlan"]);
}

#[test]
fn customisation_generator_handles_backslashes_in_wifi_ssid() {
    let s = script(&json!({
        "wifiConfigured": true,
        "wifiSSID": "Network\\With\\Backslashes",
        "wifiPasswordCrypt": "fakehash",
        "recommendedWifiCountry": "US",
    }));
    has(&s, &["ssid=\"Network\\\\With\\\\Backslashes\""]);
}

#[test]
fn customisation_generator_handles_non_ascii_utf_8_wifi_ssid() {
    let s = script(&json!({
        "wifiConfigured": true,
        "wifiSSID": "Café ☕ 日本語",
        "wifiPasswordCrypt": "fakehash",
        "recommendedWifiCountry": "FR",
    }));
    has(&s, &["wpa_supplicant.conf", "Café ☕ 日本語", "set_wlan"]);
}
