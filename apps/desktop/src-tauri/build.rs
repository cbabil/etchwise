//! Tauri build script: generates the embedded context and declares the app's
//! own commands so the window must be granted each one in `capabilities/`.
//! Windows release builds embed a manifest that runs the app as
//! administrator, which raw disk writes need; debug builds (and their test
//! binaries) run unelevated so `cargo test` works without an admin shell.

const COMMANDS: &[&str] = &["imager"];

fn main() {
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    let mut windows = tauri_build::WindowsAttributes::new();
    if std::env::var("PROFILE").as_deref() == Ok("release") {
        windows = windows.app_manifest(include_str!("windows-app-manifest.xml"));
    }
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(manifest)
            .windows_attributes(windows),
    )
    .unwrap_or_else(|error| panic!("tauri build failed: {error}"));
}
