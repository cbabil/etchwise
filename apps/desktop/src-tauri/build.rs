//! Tauri build script: generates the embedded context and declares the app's
//! own commands so the window must be granted each one in `capabilities/`.

const COMMANDS: &[&str] = &["imager"];

fn main() {
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
        .unwrap_or_else(|error| panic!("tauri build failed: {error}"));
}
