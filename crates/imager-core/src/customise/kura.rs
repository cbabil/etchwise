//! `kura-setup.json` for Kura images (not in upstream): written to the boot
//! partition, applied and deleted by Kura's first boot. The format is Kura's
//! (`scripts/common/firstboot-setup.sh` in cbabil/kura); a key left out keeps
//! Kura's default.

use serde_json::{Map, Value, json};

use super::text::{Settings, is_safe_username, ssh_keys};

/// The setup file's version this generator writes.
const VERSION: u64 = 1;

/// Generates the file's contents; empty when nothing is customised.
pub fn kura_setup(settings: Settings<'_>) -> String {
    let mut out = Map::new();
    let mut put = |key: &str, value: Option<Value>| {
        if let Some(value) = value {
            out.insert(key.to_owned(), value);
        }
    };
    put("hostname", non_empty(settings.trimmed("hostname")));
    put("user", user(settings));
    put("ssh", ssh(settings));
    put("wifi", wifi(settings));
    put("timezone", non_empty(settings.trimmed("timezone")));
    put("keyboard", non_empty(settings.trimmed("keyboard")));
    if out.is_empty() {
        return String::new();
    }
    let mut file = Map::new();
    file.insert("version".into(), json!(VERSION));
    file.extend(out);
    Value::Object(file).to_string()
}

fn non_empty(value: String) -> Option<Value> {
    (!value.is_empty()).then(|| Value::String(value))
}

/// The password is already a crypt hash (the UI hashes it before saving).
fn user(s: Settings<'_>) -> Option<Value> {
    let name = s.trimmed("sshUserName");
    if !is_safe_username(&name) {
        return None;
    }
    let mut user = json!({"name": name});
    if let Some(hash) = non_empty(s.string("sshUserPassword")) {
        user["passwordHash"] = hash;
    }
    Some(user)
}

/// Only an enabled SSH is sent: Kura is headless, so an unticked box keeps
/// Kura's default (SSH on) rather than locking the owner out.
fn ssh(s: Settings<'_>) -> Option<Value> {
    if !s.bool("sshEnabled") {
        return None;
    }
    let keys: Vec<String> = ssh_keys(s).into_iter().filter(|k| !k.is_empty()).collect();
    let mut ssh = json!({"enabled": true, "passwordAuth": s.bool("sshPasswordAuth")});
    if !keys.is_empty() {
        ssh["authorizedKeys"] = json!(keys);
    }
    Some(ssh)
}

/// Kura's Wi-Fi needs a key, so an open network is not sent.
fn wifi(s: Settings<'_>) -> Option<Value> {
    let ssid = s.ssid();
    let psk = s.effective_psk(&ssid);
    if ssid.is_empty() || psk.is_empty() {
        return None;
    }
    let mut wifi = json!({"ssid": ssid, "psk": psk, "hidden": s.bool("wifiHidden")});
    if let Some(country) = non_empty(s.trimmed("recommendedWifiCountry")) {
        wifi["country"] = country;
    }
    Some(wifi)
}
