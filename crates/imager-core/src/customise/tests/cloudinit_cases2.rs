//! `generateCloudInitUserData` cases: password auth, rpi interfaces,
//! Pi Connect, WiFi country, and empty inputs.

use serde_json::json;

use super::helpers::{assert_no_network, has, lacks, network, user_data};
use crate::customise::text::{PI_CONNECT_CONFIG_PATH, PI_CONNECT_DEPLOY_KEY_FILENAME};

#[test]
fn customisation_generator_generates_cloud_init_user_data_with_password_auth() {
    let y = user_data(
        &json!({"sshPasswordAuth": true}),
        "",
        false,
        true,
        "testuser",
    );
    has(&y, &["ssh_pwauth: true"]);
}

#[test]
fn customisation_generator_generates_cloud_init_user_data_with_password_auth_and_ssh_keys() {
    let settings = json!({
        "sshUserName": "testuser",
        "sshPasswordAuth": true,
        "sshAuthorizedKeys": "ssh-rsa AAAAB3...key1",
    });
    let y = user_data(&settings, "", false, true, "testuser");
    has(&y, &["ssh_authorized_keys:", "ssh_pwauth: true"]);
    lacks(&y, &["ssh_pwauth: false"]);
}

#[test]
fn customisation_generator_generates_cloud_init_user_data_with_raspberry_pi_interfaces() {
    let settings = json!({
        "enableI2C": true,
        "enableSPI": true,
        "enableSerial": "Console & Hardware",
        "enableUsbGadget": true,
    });
    let y = user_data(&settings, "", true, false, "");
    has(
        &y,
        &[
            "rpi:",
            "enable_usb_gadget: true",
            "interfaces:",
            "i2c: true",
            "spi: true",
            "serial:",
            "console: true",
            "hardware: true",
        ],
    );
}

#[test]
fn customisation_generator_generates_cloud_init_user_data_with_pi_connect_token() {
    let settings = json!({"sshUserName": "testuser", "piConnectEnabled": true});
    let y = user_data(&settings, "test-token-abcd-1234", false, true, "testuser");
    let expected_install_dir =
        format!("install -o testuser -m 700 -d /home/testuser/{PI_CONNECT_CONFIG_PATH}");
    has(
        &y,
        &[
            "runcmd:",
            &expected_install_dir,
            "printf",
            "test-token-abcd-1234",
            PI_CONNECT_DEPLOY_KEY_FILENAME,
            "chmod 600",
            ".config/systemd/user/default.target.wants",
            ".config/systemd/user/paths.target.wants",
            "UNIT_SRC=/usr/lib/systemd/user/rpi-connect.service",
            "UNIT_SRC=/lib/systemd/user/rpi-connect.service",
            "ln -sf $UNIT_SRC",
            "rpi-connect.service",
            "rpi-connect-signin.path",
            "rpi-connect-wayvnc.service",
            "chown -R testuser:testuser",
            "/var/lib/systemd/linger",
            "install -m 0644 /dev/null /var/lib/systemd/linger/testuser",
        ],
    );
}

#[test]
fn customisation_generator_cloud_init_wifi_country_only_no_ssid() {
    let settings = json!({"recommendedWifiCountry": "FR"});
    let y = user_data(&settings, "", false, false, "pi");
    has(
        &y,
        &[
            "runcmd:",
            "rfkill, unblock, wifi",
            "/var/lib/systemd/rfkill/*:wlan",
        ],
    );
    assert_no_network(&network(&settings));
}

#[test]
fn customisation_generator_handles_empty_cloud_init_settings_gracefully() {
    let settings = json!({});
    let y = user_data(&settings, "", false, false, "");
    // Upstream adds `manage_resolv_conf` only to a non-empty payload
    // (`customization_generator.cpp:554`); its Catch2 case is stale.
    assert!(y.is_empty(), "expected empty user-data, got:\n{y}");
    lacks(
        &y,
        &[
            "hostname:",
            "users:",
            "enable_ssh:",
            "timezone:",
            "keyboard:",
            "rpi:",
        ],
    );
    assert_no_network(&network(&settings));
}

#[test]
fn customisation_generator_cloud_init_handles_empty_pi_connect_token() {
    let settings = json!({"sshUserName": "testuser", "piConnectEnabled": true});
    let y = user_data(&settings, "", false, true, "testuser");
    lacks(&y, &["write_files:", PI_CONNECT_CONFIG_PATH]);
}
