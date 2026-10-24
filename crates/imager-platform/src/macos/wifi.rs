//! Wi-Fi prefill (upstream `mac/ssid_helper.mm`, `macwlancredentials.cpp`):
//! the current SSID from CoreWLAN (default interface first, then all),
//! falling back to `networksetup -getairportnetwork`, and the saved
//! password from the "AirPort" keychain items, System keychain first.

use std::process::Command;

use objc2_core_wlan::CWWiFiClient;
use security_framework::os::macos::keychain::SecKeychain;
use security_framework::os::macos::passwords::find_generic_password;

const SYSTEM_KEYCHAIN: &str = "/Library/Keychains/System.keychain";
const AIRPORT_SERVICE: &str = "AirPort";

/// SSID of the network this Mac is on, if any. Newer macOS versions only
/// report it to apps with location permission.
pub fn current_ssid() -> Option<String> {
    // SAFETY: CoreWLAN class methods and getters without preconditions.
    let (ssid, interface) = unsafe {
        let client = CWWiFiClient::sharedWiFiClient();
        let ssid = client.interface().and_then(|i| i.ssid()).or_else(|| {
            client
                .interfaces()
                .into_iter()
                .flatten()
                .find_map(|i| i.ssid())
        });
        (
            ssid.map(|s| s.to_string()),
            client
                .interface()
                .and_then(|i| i.interfaceName())
                .map(|n| n.to_string()),
        )
    };
    ssid.or_else(|| networksetup_ssid(&interface?))
}

fn networksetup_ssid(interface: &str) -> Option<String> {
    let out = Command::new("/usr/sbin/networksetup")
        .args(["-getairportnetwork", interface])
        .output()
        .ok()?;
    parse_networksetup(&String::from_utf8_lossy(&out.stdout))
}

/// `Current Wi-Fi Network: <ssid>`.
fn parse_networksetup(output: &str) -> Option<String> {
    output
        .lines()
        .find_map(|l| l.strip_prefix("Current Wi-Fi Network: "))
        .map(|s| s.trim_end().to_owned())
        .filter(|s| !s.is_empty())
}

/// The saved passphrase for `ssid`, from the System keychain, then the
/// user's keychains. macOS asks the user (an administrator for the System
/// keychain) before releasing it; upstream asks for consent first.
pub fn saved_password(ssid: &str) -> Option<String> {
    let system = SecKeychain::open(SYSTEM_KEYCHAIN).ok().map(|k| vec![k]);
    [system, None].into_iter().find_map(|keychains| {
        let (password, _) =
            find_generic_password(keychains.as_deref(), AIRPORT_SERVICE, ssid).ok()?;
        String::from_utf8(password.to_vec()).ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_networksetup_output() {
        assert_eq!(
            parse_networksetup("Current Wi-Fi Network: Home Net\n").as_deref(),
            Some("Home Net")
        );
        assert_eq!(
            parse_networksetup("You are not associated with an AirPort network.\n"),
            None
        );
        assert_eq!(parse_networksetup("Current Wi-Fi Network: \n"), None);
    }

    #[test]
    fn missing_keychain_items_are_none() {
        assert_eq!(
            saved_password("etchwise-test-network-that-does-not-exist"),
            None
        );
        let _ = current_ssid();
    }
}
