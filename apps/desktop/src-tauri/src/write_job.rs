//! `startWrite` (upstream `ImageWriter::startWrite` + `DownloadThread`):
//! picks the source (verified cache, local file or download), unmounts and
//! opens the disk, runs the write job, then saves the cache record, ejects,
//! beeps and sends telemetry.

use std::sync::Arc;

use imager_core::pipeline::cache::{has_room, is_cached, save_record};
use imager_core::pipeline::job::{Job, Source};
use imager_core::pipeline::{Progress, WriteOptions};
use imager_core::telemetry;
use serde_json::{Value, json};
use tauri::AppHandle;

use self::device_ops::{after_success, run, run_erase};
use crate::background::{Shared, emit, lock};
use crate::state::State;

pub(crate) mod device_ops;
mod stats;
mod usb_boot;

/// Starts the write on its own thread; progress is polled by the UI.
pub fn start_write(shared: &Shared, app: &AppHandle) -> Result<(), String> {
    let mut state = lock(shared);
    if state.writing {
        return Err("A write is already running".into());
    }
    let (Some(src), Some((device, _))) = (state.src.clone(), state.dst.clone()) else {
        return Err("Select an operating system and a storage device first".into());
    };
    let erase = src.url == "internal://format";
    let usb_boot = state.rpiboot_device.clone();
    let (bmap_url, cache_dir) = (src.bmap_url.clone(), state.cache_dir.clone());
    let job = build_job(&state, &src.url, &src.expected_hash, src.extract_len);
    if stats::should_report(
        &state,
        matches!(job.source, Some(Source::Remote(_))),
        &src.url,
    ) {
        telemetry::send_in_background(telemetry::body(&report(&src)));
    }
    state.progress = Arc::new(Progress::default());
    state.writing = true;
    let (shared, app, progress) = (Arc::clone(shared), app.clone(), Arc::clone(&state.progress));
    drop(state);
    crate::monitor::spawn(app.clone(), Arc::clone(&progress), device.clone());
    std::thread::spawn(move || {
        let outcome = match (&usb_boot, erase) {
            (Some(board), _) => usb_boot::run_usb_boot(
                board,
                &job,
                &bmap_url,
                job.extract_size,
                &cache_dir,
                &progress,
            ),
            (None, true) => run_erase(&device, &progress),
            (None, false) => run(&device, &job, &progress),
        };
        finish(&shared, &app, outcome, &device);
    });
    Ok(())
}

fn build_job(state: &State, url: &str, sha: &str, extract_len: u64) -> Job {
    let expected = (!sha.is_empty()).then(|| sha.to_owned());
    let cached = is_cached(state.cache_record.as_ref(), state.cache_verified, sha);
    let source = match (url.strip_prefix("file://"), cached, &state.cache_record) {
        (Some(path), _, _) => Source::Local(path.into()),
        (None, true, Some(record)) => Source::Local(record.file.clone()),
        _ => Source::Remote(url.to_owned()),
    };
    let remote = matches!(source, Source::Remote(_));
    let _ = std::fs::create_dir_all(&state.cache_dir);
    let room = available_space(&state.cache_dir)
        .is_some_and(|free| has_room(free, state.src.as_ref().map_or(0, |s| s.download_len)));
    let caching = remote && imager_core::pipeline::cache::caching_enabled(&state.settings) && room;
    Job {
        source: Some(source),
        extract_size: extract_len,
        write: WriteOptions {
            expected_sha256: expected,
            skip_end_of_device: state.debug.skip_end_of_device,
            ..WriteOptions::default()
        },
        verify: state.verify_enabled,
        plan: state.plan.clone(),
        cache_dir: caching.then(|| state.cache_dir.clone()),
        multiple_files: state.src.as_ref().is_some_and(|s| s.multiple_files),
    }
}

fn report(src: &crate::state::Source) -> telemetry::Report<'_> {
    telemetry::Report {
        url: &src.url,
        os: &src.parent_category,
        image: &src.os_name,
        os_type: "macos",
        os_version: "",
        arch: std::env::consts::ARCH,
        locale: "",
    }
}

fn finish(
    shared: &Shared,
    app: &AppHandle,
    outcome: Result<Option<imager_core::pipeline::cache::CacheRecord>, String>,
    device: &str,
) {
    let mut state = lock(shared);
    state.writing = false;
    match outcome {
        Ok(record) => {
            if let Some(record) = record {
                let _ = save_record(&mut state.settings, &record);
                state.cache_record = Some(record);
                state.cache_verified = true;
            }
            let (eject, beep) = (state.settings.bool("eject"), state.settings.bool("beep"));
            drop(state);
            after_success(device, eject, beep);
            emit(app, "success", Value::Null);
        }
        Err(message) if message == "Cancelled" => {
            drop(state);
            emit(app, "cancelled", Value::Null);
        }
        Err(message) => {
            drop(state);
            emit(app, "error", json!(message));
        }
    }
}

/// Background re-hash of the cached download at startup.
pub fn verify_cache_in_background(shared: &Shared) {
    let shared = Arc::clone(shared);
    std::thread::spawn(move || {
        let record = lock(&shared).cache_record.clone();
        let valid = record
            .as_ref()
            .is_some_and(|r| imager_core::pipeline::cache::verify_file(r).unwrap_or(false));
        lock(&shared).cache_verified = valid;
    });
}

/// Free bytes on the volume holding `path`.
fn available_space(path: &std::path::Path) -> Option<u64> {
    let dir = std::ffi::CString::new(path.to_string_lossy().as_bytes()).ok()?;
    // SAFETY: statvfs is plain data; zeroed is a valid value.
    let mut stats: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: valid C path and out-parameter.
    (unsafe { libc::statvfs(dir.as_ptr(), &mut stats) } == 0)
        .then(|| stats.f_bavail as u64 * stats.f_frsize as u64)
}
