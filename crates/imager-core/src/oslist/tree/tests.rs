use serde_json::{Value, json};

use super::*;

fn tags(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

fn array(v: Value) -> Vec<Value> {
    v.as_array().unwrap().clone()
}

/// `levels` nested groups around one leaf.
fn nested(levels: usize, leaf: Value) -> Vec<Value> {
    (0..levels).fold(vec![leaf], |inner, _| {
        vec![json!({"name": "g", "subitems": inner})]
    })
}

#[test]
fn inserts_sublist_at_matching_subitems_url() {
    let mut list = array(json!([
        {"name": "A", "subitems_url": "https://x/a.json"},
        {"name": "B", "subitems_url": "https://x/b.json"}
    ]));
    insert_sublist(&mut list, &[json!({"name": "A1"})], "https://x/a.json");
    assert_eq!(list[0], json!({"name": "A", "subitems": [{"name": "A1"}]}));
    assert_eq!(
        list[1],
        json!({"name": "B", "subitems_url": "https://x/b.json"})
    );
}

#[test]
fn inserts_into_nested_subitems() {
    let mut list = array(json!([{"name": "G", "subitems": [{"name": "N", "subitems_url": "u"}]}]));
    insert_sublist(&mut list, &[json!({"name": "X"})], "u");
    assert_eq!(list[0]["subitems"][0]["subitems"], json!([{"name": "X"}]));
}

#[test]
fn insert_normalises_like_qjson() {
    let mut list = array(json!([5, {"name": "g", "subitems": "oops"}]));
    insert_sublist(&mut list, &[], "u");
    assert_eq!(list, [json!({}), json!({"name": "g", "subitems": []})]);
}

#[test]
fn insert_empties_groups_beyond_max_depth() {
    let leaf = json!({"subitems_url": "u"});
    let mut shallow = nested(usize::from(MAX_SUBITEMS_DEPTH) - 1, leaf.clone());
    insert_sublist(&mut shallow, &[json!({"name": "X"})], "u");
    assert!(shallow[0].to_string().contains("\"X\""));
    let mut deep = nested(usize::from(MAX_SUBITEMS_DEPTH), leaf);
    insert_sublist(&mut deep, &[json!({"name": "X"})], "u");
    assert!(!deep[0].to_string().contains("\"X\""));
}

#[test]
fn hw_filter_keeps_leaves_with_matching_device() {
    let list = array(json!([
        {"name": "pi5", "devices": ["pi5-64bit"]},
        {"name": "pi4", "devices": ["pi4-64bit"]},
        {"name": "any"}
    ]));
    let out = filter_with_hw_tags(&list, &tags(&["pi5-64bit"]), false);
    let names: Vec<_> = out.iter().map(|v| v["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["pi5"]);
}

#[test]
fn hw_filter_inclusive_keeps_untagged_leaves() {
    let list = array(json!([{"name": "any"}, {"name": "pi4", "devices": ["pi4-64bit"]}]));
    assert_eq!(
        filter_with_hw_tags(&list, &tags(&["pi5-64bit"]), true),
        [json!({"name": "any"})]
    );
}

#[test]
fn hw_filter_drops_empty_parents_and_keeps_full_ones() {
    let list = array(json!([
        {"name": "empty", "subitems": [{"name": "x", "devices": ["pi1-32bit"]}]},
        {"name": "full", "subitems": [{"name": "y", "devices": ["pi5-64bit"]}]}
    ]));
    let out = filter_with_hw_tags(&list, &tags(&["pi5-64bit"]), false);
    assert_eq!(
        out,
        [json!({"name": "full", "subitems": [{"name": "y", "devices": ["pi5-64bit"]}]})]
    );
}

#[test]
fn hw_filter_drops_leaves_beyond_max_depth() {
    let leaf = json!({"name": "any"});
    assert_eq!(
        filter_with_hw_tags(
            &nested(usize::from(MAX_SUBITEMS_DEPTH) - 1, leaf.clone()),
            &[],
            true
        )
        .len(),
        1
    );
    assert!(
        filter_with_hw_tags(&nested(usize::from(MAX_SUBITEMS_DEPTH), leaf), &[], true).is_empty()
    );
}

#[test]
fn collects_valid_sublist_urls_recursively() {
    let list = array(json!([
        {"subitems_url": "https://downloads.example.com/a.json"},
        {"subitems": [{"subitems_url": "file:///tmp/b.json"}]},
        {"subitems_url": "relative/c.json"},
        {"subitems_url": "https:///nohost.json"}
    ]));
    assert_eq!(
        sublist_urls(&list),
        ["https://downloads.example.com/a.json", "file:///tmp/b.json"]
    );
}

#[test]
fn sublist_urls_stop_beyond_max_depth() {
    let leaf = json!({"subitems_url": "https://h/a"});
    assert_eq!(
        sublist_urls(&nested(usize::from(MAX_SUBITEMS_DEPTH) - 1, leaf.clone())).len(),
        1
    );
    assert!(sublist_urls(&nested(usize::from(MAX_SUBITEMS_DEPTH), leaf)).is_empty());
}

#[test]
fn validates_urls_like_preflight() {
    assert!(is_fetchable_url("https://downloads.raspberrypi.com/x.json"));
    assert!(is_fetchable_url("HTTP://host/x"));
    assert!(is_fetchable_url("file:///Users/me/list.json"));
    assert!(!is_fetchable_url("list.json"));
    assert!(!is_fetchable_url("https://"));
    assert!(!is_fetchable_url("http:///path"));
    assert!(!is_fetchable_url(""));
}
