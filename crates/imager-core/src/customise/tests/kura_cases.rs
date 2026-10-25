//! Kura's `kura-setup.json` (Etchwise, not upstream). The format is Kura's
//! (`scripts/common/firstboot-setup.sh` in cbabil/kura).

use serde_json::{Value, json};

use super::map;
use crate::customise::{Settings, kura_setup};

fn setup(settings: &Value) -> Value {
    let text = kura_setup(Settings(&map(settings.clone())));
    serde_json::from_str(&text).expect("valid JSON")
}

#[test]
fn writes_a_full_pi_setup() {
    let s = setup(&json!({
        "hostname": "kura-01",
        "sshUserName": "chris",
        "sshUserPassword": "$y$jB5$salt$hash",
        "timezone": "Europe/Paris",
        "keyboard": "fr",
        "sshEnabled": true,
        "sshPasswordAuth": false,
        "sshAuthorizedKeys": "ssh-ed25519 AAAA one@mac\nssh-rsa BBBB two@pc",
        "wifiSSID": "Home Net",
        "wifiPasswordCrypt": "a".repeat(64),
        "wifiHidden": true,
        "recommendedWifiCountry": "FR",
    }));
    assert_eq!(
        s,
        json!({
            "version": 1,
            "hostname": "kura-01",
            "user": {"name": "chris", "passwordHash": "$y$jB5$salt$hash"},
            "ssh": {
                "enabled": true,
                "passwordAuth": false,
                "authorizedKeys": ["ssh-ed25519 AAAA one@mac", "ssh-rsa BBBB two@pc"],
            },
            "wifi": {"ssid": "Home Net", "psk": "a".repeat(64), "hidden": true, "country": "FR"},
            "timezone": "Europe/Paris",
            "keyboard": "fr",
        })
    );
}

#[test]
fn nothing_customised_writes_nothing() {
    assert_eq!(kura_setup(Settings(&map(json!({})))), "");
    // Settings that Kura doesn't use alone produce no file either
    assert_eq!(kura_setup(Settings(&map(json!({"sshEnabled": false})))), "");
}

#[test]
fn keys_left_out_keep_kuras_defaults() {
    let s = setup(&json!({"hostname": "lab"}));
    assert_eq!(s, json!({"version": 1, "hostname": "lab"}));
}

#[test]
fn ssh_off_is_never_sent() {
    // Kura is headless: an unticked SSH box keeps Kura's default (SSH on)
    // instead of locking the owner out.
    let s = setup(
        &json!({"hostname": "lab", "sshEnabled": false, "sshAuthorizedKeys": "ssh-ed25519 AAAA k"}),
    );
    assert!(s.get("ssh").is_none());
}

#[test]
fn an_unsafe_user_name_is_left_out() {
    let s =
        setup(&json!({"hostname": "lab", "sshUserName": "Bad Name", "sshUserPassword": "$y$x$y"}));
    assert!(s.get("user").is_none());
}

#[test]
fn a_user_without_a_password_only_renames() {
    let s = setup(&json!({"sshUserName": "chris"}));
    assert_eq!(s["user"], json!({"name": "chris"}));
}

#[test]
fn wifi_passphrase_becomes_the_derived_psk_and_open_networks_are_skipped() {
    let s = setup(&json!({"wifiSSID": "Home", "wifiPassword": "correct horse"}));
    assert_eq!(s["wifi"]["psk"].as_str().unwrap().len(), 64);
    assert_eq!(s["wifi"]["hidden"], json!(false));
    assert!(s["wifi"].get("country").is_none());
    // Kura's Wi-Fi needs a key; an open network is not sent
    assert_eq!(kura_setup(Settings(&map(json!({"wifiSSID": "Cafe"})))), "");
}

#[test]
fn wifi_switched_off_is_not_sent() {
    let s = setup(&json!({
        "hostname": "lab", "wifiConfigured": false, "wifiSSID": "Home", "wifiPasswordCrypt": "a".repeat(64),
    }));
    assert!(s.get("wifi").is_none());
}
