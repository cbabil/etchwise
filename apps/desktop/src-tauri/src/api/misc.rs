//! Environment flags, debug options and Raspberry Pi Connect tokens.

use serde_json::{Value, json};

use super::args::{Args, Handled, ok};
use crate::state::{DebugOptions, State};

pub fn handle(state: &mut State, method: &str, args: &Args<'_>) -> Handled {
    match method {
        "isEmbeddedMode"
        | "isScreenReaderActive"
        | "isElevatableBundle"
        | "isSecureBootForcedByCliFlag"
        | "hasPerformanceData" => ok(false),
        "isBeepAvailable" | "nativeFileDialogAvailable" | "hasWindowDecorations" => ok(true),
        // Upstream `main.cpp`: shown for a saved language preference, in
        // embedded mode, or when the system language is unknown (never on
        // macOS, which always reports one).
        "showLanguageSelection" => ok(!state.settings.string("savedLanguage").is_empty()),
        // Upstream runs `reboot`, reachable only in embedded mode (never on macOS).
        "reboot" => ok(Value::Null),
        "getExtraRepoError" => ok(state.extra_repo_error.clone()),
        "getDebugOptions" => ok(json!(state.debug)),
        "setDebugOptions" => {
            state.debug = serde_json::from_value::<DebugOptions>(args.value(0)).unwrap_or_default();
            ok(Value::Null)
        }
        "getRuntimeConnectToken" => ok(state.connect_token.clone()),
        "verifyAuthKey" => ok(verify_auth_key(&args.str(0), args.bool(1))),
        "getRsaKeyFingerprint" => ok(std::fs::read_to_string(args.str(0))
            .ok()
            .and_then(|pem| imager_core::secureboot::key_fingerprint(&pem).ok())
            .unwrap_or_default()),
        _ => None,
    }
}

/// `rpuak_` / `rpoak_` followed by 24+ base58 characters.
pub fn is_connect_token(token: &str) -> bool {
    verify_auth_key(token, false)
}

/// Upstream `verifyAuthKey(token, strict)`: strict demands exactly 24.
fn verify_auth_key(token: &str, strict: bool) -> bool {
    let token = token.trim();
    let body = token
        .strip_prefix("rpuak_")
        .or_else(|| token.strip_prefix("rpoak_"));
    let base58 = |b: &str| {
        !b.is_empty()
            && b.chars()
                .all(|c| c.is_ascii_alphanumeric() && !"0OIl".contains(c))
    };
    body.is_some_and(|b| base58(b) && if strict { b.len() == 24 } else { b.len() >= 24 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_why_the_extra_repository_failed() {
        let mut state = State::new(
            imager_core::settings::Settings::in_memory(),
            std::env::temp_dir(),
        );
        let ask = |state: &mut State| handle(state, "getExtraRepoError", &Args(&[]));
        assert_eq!(ask(&mut state), Some(Ok(json!(""))));
        state.extra_repo_error = "not found".into();
        assert_eq!(ask(&mut state), Some(Ok(json!("not found"))));
    }

    #[test]
    fn shows_language_selection_only_for_a_saved_language() {
        let mut state = State::new(
            imager_core::settings::Settings::in_memory(),
            std::env::temp_dir(),
        );
        let ask = |state: &mut State| handle(state, "showLanguageSelection", &Args(&[]));
        assert_eq!(ask(&mut state), Some(Ok(Value::Bool(false))));
        state.settings.set("savedLanguage", json!("fr")).unwrap();
        assert_eq!(ask(&mut state), Some(Ok(Value::Bool(true))));
    }

    #[test]
    fn validates_connect_tokens() {
        assert!(is_connect_token("rpuak_abcdefghijkmnpqrstuvwxyz123"));
        assert!(is_connect_token(" rpoak_ABCDEFGHJKLMNPQRSTUVWXYZ "));
        assert!(!is_connect_token("rpuak_short"));
        assert!(!is_connect_token("rpuak_0OIlabcdefghijkmnpqrstuvwx"));
        assert!(!is_connect_token("token_abcdefghijkmnpqrstuvwxyz123"));
        assert!(verify_auth_key("rpuak_abcdefghijkmnpqrstuvwxyz", true));
        assert!(!verify_auth_key("rpuak_abcdefghijkmnpqrstuvwxyz123", true));
    }
}
