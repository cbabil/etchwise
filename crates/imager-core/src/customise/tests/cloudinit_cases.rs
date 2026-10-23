//! `generateCloudInitUserData` cases: SSH keys, hostname, locale, users.

use serde_json::json;

use super::helpers::{has, lacks, user_data};

const GROUPS: &str = "groups: users,adm,dialout,audio,netdev,video,plugdev,cdrom,games,input,gpio,spi,i2c,render,sudo";

#[test]
fn customisation_generator_cloud_init_handles_ssh_public_key_only_no_username_password() {
    let settings = json!({
        "sshPublicKey": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAITestPublicKey user@host",
    });
    let y = user_data(&settings, "", false, true, "pi");
    has(
        &y,
        &[
            "enable_ssh: true",
            "users:",
            "- name: pi",
            "ssh_authorized_keys:",
            "ssh-ed25519",
            "lock_passwd: true",
        ],
    );
    lacks(&y, &["sudo: ALL=(ALL) NOPASSWD:ALL"]);
}

#[test]
fn customisation_generator_cloud_init_handles_multiple_ssh_keys_in_pub_file() {
    let settings = json!({
        "sshPublicKey": "ssh-rsa AAAAB3...key1 user@host1\nssh-ed25519 AAAAC3...key2 user@host2",
    });
    let y = user_data(&settings, "", false, true, "pi");
    has(
        &y,
        &[
            "ssh_authorized_keys:",
            "- \"ssh-rsa AAAAB3...key1 user@host1\"",
            "- \"ssh-ed25519 AAAAC3...key2 user@host2\"",
        ],
    );
}

#[test]
fn customisation_generator_cloud_init_quotes_ssh_keys_with_yaml_special_characters() {
    let settings = json!({
        "sshPublicKey": "sk-ssh-ed25519@openssh.com AAAAGnNr...DMtkAAAABHNzaDo= ssh:",
    });
    let y = user_data(&settings, "", false, true, "pi");
    has(
        &y,
        &[
            "ssh_authorized_keys:",
            "- \"sk-ssh-ed25519@openssh.com AAAAGnNr...DMtkAAAABHNzaDo= ssh:\"",
        ],
    );
}

#[test]
fn customisation_generator_generates_cloud_init_user_data_with_hostname() {
    let y = user_data(&json!({"hostname": "testpi"}), "", false, false, "");
    has(
        &y,
        &[
            "manage_resolv_conf: false",
            "hostname: testpi",
            "manage_etc_hosts: true",
        ],
    );
    lacks(&y, &["preserve_hostname"]);
    has(
        &y,
        &[
            "packages:",
            "- avahi-daemon",
            "apt:",
            "preserve_sources_list: true",
        ],
    );
}

#[test]
fn customisation_generator_generates_cloud_init_user_data_with_timezone() {
    let y = user_data(&json!({"timezone": "Europe/London"}), "", false, false, "");
    has(&y, &["timezone: Europe/London"]);
}

#[test]
fn customisation_generator_generates_cloud_init_user_data_with_keyboard_layout() {
    let y = user_data(&json!({"keyboard": "gb"}), "", false, false, "");
    has(&y, &["keyboard:", "model: pc105", "layout: \"gb\""]);
}

#[test]
fn customisation_generator_generates_cloud_init_user_data_with_ssh_user() {
    let settings = json!({
        "sshUserName": "testuser",
        "sshUserPassword": "$5$fakesalt$fakehash123",
    });
    let y = user_data(&settings, "", false, true, "testuser");
    has(
        &y,
        &[
            "enable_ssh: true",
            "users:",
            "- name: testuser",
            GROUPS,
            "shell: /bin/bash",
            "lock_passwd: false",
            "passwd: \"$5$fakesalt$fakehash123\"",
        ],
    );
}

#[test]
fn customisation_generator_generates_cloud_init_user_data_with_user_credentials_but_no_ssh() {
    let settings = json!({
        "sshUserName": "localuser",
        "sshUserPassword": "$5$fakesalt$fakehash456",
    });
    let y = user_data(&settings, "", false, false, "pi");
    has(
        &y,
        &[
            "users:",
            "- name: localuser",
            GROUPS,
            "shell: /bin/bash",
            "lock_passwd: false",
            "passwd: \"$5$fakesalt$fakehash456\"",
        ],
    );
    lacks(&y, &["enable_ssh:", "ssh_pwauth:", "ssh_authorized_keys:"]);
}

#[test]
fn customisation_generator_generates_cloud_init_user_data_with_ssh_keys() {
    let settings = json!({
        "sshUserName": "testuser",
        "sshAuthorizedKeys": "ssh-rsa AAAAB3...key1\nssh-rsa AAAAB3...key2",
    });
    let y = user_data(&settings, "", false, true, "testuser");
    has(
        &y,
        &[
            "enable_ssh: true",
            "ssh_authorized_keys:",
            "- \"ssh-rsa AAAAB3...key1\"",
            "- \"ssh-rsa AAAAB3...key2\"",
            "lock_passwd: true",
        ],
    );
    lacks(&y, &["sudo: ALL=(ALL) NOPASSWD:ALL"]);
    has(&y, &["ssh_pwauth: false"]);
}

#[test]
fn customisation_generator_cloud_init_passwordless_sudo_when_explicitly_enabled() {
    let settings = json!({
        "sshUserName": "testuser",
        "sshUserPassword": "$y$j9T$test$hash",
        "passwordlessSudo": true,
    });
    let y = user_data(&settings, "", false, false, "testuser");
    has(&y, &["- name: testuser", "sudo: ALL=(ALL) NOPASSWD:ALL"]);
}

#[test]
fn customisation_generator_cloud_init_no_passwordless_sudo_by_default() {
    let settings = json!({"sshUserName": "testuser", "sshUserPassword": "$y$j9T$test$hash"});
    let y = user_data(&settings, "", false, false, "testuser");
    has(&y, &["- name: testuser"]);
    lacks(&y, &["sudo: ALL=(ALL) NOPASSWD:ALL"]);
}
