//! Shared helpers for the ported Catch2 cases: generator wrappers that take
//! JSON settings, plus substring assertions with readable failure output.

use serde_json::Value;

use super::map;
use crate::customise::{
    Settings, UserDataOptions, cloudinit_network_config, cloudinit_user_data, systemd_script,
};

/// `generateSystemdScript(settings, token)`.
pub(super) fn script_with_token(settings: &Value, token: &str) -> String {
    systemd_script(Settings(&map(settings.clone())), token)
}

/// `generateSystemdScript(settings)`.
pub(super) fn script(settings: &Value) -> String {
    script_with_token(settings, "")
}

/// `generateCloudInitUserData(settings, token, hasCcRpi, sshEnabled, currentUser)`.
pub(super) fn user_data(
    settings: &Value,
    pi_connect_token: &str,
    has_cc_rpi: bool,
    ssh_enabled: bool,
    current_user: &str,
) -> String {
    let options = UserDataOptions {
        pi_connect_token,
        has_cc_rpi,
        ssh_enabled,
        current_user,
    };
    cloudinit_user_data(Settings(&map(settings.clone())), options)
}

/// `generateCloudInitNetworkConfig(settings, false)`.
pub(super) fn network(settings: &Value) -> String {
    cloudinit_network_config(Settings(&map(settings.clone())))
}

/// `REQUIRE_THAT(s, ContainsSubstring(n))` for every needle.
pub(super) fn has(haystack: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            haystack.contains(needle),
            "expected {needle:?} in output:\n{haystack}"
        );
    }
}

/// `REQUIRE_THAT(s, !ContainsSubstring(n))` for every needle.
pub(super) fn lacks(haystack: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            !haystack.contains(needle),
            "unexpected {needle:?} in output:\n{haystack}"
        );
    }
}

/// `QString::indexOf`: byte offset, or -1 when absent.
pub(super) fn index_of(haystack: &str, needle: &str) -> i64 {
    haystack
        .find(needle)
        .map_or(-1, |pos| i64::try_from(pos).unwrap())
}

/// The DHCP-only `eth0` block every network-config carries.
pub(super) const ETH0_DHCP: &[&str] = &["ethernets:", "eth0:", "dhcp4: true", "dhcp6: true"];

/// Upstream emits no `network-config` without an SSID
/// (`customization_generator.cpp:584`). The upstream Catch2 cases that expect
/// `eth0` there are stale: they fail against v2.0.7 as well.
pub(super) fn assert_no_network(netcfg: &str) {
    assert!(
        netcfg.is_empty(),
        "expected no network-config, got:\n{netcfg}"
    );
}
