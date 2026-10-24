//! Ubuntu Server autoinstall `user-data` for PCs such as the Intel NUC (not
//! in upstream). Ubuntu's installer reads it from a NoCloud `CIDATA` volume;
//! it asks "Continue with autoinstall?" before touching disks, because the
//! kernel command line has no `autoinstall` keyword.

use super::cloudinit::Yaml;
use super::text::{Settings, ssh_keys, yaml_quoted as quoted};

/// Where passwordless sudo is granted on the installed system.
const SUDOERS_FILE: &str = "/target/etc/sudoers.d/90-etchwise";

/// Generates the `user-data` body (the `#cloud-config` header is added when
/// the file is written); empty when nothing is customised.
pub fn autoinstall_user_data(settings: Settings<'_>) -> String {
    if settings.0.is_empty() {
        return String::new();
    }
    let mut out = Yaml::default();
    out.lines(&["autoinstall:", "  version: 1"]);
    identity(&mut out, settings);
    locale(&mut out, settings);
    ssh(&mut out, settings);
    network(&mut out, settings);
    out.lines(&["  storage:", "    layout:", "      name: lvm"]);
    passwordless_sudo(&mut out, settings);
    out.finish()
}

/// The installer requires a valid user with a password; without one it asks.
fn identity(out: &mut Yaml, s: Settings<'_>) {
    let (user, password) = (s.trimmed("sshUserName"), s.string("sshUserPassword"));
    if !is_safe_username(&user) || password.is_empty() {
        out.lines(&["  interactive-sections:", "    - identity"]);
        return;
    }
    let hostname = s.trimmed("hostname");
    out.push("  identity:");
    out.push(format!(
        "    hostname: {}",
        quoted(if hostname.is_empty() {
            "ubuntu"
        } else {
            &hostname
        })
    ));
    out.push(format!("    username: {}", quoted(&user)));
    out.push(format!("    password: {}", quoted(&password)));
}

fn locale(out: &mut Yaml, s: Settings<'_>) {
    let timezone = s.trimmed("timezone");
    if !timezone.is_empty() {
        out.push(format!("  timezone: {}", quoted(&timezone)));
    }
    let keyboard = s.trimmed("keyboard");
    if !keyboard.is_empty() {
        out.push("  keyboard:");
        out.push(format!("    layout: {}", quoted(&keyboard)));
    }
}

fn ssh(out: &mut Yaml, s: Settings<'_>) {
    if !s.bool("sshEnabled") {
        return;
    }
    out.lines(&["  ssh:", "    install-server: true"]);
    out.push(format!("    allow-pw: {}", s.bool("sshPasswordAuth")));
    let keys: Vec<String> = ssh_keys(s).into_iter().filter(|k| !k.is_empty()).collect();
    if !keys.is_empty() {
        out.push("    authorized-keys:");
        keys.iter()
            .for_each(|k| out.push(format!("      - {}", quoted(k))));
    }
}

/// Netplan for any wired and wireless interface (NUC names vary: en*, wl*).
fn network(out: &mut Yaml, s: Settings<'_>) {
    let ssid = s.ssid();
    if ssid.is_empty() {
        return;
    }
    out.lines(&[
        "  network:",
        "    version: 2",
        "    ethernets:",
        "      lan:",
    ]);
    interface(out, "en*");
    out.lines(&["    wifis:", "      wlan:"]);
    interface(out, "wl*");
    let country = s.trimmed("recommendedWifiCountry");
    if !country.is_empty() {
        out.push(format!("        regulatory-domain: {}", quoted(&country)));
    }
    out.push("        access-points:");
    out.push(format!("          {}:", quoted(&ssid)));
    let psk = s.effective_psk(&ssid);
    if !psk.is_empty() {
        out.push(format!("            password: {}", quoted(&psk)));
    }
    if s.bool("wifiHidden") {
        out.push("            hidden: true");
    }
}

fn interface(out: &mut Yaml, pattern: &str) {
    out.push("        match:");
    out.push(format!("          name: {}", quoted(pattern)));
    out.lines(&["        dhcp4: true", "        optional: true"]);
}

/// The UI's username rule (`[a-z_][a-z0-9_-]*`), checked again because the
/// name goes into a shell command the installer runs as root.
fn is_safe_username(user: &str) -> bool {
    let mut chars = user.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c == '_')
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

fn passwordless_sudo(out: &mut Yaml, s: Settings<'_>) {
    let user = s.trimmed("sshUserName");
    if !s.bool("passwordlessSudo") || !is_safe_username(&user) {
        return;
    }
    let rule = format!("echo '{user} ALL=(ALL) NOPASSWD:ALL' > {SUDOERS_FILE}");
    out.push("  late-commands:");
    out.push(format!("    - {}", quoted(&rule)));
    out.push(format!(
        "    - {}",
        quoted(&format!("chmod 0440 {SUDOERS_FILE}"))
    ));
}
