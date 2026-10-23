//! Positional argument access for QML-style calls.

use serde_json::Value;

/// Arguments of one call.
pub struct Args<'a>(pub &'a [Value]);

impl Args<'_> {
    pub fn value(&self, i: usize) -> Value {
        self.0.get(i).cloned().unwrap_or(Value::Null)
    }

    pub fn str(&self, i: usize) -> String {
        match self.0.get(i) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Null) | None => String::new(),
            Some(other) => other.to_string(),
        }
    }

    pub fn bool(&self, i: usize) -> bool {
        self.0
            .get(i)
            .is_some_and(imager_core::settings::variant_to_bool)
    }

    /// A signed integer (negative indices mean "none"), default -1.
    pub fn i64(&self, i: usize) -> i64 {
        self.0
            .get(i)
            .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
            .unwrap_or(-1)
    }

    pub fn u64(&self, i: usize) -> u64 {
        self.0
            .get(i)
            .and_then(|v| v.as_u64().or_else(|| v.as_f64().map(|f| f as u64)))
            .unwrap_or(0)
    }
}

/// A handler's result: `None` when the method belongs to another module.
pub type Handled = Option<Result<Value, String>>;

/// Wraps an infallible value.
pub fn ok(value: impl Into<Value>) -> Handled {
    Some(Ok(value.into()))
}

/// Wraps a fallible value, turning errors into strings.
pub fn done<T: Into<Value>, E: std::fmt::Display>(result: Result<T, E>) -> Handled {
    Some(result.map(Into::into).map_err(|e| e.to_string()))
}
