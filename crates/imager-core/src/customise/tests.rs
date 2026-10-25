//! Ports of upstream `src/test/customization_generator_test.cpp` (Catch2).

use serde_json::{Map, Value};

mod autoinstall_cases;
mod cloudinit_cases;
mod cloudinit_cases2;
mod combined_cases;
mod helpers;
mod independent_cases;
mod independent_cases2;
mod kura_cases;
mod network_cases;
mod plan_cases;
mod systemd_cases;
mod systemd_cases2;
mod systemd_cases3;
mod systemd_cases4;

/// Builds a settings map from `serde_json::json!({...})`.
pub(super) fn map(value: Value) -> Map<String, Value> {
    value.as_object().cloned().unwrap_or_default()
}
