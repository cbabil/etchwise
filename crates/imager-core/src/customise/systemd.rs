//! `firstrun.sh` for systemd images, ported line for line from upstream
//! `CustomisationGenerator::generateSystemdScript`.

use super::text::{Settings, shell_quote, ssh_keys};

pub(super) const IMAGER_CUSTOM: &str = "/usr/lib/raspberrypi-sys-mods/imager_custom";
pub(super) const IF_IMAGER_CUSTOM: &str =
    "if [ -f /usr/lib/raspberrypi-sys-mods/imager_custom ]; then";

/// Generates the first-boot script. Every line ends with `\n`.
pub fn systemd_script(settings: Settings<'_>, pi_connect_token: &str) -> String {
    let mut out = Script::default();
    let keys = ssh_keys(settings);
    let user = settings.trimmed("sshUserName");
    let pass = settings.string("sshUserPassword");
    let effective_user = if user.is_empty() {
        "pi".to_owned()
    } else {
        user.clone()
    };
    let configures_user = !user.is_empty() || !pass.is_empty() || !keys.is_empty();

    out.lines(&["#!/bin/sh", "", "set +e", ""]);
    hostname(&mut out, &settings.trimmed("hostname"));
    out.lines(&[
        "FIRSTUSER=$(getent passwd 1000 | cut -d: -f1)",
        "FIRSTUSERHOME=$(getent passwd 1000 | cut -d: -f6)",
    ]);
    ssh(&mut out, settings, &keys);
    if configures_user {
        user_setup(&mut out, &effective_user, &pass);
        if settings.bool("passwordlessSudo") {
            let file = format!("/etc/sudoers.d/010_{effective_user}-nopasswd");
            out.line(format!(
                "echo {} >{}",
                shell_quote(&format!("{effective_user} ALL=(ALL) NOPASSWD:ALL")),
                shell_quote(&file)
            ));
            out.line(format!("chmod 0440 {}", shell_quote(&file)));
        }
    }
    super::systemd_wifi::wifi(&mut out, settings);
    let token = pi_connect_token.trim();
    if settings.bool("piConnectEnabled") && !token.is_empty() {
        super::systemd_extras::pi_connect(&mut out, &effective_user, token);
    }
    super::systemd_extras::locale(
        &mut out,
        &settings.trimmed("keyboard"),
        &settings.trimmed("timezone"),
    );
    out.lines(&[
        "rm -f /boot/firstrun.sh",
        "sed -i 's| systemd.run.*||g' /boot/cmdline.txt",
        "exit 0",
    ]);
    out.finish()
}

/// Accumulates script lines.
#[derive(Default)]
pub(super) struct Script(String);

impl Script {
    pub(super) fn line(&mut self, line: impl AsRef<str>) {
        self.0.push_str(line.as_ref());
        self.0.push('\n');
    }

    pub(super) fn lines(&mut self, lines: &[&str]) {
        lines.iter().for_each(|l| self.line(l));
    }

    pub(super) fn finish(self) -> String {
        self.0
    }
}

fn hostname(out: &mut Script, hostname: &str) {
    if hostname.is_empty() {
        return;
    }
    out.line("CURRENT_HOSTNAME=$(cat /etc/hostname | tr -d \" \\t\\n\\r\")");
    out.line(IF_IMAGER_CUSTOM);
    out.line(format!("   {IMAGER_CUSTOM} set_hostname {hostname}"));
    out.line("else");
    out.line(format!("   echo {hostname} >/etc/hostname"));
    out.line(format!(
        "   sed -i \"s/127.0.1.1.*$CURRENT_HOSTNAME/127.0.1.1\\t{hostname}/g\" /etc/hosts"
    ));
    out.line("fi");
}

fn ssh(out: &mut Script, settings: Settings<'_>, keys: &[String]) {
    if !keys.is_empty() {
        let args: String = keys
            .iter()
            .map(|k| format!(" {}", shell_quote(k)))
            .collect();
        out.line(IF_IMAGER_CUSTOM);
        out.line(format!("   {IMAGER_CUSTOM} enable_ssh -k{args}"));
        out.line("else");
        out.line("   install -o \"$FIRSTUSER\" -m 700 -d \"$FIRSTUSERHOME/.ssh\"");
        out.line("cat > \"$FIRSTUSERHOME/.ssh/authorized_keys\" <<'EOF'");
        out.line(keys.join("\n"));
        out.lines(&[
            "EOF",
            "   chown \"$FIRSTUSER:$FIRSTUSER\" \"$FIRSTUSERHOME/.ssh/authorized_keys\"",
            "   chmod 600 \"$FIRSTUSERHOME/.ssh/authorized_keys\"",
            "   echo 'PasswordAuthentication no' >>/etc/ssh/sshd_config",
            "   systemctl enable ssh",
            "fi",
        ]);
    }
    if settings.bool("sshEnabled") && settings.bool("sshPasswordAuth") && keys.is_empty() {
        out.line(IF_IMAGER_CUSTOM);
        out.line(format!("   {IMAGER_CUSTOM} enable_ssh"));
        out.lines(&["else", "   systemctl enable ssh", "fi"]);
    }
}

fn user_setup(out: &mut Script, user: &str, pass: &str) {
    out.line("if [ -f /usr/lib/userconf-pi/userconf ]; then");
    out.line(format!(
        "   /usr/lib/userconf-pi/userconf {} {}",
        shell_quote(user),
        shell_quote(pass)
    ));
    out.line("else");
    if !pass.is_empty() {
        out.line(format!("   echo \"$FIRSTUSER:{pass}\" | chpasswd -e"));
    }
    out.line(format!("   if [ \"$FIRSTUSER\" != \"{user}\" ]; then"));
    out.line(format!("      usermod -l \"{user}\" \"$FIRSTUSER\""));
    out.line(format!("      usermod -m -d \"/home/{user}\" \"{user}\""));
    out.line(format!("      groupmod -n \"{user}\" \"$FIRSTUSER\""));
    out.line("      if grep -q \"^autologin-user=\" /etc/lightdm/lightdm.conf ; then");
    out.line(format!("         sed /etc/lightdm/lightdm.conf -i -e \"s/^autologin-user=.*/autologin-user={user}/\""));
    out.line("      fi");
    out.line("      if [ -f /etc/systemd/system/getty@tty1.service.d/autologin.conf ]; then");
    out.line(format!("         sed /etc/systemd/system/getty@tty1.service.d/autologin.conf -i -e \"s/$FIRSTUSER/{user}/\""));
    out.line("      fi");
    out.line("      if [ -f /etc/sudoers.d/010_pi-nopasswd ]; then");
    out.line(format!(
        "         sed -i \"s/^$FIRSTUSER /{user} /\" /etc/sudoers.d/010_pi-nopasswd"
    ));
    out.lines(&["      fi", "   fi", "fi"]);
}
