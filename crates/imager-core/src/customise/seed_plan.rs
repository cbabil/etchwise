//! The NoCloud files of a plan: `user-data` and `meta-data`, written to the
//! boot partition for cloud-init images and to a `CIDATA` seed partition for
//! Ubuntu autoinstall.

use super::plan::{AUTOINSTALL_UBUNTU, CustomisationPlan};

/// `user-data` file contents for the plan's cloud-init body.
pub(super) fn user_data_file(cloudinit: &str) -> String {
    format!("#cloud-config\n{cloudinit}")
}

/// `meta-data` file contents.
pub(super) fn meta_data_file(instance_id: &str) -> String {
    format!("instance-id: {instance_id}\n")
}

impl CustomisationPlan {
    /// The image is set up through a `CIDATA` seed partition after it
    /// (Ubuntu autoinstall with something to apply) rather than by editing
    /// its boot partition.
    pub fn needs_seed(&self) -> bool {
        self.init_format == AUTOINSTALL_UBUNTU && !self.cloudinit.is_empty()
    }

    /// The seed partition's files, by name.
    pub fn seed_files(&self, instance_id: &str) -> [(&'static str, Vec<u8>); 2] {
        [
            ("user-data", user_data_file(&self.cloudinit).into_bytes()),
            ("meta-data", meta_data_file(instance_id).into_bytes()),
        ]
    }
}
