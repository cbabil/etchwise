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
