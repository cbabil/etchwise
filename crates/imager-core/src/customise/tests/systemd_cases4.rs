//! Ported upstream systemd `firstrun.sh` cases (continued).

use serde_json::json;

use super::helpers::{has, lacks, script};

#[test]
fn customisation_generator_reference_script_comparison() {
    let s = script(&json!({
        "hostname": "raspberrypi",
        "sshEnabled": true,
        "sshUserName": "testuserfoobar",
        "sshUserPassword": "$5$salt$hash",
        "sshAuthorizedKeys": "ssh-rsa AAAAB3...testkey",
        "timezone": "Europe/London",
        "keyboard": "gb",
    }));
    // SECTION "Script structure matches reference"
    has(
        &s,
        &[
            "#!/bin/sh",
            "set +e",
            "CURRENT_HOSTNAME=$(cat /etc/hostname | tr -d",
            "FIRSTUSER=$(getent passwd 1000 | cut -d: -f1)",
            "FIRSTUSERHOME=$(getent passwd 1000 | cut -d: -f6)",
        ],
    );
    // SECTION "SSH key handling matches reference"
    has(
        &s,
        &["cat > \"$FIRSTUSERHOME/.ssh/authorized_keys\" <<'EOF'"],
    );
    lacks(&s, &["install -o \"$FIRSTUSER\" -m 600 <(printf"]);
    // SECTION "User management matches reference"
    has(
        &s,
        &[
            "/usr/lib/userconf-pi/userconf",
            "echo \"$FIRSTUSER:$5$salt$hash",
            "if [ \"$FIRSTUSER\" != \"testuserfoobar\" ]; then",
            "usermod -l \"testuserfoobar\" \"$FIRSTUSER\"",
            "usermod -m -d \"/home/testuserfoobar\" \"testuserfoobar\"",
            "groupmod -n \"testuserfoobar\" \"$FIRSTUSER\"",
        ],
    );
    // SECTION "Locale configuration matches reference"
    has(
        &s,
        &[
            "set_keymap 'gb'",
            "set_timezone 'Europe/London'",
            "cat >/etc/default/keyboard <<'KBEOF'",
        ],
    );
    // SECTION "Script ends with cleanup"
    has(
        &s,
        &[
            "rm -f /boot/firstrun.sh",
            "sed -i 's| systemd.run.*||g' /boot/cmdline.txt",
        ],
    );
    assert!(s.ends_with("exit 0\n"));
}
