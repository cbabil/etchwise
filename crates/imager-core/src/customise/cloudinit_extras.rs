//! SSH daemon and `rpi:` interface sections of cloud-init `user-data`, and
//! the netplan `network-config` (upstream
//! `generateCloudInitNetworkConfig`).

use super::cloudinit::Yaml;
use super::text::{Settings, ssh_keys, yaml_escape};

pub(super) fn ssh_daemon(out: &mut Yaml, settings: Settings<'_>, ssh_enabled: bool) {
    if !ssh_enabled {
        return;
    }
    out.push("enable_ssh: true");
    if settings.bool("sshPasswordAuth") {
        out.push("ssh_pwauth: true");
    } else if !ssh_keys(settings).is_empty() {
        out.push("ssh_pwauth: false");
    }
}

pub(super) fn rpi_interfaces(out: &mut Yaml, settings: Settings<'_>, has_cc_rpi: bool) {
    let (i2c, spi, onewire) = (
        settings.bool("enableI2C"),
        settings.bool("enableSPI"),
        settings.bool("enable1Wire"),
    );
    let serial = settings.string("enableSerial");
    let interfaces = i2c || spi || onewire || serial != "Disabled";
    let gadget = settings.bool("enableUsbGadget");
    if !has_cc_rpi || !(gadget || interfaces) {
        return;
    }
    out.push("rpi:");
    if gadget {
        out.push("  enable_usb_gadget: true");
    }
    if !interfaces {
        return;
    }
    out.push("  interfaces:");
    for (enabled, line) in [
        (i2c, "    i2c: true"),
        (spi, "    spi: true"),
        (onewire, "    onewire: true"),
    ] {
        if enabled {
            out.push(line);
        }
    }
    let (console, hardware) = match serial.as_str() {
        "Disabled" => return,
        "" | "Default" => return out.push("    serial: true"),
        "Console & Hardware" => (Some(true), Some(true)),
        "Console" => (Some(true), Some(false)),
        "Hardware" => (Some(false), Some(true)),
        _ => (None, None),
    };
    out.push("    serial:");
    if let (Some(console), Some(hardware)) = (console, hardware) {
        out.push(format!("      console: {console}"));
        out.push(format!("      hardware: {hardware}"));
    }
}

/// Generates netplan v2 `network-config`; empty without an SSID.
pub fn cloudinit_network_config(settings: Settings<'_>) -> String {
    let mut out = Yaml::default();
    let ssid = settings.ssid();
    if ssid.is_empty() {
        return out.finish();
    }
    let hidden = settings.wifi_configured() && settings.bool("wifiHidden");
    let regdom = settings.trimmed("recommendedWifiCountry").to_uppercase();
    out.lines(&[
        "network:",
        "  version: 2",
        "  ethernets:",
        "    eth0:",
        "      dhcp4: true",
        "      dhcp6: true",
        "      optional: true",
        "  wifis:",
        "    wlan0:",
        "      dhcp4: true",
    ]);
    if !regdom.is_empty() {
        out.push(format!("      regulatory-domain: \"{regdom}\""));
    }
    out.push("      access-points:");
    out.push(format!("        \"{}\":", yaml_escape(&ssid)));
    if hidden {
        out.push("          hidden: true");
    }
    let psk = settings.effective_psk(&ssid);
    if psk.is_empty() {
        out.push("          auth:");
        out.push("            key-management: none");
    } else {
        out.push(format!(
            "          password: \"{}\"",
            psk.replace('"', "\\\"")
        ));
    }
    out.push("      optional: true");
    out.finish()
}
