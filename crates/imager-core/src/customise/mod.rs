//! OS customisation: `firstrun.sh` for systemd images and cloud-init
//! `user-data` / `network-config`, ported from upstream
//! `customization_generator.cpp`.

mod cloudinit;
mod cloudinit_extras;
pub mod plan;
mod systemd;
mod systemd_extras;
mod systemd_wifi;
mod text;

pub use cloudinit::{UserDataOptions, cloudinit_user_data};
pub use cloudinit_extras::cloudinit_network_config;
pub use systemd::systemd_script;
pub use text::{Settings, pbkdf2_psk};

#[cfg(test)]
mod tests;
