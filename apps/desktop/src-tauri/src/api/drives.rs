//! Storage devices (`getDriveList`, `setDst`). The UI polls the list every
//! second, as upstream's poll thread does.

use imager_core::drives::{DriveInfo, visible_drives};
use serde_json::{Value, json};

use super::args::{Args, Handled, ok};
use crate::state::State;

pub fn handle(state: &mut State, method: &str, args: &Args<'_>) -> Handled {
    match method {
        "getDriveList" => ok(with_rpiboot(drive_list(), state)),
        "setRpibootDevice" => {
            state.dst = Some((args.str(0), 0));
            state.rpiboot_device = Some(args.str(0));
            ok(Value::Null)
        }
        "isRpibootDevice" => ok(state.rpiboot_device.is_some()),
        "setDst" => {
            state.rpiboot_device = None;
            let device = args.str(0);
            if device.is_empty() && !state.writing {
                state.progress = std::sync::Arc::default();
            }
            state.dst = (!device.is_empty()).then(|| (device, args.u64(1)));
            ok(Value::Null)
        }
        _ => None,
    }
}

/// Boards in USB boot mode join the list only when the debug option
/// "Rpiboot/Fastboot support" is on (upstream default: off); their chips
/// are kept for the device list's "connected" marks.
fn with_rpiboot(mut list: Value, state: &mut State) -> Value {
    if !state.debug.rpiboot {
        state.rpiboot_chips.clear();
        return list;
    }
    let scanned = imager_rpiboot::rpiboot::scan_rpiboot_devices();
    state.rpiboot_chips = scanned.iter().map(|d| d.chip_generation.name()).collect();
    let boards: Vec<Value> = scanned
        .iter()
        .map(|d| {
            json!(DriveInfo {
                key: d.unique_key(),
                device: d.device_id(),
                description: d.description().to_owned(),
                size: 0,
                is_usb: true,
                is_scsi: false,
                is_read_only: false,
                is_system: false,
                mountpoints: Vec::new(),
                child_devices: Vec::new(),
                is_rpiboot: true,
            })
        })
        .collect();
    if let Some(drives) = list.get_mut("drives").and_then(Value::as_array_mut) {
        drives.extend(boards);
    }
    list
}

#[cfg(target_os = "macos")]
fn drive_list() -> Value {
    match imager_platform::macos::drivelist::list_drives() {
        Ok(devices) => json!({"drives": visible_drives(&devices), "error": ""}),
        Err(e) => json!({"drives": [], "error": e.to_string()}),
    }
}

#[cfg(not(target_os = "macos"))]
fn drive_list() -> Value {
    json!({"drives": [], "error": "Drive listing is only implemented on macOS"})
}
