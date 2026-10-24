//! An extra repository merged on top of the main one (not in upstream):
//! its devices join the device list before "No filtering" and its images
//! join the top of the OS tree, so e.g. an Intel NUC list can sit beside
//! the official Raspberry Pi one.

use std::io;

use serde_json::Value;

use super::OsListStore;
use crate::oslist::fetch::load_repository;
use crate::oslist::json::into_vec;

/// Loads the repository at `url` (with its sublists) and merges it into
/// `store`, which must already hold the main list.
///
/// # Errors
/// When the main list is not loaded, or the extra one cannot be fetched or
/// is not a valid OS list.
pub fn load_extra_repository(store: &mut OsListStore, url: &str) -> io::Result<()> {
    if !store.is_available() {
        return Err(io::Error::other("the main OS list is not loaded"));
    }
    let mut extra = OsListStore::default();
    load_repository(&mut extra, url)?;
    store.merge_extra(&extra);
    Ok(())
}

impl OsListStore {
    /// Adds `extra`'s devices and images. Its devices lose any `default`
    /// flag (the main repository picks the default) and are skipped when a
    /// device of the same name exists. Does nothing unless both lists are
    /// available.
    pub fn merge_extra(&mut self, extra: &OsListStore) {
        let (Some(current), Some(extra)) = (self.complete.as_mut(), extra.complete.as_ref()) else {
            return;
        };
        let mut devices = into_vec(current["imager"]["devices"].take());
        let at = devices
            .iter()
            .position(is_no_filtering)
            .unwrap_or(devices.len());
        let added: Vec<Value> = extra["imager"]["devices"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|d| !devices.iter().any(|known| known["name"] == d["name"]))
            .map(without_default)
            .collect();
        devices.splice(at..at, added);
        current["imager"]["devices"] = Value::Array(devices);
        let mut os_list = into_vec(current["os_list"].take());
        os_list.extend(extra["os_list"].as_array().into_iter().flatten().cloned());
        current["os_list"] = Value::Array(os_list);
    }
}

/// Upstream's catch-all device: inclusive matching with no tags.
fn is_no_filtering(device: &Value) -> bool {
    device["matching_type"] == "inclusive" && device["tags"].as_array().is_none_or(Vec::is_empty)
}

fn without_default(device: &Value) -> Value {
    let mut device = device.clone();
    if let Some(fields) = device.as_object_mut() {
        fields.remove("default");
    }
    device
}

#[cfg(test)]
mod tests {
    use serde_json::json;

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
}
