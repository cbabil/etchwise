use serde_json::json;

use super::*;

#[test]
fn sanitizes_icons_like_upstream() {
    assert_eq!(sanitize_icon(""), "");
    assert_eq!(
        sanitize_icon("icons/cat_media_players.png"),
        "icons/cat_media_players.png"
    );
    assert_eq!(sanitize_icon("qrc:/icons/x.png"), "qrc:/icons/x.png");
    assert_eq!(
        sanitize_icon("https://downloads.raspberrypi.com/x.png"),
        "https://downloads.raspberrypi.com/x.png"
    );
    assert_eq!(sanitize_icon("https:///x.png"), "");
    assert_eq!(sanitize_icon("file:///tmp/x.png"), "file:///tmp/x.png");
    // QUrl::isLocalFile() is true for every file: URL, UNC hosts included.
    assert_eq!(sanitize_icon("file://server/x.png"), "file://server/x.png");
    assert_eq!(sanitize_icon("ftp://h/x.png"), "ftp://h/x.png");
    assert_eq!(sanitize_icon("relative/x.png"), "relative/x.png");
}

#[test]
fn builds_entry_from_json() {
    let v = json!({
        "name": "Raspberry Pi OS (64-bit)", "description": "Desktop", "devices": ["pi5-64bit"],
        "capabilities": ["rpi_connect"], "extract_size": 6744440832u64, "image_download_size": 1364720984u64,
        "extract_sha256": "ab", "init_format": "cloudinit-rpi", "release_date": "2026-10-06",
        "url": "https://h/i.img.xz", "icon": "icons/x.png", "architecture": "arm64",
        "enable_rpi_connect": true, "random": true, "bmap_url": "b", "tooltip": "t", "website": "w"
    });
    let e = OsEntry::from_json(&v);
    assert_eq!(e.name, "Raspberry Pi OS (64-bit)");
    assert_eq!(e.extract_size, 6_744_440_832);
    assert_eq!(e.image_download_size, 1_364_720_984);
    assert_eq!(e.capabilities, ["rpi_connect"]);
    assert_eq!(e.icon, "icons/x.png");
    assert!(e.enable_rpi_connect && e.random && !e.recommended);
    assert!(!e.is_group());
}

#[test]
fn missing_fields_default_like_qjson() {
    let e = OsEntry::from_json(&json!({"name": 3, "extract_size": "big"}));
    assert_eq!(e.name, "");
    assert_eq!(e.extract_size, 0);
    assert!(!e.enable_rpi_connect);
}

#[test]
fn groups_are_detected_by_subitems() {
    assert!(
        OsEntry {
            subitems_json: "[]".into(),
            ..OsEntry::default()
        }
        .is_group()
    );
    assert!(
        OsEntry {
            subitems_url: "https://h/s.json".into(),
            ..OsEntry::default()
        }
        .is_group()
    );
    assert!(
        !OsEntry {
            subitems_url: "internal://back".into(),
            ..OsEntry::default()
        }
        .is_group()
    );
}

#[test]
fn flags_first_real_leaf_and_strips_repository_labels() {
    let mut list = vec![
        OsEntry {
            url: "internal://format".into(),
            description: "Erase".into(),
            ..OsEntry::default()
        },
        OsEntry {
            description: "Desktop (Recommended)".into(),
            ..OsEntry::default()
        },
        OsEntry {
            description: "Lite (Recommended for Pi 5)".into(),
            ..OsEntry::default()
        },
    ];
    mark_first_recommended(&mut list);
    assert!(!list[0].recommended);
    assert_eq!(list[1].description, "Desktop");
    assert!(list[1].recommended);
    assert_eq!(list[2].description, "Lite");
    assert!(!list[2].recommended);
}

#[test]
fn does_not_flag_a_first_entry_that_is_a_group_or_undescribed() {
    let mut list = vec![
        OsEntry {
            description: "Group".into(),
            subitems_json: "[]".into(),
            ..OsEntry::default()
        },
        OsEntry {
            description: "Leaf".into(),
            ..OsEntry::default()
        },
    ];
    mark_first_recommended(&mut list);
    assert!(!list[0].recommended && !list[1].recommended);
    let mut bare = vec![OsEntry::default()];
    mark_first_recommended(&mut bare);
    assert!(!bare[0].recommended);
}

#[test]
fn sublist_entries_flatten_and_inherit_init_format() {
    let parent = OsEntry {
        init_format: "systemd".into(),
        subitems_json: json!([
            {"name": "a", "icon": "icons/a.png"},
            {"name": "b", "init_format": "cloudinit", "subitems": [{"name": "b1"}]},
            "junk"
        ])
        .to_string(),
        ..OsEntry::default()
    };
    let subs = sublist_entries(&parent);
    assert_eq!(subs.len(), 2);
    assert_eq!(subs[0].init_format, "systemd");
    assert_eq!(subs[0].icon, "icons/a.png");
    assert_eq!(subs[1].init_format, "cloudinit");
    assert!(subs[1].subitems_json.contains("b1"));
}

#[test]
fn sublist_entries_of_leaf_or_bad_json_are_empty() {
    assert!(sublist_entries(&OsEntry::default()).is_empty());
    let bad = OsEntry {
        subitems_json: "{not json".into(),
        ..OsEntry::default()
    };
    assert!(sublist_entries(&bad).is_empty());
}

#[test]
fn internal_entries_match_upstream() {
    let [erase, custom] = internal_entries();
    assert_eq!(erase["url"], "internal://format");
    assert_eq!(erase["icon"], "icons/erase.png");
    assert_eq!(erase["description"], "Format card as FAT32");
    assert_eq!(custom["name"], "Use custom");
    assert_eq!(custom["url"], "internal://custom");
}

#[test]
fn builds_model_with_sort_and_recommendation() {
    let list = vec![
        json!({"name": "a", "architecture": "armhf", "description": "A"}),
        json!({"name": "b", "architecture": "arm64", "description": "B"}),
    ];
    let model = build_model(list, "arm64");
    assert_eq!(model[0].name, "b");
    assert!(model[0].recommended);
}
