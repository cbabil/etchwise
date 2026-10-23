//! `generateSystemdScript` cases: structure, user, locale, WiFi, Pi Connect.

use serde_json::json;

use super::helpers::{has, index_of, script};

#[test]
fn customisation_generator_generates_valid_sh_script_header() {
    let s = script(&json!({"hostname": "testpi"}));
    has(&s, &["#!/bin/sh", "set +e"]);
}

#[test]
fn customisation_generator_handles_hostname_configuration() {
    let s = script(&json!({"hostname": "testpi"}));
    has(
        &s,
        &[
            "CURRENT_HOSTNAME=$(cat /etc/hostname",
            "raspberrypi-sys-mods/imager_custom",
            "set_hostname testpi",
            "echo testpi >/etc/hostname",
        ],
    );
}

#[test]
fn customisation_generator_sets_firstuser_variables_early() {
    let s = script(&json!({
        "hostname": "testpi",
        "sshUserName": "user1",
        "sshAuthorizedKeys": "ssh-rsa AAAAB3...testkey",
    }));
    has(
        &s,
        &[
            "FIRSTUSER=$(getent passwd 1000",
            "FIRSTUSERHOME=$(getent passwd 1000",
        ],
    );
    let first_user_pos = index_of(&s, "FIRSTUSER=$(getent");
    let ssh_key_pos = index_of(&s, ".ssh/authorized_keys");
    assert!(first_user_pos < ssh_key_pos);
}

#[test]
fn customisation_generator_handles_ssh_keys_with_heredoc() {
    let s = script(&json!({
        "sshEnabled": true,
        "sshAuthorizedKeys": "ssh-rsa AAAAB3...key1\nssh-rsa AAAAB3...key2",
    }));
    has(
        &s,
        &[
            "cat > \"$FIRSTUSERHOME/.ssh/authorized_keys\" <<'EOF'",
            "ssh-rsa AAAAB3...key1",
            "ssh-rsa AAAAB3...key2",
            "EOF",
            "PasswordAuthentication no",
        ],
    );
}

#[test]
fn customisation_generator_handles_user_renaming_with_userconf() {
    let s = script(&json!({
        "sshUserName": "testuser",
        "sshUserPassword": "$5$fakesalt$fakehash123",
    }));
    has(
        &s,
        &[
            "/usr/lib/userconf-pi/userconf",
            "'testuser'",
            "usermod -l",
            "usermod -m -d",
            "groupmod -n",
            "lightdm/lightdm.conf",
            "getty@tty1.service.d/autologin.conf",
            "010_pi-nopasswd",
        ],
    );
}

#[test]
fn customisation_generator_handles_yescrypt_password_format() {
    let s = script(&json!({
        "sshUserName": "testuser",
        "sshUserPassword": "$y$j9T$saltsaltsalt$hashhashhashhashhashhashhashhashhashhash",
    }));
    has(
        &s,
        &[
            "/usr/lib/userconf-pi/userconf",
            "$y$j9T$",
            "echo \"$FIRSTUSER:$y$j9T$",
        ],
    );
}

#[test]
fn customisation_generator_handles_sha256crypt_password_format() {
    let s = script(&json!({
        "sshUserName": "testuser",
        "sshUserPassword": "$5$rounds=5000$saltsalt$hashhashhashhashhashhash",
    }));
    has(
        &s,
        &[
            "/usr/lib/userconf-pi/userconf",
            "$5$rounds=5000$",
            "echo \"$FIRSTUSER:$5$rounds=5000$",
        ],
    );
}

#[test]
fn customisation_generator_handles_timezone_and_keyboard_at_end() {
    let s = script(&json!({"timezone": "Europe/London", "keyboard": "gb"}));
    has(
        &s,
        &[
            "set_timezone 'Europe/London'",
            "echo \"Europe/London\" >/etc/timezone",
            "dpkg-reconfigure -f noninteractive tzdata",
            "set_keymap 'gb'",
            "cat >/etc/default/keyboard <<'KBEOF'",
            "XKBLAYOUT=\"gb\"",
        ],
    );
    let user_pos = index_of(&s, "userconf-pi/userconf");
    let timezone_pos = index_of(&s, "set_timezone");
    let cleanup_pos = index_of(&s, "rm -f /boot/firstrun.sh");
    assert!(user_pos < timezone_pos);
    assert!(timezone_pos < cleanup_pos);
}

#[test]
fn customisation_generator_includes_cleanup_at_end() {
    let s = script(&json!({"hostname": "test"}));
    has(
        &s,
        &[
            "rm -f /boot/firstrun.sh",
            "sed -i 's| systemd.run.*||g' /boot/cmdline.txt",
        ],
    );
    assert!(s.ends_with("exit 0\n"));
}
