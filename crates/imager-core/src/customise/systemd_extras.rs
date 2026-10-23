//! Raspberry Pi Connect and locale sections of `firstrun.sh` (upstream
//! `generateSystemdScript`).

use super::systemd::{IF_IMAGER_CUSTOM, IMAGER_CUSTOM, Script};
use super::text::{
    PI_CONNECT_CONFIG_PATH, PI_CONNECT_DEPLOY_KEY_FILENAME, PI_CONNECT_UNITS, shell_quote,
};

pub(super) fn pi_connect(out: &mut Script, user: &str, token: &str) {
    let config_dir = format!("$TARGET_HOME/{PI_CONNECT_CONFIG_PATH}");
    let key_path = format!("{config_dir}/{PI_CONNECT_DEPLOY_KEY_FILENAME}");
    out.line(format!("TARGET_USER=\"{user}\""));
    out.line("TARGET_HOME=$(getent passwd \"$TARGET_USER\" | cut -d: -f6)");
    out.line(format!("if [ -z \"$TARGET_HOME\" ] || [ ! -d \"$TARGET_HOME\" ]; then TARGET_HOME=\"/home/{user}\"; fi"));
    out.line(format!(
        "install -o \"$TARGET_USER\" -m 700 -d \"{config_dir}\""
    ));
    out.line(format!("cat > \"{key_path}\" <<'EOF'"));
    out.line(token);
    out.line("EOF");
    out.line(format!(
        "chown \"$TARGET_USER:$TARGET_USER\" \"{key_path}\""
    ));
    out.line(format!("chmod 600 \"{key_path}\""));
    out.lines(&[
        "# Enable Raspberry Pi Connect systemd units",
        "SYSTEMD_USER_BASE=\"$TARGET_HOME/.config/systemd/user\"",
        "install -o \"$TARGET_USER\" -m 700 -d \"$SYSTEMD_USER_BASE/default.target.wants\" \"$SYSTEMD_USER_BASE/paths.target.wants\"",
    ]);
    for (unit, wants) in PI_CONNECT_UNITS {
        out.line(format!("UNIT_SRC=\"/usr/lib/systemd/user/{unit}\"; [ -f \"$UNIT_SRC\" ] || UNIT_SRC=\"/lib/systemd/user/{unit}\""));
        out.line(format!(
            "ln -sf \"$UNIT_SRC\" \"$SYSTEMD_USER_BASE/{wants}.target.wants/{unit}\""
        ));
    }
    out.lines(&[
        "chown -R \"$TARGET_USER:$TARGET_USER\" \"$TARGET_HOME/.config/systemd\" || true",
        "install -d -m 0755 /var/lib/systemd/linger",
        "install -m 0644 /dev/null \"/var/lib/systemd/linger/$TARGET_USER\"",
        "loginctl enable-linger \"$TARGET_USER\" 2>/dev/null || true",
        "# Give user manager time to start if it wasn't already running",
        "sleep 2",
        "# Reload user systemd manager and start services",
        "systemctl --quiet --user --machine ${TARGET_USER}@.host daemon-reload",
        "systemctl --quiet --user --machine ${TARGET_USER}@.host start rpi-connect.service rpi-connect-signin.path rpi-connect-wayvnc.service",
    ]);
}

pub(super) fn locale(out: &mut Script, keyboard: &str, timezone: &str) {
    if keyboard.is_empty() && timezone.is_empty() {
        return;
    }
    out.line(IF_IMAGER_CUSTOM);
    if !keyboard.is_empty() {
        out.line(format!(
            "   {IMAGER_CUSTOM} set_keymap {}",
            shell_quote(keyboard)
        ));
    }
    if !timezone.is_empty() {
        out.line(format!(
            "   {IMAGER_CUSTOM} set_timezone {}",
            shell_quote(timezone)
        ));
    }
    out.line("else");
    if !timezone.is_empty() {
        out.line("   rm -f /etc/localtime");
        out.line(format!("   echo \"{timezone}\" >/etc/timezone"));
        out.line("   dpkg-reconfigure -f noninteractive tzdata");
    }
    if !keyboard.is_empty() {
        out.lines(&["cat >/etc/default/keyboard <<'KBEOF'", "XKBMODEL=\"pc105\""]);
        out.line(format!("XKBLAYOUT=\"{keyboard}\""));
        out.lines(&[
            "XKBVARIANT=\"\"",
            "XKBOPTIONS=\"\"",
            "",
            "KBEOF",
            "   dpkg-reconfigure -f noninteractive keyboard-configuration",
        ]);
    }
    out.line("fi");
}
