//! cloud-init `user-data`, ported from upstream
//! `CustomisationGenerator::generateCloudInitUserData`. Empty lines are
//! never emitted (upstream's `push` skips them).

use super::text::{
    PI_CONNECT_CONFIG_PATH, PI_CONNECT_DEPLOY_KEY_FILENAME, PI_CONNECT_UNITS, Settings,
    USER_GROUPS, shell_quote, ssh_keys,
};

/// Inputs beyond the settings map.
#[derive(Debug, Clone, Copy, Default)]
pub struct UserDataOptions<'a> {
    pub pi_connect_token: &'a str,
    /// The OS supports the `cc_raspberry_pi` module (`cloudinit-rpi`).
    pub has_cc_rpi: bool,
    pub ssh_enabled: bool,
    /// Login name on this computer, used for key-only SSH without a username.
    pub current_user: &'a str,
}

/// Appends non-empty lines.
#[derive(Default)]
pub(super) struct Yaml(String);

impl Yaml {
    pub(super) fn lines(&mut self, lines: &[&str]) {
        lines.iter().for_each(|l| self.push(l));
    }

    pub(super) fn finish(self) -> String {
        self.0
    }

    pub(super) fn push(&mut self, line: impl AsRef<str>) {
        let line = line.as_ref();
        if !line.is_empty() {
            self.0.push_str(line);
            self.0.push('\n');
        }
    }
}

/// Generates `user-data`; empty when nothing is customised.
pub fn cloudinit_user_data(settings: Settings<'_>, options: UserDataOptions<'_>) -> String {
    let mut out = Yaml::default();
    hostname_and_locale(&mut out, settings);
    let user = settings.trimmed("sshUserName");
    let effective_user = match (user.is_empty(), options.ssh_enabled) {
        (false, _) => user.clone(),
        (true, true) => options.current_user.to_owned(),
        (true, false) => "pi".to_owned(),
    };
    users(&mut out, settings, options.ssh_enabled, &effective_user);
    super::cloudinit_extras::ssh_daemon(&mut out, settings, options.ssh_enabled);
    super::cloudinit_extras::rpi_interfaces(&mut out, settings, options.has_cc_rpi);
    let token = options.pi_connect_token.trim();
    let for_connect = settings.bool("piConnectEnabled") && !token.is_empty();
    let for_wifi =
        !settings.trimmed("recommendedWifiCountry").is_empty() && settings.ssid().is_empty();
    if for_connect || for_wifi {
        out.push("runcmd:");
        if for_connect {
            pi_connect(&mut out, &effective_user, token);
        }
        if for_wifi {
            out.push("  - [ rfkill, unblock, wifi ]");
            out.push("  - [ sh, -c, \"for f in /var/lib/systemd/rfkill/*:wlan; do echo 0 > \\\"$f\\\"; done\" ]");
        }
    }
    let yaml = out.finish();
    if yaml.is_empty() {
        yaml
    } else {
        format!(
            "manage_resolv_conf: false

{yaml}"
        )
    }
}

fn hostname_and_locale(out: &mut Yaml, settings: Settings<'_>) {
    let hostname = settings.trimmed("hostname");
    if !hostname.is_empty() {
        out.push(format!("hostname: {hostname}"));
        out.lines(&[
            "manage_etc_hosts: true",
            "packages:",
            "- avahi-daemon",
            "apt:",
            "  preserve_sources_list: true",
            "  conf: |",
            "    Acquire {",
            "      Check-Date \"false\";",
            "    };",
        ]);
    }
    let timezone = settings.trimmed("timezone");
    if !timezone.is_empty() {
        out.push(format!("timezone: {timezone}"));
    }
    let keyboard = settings.trimmed("keyboard");
    if !keyboard.is_empty() {
        out.push("keyboard:");
        out.push("  model: pc105");
        out.push(format!("  layout: \"{keyboard}\""));
    }
}

fn users(out: &mut Yaml, settings: Settings<'_>, ssh_enabled: bool, effective_user: &str) {
    let pass = settings.string("sshUserPassword");
    let keys = ssh_keys(settings);
    let has_credentials = !settings.trimmed("sshUserName").is_empty() && !pass.is_empty();
    let has_keys = ssh_enabled && !keys.is_empty();
    if !has_credentials && !has_keys {
        return;
    }
    out.push("users:");
    out.push(format!("- name: {effective_user}"));
    out.push(format!("  groups: {USER_GROUPS}"));
    out.push("  shell: /bin/bash");
    if !pass.is_empty() {
        out.push("  lock_passwd: false");
        out.push(format!("  passwd: \"{pass}\""));
    } else if has_keys {
        out.push("  lock_passwd: true");
    }
    if has_keys {
        out.push("  ssh_authorized_keys:");
        for key in keys {
            out.push(format!("    - \"{key}\""));
        }
    }
    if settings.bool("passwordlessSudo") {
        out.push("  sudo: ALL=(ALL) NOPASSWD:ALL");
    }
}

fn pi_connect(out: &mut Yaml, user: &str, token: &str) {
    let home = format!("/home/{user}");
    let config_dir = format!("{home}/{PI_CONNECT_CONFIG_PATH}");
    let target = format!("{config_dir}/{PI_CONNECT_DEPLOY_KEY_FILENAME}");
    out.push(format!(
        "  - [ sh, -c, \"install -o {user} -m 700 -d {config_dir}\" ]"
    ));
    out.push(format!("  - [ sh, -c, \"printf '%s\\n' {} > {target} && chown {user}:{user} {target} && chmod 600 {target}\" ]", shell_quote(token)));
    out.push(format!("  - [ sh, -c, \"install -o {user} -m 700 -d {home}/.config/systemd/user/default.target.wants {home}/.config/systemd/user/paths.target.wants\" ]"));
    for (unit, wants) in PI_CONNECT_UNITS {
        out.push(format!(
            "  - [ sh, -c, \"UNIT_SRC=/usr/lib/systemd/user/{unit}; [ -f $UNIT_SRC ] || UNIT_SRC=/lib/systemd/user/{unit}; ln -sf $UNIT_SRC {home}/.config/systemd/user/{wants}.target.wants/{unit}\" ]"
        ));
    }
    out.push(format!(
        "  - [ sh, -c, \"chown -R {user}:{user} {home}/.config/systemd\" ]"
    ));
    out.push("  - [ sh, -c, \"install -d -m 0755 /var/lib/systemd/linger\" ]");
    out.push(format!(
        "  - [ sh, -c, \"install -m 0644 /dev/null /var/lib/systemd/linger/{user}\" ]"
    ));
    out.push(format!(
        "  - [ sh, -c, \"loginctl enable-linger {user} 2>/dev/null || true\" ]"
    ));
    out.push("  - [ sleep, \"2\" ]");
    out.push(format!(
        "  - [ sh, -c, \"systemctl --quiet --user --machine={user}@.host daemon-reload || true\" ]"
    ));
    for (unit, _) in PI_CONNECT_UNITS {
        out.push(format!("  - [ sh, -c, \"systemctl --quiet --user --machine={user}@.host start {unit} || true\" ]"));
    }
}
