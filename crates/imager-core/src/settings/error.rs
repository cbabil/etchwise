//! Settings file errors.

use std::path::PathBuf;

/// Errors reading or writing the settings file.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("could not access settings file {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("settings file {path} is not valid JSON: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
}
