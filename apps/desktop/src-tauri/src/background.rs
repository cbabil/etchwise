//! Calls that run off the UI thread and report back through events
//! (upstream signals): repository loading and the write job.

use std::sync::{Arc, Mutex, PoisonError};

use imager_core::oslist::fetch::load_repository;
use imager_core::oslist::store::OsListStore;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

use crate::state::State;

/// Shared backend state.
pub type Shared = Arc<Mutex<State>>;

/// Event channel name; payload `{name, data}` mirrors upstream signals.
pub const EVENT: &str = "imager://event";

pub fn emit(app: &AppHandle, name: &str, data: Value) {
    let _ = app.emit(EVENT, json!({"name": name, "data": data}));
}

pub fn lock(shared: &Shared) -> std::sync::MutexGuard<'_, State> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The signals for a finished fetch, in upstream's order: the first list
/// to arrive also flips `isOsListUnavailable` (which moves the wizard from
/// OS back to Device); a failure leaves the list empty, so unavailable.
fn fetch_signals(was_available: bool, loaded: Result<(), String>) -> Vec<(&'static str, Value)> {
    match loaded {
        Ok(()) if was_available => vec![("osListPrepared", Value::Null)],
        Ok(()) => vec![
            ("osListUnavailableChanged", Value::Null),
            ("osListPrepared", Value::Null),
        ],
        Err(e) => vec![("osListUnavailableChanged", json!(e))],
    }
}

/// `beginOSListFetch`: loads the current repository into a fresh store,
/// then swaps it in and emits the [`fetch_signals`].
pub fn begin_os_list_fetch(shared: &Shared, app: &AppHandle) {
    let (shared, app) = (Arc::clone(shared), app.clone());
    let repo = lock(&shared).repo.clone();
    std::thread::spawn(move || {
        let mut store = OsListStore::default();
        let loaded = load_repository(&mut store, &repo);
        let mut state = lock(&shared);
        let was_available = state.store.is_available();
        state.store = store;
        state.hw_devices.clear();
        let loaded = loaded
            .map(|loaded| state.repo_display = loaded.final_url)
            .map_err(|e| e.to_string());
        drop(state);
        for (name, data) in fetch_signals(was_available, loaded) {
            emit(&app, name, data);
        }
    });
}

/// `refreshOsListFrom(url)`: switch repository and reload.
pub fn refresh_os_list_from(shared: &Shared, app: &AppHandle, url: &str) {
    {
        let mut state = lock(shared);
        state.repo = url.to_owned();
        state.repo_display = url.to_owned();
        state.store.clear();
    }
    emit(app, "customRepoChanged", Value::Null);
    begin_os_list_fetch(shared, app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_load_announces_availability_then_the_list() {
        assert_eq!(
            fetch_signals(false, Ok(())),
            [
                ("osListUnavailableChanged", Value::Null),
                ("osListPrepared", Value::Null)
            ]
        );
    }

    #[test]
    fn reload_only_announces_the_list() {
        assert_eq!(
            fetch_signals(true, Ok(())),
            [("osListPrepared", Value::Null)]
        );
    }

    #[test]
    fn failure_announces_unavailability() {
        assert_eq!(
            fetch_signals(false, Err("offline".into())),
            [("osListUnavailableChanged", json!("offline"))]
        );
    }
}
