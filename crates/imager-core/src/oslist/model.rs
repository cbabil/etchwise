//! The OS list as the UI sees it: typed entries, the recommended marker and
//! sublist expansion. Ported from upstream `oslistmodel.cpp` and
//! `OSSelectionStep.qml::populateSublistInto`.
//!
//! Display text stays in the UI: the core flags `recommended` instead of
//! appending a translated "(Recommended)", keeps repository-relative icon
//! paths (`icons/x.png`) raw, and gives the synthetic rows English source
//! strings that the UI translates by their `internal://` URL.

use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::json::{bool_field, str_field, str_list};
use super::parse::{flatten_subitems, sort_by_architecture};
use crate::urlparts;

/// One row of the OS list (upstream `OSListModel::OS`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OsEntry {
    pub name: String,
    pub description: String,
    pub devices: Vec<String>,
    pub capabilities: Vec<String>,
    pub extract_size: u64,
    pub image_download_size: u64,
    pub random: bool,
    pub extract_sha256: String,
    pub bmap_url: String,
    pub icon: String,
    pub init_format: String,
    pub release_date: String,
    pub url: String,
    pub subitems_json: String,
    pub subitems_url: String,
    pub tooltip: String,
    pub website: String,
    pub architecture: String,
    pub enable_rpi_connect: bool,
    pub contains_multiple_files: bool,
    /// Upstream appends " (Recommended)" to this row's description.
    pub recommended: bool,
}

impl OsEntry {
    /// Reads an entry with `QJsonValue` defaults: wrong types read as empty.
    pub fn from_json(v: &Value) -> Self {
        let size = |k: &str| v.get(k).and_then(Value::as_f64).unwrap_or(0.0) as u64;
        Self {
            name: str_field(v, "name"),
            description: str_field(v, "description"),
            devices: str_list(v, "devices"),
            capabilities: str_list(v, "capabilities"),
            extract_size: size("extract_size"),
            image_download_size: size("image_download_size"),
            random: bool_field(v, "random"),
            extract_sha256: str_field(v, "extract_sha256"),
            bmap_url: str_field(v, "bmap_url"),
            icon: sanitize_icon(&str_field(v, "icon")),
            init_format: str_field(v, "init_format"),
            release_date: str_field(v, "release_date"),
            url: str_field(v, "url"),
            subitems_json: str_field(v, "subitems_json"),
            subitems_url: str_field(v, "subitems_url"),
            tooltip: str_field(v, "tooltip"),
            website: str_field(v, "website"),
            architecture: str_field(v, "architecture"),
            enable_rpi_connect: bool_field(v, "enable_rpi_connect"),
            contains_multiple_files: bool_field(v, "contains_multiple_files"),
            recommended: false,
        }
    }

    /// True when this row opens a sublist rather than being an image.
    pub fn is_group(&self) -> bool {
        !self.subitems_json.is_empty()
            || (!self.subitems_url.is_empty() && self.subitems_url != "internal://back")
    }
}

/// Upstream `sanitizeIconSource` minus its QML-relative `../` prefix:
/// drops http(s) URLs without a host, passes everything else through.
pub fn sanitize_icon(raw: &str) -> String {
    match urlparts::scheme(raw) {
        Some(scheme) if urlparts::is_http(&scheme) && urlparts::host(raw).is_empty() => {
            String::new()
        }
        _ => raw.to_owned(),
    }
}

static RECOMMENDED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r" \([^)]*\bRecommended\b[^)]*\)").expect("constant regex is valid")
});

/// Strips "(Recommended)" variants the repository put in descriptions, then
/// flags the first real (non-internal) entry when it is a described leaf.
pub fn mark_first_recommended(entries: &mut [OsEntry]) {
    for entry in entries.iter_mut() {
        if RECOMMENDED.is_match(&entry.description) {
            entry.description = RECOMMENDED.replace_all(&entry.description, "").into_owned();
        }
    }
    if let Some(first) = entries
        .iter_mut()
        .find(|e| !e.url.starts_with("internal://"))
        && !first.description.is_empty()
        && first.subitems_json.is_empty()
    {
        first.recommended = true;
    }
}

/// `OSListModel::reload` after parsing: architecture sort, typed entries,
/// recommendation flag.
pub fn build_model(mut list: Vec<Value>, architecture: &str) -> Vec<OsEntry> {
    sort_by_architecture(&mut list, architecture);
    let mut entries: Vec<OsEntry> = list.iter().map(OsEntry::from_json).collect();
    mark_first_recommended(&mut entries);
    entries
}

/// Rows of a group's sublist; children inherit the parent's `init_format`
/// when they have none (`populateSublistInto`).
pub fn sublist_entries(parent: &OsEntry) -> Vec<OsEntry> {
    let Ok(Value::Array(mut items)) = serde_json::from_str::<Value>(&parent.subitems_json) else {
        return Vec::new();
    };
    flatten_subitems(&mut items);
    items
        .into_iter()
        .filter(Value::is_object)
        .map(|mut item| {
            if item.get("init_format").is_none() && !parent.init_format.is_empty() {
                item["init_format"] = Value::String(parent.init_format.clone());
            }
            OsEntry::from_json(&item)
        })
        .collect()
}

/// The "Erase" and "Use custom" rows appended to every list, with
/// upstream's English source strings.
pub fn internal_entries() -> [Value; 2] {
    [
        json!({"name": "Erase", "description": "Format card as FAT32", "icon": "icons/erase.png", "url": "internal://format"}),
        json!({"name": "Use custom", "description": "Select a custom .img from your computer", "icon": "icons/use_custom.png", "url": "internal://custom"}),
    ]
}

#[cfg(test)]
mod tests;
