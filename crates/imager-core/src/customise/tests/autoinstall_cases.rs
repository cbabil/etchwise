//! Ubuntu Server autoinstall `user-data` for PCs (Etchwise, not upstream).

use serde_json::json;

use super::helpers::{has, lacks};
use super::map;
use crate::customise::{Settings, autoinstall_user_data};

fn user_data(settings: &serde_json::Value) -> String {
    autoinstall_user_data(Settings(&map(settings.clone())))
}

#[test]
fn writes_a_full_nuc_setup() {
    let y = user_data(&json!({
        "hostname": "nuc-01",
        "sshUserName": "chris",
        "sshUserPassword": "$y$jB5$salt$hash",
        "timezone": "Europe/Paris",
        "keyboard": "fr",
        "sshEnabled": true,
        "sshPasswordAuth": false,
        "sshAuthorizedKeys": "ssh-ed25519 AAAA key@mac",
        "wifiSSID": "Home \"5G\"",
        "wifiPasswordCrypt": "a".repeat(64),
        "recommendedWifiCountry": "FR",
    }));
    let expected = [
        "autoinstall:",
        "  version: 1",
        "  identity:",
        "    hostname: \"nuc-01\"",
        "    username: \"chris\"",
        "    password: \"$y$jB5$salt$hash\"",
        "  timezone: \"Europe/Paris\"",
        "  keyboard:",
        "    layout: \"fr\"",
        "  ssh:",
        "    install-server: true",
        "    allow-pw: false",
        "    authorized-keys:",
        "      - \"ssh-ed25519 AAAA key@mac\"",
        "  network:",
        "    version: 2",
        "    ethernets:",
        "      lan:",
        "        match:",
        "          name: \"en*\"",
        "        dhcp4: true",
        "        optional: true",
        "    wifis:",
        "      wlan:",
        "        match:",
        "          name: \"wl*\"",
        "        dhcp4: true",
        "        optional: true",
        "        regulatory-domain: \"FR\"",
        "        access-points:",
        "          \"Home \\\"5G\\\"\":",
        &format!("            password: \"{}\"", "a".repeat(64)),
        "  storage:",
        "    layout:",
        "      name: lvm",
        "",
    ]
    .join("\n");
    assert_eq!(y, expected);
}

#[test]
fn leaves_networking_to_the_installer_without_wifi() {
    let y = user_data(&json!({"hostname": "nuc", "sshUserName": "u", "sshUserPassword": "$5$x"}));
    lacks(&y, &["network:", "wifis:", "ssh:"]);
    has(&y, &["identity:", "storage:"]);
}

#[test]
fn marks_hidden_networks() {
    let y = user_data(&json!({
        "sshUserName": "u", "sshUserPassword": "$5$x",
        "wifiSSID": "lab", "wifiPasswordCrypt": "p".repeat(64), "wifiHidden": true,
    }));
    has(&y, &["          \"lab\":", "            hidden: true"]);
    lacks(&y, &["regulatory-domain"]);
}

#[test]
fn asks_on_the_nuc_for_a_missing_user() {
    let y = user_data(&json!({"hostname": "nuc", "timezone": "UTC"}));
    has(&y, &["  interactive-sections:", "    - identity"]);
    lacks(&y, &["  identity:"]);
}

#[test]
fn grants_passwordless_sudo_when_asked() {
    let y = user_data(&json!({
        "sshUserName": "chris", "sshUserPassword": "$5$x", "passwordlessSudo": true,
    }));
    has(
        &y,
        &[
            "  late-commands:",
            "    - \"echo 'chris ALL=(ALL) NOPASSWD:ALL' > /target/etc/sudoers.d/90-etchwise\"",
            "    - \"chmod 0440 /target/etc/sudoers.d/90-etchwise\"",
        ],
    );
}

#[test]
fn is_empty_without_any_setting() {
    assert_eq!(user_data(&json!({})), "");
}

#[test]
fn plans_only_the_autoinstall_user_data() {
    use crate::customise::plan::{AUTOINSTALL_UBUNTU, PlanInputs, plan_from_settings};
    let m = map(json!({
        "sshUserName": "u", "sshUserPassword": "$5$x",
        "recommendedWifiCountry": "GB", "secureBootEnabled": true, "wifiSSID": "n",
    }));
    let inputs = PlanInputs {
        init_format: AUTOINSTALL_UBUNTU,
        secure_boot_key_path: "/k.pem",
        ..PlanInputs::default()
    };
    let p = plan_from_settings(Settings(&m), inputs);
    assert!(p.cloudinit.starts_with("autoinstall:\n"));
    assert!(p.needs_seed());
    let [(user_name, user), (meta_name, meta)] = p.seed_files("etchwise-7");
    assert_eq!((user_name, meta_name), ("user-data", "meta-data"));
    assert_eq!(user, format!("#cloud-config\n{}", p.cloudinit).into_bytes());
    assert_eq!(meta, b"instance-id: etchwise-7\n");
    assert_eq!(p.init_format, "autoinstall-ubuntu");
    assert!(p.network.is_empty() && p.cmdline.is_empty() && p.config.is_empty());
    assert!(p.firstrun.is_empty() && p.secure_boot_key_path.is_empty());
}

#[test]
fn never_puts_an_unsafe_username_into_the_sudo_command() {
    let y = user_data(&json!({
        "sshUserName": "x'; rm -rf /target; '", "sshUserPassword": "$5$x", "passwordlessSudo": true,
    }));
    lacks(&y, &["late-commands:", "rm -rf"]);
    has(&y, &["    - identity"]);
}

#[test]
fn needs_a_seed_only_for_autoinstall_with_content() {
    use crate::customise::plan::{AUTOINSTALL_UBUNTU, CustomisationPlan};
    let plan = |init_format: &str, cloudinit: &str| CustomisationPlan {
        init_format: init_format.into(),
        cloudinit: cloudinit.into(),
        ..CustomisationPlan::default()
    };
    assert!(plan(AUTOINSTALL_UBUNTU, "autoinstall:\n").needs_seed());
    assert!(
        !plan(AUTOINSTALL_UBUNTU, "").needs_seed(),
        "nothing set up: plain ISO"
    );
    assert!(!plan("cloudinit", "hostname: x\n").needs_seed());
}
