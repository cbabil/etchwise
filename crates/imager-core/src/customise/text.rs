//! Text helpers shared by the customisation generators: settings access
//! with QVariant semantics, shell and YAML quoting, SSH key splitting and
//! the Wi-Fi PSK derivation.

use pbkdf2::pbkdf2_hmac;
use serde_json::{Map, Value};
use sha1::Sha1;

use crate::settings::{variant_to_bool, variant_to_string};

/// Read-only view of a customisation settings map (`QVariantMap`).
#[derive(Clone, Copy)]
pub struct Settings<'a>(pub &'a Map<String, Value>);

impl Settings<'_> {
    /// `value(key).toString()`.
    pub fn string(&self, key: &str) -> String {
        self.0.get(key).map(variant_to_string).unwrap_or_default()
    }

    /// `value(key).toString().trimmed()`.
    pub fn trimmed(&self, key: &str) -> String {
        self.string(key).trim().to_owned()
    }

    /// `value(key).toBool()`.
    pub fn bool(&self, key: &str) -> bool {
        self.0.get(key).is_some_and(variant_to_bool)
    }

    /// Wi-Fi is configured unless `wifiConfigured` is explicitly false.
    pub fn wifi_configured(&self) -> bool {
        self.0.get("wifiConfigured").is_none_or(variant_to_bool)
    }

    /// The SSID when Wi-Fi is configured.
    pub fn ssid(&self) -> String {
        if self.wifi_configured() {
            self.string("wifiSSID")
        } else {
            String::new()
        }
    }

    /// Persisted crypted PSK, else one derived from the legacy plaintext
    /// `wifiPassword` (8-63 UTF-16 units is a passphrase; anything else is
    /// used as-is).
    pub fn effective_psk(&self, ssid: &str) -> String {
        let crypted = if self.wifi_configured() {
            self.string("wifiPasswordCrypt")
        } else {
            String::new()
        };
        if !crypted.is_empty() {
            return crypted;
        }
        let legacy = self.string("wifiPassword");
        let units = legacy.encode_utf16().count();
        if legacy.is_empty() || !(8..64).contains(&units) {
            legacy
        } else {
            pbkdf2_psk(legacy.as_bytes(), ssid.as_bytes())
        }
    }
}

/// WPA PSK: PBKDF2-HMAC-SHA1(passphrase, ssid, 4096, 32 bytes), hex.
pub fn pbkdf2_psk(password: &[u8], ssid: &[u8]) -> String {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha1>(password, ssid, 4096, &mut key);
    hex::encode(key)
}

/// Single-quotes for POSIX sh: `'` becomes `'"'"'`.
pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

/// Escapes a string for a YAML double-quoted scalar, byte-for-byte as
/// upstream `yamlEscapeString`.
pub fn yaml_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len() * 2);
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\0' => out.push_str("\\0"),
            c if (c as u32) < 0x20 || c == '\u{7f}' => {
                out.push_str(&format!("\\x{:02x}", c as u32))
            }
            c => out.push(c),
        }
    }
    out
}

/// A YAML double-quoted scalar: `"` + [`yaml_escape`] + `"`.
pub fn yaml_quoted(value: &str) -> String {
    format!("\"{}\"", yaml_escape(value))
}

/// Splits on `\r?\n`, skipping empty parts, then trims each part (so a
/// whitespace-only line yields an empty key, as upstream).
pub fn split_keys(raw: &str) -> Vec<String> {
    raw.split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .filter(|l| !l.is_empty())
        .map(|l| l.trim().to_owned())
        .collect()
}

/// `sshAuthorizedKeys` lines, else `sshPublicKey` lines; non-empty exactly
/// when either trimmed setting is.
pub fn ssh_keys(s: Settings<'_>) -> Vec<String> {
    let authorized = s.trimmed("sshAuthorizedKeys");
    if authorized.is_empty() {
        split_keys(&s.trimmed("sshPublicKey"))
    } else {
        split_keys(&authorized)
    }
}

/// The UI's username rule (`[a-z_][a-z0-9_-]*`), checked again because the
/// name reaches commands run as root on the device.
pub fn is_safe_username(user: &str) -> bool {
    let mut chars = user.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c == '_')
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// Groups the first user is added to.
pub const USER_GROUPS: &str =
    "users,adm,dialout,audio,netdev,video,plugdev,cdrom,games,input,gpio,spi,i2c,render,sudo";
/// Raspberry Pi Connect config dir under the user's home.
pub const PI_CONNECT_CONFIG_PATH: &str = ".config/com.raspberrypi.connect";
/// Raspberry Pi Connect systemd user units and the `.target.wants` they
/// are enabled in.
pub const PI_CONNECT_UNITS: [(&str, &str); 3] = [
    ("rpi-connect.service", "default"),
    ("rpi-connect-signin.path", "paths"),
    ("rpi-connect-wayvnc.service", "default"),
];
/// Raspberry Pi Connect deploy key file name.
pub const PI_CONNECT_DEPLOY_KEY_FILENAME: &str = "auth.key";

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn quotes_and_escapes() {
        assert_eq!(shell_quote("it's"), "'it'\"'\"'s'");
        assert_eq!(
            yaml_escape("a\"b\\c\n\u{1}\u{7f}\0é"),
            "a\\\"b\\\\c\\n\\x01\\x7f\\0é"
        );
        assert_eq!(yaml_escape("\t\r\u{8}\u{c}"), "\\t\\r\\b\\f");
    }

    #[test]
    fn derives_wpa_psk() {
        // IEEE 802.11i test vector: passphrase "password", SSID "IEEE".
        assert_eq!(
            pbkdf2_psk(b"password", b"IEEE"),
            "f42c6fc52df0ebef9ebb4b90b38a5f902e83fe1b135a70e23aed762e9710a12e"
        );
    }

    #[test]
    fn effective_psk_prefers_crypted_then_legacy() {
        let m = json!({"wifiPasswordCrypt": "abc", "wifiPassword": "password"});
        assert_eq!(
            Settings(m.as_object().unwrap()).effective_psk("IEEE"),
            "abc"
        );
        let m = json!({"wifiPassword": "password"});
        assert_eq!(
            Settings(m.as_object().unwrap()).effective_psk("IEEE"),
            pbkdf2_psk(b"password", b"IEEE")
        );
        let hex64 = "a".repeat(64);
        let m = json!({"wifiPassword": hex64});
        assert_eq!(
            Settings(m.as_object().unwrap()).effective_psk("x"),
            "a".repeat(64)
        );
        let m = json!({"wifiConfigured": false, "wifiPasswordCrypt": "abc", "wifiSSID": "n"});
        let s = Settings(m.as_object().unwrap());
        assert_eq!(s.effective_psk("x"), "");
        assert_eq!(s.ssid(), "");
    }

    #[test]
    fn splits_keys_like_upstream() {
        assert_eq!(split_keys("a\r\n\n b \n  \n"), ["a", "b", ""]);
        let m = json!({"sshPublicKey": "k1\nk2", "sshAuthorizedKeys": " "});
        assert_eq!(ssh_keys(Settings(m.as_object().unwrap())), ["k1", "k2"]);
    }
}
