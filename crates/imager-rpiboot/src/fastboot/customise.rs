//! OS customisation over the fastboot file protocol (upstream
//! `FastbootFlashThread::applyCustomisation`): the same edits as
//! `imager_core::customise::plan::apply_plan`, made to
//! `/boot/firmware/...` on the device with `oem upload-file` /
//! `oem download-file` instead of through a mounted FAT partition.
//! Upstream's fastboot path does no secure-boot signing; neither does this.

use imager_core::customise::plan::{CustomisationPlan, merge_config};

use super::flash::FlashError;
use super::protocol::{Cancel, read_device_file, write_device_file};
use crate::transport::UsbTransport;

/// Where the Pi gadget mounts the boot partition.
pub const BOOT: &str = "/boot/firmware/";

const FIRSTRUN_CMDLINE: &str = " systemd.run=/boot/firstrun.sh systemd.run_success_action=reboot systemd.unit=kernel-command-line.target";

fn has_customisation(plan: &CustomisationPlan) -> bool {
    !plan.config.is_empty()
        || !plan.cmdline.is_empty()
        || !plan.firstrun.is_empty()
        || !plan.cloudinit.is_empty()
        || !plan.network.is_empty()
}

/// Reads `/boot/firmware/<name>`; an empty file counts as a failure, as
/// upstream (whose empty result doubles as its error signal).
fn read<T: UsbTransport + ?Sized>(
    t: &mut T,
    name: &'static str,
    cancel: Cancel<'_>,
) -> Result<Vec<u8>, FlashError> {
    match read_device_file(t, &format!("{BOOT}{name}"), cancel) {
        Ok(data) if !data.is_empty() => Ok(data),
        Ok(_) => Err(FlashError::ReadFile(name, String::new())),
        Err(e) => Err(FlashError::ReadFile(name, e.0)),
    }
}

fn write<T: UsbTransport + ?Sized>(
    t: &mut T,
    name: &'static str,
    data: &[u8],
    cancel: Cancel<'_>,
) -> Result<(), FlashError> {
    write_device_file(t, &format!("{BOOT}{name}"), data, cancel)
        .map_err(|e| FlashError::WriteFile(name, e.0))
}

/// Applies `plan` to the device. `instance_id` is
/// `rpi-imager-<ms since epoch>` upstream.
///
/// # Errors
/// "Failed to read/write <file>: <reason>" for the first failing file.
pub fn apply_customisation<T: UsbTransport + ?Sized>(
    t: &mut T,
    plan: &CustomisationPlan,
    instance_id: &str,
    cancel: Cancel<'_>,
) -> Result<(), FlashError> {
    if !has_customisation(plan) || plan.init_format.is_empty() {
        return Ok(());
    }
    if !plan.config.is_empty() {
        let existing = read(t, "config.txt", cancel)?;
        write(
            t,
            "config.txt",
            &merge_config(&existing, &plan.config),
            cancel,
        )?;
    }
    let mut cmdline = plan.cmdline.clone();
    if !plan.firstrun.is_empty() && plan.init_format == "systemd" {
        write(t, "firstrun.sh", plan.firstrun.as_bytes(), cancel)?;
        cmdline.push_str(FIRSTRUN_CMDLINE);
    }
    let cloud = plan.init_format == "cloudinit" || plan.init_format == "cloudinit-rpi";
    if cloud && (!plan.cloudinit.is_empty() || !plan.network.is_empty()) {
        let meta = format!("instance-id: {instance_id}\n");
        write(t, "meta-data", meta.as_bytes(), cancel)?;
        cmdline.push_str(&format!(" ds=nocloud;i={instance_id}"));
        if !plan.cloudinit.is_empty() {
            let user_data = format!("#cloud-config\n{}", plan.cloudinit);
            write(t, "user-data", user_data.as_bytes(), cancel)?;
        }
        if !plan.network.is_empty() {
            write(t, "network-config", plan.network.as_bytes(), cancel)?;
        }
    }
    if !cmdline.is_empty() {
        let existing = read(t, "cmdline.txt", cancel)?;
        let mut updated = existing.trim_ascii().to_vec();
        updated.extend_from_slice(cmdline.as_bytes());
        write(t, "cmdline.txt", &updated, cancel)?;
    }
    Ok(())
}
