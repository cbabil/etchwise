//! An extra repository merged on top of the main one (not in upstream):
//! its devices join the device list before "No filtering" and its images
//! join the top of the OS tree, so e.g. an Intel NUC list can sit beside
//! the official Raspberry Pi one.

use std::io;

use serde_json::Value;

use super::OsListStore;
use crate::oslist::fetch::{load_list_text, load_repository};
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

/// Like [`load_extra_repository`], for a list shipped as text (its
/// sublists are still fetched).
///
/// # Errors
/// When the main list is not loaded, or `text` is not a valid OS list.
pub fn load_extra_document(store: &mut OsListStore, text: &str) -> io::Result<()> {
    if !store.is_available() {
        return Err(io::Error::other("the main OS list is not loaded"));
    }
    let mut extra = OsListStore::default();
    load_list_text(&mut extra, text, "built-in")?;
    store.merge_extra(&extra);
    Ok(())
}

impl OsListStore {
    /// Adds `extra`'s devices, and its images (folders unfolded) above the main list's. Its devices lose any `default`
    /// flag (the main repository picks the default); devices and images
    /// whose name is already listed are skipped. Does nothing unless both
    /// lists are available.
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
        let listed = into_vec(current["os_list"].take());
        let mut os_list: Vec<Value> = extra["os_list"]
            .as_array()
            .into_iter()
            .flatten()
            .flat_map(unfolded)
            .filter(|os| !listed.iter().any(|known| known["name"] == os["name"]))
            .collect();
        os_list.extend(listed);
        current["os_list"] = Value::Array(os_list);
    }
}

/// A loaded folder's images in its place, so each shows its own name and
/// version ("Kura 0.3.0", not "Kura"); a folder whose sublist failed stays.
fn unfolded(os: &Value) -> Vec<Value> {
    match os["subitems"].as_array() {
        Some(items) => items.iter().flat_map(unfolded).collect(),
        None => vec![os.clone()],
    }
}

/// Upstream's catch-all device: inclusive matching with no tags.
fn is_no_filtering(device: &Value) -> bool {
    device["matching_type"] == "inclusive" && device["tags"].as_array().is_none_or(Vec::is_empty)
}

/// The tab extra devices are listed under on the Device step.
const EXTRA_DEVICE_GROUP: &str = "other";

/// `device` without its `default` flag, filed under the extra tab.
fn without_default(device: &Value) -> Value {
    let mut device = device.clone();
    if let Some(fields) = device.as_object_mut() {
        fields.remove("default");
        fields.insert("group".into(), EXTRA_DEVICE_GROUP.into());
    }
    device
}

#[cfg(test)]
mod tests;
