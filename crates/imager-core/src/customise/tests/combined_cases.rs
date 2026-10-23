//! "Combined steps" cases: steps must not interfere with each other.

use serde_json::json;

use super::helpers::{has, lacks, network, user_data};

#[test]
fn combined_steps_user_hostname_no_ssh() {
    let settings = json!({
        "hostname": "workstation",
        "sshUserName": "developer",
        "sshUserPassword": "$6$salt$hash",
    });
    let y = user_data(&settings, "", false, false, "pi");
    has(
        &y,
        &[
            "hostname: workstation",
            "users:",
            "- name: developer",
            "passwd:",
        ],
    );
    lacks(&y, &["enable_ssh:"]);
}

#[test]
fn combined_steps_user_wifi_no_ssh() {
    let settings = json!({
        "sshUserName": "wifiuser",
        "sshUserPassword": "$6$salt$hash",
        "wifiConfigured": true,
        "wifiSSID": "OfficeWiFi",
        "wifiPasswordCrypt": "wifihash",
    });
    let y = user_data(&settings, "", false, false, "pi");
    let n = network(&settings);
    has(&y, &["users:", "- name: wifiuser"]);
    has(&n, &["\"OfficeWiFi\":"]);
    lacks(&y, &["enable_ssh:"]);
}

#[test]
fn combined_steps_all_locale_user_no_ssh() {
    let settings = json!({
        "timezone": "Asia/Tokyo",
        "keyboard": "jp",
        "sshUserName": "jpuser",
        "sshUserPassword": "$6$salt$hash",
    });
    let y = user_data(&settings, "", false, false, "pi");
    has(
        &y,
        &[
            "timezone: Asia/Tokyo",
            "layout: \"jp\"",
            "users:",
            "- name: jpuser",
        ],
    );
    lacks(&y, &["enable_ssh:"]);
}

#[test]
fn combined_steps_user_interfaces_no_ssh() {
    let settings = json!({
        "sshUserName": "iotuser",
        "sshUserPassword": "$6$salt$hash",
        "enableI2C": true,
        "enableSPI": true,
    });
    let y = user_data(&settings, "", true, false, "pi");
    has(&y, &["users:", "- name: iotuser"]);
    has(&y, &["rpi:", "i2c: true", "spi: true"]);
    lacks(&y, &["enable_ssh:"]);
}

#[test]
fn combined_steps_full_customization_without_ssh() {
    let settings = json!({
        "hostname": "fullpi",
        "timezone": "Europe/Berlin",
        "keyboard": "de",
        "sshUserName": "fulluser",
        "sshUserPassword": "$6$salt$hash",
        "wifiConfigured": true,
        "wifiSSID": "FullWiFi",
        "wifiPasswordCrypt": "wifihash",
        "recommendedWifiCountry": "DE",
        "enableI2C": true,
    });
    let y = user_data(&settings, "", true, false, "pi");
    let n = network(&settings);
    has(
        &y,
        &[
            "hostname: fullpi",
            "timezone: Europe/Berlin",
            "layout: \"de\"",
            "users:",
            "- name: fulluser",
            "passwd:",
            "rpi:",
            "i2c: true",
        ],
    );
    has(&n, &["\"FullWiFi\":", "regulatory-domain: \"DE\""]);
    lacks(&y, &["enable_ssh:", "ssh_pwauth:"]);
}

#[test]
fn combined_steps_full_customization_with_ssh() {
    let settings = json!({
        "hostname": "sshpi",
        "timezone": "UTC",
        "keyboard": "us",
        "sshUserName": "sshuser",
        "sshUserPassword": "$6$salt$hash",
        "sshPasswordAuth": true,
        "wifiConfigured": true,
        "wifiSSID": "SSHWiFi",
        "wifiPasswordCrypt": "wifihash",
        "enableSPI": true,
    });
    let y = user_data(&settings, "", true, true, "pi");
    let n = network(&settings);
    has(
        &y,
        &[
            "hostname: sshpi",
            "timezone: UTC",
            "layout: \"us\"",
            "users:",
            "- name: sshuser",
            "passwd:",
            "rpi:",
            "spi: true",
        ],
    );
    has(&y, &["enable_ssh: true", "ssh_pwauth: true"]);
    has(&n, &["\"SSHWiFi\":"]);
}
