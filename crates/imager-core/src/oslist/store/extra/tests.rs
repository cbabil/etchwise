//! Tests of the extra-repository merge.
use serde_json::json;

use super::super::OsListStore;
use super::*;
use crate::oslist::store::HwFilter;

fn store(doc: &Value) -> OsListStore {
    let mut store = OsListStore::default();
    store.on_fetch_complete(doc.to_string().as_bytes(), "top", true);
    store
}

fn official() -> OsListStore {
    store(&json!({
        "imager": {"latest_version": "2.0.12", "devices": [
            {"name": "Raspberry Pi 5", "tags": ["pi5-64bit"], "matching_type": "exclusive", "default": true},
            {"name": "No filtering", "tags": [], "matching_type": "inclusive"}
        ]},
        "os_list": [{"name": "Raspberry Pi OS", "devices": ["pi5-64bit"], "url": "https://pi/os.img.xz"}]
    }))
}

fn nuc() -> OsListStore {
    store(&nuc_doc())
}

fn nuc_doc() -> Value {
    json!({
        "imager": {"devices": [
            {"name": "Intel NUC", "tags": ["x86-64"], "matching_type": "exclusive", "default": true},
            {"name": "Raspberry Pi 5", "tags": ["dup"]}
        ]},
        "os_list": [{"name": "Ubuntu Server", "devices": ["x86-64"], "url": "https://u/s.iso"}]
    })
}

fn names(values: &Value) -> Vec<&str> {
    values
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v["name"].as_str())
        .collect()
}

#[test]
fn adds_devices_before_no_filtering_without_taking_the_default() {
    let mut main = official();
    main.merge_extra(&nuc());
    let devices = &main.imager_metadata()["devices"];
    assert_eq!(
        names(devices),
        ["Raspberry Pi 5", "Intel NUC", "No filtering"]
    );
    assert_eq!(devices[0]["default"], true);
    assert!(devices[1].get("default").is_none());
    assert_eq!(main.imager_metadata()["latest_version"], "2.0.12");
}

#[test]
fn device_filters_pick_each_repository_images() {
    let mut main = official();
    main.merge_extra(&nuc());
    let shown = |tag: &str| {
        let filter = HwFilter {
            tags: vec![tag.to_owned()],
            inclusive: false,
        };
        let doc = main.filtered_document(&filter);
        names(&doc["os_list"])
            .into_iter()
            .filter(|n| !n.starts_with("Erase") && !n.starts_with("Use custom"))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    assert_eq!(shown("x86-64"), ["Ubuntu Server"]);
    assert_eq!(shown("pi5-64bit"), ["Raspberry Pi OS"]);
    let all = main.filtered_document(&HwFilter::default());
    assert_eq!(
        &names(&all["os_list"])[..2],
        ["Raspberry Pi OS", "Ubuntu Server"]
    );
}

#[test]
fn needs_both_lists() {
    let mut main = official();
    main.merge_extra(&OsListStore::default());
    assert_eq!(names(&main.imager_metadata()["devices"]).len(), 2);
    let mut empty = OsListStore::default();
    empty.merge_extra(&nuc());
    assert!(!empty.is_available());
}

#[test]
fn loads_an_extra_repository_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("nuc list.json");
    std::fs::write(&file, nuc_doc().to_string()).unwrap();
    let url = format!("file://{}", file.display()).replace(' ', "%20");
    let mut main = official();
    load_extra_repository(&mut main, &url).unwrap();
    assert_eq!(names(&main.imager_metadata()["devices"])[1], "Intel NUC");
    let missing = format!("file://{}/nope.json", dir.path().display());
    assert!(load_extra_repository(&mut main, &missing).is_err());
    assert!(load_extra_repository(&mut OsListStore::default(), &url).is_err());
}

#[test]
fn appends_when_there_is_no_catch_all_device() {
    let mut main = store(&json!({"imager": {}, "os_list": []}));
    main.merge_extra(&nuc());
    assert_eq!(
        names(&main.imager_metadata()["devices"]),
        ["Intel NUC", "Raspberry Pi 5"]
    );
}

#[test]
fn remembers_which_images_came_from_the_extra_repository() {
    let mut main = official();
    let extra = store(&json!({"imager": {}, "os_list": [
        {"name": "Ubuntu Server", "url": "https://u/s.iso"},
        {"name": "Folder", "subitems": [{"name": "Deep", "url": "https://u/deep.iso"}]}
    ]}));
    main.merge_extra(&extra);
    assert!(main.is_from_extra("https://u/s.iso"));
    assert!(main.is_from_extra("https://u/deep.iso"));
    assert!(!main.is_from_extra("https://pi/os.img.xz"));
    main.clear();
    assert!(!main.is_from_extra("https://u/s.iso"));
}
