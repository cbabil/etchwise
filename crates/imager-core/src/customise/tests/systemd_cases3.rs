//! `generateSystemdScript` cases: SSH keys, negative inputs, sudo.

use serde_json::json;

use super::helpers::{has, index_of, lacks, script, script_with_token};
use crate::customise::text::{PI_CONNECT_CONFIG_PATH, PI_CONNECT_DEPLOY_KEY_FILENAME};

#[test]
fn customisation_generator_handles_multiline_ssh_key() {
    let s = script(&json!({
        "sshEnabled": true,
        "sshAuthorizedKeys": "ssh-rsa AAAAB3...key1\n\n\nssh-rsa AAAAB3...key2\n\n",
    }));
    has(&s, &["ssh-rsa AAAAB3...key1", "ssh-rsa AAAAB3...key2"]);
}

#[test]
fn customisation_generator_handles_ssh_public_key_only_no_username_password() {
    let s = script(&json!({
        "sshEnabled": true,
        "sshPublicKey": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAITestPublicKey user@host",
    }));
    has(
        &s,
        &[
            "cat > \"$FIRSTUSERHOME/.ssh/authorized_keys\" <<'EOF'",
            "ssh-ed25519",
            "PasswordAuthentication no",
            "/usr/lib/userconf-pi/userconf",
            "'pi'",
        ],
    );
}

#[test]
fn customisation_generator_handles_multiple_ssh_keys_in_pub_file() {
    let s = script(&json!({
        "sshEnabled": true,
        "sshPublicKey": "ssh-rsa AAAAB3...key1 user@host1\nssh-ed25519 AAAAC3...key2 user@host2",
    }));
    has(
        &s,
        &[
            "cat > \"$FIRSTUSERHOME/.ssh/authorized_keys\" <<'EOF'",
            "ssh-rsa AAAAB3...key1",
            "ssh-ed25519 AAAAC3...key2",
        ],
    );
    let key1_pos = index_of(&s, "ssh-rsa AAAAB3...key1");
    let key2_pos = index_of(&s, "ssh-ed25519 AAAAC3...key2");
    assert!(key1_pos != -1);
    assert!(key2_pos != -1);
    assert!(key2_pos > key1_pos);
}

#[test]
fn customisation_generator_handles_very_long_hostname() {
    let s = script(&json!({
        "hostname": "verylonghostnameverylonghostnameverylonghostnameverylonghostnameverylonghost",
    }));
    has(&s, &["set_hostname"]);
    assert!(s.ends_with("exit 0\n"));
}

#[test]
fn customisation_generator_handles_null_empty_piconnect_token() {
    let s = script_with_token(
        &json!({"sshUserName": "testuser", "piConnectEnabled": true}),
        "",
    );
    lacks(
        &s,
        &[PI_CONNECT_CONFIG_PATH, PI_CONNECT_DEPLOY_KEY_FILENAME],
    );
}

#[test]
fn customisation_generator_handles_invalid_keyboard_layout() {
    let s = script(&json!({"keyboard": "invalid_layout_xyz"}));
    has(&s, &["XKBLAYOUT=\"invalid_layout_xyz\""]);
}

#[test]
fn customisation_generator_handles_backticks_in_username_command_substitution_attempt() {
    let s = script(&json!({"sshUserName": "user`whoami`", "sshUserPassword": "$5$salt$hash"}));
    has(&s, &["'user`whoami`'"]);
}

#[test]
fn customisation_generator_handles_dollar_signs_in_password_variable_expansion_attempt() {
    let s = script(&json!({
        "sshUserName": "testuser",
        "sshUserPassword": "$5$salt$(rm -rf /)hash",
    }));
    has(&s, &["'$5$salt$(rm -rf /)hash'"]);
}

#[test]
fn customisation_generator_systemd_script_passwordless_sudo() {
    let s = script(&json!({
        "sshUserName": "testuser",
        "sshUserPassword": "$y$j9T$test$hash",
        "passwordlessSudo": true,
    }));
    has(
        &s,
        &[
            "testuser ALL=(ALL) NOPASSWD:ALL",
            "/etc/sudoers.d/010_testuser-nopasswd",
            "chmod 0440",
        ],
    );
}

#[test]
fn customisation_generator_systemd_script_no_passwordless_sudo_by_default() {
    let s = script(&json!({
        "sshUserName": "testuser",
        "sshUserPassword": "$y$j9T$test$hash",
    }));
    lacks(&s, &["NOPASSWD"]);
}
