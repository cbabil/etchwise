use rand::SeedableRng;
use rand::rngs::StdRng;
use serde_json::json;

use super::*;

#[test]
fn locale_list_has_priority_then_language_then_default() {
    let root = json!({"os_list_de_DE": [{"name": "de_DE"}], "os_list_fr": [{"name": "fr"}], "os_list": [{"name": "default"}]});
    assert_eq!(list_for_locale(&root, "de_DE")[0]["name"], "de_DE");
    assert_eq!(list_for_locale(&root, "fr_CA")[0]["name"], "fr");
    assert_eq!(list_for_locale(&root, "en_GB")[0]["name"], "default");
    assert_eq!(list_for_locale(&root, "it")[0]["name"], "default");
}

#[test]
fn empty_locale_list_falls_back_to_default() {
    let root = json!({"os_list_de_DE": [], "os_list": [{"name": "default"}]});
    assert_eq!(list_for_locale(&root, "de_DE")[0]["name"], "default");
}

#[test]
fn missing_lists_give_empty() {
    assert!(list_for_locale(&json!({"imager": {}}), "en_GB").is_empty());
}

#[test]
fn prunes_invalid_init_formats_and_empty_parents() {
    let list = json!([
        {"name": "ok", "init_format": "systemd"},
        {"name": "bad", "init_format": "ignition"},
        {"name": "none", "init_format": "none"},
        {"name": "parent-bad", "subitems": [{"name": "x", "init_format": "weird"}]},
        {"name": "parent-ok", "subitems": [{"name": "y"}, {"name": "z", "init_format": "?"}]}
    ]);
    let out = prune_invalid_init_formats(list.as_array().unwrap().clone());
    let names: Vec<_> = out.iter().map(|v| v["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["ok", "none", "parent-ok"]);
    assert_eq!(out[2]["subitems"], json!([{"name": "y"}]));
}

#[test]
fn non_string_init_format_counts_as_empty() {
    let list = json!([{"name": "n", "init_format": 5}]);
    assert_eq!(
        prune_invalid_init_formats(list.as_array().unwrap().clone()).len(),
        1
    );
    assert_eq!(prune_invalid_init_formats(vec![json!(7)]), [json!({})]);
}

#[test]
fn shuffles_only_random_subitems_deterministically() {
    let items: Vec<_> = (0..20).map(|i| json!({"name": i})).collect();
    let mut list = vec![
        json!({"name": "r", "random": true, "subitems": items.clone()}),
        json!({"name": "s", "subitems": items.clone()}),
    ];
    shuffle_random(&mut list, &mut StdRng::seed_from_u64(7));
    assert_ne!(list[0]["subitems"], json!(items));
    assert_eq!(list[1]["subitems"], json!(items));
    let mut sorted: Vec<i64> = list[0]["subitems"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["name"].as_i64().unwrap())
        .collect();
    sorted.sort_unstable();
    assert_eq!(sorted, (0..20).collect::<Vec<_>>());
}

#[test]
fn flattens_subitems_to_json_string() {
    let mut list = vec![
        json!({"name": "g", "subitems": [{"name": "a"}]}),
        json!({"name": "leaf"}),
    ];
    flatten_subitems(&mut list);
    assert!(list[0].get("subitems").is_none());
    let parsed: serde_json::Value =
        serde_json::from_str(list[0]["subitems_json"].as_str().unwrap()).unwrap();
    assert_eq!(parsed, json!([{"name": "a"}]));
    assert!(list[1].get("subitems_json").is_none());
}

#[test]
fn sorts_preferred_architecture_first_stably() {
    let mut list = vec![
        json!({"name": "a", "architecture": "armhf"}),
        json!({"name": "b", "architecture": "arm64"}),
        json!({"name": "c"}),
        json!({"name": "d", "architecture": "arm64", "subitems": [
            {"name": "d1", "architecture": "armhf"}, {"name": "d2", "architecture": "arm64"}
        ]}),
    ];
    sort_by_architecture(&mut list, "arm64");
    let names: Vec<_> = list.iter().map(|v| v["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["b", "d", "a", "c"]);
    assert_eq!(list[1]["subitems"][0]["name"], "d2");
}

#[test]
fn empty_architecture_leaves_order() {
    let mut list = vec![
        json!({"name": "a", "architecture": "armhf"}),
        json!({"name": "b", "architecture": "arm64"}),
    ];
    sort_by_architecture(&mut list, "");
    assert_eq!(list[0]["name"], "a");
}

#[test]
fn parse_os_json_runs_full_pipeline() {
    let root = json!({"os_list": [
        {"name": "bad", "init_format": "x"},
        {"name": "g", "subitems": [{"name": "a"}]}
    ]});
    let out = parse_os_json(&root, "en_GB", &mut StdRng::seed_from_u64(1));
    assert_eq!(out.len(), 1);
    assert!(out[0]["subitems_json"].is_string());
}

#[test]
fn parse_os_json_without_list_is_empty() {
    assert!(parse_os_json(&json!({}), "en_GB", &mut StdRng::seed_from_u64(1)).is_empty());
}
