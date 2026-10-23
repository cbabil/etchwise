//! `QJsonValue`-style accessors: a value of the wrong type reads as empty.

use serde_json::{Map, Value};

/// A JSON array as a slice; anything else reads as empty (`toArray()`).
pub(crate) fn as_slice(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}

/// Takes ownership of an array's items; anything else yields none.
pub(crate) fn into_vec(value: Value) -> Vec<Value> {
    match value {
        Value::Array(items) => items,
        _ => Vec::new(),
    }
}

/// A string field, empty when missing or not a string (`toString()`).
pub(crate) fn str_field(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

/// A bool field, false when missing or not a bool (`toBool()`).
pub(crate) fn bool_field(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

/// Array items as strings; non-strings read as empty.
pub(crate) fn strings(items: &[Value]) -> Vec<String> {
    items
        .iter()
        .map(|x| x.as_str().unwrap_or_default().to_owned())
        .collect()
}

/// A string-array field.
pub(crate) fn str_list(v: &Value, key: &str) -> Vec<String> {
    v.get(key).map(|a| strings(as_slice(a))).unwrap_or_default()
}

/// The object itself, or `{}` for anything else (`toObject()`).
pub(crate) fn object_or_empty(value: Option<Value>) -> Value {
    match value {
        Some(object @ Value::Object(_)) => object,
        _ => Value::Object(Map::new()),
    }
}

/// A copy of `entry` (as an object) with `key` set to `value`, without
/// cloning the field being replaced.
pub(crate) fn with_field(entry: &Value, key: &str, value: Value) -> Value {
    let mut object: Map<String, Value> = entry
        .as_object()
        .map(|o| {
            o.iter()
                .filter(|(k, _)| *k != key)
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        })
        .unwrap_or_default();
    object.insert(key.to_owned(), value);
    Value::Object(object)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn accessors_default_like_qjson() {
        let v = json!({"s": "x", "n": 1, "b": true, "l": ["a", 2]});
        assert_eq!(str_field(&v, "s"), "x");
        assert_eq!(str_field(&v, "n"), "");
        assert!(bool_field(&v, "b") && !bool_field(&v, "s"));
        assert_eq!(str_list(&v, "l"), ["a", ""]);
        assert!(str_list(&v, "none").is_empty());
        assert!(into_vec(json!(3)).is_empty());
        assert_eq!(object_or_empty(Some(json!([1]))), json!({}));
        assert_eq!(object_or_empty(Some(json!({"a": 1}))), json!({"a": 1}));
        assert_eq!(
            with_field(&json!({"a": 1, "k": [9]}), "k", json!(2)),
            json!({"a": 1, "k": 2})
        );
        assert_eq!(with_field(&json!(5), "k", json!(2)), json!({"k": 2}));
    }
}
