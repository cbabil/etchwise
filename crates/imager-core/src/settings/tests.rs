use serde_json::json;

use super::*;

#[test]
fn bool_defaults_match_upstream() {
    let s = Settings::in_memory();
    assert!(s.bool("eject"));
    assert!(s.bool("check_version"));
    assert!(!s.bool("beep"));
    assert!(!s.bool("disable_warnings"));
}

#[test]
fn bools_follow_qvariant_conversion() {
    let mut s = Settings::in_memory();
    for (value, expected) in [
        (json!(false), false),
        (json!("false"), false),
        (json!("0"), false),
        (json!(""), false),
        (json!("yes"), true),
        (json!(1), true),
        (json!(0), false),
        (json!(null), false),
    ] {
        s.set("eject", value).unwrap();
        assert_eq!(s.bool("eject"), expected);
    }
}

#[test]
fn strings_round_trip() {
    let mut s = Settings::in_memory();
    assert_eq!(s.string("savedLanguage"), "");
    s.set("savedLanguage", json!("de")).unwrap();
    assert_eq!(s.string("savedLanguage"), "de");
    s.set("sidebarWidth", json!(240)).unwrap();
    assert_eq!(s.string("sidebarWidth"), "240");
    assert_eq!(s.value("sidebarWidth"), Some(json!(240)));
    assert_eq!(s.value("eject"), Some(json!(true)));
    assert_eq!(s.value("beep"), None);
}

#[test]
fn customisation_group_replace_set_remove_clear() {
    let mut s = Settings::in_memory();
    let map = json!({"hostname": "pi", "timezone": "Europe/London"})
        .as_object()
        .unwrap()
        .clone();
    s.set_customisation(map).unwrap();
    s.set_group_value(CUSTOMISATION_GROUP, "keyboard", json!("gb"))
        .unwrap();
    s.remove_group_value(CUSTOMISATION_GROUP, "timezone")
        .unwrap();
    let got = s.customisation();
    assert_eq!(got.get("hostname"), Some(&json!("pi")));
    assert_eq!(got.get("keyboard"), Some(&json!("gb")));
    assert!(got.get("timezone").is_none());
    s.clear_customisation().unwrap();
    assert!(s.customisation().is_empty());
}

#[test]
fn authorized_keys_are_deduplicated_in_order() {
    let mut s = Settings::in_memory();
    s.set_group_value(
        CUSTOMISATION_GROUP,
        "sshAuthorizedKeys",
        json!("ssh-ed25519 A\r\n ssh-ed25519 A \n\nssh-rsa B\nssh-ed25519 A"),
    )
    .unwrap();
    assert_eq!(
        s.customisation()["sshAuthorizedKeys"],
        json!("ssh-ed25519 A\nssh-rsa B")
    );
}

#[test]
fn group_values_replace_a_non_object_group() {
    let mut s = Settings::in_memory();
    s.set("caching", json!(5)).unwrap();
    s.set_group_value("caching", "enabled", json!(true))
        .unwrap();
    assert_eq!(s.group_value("caching", "enabled"), Some(&json!(true)));
}

#[test]
fn persists_to_disk_and_reloads() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sub/settings.json");
    let mut s = Settings::load(path.clone()).unwrap();
    s.set("beep", json!(true)).unwrap();
    s.set_group_value("caching", "lastFileName", json!("x.img.xz"))
        .unwrap();
    let reloaded = Settings::load(path).unwrap();
    assert!(reloaded.bool("beep"));
    assert_eq!(
        reloaded.group_value("caching", "lastFileName"),
        Some(&json!("x.img.xz"))
    );
}

#[test]
fn corrupt_file_is_a_parse_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    std::fs::write(&path, b"{oops").unwrap();
    let err = Settings::load(path).unwrap_err();
    assert!(err.to_string().contains("not valid JSON"));
}

#[test]
fn unreadable_path_is_an_io_error() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        Settings::load(dir.path().to_path_buf()),
        Err(SettingsError::Io { .. })
    ));
}
