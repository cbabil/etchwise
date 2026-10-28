//! The first user's name rule (Etchwise, not upstream), the same as the UI's
//! (`apps/desktop/src/wizard/steps/custom/username.ts`): upstream's pattern,
//! at most 32 characters, and not a system account or group name. Kura's
//! first boot refuses those and then ignores its whole setup file.

/// Debian system account and group names, plus those of common packages.
const RESERVED: [&str; 72] = [
    "_apt",
    "_chrony",
    "_rpc",
    "_ssh",
    "adm",
    "audio",
    "avahi",
    "avahi-autoipd",
    "backup",
    "bin",
    "bluetooth",
    "cdrom",
    "colord",
    "crontab",
    "daemon",
    "dialout",
    "dip",
    "disk",
    "dnsmasq",
    "docker",
    "fax",
    "floppy",
    "games",
    "gnats",
    "gpio",
    "i2c",
    "input",
    "irc",
    "kmem",
    "kvm",
    "list",
    "lp",
    "lpadmin",
    "mail",
    "man",
    "messagebus",
    "netdev",
    "news",
    "nobody",
    "nogroup",
    "ntp",
    "operator",
    "plugdev",
    "polkitd",
    "proxy",
    "pulse",
    "rdma",
    "render",
    "root",
    "sasl",
    "sgx",
    "shadow",
    "spi",
    "src",
    "ssl-cert",
    "sshd",
    "staff",
    "statd",
    "sudo",
    "sync",
    "sys",
    "syslog",
    "tape",
    "tss",
    "tty",
    "users",
    "utmp",
    "uucp",
    "uuidd",
    "video",
    "voice",
    "www-data",
];
/// Names under this prefix belong to systemd.
const RESERVED_PREFIX: &str = "systemd-";
const MAX_LEN: usize = 32;

/// The UI's username rule, checked again because the name reaches commands
/// run as root on the device and a refused name voids Kura's setup file.
pub fn is_safe_username(user: &str) -> bool {
    let mut chars = user.chars();
    user.len() <= MAX_LEN
        && chars
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c == '_')
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        && !RESERVED.contains(&user)
        && !user.starts_with(RESERVED_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_system_names_and_long_names() {
        for name in [
            "root",
            "daemon",
            "www-data",
            "_apt",
            "nobody",
            "systemd-network",
            "docker",
        ] {
            assert!(!is_safe_username(name), "{name}");
        }
        assert!(!is_safe_username(&"a".repeat(33)));
        assert!(!is_safe_username("Bad Name"));
        assert!(!is_safe_username(""));
        for name in ["chris", "admin", "rooted", "systemd", "_x", "a-1"] {
            assert!(is_safe_username(name), "{name}");
        }
        assert!(is_safe_username(&"a".repeat(32)));
    }

    #[test]
    fn reserved_names_match_the_ui() {
        let ts = include_str!("../../../../apps/desktop/src/wizard/steps/custom/username.ts");
        let block = ts
            .split("// reserved-usernames:start")
            .nth(1)
            .and_then(|rest| rest.split("// reserved-usernames:end").next())
            .expect("markers in username.ts");
        let ui: Vec<&str> = block.split('\'').skip(1).step_by(2).collect();
        assert_eq!(ui, RESERVED);
    }
}
