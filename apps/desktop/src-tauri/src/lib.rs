//! Tauri shell of the Pi Imager port: one `imager` command carrying
//! upstream's `imageWriter.<method>(…)` calls, events for its signals.

mod api;
mod app_dirs;
mod background;
pub mod cli;
mod cli_plan;
mod deeplink;
mod launch_args;
mod monitor;
mod state;
mod write_job;

use std::sync::{Arc, Mutex};

use imager_core::oslist::fetch::DEFAULT_REPO;
use imager_core::settings::Settings;
use imager_core::{APP_NAME, IMAGER_VERSION};
use serde_json::Value;
use tauri::{AppHandle, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_opener::OpenerExt;

use self::background::{Shared, begin_os_list_fetch, lock, refresh_os_list_from};
use self::state::State;

/// The window title: the app name and version.
fn window_title() -> String {
    format!("{APP_NAME} {IMAGER_VERSION}")
}

/// `imageWriter.<method>(args…)`.
#[tauri::command]
fn imager(
    method: String,
    args: Vec<Value>,
    shared: tauri::State<'_, Shared>,
    app: AppHandle,
) -> Result<Value, String> {
    match method.as_str() {
        "beginOSListFetch" => begin_os_list_fetch(&shared, &app),
        "refreshOsListFrom" => refresh_os_list_from(
            &shared,
            &app,
            args.first().and_then(Value::as_str).unwrap_or_default(),
        ),
        "refreshOsListFromDefaultUrl" => refresh_os_list_from(&shared, &app, DEFAULT_REPO),
        "startWrite" => write_job::start_write(&shared, &app)?,
        "overwriteConnectToken" => {
            let token = args
                .first()
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_owned();
            lock(&shared).connect_token.clone_from(&token);
            background::emit(&app, "connectTokenReceived", Value::String(token));
        }
        "openUrl" => app
            .opener()
            .open_url(
                args.first().and_then(Value::as_str).unwrap_or_default(),
                None::<&str>,
            )
            .map_err(|e| e.to_string())?,
        _ => return api::dispatch(&mut lock(&shared), &method, &args),
    }
    Ok(Value::Null)
}

fn load_state(repo: Option<String>) -> State {
    let config = app_dirs::app_dir(&dirs::config_dir().unwrap_or_default());
    let settings =
        Settings::load(config.join("settings.json")).unwrap_or_else(|_| Settings::in_memory());
    let mut state = State::new(
        settings,
        app_dirs::app_dir(&dirs::cache_dir().unwrap_or_default()),
    );
    if let Some(repo) = repo {
        state.repo_display.clone_from(&repo);
        state.repo = repo;
    }
    state
}

/// Builds and runs the Tauri application.
///
/// # Panics
/// Panics if the Tauri runtime fails to start, which leaves nothing to recover.
pub fn run() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let repo = launch_args::repo_from_args(&args).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });
    let shared: Shared = Arc::new(Mutex::new(load_state(repo)));
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .manage(Arc::clone(&shared))
        .setup(move |app| {
            if let Some(window) = app.get_webview_window("main") {
                window.set_title(&window_title())?;
            }
            write_job::verify_cache_in_background(&shared);
            let (links, handle) = (Arc::clone(&shared), app.handle().clone());
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    deeplink::handle_incoming_url(&links, &handle, url.as_str());
                }
            });
            begin_os_list_fetch(&shared, app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![imager])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| panic!("failed to run Pi Imager: {error}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_names_the_app_and_its_version() {
        assert_eq!(window_title(), "Etchwise v1.0.0");
    }

    #[test]
    fn dispatches_synchronous_calls() {
        let mut state = State::new(Settings::in_memory(), std::env::temp_dir());
        assert_eq!(
            api::dispatch(&mut state, "getBoolSetting", &["eject".into()]),
            Ok(Value::Bool(true))
        );
        assert_eq!(
            api::dispatch(&mut state, "constantVersion", &[]),
            Ok(Value::String("v1.0.0".into()))
        );
        assert_eq!(
            api::dispatch(&mut state, "isOsListUnavailable", &[]),
            Ok(Value::Bool(true))
        );
        assert!(api::dispatch(&mut state, "noSuchMethod", &[]).is_err());
    }
}
