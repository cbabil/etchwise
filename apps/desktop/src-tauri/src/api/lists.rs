//! Device and OS lists (`getHWList`, `getOSList`, repository calls).

use imager_core::IMAGER_VERSION;
use imager_core::format::is_version_newer;
use imager_core::oslist::hw::{parse_devices, parse_sw_capabilities, tags_match_chip};
use imager_core::oslist::model::{OsEntry, build_model, sublist_entries};
use imager_core::oslist::parse::parse_os_json;
use imager_core::oslist::store::HwFilter;
use serde_json::{Value, json};

use super::args::{Args, Handled, ok};
use crate::state::State;

pub fn handle(state: &mut State, method: &str, args: &Args<'_>) -> Handled {
    match method {
        "getHWList" => ok(hw_list(state)),
        "setHWIndex" => ok(select_device(state, args.i64(0))),
        "getOSList" => ok(os_list(state)),
        "getSublist" => ok(serde_json::to_value(sublist_entries(
            &serde_json::from_value::<OsEntry>(args.value(0)).unwrap_or_default(),
        ))
        .unwrap_or_default()),
        "getFilteredOSlist" => ok(state.store.filtered_document(&state.hw_filter)),
        "isOsListUnavailable" => ok(!state.store.is_available()),
        "imagerMetadata" => ok(state.store.imager_metadata()),
        "constantVersion" => ok(IMAGER_VERSION),
        "isVersionNewer" => ok(is_version_newer(&args.str(0), IMAGER_VERSION)),
        "osListUrl" => ok(state.repo_display.clone()),
        "customRepo" => ok(state.custom_repo()),
        "customRepoHost" => ok(host_of(&state.repo_display)),
        "isValidRepoUrl" => ok(is_valid_repo_url(&args.str(0))),
        "setSWCapabilitiesList" => {
            state.caps.software = match args.value(0) {
                Value::Array(items) => items
                    .iter()
                    .map(|v| v.as_str().unwrap_or_default().trim().to_lowercase())
                    .collect(),
                other => parse_sw_capabilities(other.as_str().unwrap_or_default()),
            };
            ok(Value::Null)
        }
        "checkSWCapability" => ok(imager_core::oslist::hw::has_capability(
            &state.caps.software,
            &args.str(0),
        )),
        "checkHWCapability" => ok(imager_core::oslist::hw::has_capability(
            &state.caps.hardware,
            &args.str(0),
        )),
        "checkHWAndSWCapability" => {
            let sw = args.str(1);
            ok(state
                .caps
                .hw_and_sw(&args.str(0), (!sw.is_empty()).then_some(sw.as_str())))
        }
        _ => None,
    }
}

/// `{devices, currentIndex}`; selects the default device on first load.
fn hw_list(state: &mut State) -> Value {
    let doc = state.store.filtered_document(&HwFilter::default());
    if let Some((devices, default)) = parse_devices(&doc) {
        let fresh = state.hw_devices.is_empty();
        state.hw_devices = devices;
        if fresh && let Some(index) = default {
            select_device(state, index as i64);
        }
    }
    let devices: Vec<Value> = state
        .hw_devices
        .iter()
        .map(|d| {
            let mut row = json!(d);
            row["isUsbBootConnected"] = json!(
                state
                    .rpiboot_chips
                    .iter()
                    .any(|chip| tags_match_chip(&d.tags, chip))
            );
            row
        })
        .collect();
    json!({"devices": devices, "currentIndex": state.hw_index.map_or(-1, |i| i as i64)})
}

/// `HWListModel::setCurrentIndex`: sets the tag filter and hardware
/// capabilities; a different device clears the OS selection.
fn select_device(state: &mut State, index: i64) -> Value {
    let Some(device) = usize::try_from(index)
        .ok()
        .and_then(|i| state.hw_devices.get(i))
        .cloned()
    else {
        state.hw_index = None;
        return Value::Null;
    };
    let changed = state
        .hw_index
        .and_then(|i| state.hw_devices.get(i))
        .is_none_or(|d| d.name != device.name);
    state.hw_filter = HwFilter {
        tags: device.tags.clone(),
        inclusive: device.is_inclusive(),
    };
    state.caps.hardware = device.capabilities.clone();
    if changed {
        state.src = None;
    }
    state.hw_index = usize::try_from(index).ok();
    Value::Null
}

fn os_list(state: &State) -> Value {
    let doc = state.store.filtered_document(&state.hw_filter);
    let locale = sys_locale();
    let list = parse_os_json(&doc, &locale, &mut rand::rng());
    serde_json::to_value(build_model(list, state.architecture())).unwrap_or_default()
}

/// `QLocale::system().name()` style (`en_GB`).
fn sys_locale() -> String {
    std::env::var("LANG")
        .ok()
        .and_then(|l| l.split('.').next().map(str::to_owned))
        .filter(|l| !l.is_empty())
        .unwrap_or_else(|| "en_GB".into())
}

fn host_of(url: &str) -> String {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_owned))
        .unwrap_or_default()
}

/// Upstream regex `^https?://\S+\.(json|rpi-imager-manifest)$`.
pub fn is_valid_repo_url(url: &str) -> bool {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"));
    rest.is_some_and(|r| {
        !r.chars().any(char::is_whitespace)
            && (r.ends_with(".json") || r.ends_with(".rpi-imager-manifest"))
            && r.len() > ".json".len()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_devices_by_plain_index() {
        let mut state = State::new(
            imager_core::settings::Settings::in_memory(),
            std::env::temp_dir(),
        );
        state.hw_devices = vec![
            imager_core::oslist::hw::HwDevice {
                name: "Pi 5".into(),
                tags: vec!["pi5-64bit".into()],
                ..Default::default()
            },
            imager_core::oslist::hw::HwDevice {
                name: "Pi 4".into(),
                tags: vec!["pi4-64bit".into()],
                ..Default::default()
            },
        ];
        handle(&mut state, "setHWIndex", &Args(&[0.into()]));
        assert_eq!(
            (state.hw_index, state.hw_filter.tags.clone()),
            (Some(0), vec!["pi5-64bit".to_owned()])
        );
        handle(&mut state, "setHWIndex", &Args(&[(-1).into()]));
        assert_eq!(state.hw_index, None);
    }

    #[test]
    fn validates_repository_urls() {
        assert!(is_valid_repo_url("https://example.com/list.json"));
        assert!(is_valid_repo_url("http://h/x.rpi-imager-manifest"));
        assert!(!is_valid_repo_url("ftp://h/x.json"));
        assert!(!is_valid_repo_url("https://h/a b.json"));
        assert!(!is_valid_repo_url("https://.json"));
        assert_eq!(
            host_of("https://downloads.raspberrypi.com/x.json"),
            "downloads.raspberrypi.com"
        );
    }
}
