//! "Independent step" cases: each customisation step on its own
//! (hostname, locale, user, WiFi, SSH).

use serde_json::json;

use super::helpers::{ETH0_DHCP, assert_no_network, has, lacks, network, user_data};

const GROUPS: &str = "groups: users,adm,dialout,audio,netdev,video,plugdev,cdrom,games,input,gpio,spi,i2c,render,sudo";

/// Upstream's stale cases expected eth0 here; see `assert_no_network`.
fn assert_eth0_only(netcfg: &str) {
    assert_no_network(netcfg);
}

#[test]
fn independent_step_hostname_only() {
    let settings = json!({"hostname": "mypi"});
    let y = user_data(&settings, "", false, false, "pi");
    has(
        &y,
        &[
            "manage_resolv_conf: false",
            "hostname: mypi",
            "manage_etc_hosts: true",
        ],
    );
    lacks(&y, &["preserve_hostname"]);
    has(
        &y,
        &[
            "packages:",
            "- avahi-daemon",
            "apt:",
            "preserve_sources_list: true",
        ],
    );
    lacks(
        &y,
        &["users:", "enable_ssh:", "timezone:", "keyboard:", "rpi:"],
    );
    assert_eth0_only(&network(&settings));
}

#[test]
fn independent_step_timezone_only() {
    let settings = json!({"timezone": "America/New_York"});
    let y = user_data(&settings, "", false, false, "pi");
    has(&y, &["timezone: America/New_York"]);
    lacks(
        &y,
        &["hostname:", "users:", "enable_ssh:", "keyboard:", "rpi:"],
    );
    assert_eth0_only(&network(&settings));
}

#[test]
fn independent_step_keyboard_only() {
    let settings = json!({"keyboard": "de"});
    let y = user_data(&settings, "", false, false, "pi");
    has(&y, &["keyboard:", "model: pc105", "layout: \"de\""]);
    lacks(
        &y,
        &["hostname:", "users:", "enable_ssh:", "timezone:", "rpi:"],
    );
    assert_eth0_only(&network(&settings));
}

#[test]
fn independent_step_locale_timezone_keyboard() {
    let settings = json!({"timezone": "Europe/Paris", "keyboard": "fr"});
    let y = user_data(&settings, "", false, false, "pi");
    has(
        &y,
        &["timezone: Europe/Paris", "keyboard:", "layout: \"fr\""],
    );
    lacks(&y, &["hostname:", "users:", "enable_ssh:", "rpi:"]);
}

#[test]
fn independent_step_user_credentials_only_no_ssh() {
    let settings = json!({
        "sshUserName": "alice",
        "sshUserPassword": "$6$rounds=4096$salt$hashvalue",
    });
    let y = user_data(&settings, "", false, false, "pi");
    has(
        &y,
        &[
            "users:",
            "- name: alice",
            GROUPS,
            "shell: /bin/bash",
            "lock_passwd: false",
            "passwd: \"$6$rounds=4096$salt$hashvalue\"",
        ],
    );
    lacks(&y, &["enable_ssh:", "ssh_pwauth:", "ssh_authorized_keys:"]);
    lacks(&y, &["hostname:", "timezone:", "keyboard:", "rpi:"]);
    assert_eth0_only(&network(&settings));
}

#[test]
fn independent_step_wifi_only() {
    let settings = json!({
        "wifiConfigured": true,
        "wifiSSID": "MyHomeNetwork",
        "wifiPasswordCrypt": "hashedwifipassword123",
        "recommendedWifiCountry": "GB",
    });
    let y = user_data(&settings, "", false, false, "pi");
    let n = network(&settings);
    has(&n, &["network:", "version: 2"]);
    has(&n, ETH0_DHCP);
    has(
        &n,
        &[
            "wifis:",
            "wlan0:",
            "\"MyHomeNetwork\":",
            "password: \"hashedwifipassword123\"",
            "regulatory-domain: \"GB\"",
        ],
    );
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
}

#[test]
fn independent_step_ssh_with_password_auth_only() {
    let settings = json!({"sshPasswordAuth": true});
    let y = user_data(&settings, "", false, true, "pi");
    has(&y, &["enable_ssh: true", "ssh_pwauth: true"]);
    lacks(&y, &["users:"]);
    lacks(&y, &["hostname:", "timezone:", "keyboard:", "rpi:"]);
    assert_eth0_only(&network(&settings));
}

#[test]
fn independent_step_ssh_with_public_keys_only() {
    let settings = json!({
        "sshAuthorizedKeys": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIG... user@host",
    });
    let y = user_data(&settings, "", false, true, "defaultuser");
    has(&y, &["enable_ssh: true", "ssh_pwauth: false"]);
    has(
        &y,
        &[
            "users:",
            "- name: defaultuser",
            "ssh_authorized_keys:",
            "- \"ssh-ed25519",
            "lock_passwd: true",
        ],
    );
    lacks(&y, &["hostname:", "timezone:", "keyboard:", "rpi:"]);
}
