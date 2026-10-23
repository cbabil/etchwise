//! Write-side calls: staging customisation, progress and control.

use imager_core::customise::Settings as CustomSettings;
use imager_core::customise::plan::{PlanInputs, plan_from_settings};
use serde_json::{Value, json};

use super::args::{Args, Handled, ok};
use crate::state::State;

pub fn handle(state: &mut State, method: &str, args: &Args<'_>) -> Handled {
    match method {
        "applyCustomisationFromSettings" => {
            let map = args.value(0).as_object().cloned().unwrap_or_default();
            let key_path = state.settings.string("secureboot_rsa_key");
            let inputs = PlanInputs {
                init_format: state.src.as_ref().map_or("", |s| s.init_format.as_str()),
                pi_connect_token: &state.connect_token,
                current_user: &current_user(),
                secure_boot_key_path: &key_path,
            };
            state.plan = Some(plan_from_settings(CustomSettings(&map), inputs));
            ok(Value::Null)
        }
        "readyToWrite" => ok(state.src.is_some() && state.dst.is_some()),
        "writeState" => ok(json!(state.progress.state())),
        "progress" => ok(json!(state.progress.snapshot())),
        "cancelWrite" => {
            state.progress.cancel();
            ok(Value::Null)
        }
        "skipCurrentVerification" => {
            state.progress.skip_verification();
            ok(Value::Null)
        }
        "setVerifyEnabled" => {
            state.verify_enabled = args.bool(0);
            ok(Value::Null)
        }
        _ => None,
    }
}

/// Login name, used for key-only SSH without a username (`getCurrentUser`).
fn current_user() -> String {
    std::env::var("USER").unwrap_or_default()
}
