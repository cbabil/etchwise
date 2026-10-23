//! The assembled OS list (upstream `_completeOsList`): merges the top-level
//! reply with sublist replies as they arrive and serves the hardware-filtered
//! document with the synthetic "Erase" / "Use custom" rows appended.

use serde_json::{Value, json};

use super::json::{as_slice, into_vec, object_or_empty};
use super::model::internal_entries;
use super::tree::{filter_with_hw_tags, insert_sublist, sublist_urls};

/// The selected device's tag filter (`_deviceFilter`, `_deviceFilterIsInclusive`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HwFilter {
    pub tags: Vec<String>,
    pub inclusive: bool,
}

/// What a finished fetch changed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FetchResult {
    /// Sublists referenced by this reply that should be fetched next.
    pub sublist_urls: Vec<String>,
    /// The list went from unavailable to available.
    pub became_available: bool,
    /// A top-level reply without `os_list` left the app offline.
    pub malformed: bool,
}

/// Holds the merged OS-list document.
#[derive(Debug, Clone, Default)]
pub struct OsListStore {
    complete: Option<Value>,
}

impl OsListStore {
    /// True once a list with `os_list` has been received.
    pub fn is_available(&self) -> bool {
        self.complete.is_some()
    }

    /// Forgets the list, e.g. before switching repository.
    pub fn clear(&mut self) {
        self.complete = None;
    }

    /// The `imager` object (devices, latest_version, refresh hints).
    pub fn imager_metadata(&self) -> Value {
        object_or_empty(
            self.complete
                .as_ref()
                .and_then(|c| c.get("imager"))
                .cloned(),
        )
    }

    /// Port of `onOsListFetchComplete`. A top-level refresh only replaces
    /// the `imager` metadata: like upstream, the tree itself is kept.
    pub fn on_fetch_complete(&mut self, data: &[u8], url: &str, is_top_level: bool) -> FetchResult {
        let response = serde_json::from_slice::<Value>(data)
            .ok()
            .filter(Value::is_object)
            .unwrap_or_else(|| json!({}));
        let Some(incoming) = response.get("os_list") else {
            return FetchResult {
                malformed: is_top_level && self.complete.is_none(),
                ..FetchResult::default()
            };
        };
        let result = FetchResult {
            sublist_urls: sublist_urls(as_slice(incoming)),
            became_available: self.complete.is_none(),
            malformed: false,
        };
        match &mut self.complete {
            None => self.complete = Some(response),
            Some(current) => merge_into(current, response, url, is_top_level),
        }
        result
    }

    /// Port of `getFilteredOSlistDocument`.
    pub fn filtered_document(&self, filter: &HwFilter) -> Value {
        let full = self
            .complete
            .as_ref()
            .and_then(|c| c.get("os_list"))
            .map(as_slice)
            .unwrap_or_default();
        let mut list = if filter.tags.is_empty() {
            full.to_vec()
        } else {
            filter_with_hw_tags(full, &filter.tags, filter.inclusive)
        };
        list.extend(internal_entries());
        json!({"imager": self.imager_metadata(), "os_list": list})
    }
}

fn merge_into(current: &mut Value, mut response: Value, url: &str, is_top_level: bool) {
    let incoming = into_vec(response["os_list"].take());
    let mut tree = into_vec(current["os_list"].take());
    insert_sublist(&mut tree, &incoming, url);
    let imager = match response.get_mut("imager") {
        Some(meta) if is_top_level => meta.take(),
        _ => current["imager"].take(),
    };
    *current = json!({"imager": object_or_empty(Some(imager)), "os_list": tree});
}

#[cfg(test)]
mod tests;
