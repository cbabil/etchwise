//! QVariant-style conversions for settings values, and the SSH key
//! de-duplication upstream applies when loading saved customisation.

use serde_json::Value;
use sha2::{Digest, Sha256};

/// `QVariant::toBool` for the JSON types settings hold.
pub fn variant_to_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !(s.is_empty() || s == "0" || s.eq_ignore_ascii_case("false")),
        _ => false,
    }
}

pub(crate) fn variant_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Unique trimmed key lines in first-seen order (SHA-256 identity, as upstream).
pub(super) fn dedupe_keys(raw: &str) -> String {
    let mut seen: Vec<[u8; 32]> = Vec::new();
    let mut unique = Vec::new();
    for line in raw
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l).trim())
        .filter(|l| !l.is_empty())
    {
        let hash: [u8; 32] = Sha256::digest(line.as_bytes()).into();
        if !seen.contains(&hash) {
            seen.push(hash);
            unique.push(line);
        }
    }
    unique.join("\n")
}
