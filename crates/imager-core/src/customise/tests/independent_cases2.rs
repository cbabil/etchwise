//! "Independent step" cases: interfaces, USB gadget, Pi Connect.

use serde_json::json;

use super::helpers::{assert_no_network, has, lacks, network, user_data};
use crate::customise::text::PI_CONNECT_CONFIG_PATH;

#[test]
fn independent_step_interfaces_only_i2c() {
    let settings = json!({"enableI2C": true, "enableSerial": "Disabled"});
    let y = user_data(&settings, "", true, false, "pi");
    has(&y, &["rpi:", "interfaces:", "i2c: true"]);
    lacks(
        &y,
        &[
            "hostname:",
            "users:",
            "enable_ssh:",
            "timezone:",
            "keyboard:",
        ],
    );
    assert_no_network(&network(&settings));
}

#[test]
fn independent_step_interfaces_only_spi() {
    let settings = json!({"enableSPI": true, "enableSerial": "Disabled"});
    let y = user_data(&settings, "", true, false, "pi");
    has(&y, &["rpi:", "interfaces:", "spi: true"]);
    lacks(&y, &["i2c:", "onewire:"]);
}

#[test]
fn independent_step_interfaces_only_1_wire() {
    let settings = json!({"enable1Wire": true, "enableSerial": "Disabled"});
    let y = user_data(&settings, "", true, false, "pi");
    has(&y, &["rpi:", "interfaces:", "onewire: true"]);
}

#[test]
fn independent_step_interfaces_only_serial() {
    let settings = json!({"enableSerial": "Console"});
    let y = user_data(&settings, "", true, false, "pi");
    has(
        &y,
        &[
            "rpi:",
            "interfaces:",
            "serial:",
            "console: true",
            "hardware: false",
        ],
    );
}

#[test]
fn independent_step_usb_gadget_only() {
    let settings = json!({"enableUsbGadget": true, "enableSerial": "Disabled"});
    let y = user_data(&settings, "", true, false, "pi");
    has(&y, &["rpi:", "enable_usb_gadget: true"]);
    lacks(&y, &["interfaces:"]);
}

#[test]
fn independent_step_pi_connect_only_with_required_user() {
    let settings = json!({
        "sshUserName": "connectuser",
        "sshUserPassword": "$5$salt$hash",
        "piConnectEnabled": true,
    });
    let y = user_data(&settings, "pi-connect-deploy-token-xyz", false, false, "pi");
    has(&y, &["users:", "- name: connectuser"]);
    has(
        &y,
        &[
            "runcmd:",
            "pi-connect-deploy-token-xyz",
            PI_CONNECT_CONFIG_PATH,
            "install -o connectuser",
            "chown",
            "rpi-connect.service",
        ],
    );
    lacks(
        &y,
        &["hostname:", "enable_ssh:", "timezone:", "keyboard:", "rpi:"],
    );
    assert_no_network(&network(&settings));
}
