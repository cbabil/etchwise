//! Signals derived from a running write (upstream `ImageWriter` timers):
//! `preparationStatusUpdate` when the status line changes,
//! `bottleneckStatusChanged` with throughput twice a second, and
//! `writeCancelledDueToDeviceRemoval` when the target disappears.

use std::sync::Arc;
use std::time::{Duration, Instant};

use imager_core::pipeline::{Progress, WriteState};
use serde_json::json;
use tauri::AppHandle;

use crate::background::emit;

/// Watches `progress` until the write ends.
pub fn spawn(app: AppHandle, progress: Arc<Progress>, device: String) {
    std::thread::spawn(move || {
        let (mut status, mut last_bytes, mut last_tick) = (String::new(), 0u64, Instant::now());
        let mut last_presence_check = Instant::now();
        while !matches!(
            progress.state(),
            WriteState::Succeeded | WriteState::Failed | WriteState::Cancelled
        ) {
            let snapshot = progress.snapshot();
            if snapshot.status != status {
                emit(&app, "preparationStatusUpdate", json!(snapshot.status));
                status.clone_from(&snapshot.status);
            }
            if last_tick.elapsed() >= Duration::from_millis(500) {
                let verifying = snapshot.state == WriteState::Verifying;
                let bytes = if verifying {
                    snapshot.verify_now
                } else {
                    snapshot.bytes_written
                };
                let kbps = bytes.saturating_sub(last_bytes) * 1000
                    / (last_tick.elapsed().as_millis().max(1) as u64 * 1024);
                emit(
                    &app,
                    "bottleneckStatusChanged",
                    json!({"status": if verifying { "Verifying" } else { "None" }, "throughputKBps": kbps}),
                );
                (last_bytes, last_tick) = (bytes, Instant::now());
            }
            if last_presence_check.elapsed() >= Duration::from_secs(1) {
                last_presence_check = Instant::now();
                if device.starts_with("/dev/") && !device_present(&device) {
                    progress.cancel();
                    emit(&app, "writeCancelledDueToDeviceRemoval", json!(device));
                    return;
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    });
}

#[cfg(target_os = "macos")]
fn device_present(device: &str) -> bool {
    imager_platform::macos::drivelist::list_drives()
        .map_or(true, |drives| drives.iter().any(|d| d.device == device))
}

#[cfg(not(target_os = "macos"))]
fn device_present(_device: &str) -> bool {
    true
}
