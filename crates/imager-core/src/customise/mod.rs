//! OS customisation: `firstrun.sh` for systemd images and cloud-init
//! `user-data` / `network-config`, ported from upstream
//! `customization_generator.cpp`; plus Etchwise's Ubuntu autoinstall and
//! Kura setup file.

mod autoinstall;
mod cloudinit;
mod cloudinit_extras;
mod kura;
pub mod plan;
mod seed_plan;
mod systemd;
mod systemd_extras;
mod systemd_wifi;
mod text;
mod username;

pub use autoinstall::autoinstall_user_data;
pub use cloudinit::{UserDataOptions, cloudinit_user_data};
pub use cloudinit_extras::cloudinit_network_config;
pub use kura::kura_setup;
pub use systemd::systemd_script;
pub use text::{Settings, pbkdf2_psk};

#[cfg(test)]
mod tests;
