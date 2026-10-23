//! Wi-Fi section of `firstrun.sh` (upstream `generateSystemdScript`).

use super::systemd::{IF_IMAGER_CUSTOM, IMAGER_CUSTOM, Script};
use super::text::{Settings, shell_quote};

pub(super) fn wifi(out: &mut Script, settings: Settings<'_>) {
    let ssid = settings.ssid();
    let country = settings.trimmed("recommendedWifiCountry");
    if ssid.is_empty() {
        if !country.is_empty() {
            out.lines(&[
                "rfkill unblock wifi",
                "for filename in /var/lib/systemd/rfkill/*:wlan ; do",
                "  echo 0 > $filename",
                "done",
            ]);
        }
        return;
    }
    let hidden = (settings.wifi_configured() && settings.bool("wifiHidden"))
        || settings.bool("wifiSSIDHidden");
    let psk = settings.effective_psk(&ssid);
    out.line(IF_IMAGER_CUSTOM);
    let flag = if hidden { " -h " } else { "" };
    out.line(format!(
        "   {IMAGER_CUSTOM} set_wlan {flag}{} {} {}",
        shell_quote(&ssid),
        shell_quote(&psk),
        shell_quote(&country)
    ));
    out.lines(&[
        "else",
        "cat >/etc/wpa_supplicant/wpa_supplicant.conf <<'WPAEOF'",
    ]);
    if !country.is_empty() {
        out.line(format!("country={country}"));
    }
    out.lines(&[
        "ctrl_interface=DIR=/var/run/wpa_supplicant GROUP=netdev",
        "ap_scan=1",
        "",
        "update_config=1",
        "network={",
    ]);
    if hidden {
        out.line("\tscan_ssid=1");
    }
    out.line(format!(
        "\tssid=\"{}\"",
        ssid.replace('\\', "\\\\").replace('"', "\\\"")
    ));
    if psk.is_empty() {
        out.line("\tkey_mgmt=NONE");
    } else {
        out.line("\tkey_mgmt=WPA-PSK SAE");
        out.line(format!("\tpsk={psk}"));
        out.line("\tieee80211w=1");
    }
    out.lines(&[
        "}",
        "WPAEOF",
        "   chmod 600 /etc/wpa_supplicant/wpa_supplicant.conf",
        "   rfkill unblock wifi",
        "   for filename in /var/lib/systemd/rfkill/*:wlan ; do",
        "       echo 0 > $filename",
        "   done",
        "fi",
    ]);
}
