//! Calls that run off the UI thread and report back through events
//! (upstream signals): repository loading and the write job.

use std::sync::{Arc, Mutex, PoisonError};

use imager_core::oslist::fetch::{Loaded, load_repository};
use imager_core::oslist::store::{OsListStore, load_extra_repository};
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

/// Setting holding the extra repository URL (empty for none).
pub const EXTRA_REPO_SETTING: &str = "extraRepo";

/// A freshly loaded OS list: the main repository, plus the extra one when
/// set. A failing extra list leaves the main one in place.
struct Lists {
    store: OsListStore,
    main: std::io::Result<Loaded>,
    extra_error: String,
}

fn load_lists(repo: &str, extra: &str) -> Lists {
    let mut store = OsListStore::default();
    let main = load_repository(&mut store, repo);
    let extra_error = if main.is_ok() && !extra.is_empty() {
        load_extra_repository(&mut store, extra)
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };
    Lists {
        store,
        main,
        extra_error,
    }
}

/// `beginOSListFetch`: loads the current repository (and the extra one)
/// into a fresh store, then swaps it in and emits the [`fetch_signals`].
pub fn begin_os_list_fetch(shared: &Shared, app: &AppHandle) {
    let (shared, app) = (Arc::clone(shared), app.clone());
    let (repo, extra) = {
        let state = lock(&shared);
        (
            state.repo.clone(),
            state.settings.string(EXTRA_REPO_SETTING),
        )
    };
    std::thread::spawn(move || {
        let lists = load_lists(&repo, &extra);
        let mut state = lock(&shared);
        let was_available = state.store.is_available();
        state.store = lists.store;
        state.extra_repo_error = lists.extra_error;
        state.hw_devices.clear();
        let loaded = lists
            .main
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

    fn write_list(dir: &std::path::Path, name: &str, doc: &Value) -> String {
        let path = dir.join(name);
        std::fs::write(&path, doc.to_string()).unwrap();
        format!("file://{}", path.display())
    }

    fn device_names(store: &OsListStore) -> Vec<String> {
        store.imager_metadata()["devices"]
            .as_array()
            .map(|d| {
                d.iter()
                    .filter_map(|x| x["name"].as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    }

    #[test]
    fn loads_the_extra_list_on_top_of_the_main_one() {
        let dir = tempfile::tempdir().unwrap();
        let main = write_list(
            dir.path(),
            "main.json",
            &json!({
                "imager": {"devices": [{"name": "Raspberry Pi 5", "tags": ["pi5"]}]}, "os_list": []
            }),
        );
        let extra = write_list(
            dir.path(),
            "nuc.json",
            &json!({
                "imager": {"devices": [{"name": "Intel NUC", "tags": ["x86-64"]}]}, "os_list": []
            }),
        );
        let lists = load_lists(&main, &extra);
        assert!(lists.main.is_ok());
        assert_eq!(lists.extra_error, "");
        assert_eq!(device_names(&lists.store), ["Raspberry Pi 5", "Intel NUC"]);

        let missing = format!("file://{}/gone.json", dir.path().display());
        let lists = load_lists(&main, &missing);
        assert!(lists.main.is_ok());
        assert!(!lists.extra_error.is_empty());
        assert_eq!(device_names(&lists.store), ["Raspberry Pi 5"]);

        let lists = load_lists(&main, "");
        assert_eq!(
            (lists.extra_error.as_str(), device_names(&lists.store).len()),
            ("", 1)
        );
    }

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
