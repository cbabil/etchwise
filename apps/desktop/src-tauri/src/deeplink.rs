//! `rpi-imager://` links and opened manifest files (upstream
//! `ImageWriter::handleIncomingUrl`): a manifest or `.json` file switches
//! the repository, `repo=` asks the UI to confirm one, `auth_key=` sets the
//! Raspberry Pi Connect token (or reports a conflict).

use reqwest::Url;
use serde_json::json;
use tauri::AppHandle;

use crate::background::{Shared, emit, lock, refresh_os_list_from};

/// Handles one URL from the OS.
pub fn handle_incoming_url(shared: &Shared, app: &AppHandle, raw: &str) {
    let Ok(url) = Url::parse(raw) else { return };
    if url.scheme() == "file" {
        let path = url.path().to_lowercase();
        if path.ends_with(".rpi-imager-manifest") || path.ends_with(".json") {
            refresh_os_list_from(shared, app, raw);
        }
        return;
    }
    let query = |key: &str| {
        url.query_pairs()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.into_owned())
            .unwrap_or_default()
    };
    let repo = query("repo");
    if !repo.is_empty() && crate::api::is_valid_repo_url(&repo) {
        emit(app, "repositoryUrlReceived", json!(repo));
    }
    let token = query("auth_key");
    if token.is_empty() || !crate::api::is_connect_token(&token) {
        return;
    }
    let mut state = lock(shared);
    if state.connect_token.is_empty() {
        state.connect_token = token.clone();
        drop(state);
        emit(app, "connectTokenReceived", json!(token));
    } else if state.connect_token != token {
        drop(state);
        emit(app, "connectTokenConflictDetected", json!(token));
    }
}
