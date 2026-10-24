//! OS-list parsing ported from upstream `oslistmodel.cpp`: locale list
//! selection, `init_format` pruning, `random` shuffling, flattening and
//! architecture ordering.

use rand::Rng;
use rand::seq::SliceRandom;
use serde_json::Value;

use super::json::{as_slice, into_vec};

/// `init_format` values the v4 schema allows, plus Etchwise's Ubuntu
/// autoinstall for PCs.
pub const VALID_INIT_FORMATS: [&str; 6] = [
    "",
    "systemd",
    "cloudinit",
    "cloudinit-rpi",
    "none",
    crate::customise::plan::AUTOINSTALL_UBUNTU,
];

/// Picks `os_list_<locale>` (or, when that key is absent,
/// `os_list_<language>`), falling back to `os_list` when the pick is empty.
/// `locale` is a POSIX name such as `en_GB` (`QLocale::system().name()`).
pub fn list_for_locale(root: &Value, locale: &str) -> Vec<Value> {
    let key = |suffix: &str| root.get(format!("os_list_{suffix}"));
    key(locale)
        .or_else(|| {
            locale
                .split_once('_')
                .and_then(|(language, _)| key(language))
        })
        .map(as_slice)
        .filter(|list| !list.is_empty())
        .or_else(|| root.get("os_list").map(as_slice))
        .unwrap_or_default()
        .to_vec()
}

/// Drops entries with an unknown `init_format` and groups left without
/// valid children; non-objects read as `{}`.
pub fn prune_invalid_init_formats(list: Vec<Value>) -> Vec<Value> {
    list.into_iter()
        .filter_map(|mut entry| {
            let format = entry
                .get("init_format")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !VALID_INIT_FORMATS.contains(&format) {
                return None;
            }
            let Value::Object(object) = &mut entry else {
                return Some(Value::Object(serde_json::Map::new()));
            };
            if let Some(subitems) = object.remove("subitems") {
                let nested = prune_invalid_init_formats(into_vec(subitems));
                if nested.is_empty() {
                    return None;
                }
                object.insert("subitems".into(), Value::Array(nested));
            }
            Some(entry)
        })
        .collect()
}

/// Shuffles the `subitems` of every group flagged `random`, after
/// recursing, with the same backward Fisher-Yates as upstream.
pub fn shuffle_random<R: Rng + ?Sized>(list: &mut [Value], rng: &mut R) {
    for entry in list {
        let random = entry
            .get("random")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let Some(Value::Array(subitems)) = entry.get_mut("subitems") else {
            continue;
        };
        shuffle_random(subitems, rng);
        if random {
            subitems.shuffle(rng);
        }
    }
}

/// Replaces `subitems` arrays with a `subitems_json` string, one level deep,
/// because the UI list model is flat.
pub fn flatten_subitems(list: &mut [Value]) {
    for object in list.iter_mut().filter_map(Value::as_object_mut) {
        if let Some(subitems) = object.remove("subitems") {
            object.insert("subitems_json".into(), Value::String(subitems.to_string()));
        }
    }
}

/// Stable-moves entries whose `architecture` equals `preferred` to the
/// front, recursively (`applyArchitectureSorting`).
pub fn sort_by_architecture(list: &mut Vec<Value>, preferred: &str) {
    if preferred.is_empty() {
        return;
    }
    let is_preferred = |v: &Value| v.get("architecture").and_then(Value::as_str) == Some(preferred);
    let (mut front, back): (Vec<Value>, Vec<Value>) = list.drain(..).partition(is_preferred);
    front.extend(back);
    *list = front;
    for entry in list.iter_mut() {
        if let Some(Value::Array(subitems)) = entry.get_mut("subitems") {
            sort_by_architecture(subitems, preferred);
        }
    }
}

/// `parseOSJson`: locale list, pruning, shuffling and flattening.
pub fn parse_os_json<R: Rng + ?Sized>(root: &Value, locale: &str, rng: &mut R) -> Vec<Value> {
    let mut list = prune_invalid_init_formats(list_for_locale(root, locale));
    shuffle_random(&mut list, rng);
    flatten_subitems(&mut list);
    list
}

#[cfg(test)]
mod tests;
