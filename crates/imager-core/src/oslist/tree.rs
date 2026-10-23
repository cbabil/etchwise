//! Tree operations on the raw OS-list JSON, ported from the anonymous
//! helpers in upstream `imagewriter.cpp`: sublist insertion, hardware-tag
//! filtering and sublist URL discovery.

use serde_json::{Map, Value};

use super::json::{as_slice, with_field};
use crate::urlparts;

/// Upstream `MAX_SUBITEMS_DEPTH`: nesting limit for sublists.
pub const MAX_SUBITEMS_DEPTH: u8 = 16;

/// Splices `incoming` in place of every `subitems_url` equal to `url`
/// (`findAndInsertJsonResult`). Edits the tree in place; non-object items
/// become `{}` and groups nested too deep are emptied, as upstream.
pub fn insert_sublist(list: &mut [Value], incoming: &[Value], url: &str) {
    insert_at(list, incoming, url, 1);
}

fn insert_at(list: &mut [Value], incoming: &[Value], url: &str, depth: u8) {
    for item in list {
        if !item.is_object() {
            *item = Value::Object(Map::new());
        }
        let Some(object) = item.as_object_mut() else {
            continue;
        };
        if let Some(subitems) = object.get_mut("subitems") {
            match subitems {
                Value::Array(nested) if depth < MAX_SUBITEMS_DEPTH => {
                    insert_at(nested, incoming, url, depth + 1)
                }
                _ => *subitems = Value::Array(Vec::new()),
            }
        } else if object.get("subitems_url").and_then(Value::as_str) == Some(url) {
            object.remove("subitems_url");
            object.insert("subitems".into(), Value::Array(incoming.to_vec()));
        }
    }
}

/// Keeps leaves whose `devices` intersect `tags`; untagged leaves are kept
/// only for inclusive filters and empty groups are dropped
/// (`filterOsListWithHWTags`).
pub fn filter_with_hw_tags(list: &[Value], tags: &[String], inclusive: bool) -> Vec<Value> {
    filter_at(list, tags, inclusive, 1)
}

fn filter_at(list: &[Value], tags: &[String], inclusive: bool, depth: u8) -> Vec<Value> {
    if depth > MAX_SUBITEMS_DEPTH {
        return Vec::new();
    }
    list.iter()
        .filter_map(|item| {
            if let Some(subitems) = item.get("subitems") {
                let nested = filter_at(as_slice(subitems), tags, inclusive, depth + 1);
                (!nested.is_empty()).then(|| with_field(item, "subitems", Value::Array(nested)))
            } else if let Some(devices) = item.get("devices") {
                let matches = as_slice(devices)
                    .iter()
                    .any(|d| tags.iter().any(|t| Some(t.as_str()) == d.as_str()));
                matches.then(|| item.clone())
            } else {
                inclusive.then(|| item.clone())
            }
        })
        .collect()
}

/// Every fetchable `subitems_url` in the tree, in document order
/// (`queueSublistFetches`).
pub fn sublist_urls(list: &[Value]) -> Vec<String> {
    let mut urls = Vec::new();
    collect_urls(list, 1, &mut urls);
    urls
}

fn collect_urls(list: &[Value], depth: u8, urls: &mut Vec<String>) {
    if depth > MAX_SUBITEMS_DEPTH {
        return;
    }
    for item in list {
        if let Some(subitems) = item.get("subitems") {
            collect_urls(as_slice(subitems), depth + 1, urls);
        } else if let Some(url) = item.get("subitems_url").and_then(Value::as_str)
            && is_fetchable_url(url)
        {
            urls.push(url.to_owned());
        }
    }
}

/// Upstream `preflightValidateUrl`: needs a scheme, and http(s) needs a host.
pub fn is_fetchable_url(url: &str) -> bool {
    urlparts::scheme(url)
        .is_some_and(|scheme| !urlparts::is_http(&scheme) || !urlparts::host(url).is_empty())
}

#[cfg(test)]
mod tests;
