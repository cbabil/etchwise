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

/// `beginOSListFetch`: loads the current repository into a fresh store,
/// then swaps it in and emits `osListPrepared` (or
/// `osListUnavailableChanged` when offline).
pub fn begin_os_list_fetch(shared: &Shared, app: &AppHandle) {
    let (shared, app) = (Arc::clone(shared), app.clone());
    let repo = lock(&shared).repo.clone();
    std::thread::spawn(move || {
        let mut store = OsListStore::default();
        let loaded = load_repository(&mut store, &repo);
        let mut state = lock(&shared);
        state.store = store;
        state.hw_devices.clear();
        match loaded {
            Ok(loaded) => {
                state.repo_display = loaded.final_url;
                drop(state);
                emit(&app, "osListPrepared", Value::Null);
            }
            Err(e) => {
                drop(state);
                emit(&app, "osListUnavailableChanged", json!(e.to_string()));
            }
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
