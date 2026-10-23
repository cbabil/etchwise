use serde_json::json;

use super::*;

const TOP: &str = "https://downloads.raspberrypi.com/os_list_imagingutility_v4.json";

fn top_json() -> Vec<u8> {
    json!({
        "imager": {"latest_version": "2.0.12", "devices": []},
        "os_list": [
            {"name": "Pi OS", "devices": ["pi5-64bit"], "url": "https://h/a.img.xz"},
            {"name": "Other", "subitems_url": "https://h/other.json"}
        ]
    })
    .to_string()
    .into_bytes()
}

#[test]
fn first_top_level_fetch_makes_list_available_and_queues_sublists() {
    let mut store = OsListStore::default();
    assert!(!store.is_available());
    let result = store.on_fetch_complete(&top_json(), TOP, true);
    assert!(result.became_available);
    assert!(!result.malformed);
    assert_eq!(result.sublist_urls, ["https://h/other.json"]);
    assert!(store.is_available());
    assert_eq!(store.imager_metadata()["latest_version"], "2.0.12");
}

#[test]
fn sublist_fetch_is_spliced_into_the_tree() {
    let mut store = OsListStore::default();
    store.on_fetch_complete(&top_json(), TOP, true);
    let sub = json!({"os_list": [{"name": "Ubuntu", "devices": ["pi5-64bit"]}]}).to_string();
    let result = store.on_fetch_complete(sub.as_bytes(), "https://h/other.json", false);
    assert!(!result.became_available);
    let doc = store.filtered_document(&HwFilter::default());
    assert_eq!(doc["os_list"][1]["subitems"][0]["name"], "Ubuntu");
    assert!(doc["os_list"][1].get("subitems_url").is_none());
}

#[test]
fn top_level_refresh_updates_metadata_but_keeps_tree() {
    let mut store = OsListStore::default();
    store.on_fetch_complete(&top_json(), TOP, true);
    let refreshed =
        json!({"imager": {"latest_version": "9.9.9"}, "os_list": [{"name": "New"}]}).to_string();
    store.on_fetch_complete(refreshed.as_bytes(), TOP, true);
    assert_eq!(store.imager_metadata()["latest_version"], "9.9.9");
    let doc = store.filtered_document(&HwFilter::default());
    assert_eq!(doc["os_list"][0]["name"], "Pi OS");
}

#[test]
fn sublist_reply_does_not_replace_metadata() {
    let mut store = OsListStore::default();
    store.on_fetch_complete(&top_json(), TOP, true);
    let sub = json!({"imager": {"latest_version": "0"}, "os_list": []}).to_string();
    store.on_fetch_complete(sub.as_bytes(), "https://h/other.json", false);
    assert_eq!(store.imager_metadata()["latest_version"], "2.0.12");
}

#[test]
fn malformed_reply_is_reported_only_when_empty_top_level() {
    let mut store = OsListStore::default();
    assert!(store.on_fetch_complete(b"not json", TOP, true).malformed);
    assert!(
        !store
            .on_fetch_complete(b"{}", "https://h/x.json", false)
            .malformed
    );
    assert!(!store.is_available());
}

#[test]
fn filtered_document_applies_hw_filter_and_appends_internal_entries() {
    let mut store = OsListStore::default();
    store.on_fetch_complete(&top_json(), TOP, true);
    let filter = HwFilter {
        tags: vec!["pi4-64bit".into()],
        inclusive: false,
    };
    let doc = store.filtered_document(&filter);
    let names: Vec<_> = doc["os_list"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["Erase", "Use custom"]);
}

#[test]
fn empty_store_still_offers_internal_entries() {
    let doc = OsListStore::default().filtered_document(&HwFilter::default());
    assert_eq!(doc["os_list"].as_array().unwrap().len(), 2);
    assert_eq!(doc["imager"], json!({}));
}

#[test]
fn clear_forgets_the_list() {
    let mut store = OsListStore::default();
    store.on_fetch_complete(&top_json(), TOP, true);
    store.clear();
    assert!(!store.is_available());
}

#[test]
fn parses_the_real_v4_fixture() {
    let data = include_bytes!("../../../tests/fixtures/os_list_v4.json");
    let mut store = OsListStore::default();
    let result = store.on_fetch_complete(data, TOP, true);
    assert!(result.became_available);
    let doc = store.filtered_document(&HwFilter::default());
    assert!(doc["os_list"].as_array().unwrap().len() >= 10);
    assert!(doc["imager"]["devices"].as_array().unwrap().len() >= 5);
}
