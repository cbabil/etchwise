//! Every `call('<method>')` in the React UI must reach a backend handler.
//! The browser mock answers any method, so without this check a method
//! missing in Rust only shows up as a blank window in the real app.

use std::path::Path;

use super::dispatch;
use crate::state::State;
use imager_core::settings::Settings;

/// Methods `lib.rs` handles itself, before `dispatch`.
const SHELL_METHODS: &[&str] = &[
    "beginOSListFetch",
    "refreshOsListFrom",
    "refreshOsListFromDefaultUrl",
    "startWrite",
    "overwriteConnectToken",
    "openUrl",
];

/// Methods that reach the keychain or Wi-Fi and could prompt the user;
/// checked by name in the handler sources instead of being called.
const PROMPTING_METHODS: &[&str] = &["getPSKForSSID", "getSSID"];

fn handled_by_name(method: &str) -> bool {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/api");
    let quoted = format!("\"{method}\"");
    std::fs::read_dir(dir)
        .expect("read api dir")
        .flatten()
        .any(|f| std::fs::read_to_string(f.path()).is_ok_and(|t| t.contains(&quoted)))
}

/// The method names in `call('name'` / `call<T>(\n  'name'` forms.
fn called_methods(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = source;
    while let Some(at) = rest.find("call") {
        let before = &rest[..at];
        rest = &rest[at + 4..];
        if before.ends_with(|c: char| c.is_alphanumeric() || c == '_' || c == '.') {
            continue;
        }
        let mut tail = rest;
        if tail.starts_with('<') {
            let mut depth = 0;
            let end = tail.find(|c: char| {
                depth += i32::from(c == '<') - i32::from(c == '>');
                depth == 0
            });
            let Some(end) = end else { continue };
            tail = &tail[end + 1..];
        }
        let Some(args) = tail.strip_prefix('(') else {
            continue;
        };
        let args = args.trim_start();
        let Some(quote) = args.chars().next().filter(|c| *c == '\'' || *c == '"') else {
            continue;
        };
        let name = args[1..].split(quote).next().unwrap_or_default();
        if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric()) {
            names.push(name.to_owned());
        }
    }
    names
}

fn ui_sources(dir: &Path, out: &mut Vec<(String, String)>) {
    for entry in std::fs::read_dir(dir).expect("read ui dir").flatten() {
        let path = entry.path();
        let name = path.to_string_lossy().into_owned();
        if path.is_dir() {
            ui_sources(&path, out);
        } else if (name.ends_with(".ts") || name.ends_with(".tsx")) && !name.contains("/api/mock") {
            out.push((name, std::fs::read_to_string(&path).expect("read ui file")));
        }
    }
}

#[test]
fn finds_calls_in_every_form() {
    let src = "call('a'); call<X | null>(\n  'b', 1); mockCall('c'); x.call('d'); call('<m>')";
    assert_eq!(called_methods(src), ["a", "b"]);
}

#[test]
fn every_ui_call_has_a_backend_handler() {
    let ui = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src");
    let mut files = Vec::new();
    ui_sources(&ui, &mut files);
    let mut state = State::new(Settings::in_memory(), std::env::temp_dir());
    let mut missing = Vec::new();
    let mut seen = 0;
    for (file, text) in &files {
        for method in called_methods(text) {
            seen += 1;
            if SHELL_METHODS.contains(&method.as_str()) {
                continue;
            }
            if PROMPTING_METHODS.contains(&method.as_str()) {
                if !handled_by_name(&method) {
                    missing.push(format!("{method} ({file})"));
                }
                continue;
            }
            if let Err(e) = dispatch(&mut state, &method, &[])
                && e.starts_with("unknown imageWriter method")
            {
                missing.push(format!("{method} ({file})"));
            }
        }
    }
    assert!(seen > 50, "found only {seen} calls; parser broken?");
    assert!(missing.is_empty(), "no backend handler for: {missing:#?}");
}
