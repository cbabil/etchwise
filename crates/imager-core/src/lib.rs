//! Platform-independent core of Etchwise.
//!
//! Everything here is pure Rust with no OS calls, so it is fully unit-tested.
//! Disk, Wi-Fi and power access live behind the traits in `imager-platform`.

pub mod crypto;
pub mod customise;
pub mod data;
pub mod drives;
pub mod erase;
pub mod fat;
pub mod format;
pub mod oslist;
pub mod pipeline;
pub mod secureboot;
pub mod settings;
#[cfg(test)]
mod testutil;
mod urlparts;

/// The app name shown to users.
pub const APP_NAME: &str = "Etchwise";

/// Version string, e.g. `v1.0.0`; the workspace `Cargo.toml` version is the
/// single source.
pub const IMAGER_VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));
