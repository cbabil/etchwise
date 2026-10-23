//! Platform-independent core of the Pi Imager port.
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
pub mod telemetry;
#[cfg(test)]
mod testutil;
mod urlparts;

/// Version string as upstream shows it (`git describe` of tag `v2.0.7`).
/// The workspace `Cargo.toml` version is the single source.
pub const IMAGER_VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));
