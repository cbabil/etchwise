//! What customisation writes to the boot partition, ported from upstream
//! `ImageWriter::applyCustomisationFromSettings` (payload assembly) and
//! `DownloadThread::_customizeImage` (applying it).

use std::io::{self, Read, Seek, Write};

use super::cloudinit::{UserDataOptions, cloudinit_user_data};
use super::cloudinit_extras::cloudinit_network_config;
use super::systemd::systemd_script;
use super::text::Settings;
use crate::fat::BootPartition;

/// Payloads staged for one write (`setImageCustomisation`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CustomisationPlan {
    /// `config.txt` lines to enable (newline separated). Upstream's GUI
    /// always stages none (imagewriter.cpp:3333/3372); kept for parity.
    pub config: String,
    /// Appended to `cmdline.txt`, each item with a leading space.
    pub cmdline: String,
    pub firstrun: String,
    pub cloudinit: String,
    pub network: String,
    /// RSA key (PEM file) to sign the boot partition with after the
    /// edits, as upstream's `_createSecureBootFiles`; empty: no signing.
    pub secure_boot_key_path: String,
    pub init_format: String,
}

/// Facts outside the settings map.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlanInputs<'a> {
    /// The selected OS's `init_format` (empty: no customisation).
    pub init_format: &'a str,
    pub pi_connect_token: &'a str,
    pub current_user: &'a str,
    /// The `secureboot_rsa_key` setting (path to an RSA-2048 PEM key).
    pub secure_boot_key_path: &'a str,
}

/// Builds the plan for the selected OS.
pub fn plan_from_settings(settings: Settings<'_>, inputs: PlanInputs<'_>) -> CustomisationPlan {
    let init_format = inputs.init_format.to_owned();
    let mut plan = CustomisationPlan {
        init_format,
        ..CustomisationPlan::default()
    };
    match inputs.init_format {
        "" => return plan,
        "systemd" => plan.firstrun = systemd_script(settings, inputs.pi_connect_token),
        _ => {
            let options = UserDataOptions {
                pi_connect_token: inputs.pi_connect_token,
                has_cc_rpi: inputs.init_format == "cloudinit-rpi",
                ssh_enabled: settings.bool("sshEnabled"),
                current_user: inputs.current_user,
            };
            plan.cloudinit = cloudinit_user_data(settings, options);
            plan.network = cloudinit_network_config(settings);
        }
    }
    if plan.firstrun.is_empty() && plan.cloudinit.is_empty() && plan.network.is_empty() {
        return plan;
    }
    let country = settings.trimmed("recommendedWifiCountry");
    if !country.is_empty() {
        plan.cmdline = format!(" cfg80211.ieee80211_regdom={country}");
    }
    if settings.bool("secureBootEnabled") && !inputs.secure_boot_key_path.is_empty() {
        plan.secure_boot_key_path = inputs.secure_boot_key_path.to_owned();
    }
    plan
}

/// Writes the plan's files and edits `config.txt` / `cmdline.txt`.
/// `instance_id` is `rpi-imager-<ms since epoch>` upstream.
///
/// # Errors
/// Fails when the boot filesystem cannot be read or written.
pub fn apply_plan<D: Read + Write + Seek>(
    boot: &mut BootPartition<D>,
    plan: &CustomisationPlan,
    instance_id: &str,
) -> io::Result<()> {
    if !plan.config.is_empty() {
        let existing = boot.read_file("config.txt")?.unwrap_or_default();
        boot.write_file("config.txt", &merge_config(&existing, &plan.config))?;
    }
    let mut cmdline = plan.cmdline.clone();
    if !plan.firstrun.is_empty() && plan.init_format == "systemd" {
        boot.write_file("firstrun.sh", plan.firstrun.as_bytes())?;
        cmdline.push_str(" systemd.run=/boot/firstrun.sh systemd.run_success_action=reboot systemd.unit=kernel-command-line.target");
    }
    let cloud = plan.init_format == "cloudinit" || plan.init_format == "cloudinit-rpi";
    if cloud && (!plan.cloudinit.is_empty() || !plan.network.is_empty()) {
        boot.write_file(
            "meta-data",
            format!("instance-id: {instance_id}\n").as_bytes(),
        )?;
        cmdline.push_str(&format!(" ds=nocloud;i={instance_id}"));
        if !plan.cloudinit.is_empty() {
            boot.write_file(
                "user-data",
                format!("#cloud-config\n{}", plan.cloudinit).as_bytes(),
            )?;
        }
        if !plan.network.is_empty() {
            boot.write_file("network-config", plan.network.as_bytes())?;
        }
    }
    if !cmdline.is_empty() {
        let existing = boot.read_file("cmdline.txt")?.unwrap_or_default();
        let mut updated = existing.trim_ascii().to_vec();
        updated.extend_from_slice(cmdline.as_bytes());
        boot.write_file("cmdline.txt", &updated)?;
    }
    if !plan.secure_boot_key_path.is_empty() {
        sign(boot, &plan.secure_boot_key_path)?;
    }
    Ok(())
}

/// Signs the edited partition (upstream: after the edits are synced, which
/// the shared block cache makes implicit).
fn sign<D: Read + Write + Seek>(boot: &mut BootPartition<D>, key_path: &str) -> io::Result<()> {
    let key = std::fs::read_to_string(key_path)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    crate::secureboot::sign_boot_partition(boot, &key, now)
        .map_err(|e| io::Error::other(format!("Failed to create secure boot files: {e}")))
}

/// Uncomments `#item`, keeps items already on a line of their own, else
/// appends them, byte for byte as upstream.
pub fn merge_config(existing: &[u8], items: &str) -> Vec<u8> {
    let mut config = existing.to_vec();
    for item in items.split('\n').filter(|i| !i.is_empty()) {
        let commented = format!("#{item}");
        if contains(&config, commented.as_bytes()) {
            config = replace_all(&config, commented.as_bytes(), item.as_bytes());
        } else if !contains(&config, format!("\n{item}").as_bytes()) {
            if config.last() != Some(&b'\n') {
                config.push(b'\n');
            }
            config.extend_from_slice(item.as_bytes());
            config.push(b'\n');
        }
    }
    config
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

fn replace_all(haystack: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(haystack.len());
    let mut i = 0;
    while i < haystack.len() {
        if haystack[i..].starts_with(from) {
            out.extend_from_slice(to);
            i += from.len();
        } else {
            out.push(haystack[i]);
            i += 1;
        }
    }
    out
}
