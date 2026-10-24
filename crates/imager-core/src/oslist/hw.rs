//! Raspberry Pi device list and capability checks, ported from upstream
//! `hwlistmodel.cpp` and the capability helpers in `imagewriter.cpp`.

use serde::Serialize;
use serde_json::Value;

use super::json::{str_field, str_list, strings};

/// One device row (upstream `HWListModel::HardwareDevice`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct HwDevice {
    pub name: String,
    pub tags: Vec<String>,
    pub capabilities: Vec<String>,
    pub icon: String,
    pub description: String,
    pub matching_type: String,
    pub architecture: String,
    /// Device-step tab: empty for the main repository's devices, "other"
    /// for those merged from an extra repository.
    pub group: String,
}

impl HwDevice {
    /// Inclusive devices also keep OS entries that list no devices.
    pub fn is_inclusive(&self) -> bool {
        self.matching_type == "inclusive"
    }
}

/// Parses `imager.devices`; returns the rows and the index flagged
/// `"default": true` (the last one wins, as upstream). `None` means the
/// list has not loaded yet.
pub fn parse_devices(doc: &Value) -> Option<(Vec<HwDevice>, Option<usize>)> {
    let devices = doc.get("imager")?.as_object()?.get("devices")?.as_array()?;
    let default = devices
        .iter()
        .rposition(|d| d.get("default").and_then(Value::as_bool) == Some(true));
    Some((devices.iter().map(device_from_json).collect(), default))
}

/// Icons stay repository-relative (`icons/x.png`); the UI resolves them.
fn device_from_json(d: &Value) -> HwDevice {
    HwDevice {
        name: str_field(d, "name"),
        tags: str_list(d, "tags"),
        capabilities: str_list(d, "capabilities"),
        icon: str_field(d, "icon"),
        description: str_field(d, "description"),
        matching_type: str_field(d, "matching_type"),
        architecture: str_field(d, "architecture"),
        group: str_field(d, "group"),
    }
}

/// Whether a device's tags fit an rpiboot-detected SoC (`tagsMatchChip`).
pub fn tags_match_chip(tags: &[String], chip: &str) -> bool {
    let prefixes: &[&str] = match chip {
        "BCM2712" => &["pi5-"],
        "BCM2711" => &["pi4-"],
        c if c.starts_with("BCM2836") => &["pi2-", "pi3-"],
        "BCM2835" => &["pi1-"],
        _ => return false,
    };
    tags.iter()
        .any(|t| prefixes.iter().any(|p| t.starts_with(p)))
}

/// Case-insensitive membership after trimming the needle.
pub fn has_capability(list: &[String], cap: &str) -> bool {
    let needle = cap.trim().to_lowercase();
    list.iter().any(|c| c.to_lowercase() == needle)
}

/// Upstream `setSWCapabilitiesList(QString)`: a JSON array, else CSV.
/// Empty parts are skipped before trimming, so `" "` yields `""` as upstream.
pub fn parse_sw_capabilities(raw: &str) -> Vec<String> {
    match serde_json::from_str::<Value>(raw) {
        Ok(Value::Array(items)) => strings(&items),
        _ => raw
            .split(',')
            .filter(|p| !p.is_empty())
            .map(|p| p.trim().to_lowercase())
            .collect(),
    }
}

/// Selected device's and OS's capability lists.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Capabilities {
    pub hardware: Vec<String>,
    pub software: Vec<String>,
}

impl Capabilities {
    /// `checkHWAndSWCapability(cap, differentSWCap)`.
    pub fn hw_and_sw(&self, cap: &str, sw_cap: Option<&str>) -> bool {
        has_capability(&self.hardware, cap) && has_capability(&self.software, sw_cap.unwrap_or(cap))
    }
}

#[cfg(test)]
mod tests;
