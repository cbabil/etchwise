//! Localisation lists, password hashing and Wi-Fi prefill.

use imager_core::crypto::crypt_password;
use imager_core::customise::pbkdf2_psk;
use imager_core::data;
use serde_json::{Value, json};

use super::args::{Args, Handled, ok};
use crate::state::State;

pub fn handle(state: &mut State, method: &str, args: &Args<'_>) -> Handled {
    match method {
        "getTimezoneList" => ok(json!(data::timezones())),
        "getCountryList" => ok(json!(data::countries())),
        "getKeymapLayoutList" => ok(json!(data::keymap_layouts())),
        "getCapitalCitiesList" => ok(json!(data::capital_cities())),
        "getLocaleDataForCapital" => {
            ok(data::locale_for_capital(&args.str(0)).map_or(json!({}), |l| json!(l)))
        }
        "getTimezone" => ok(system_timezone()),
        "crypt" => {
            let release = state
                .src
                .as_ref()
                .map(|s| s.release_date.clone())
                .unwrap_or_default();
            ok(crypt_password(&args.str(0), &release, &mut rand::rng()).unwrap_or_default())
        }
        "pbkdf2" => ok(pbkdf2_psk(args.str(0).as_bytes(), args.str(1).as_bytes())),
        "getCurrentLanguage" => ok(state.language.clone()),
        "changeLanguage" => {
            state.language = args.str(0);
            ok(Value::Null)
        }
        "getSSID" => ok(wifi_ssid()),
        "getPSKForSSID" => ok(wifi_password(&args.str(0))),
        _ => None,
    }
}

/// `QTimeZone::systemTimeZoneId()`.
fn system_timezone() -> String {
    std::fs::read_link("/etc/localtime")
        .ok()
        .and_then(|p| {
            p.to_string_lossy()
                .split("zoneinfo/")
                .nth(1)
                .map(str::to_owned)
        })
        .unwrap_or_default()
}

#[cfg(target_os = "macos")]
fn wifi_ssid() -> String {
    imager_platform::macos::wifi::current_ssid().unwrap_or_default()
}

#[cfg(target_os = "macos")]
fn wifi_password(ssid: &str) -> String {
    imager_platform::macos::wifi::saved_password(ssid).unwrap_or_default()
}

#[cfg(not(target_os = "macos"))]
fn wifi_ssid() -> String {
    String::new()
}

#[cfg(not(target_os = "macos"))]
fn wifi_password(_ssid: &str) -> String {
    String::new()
}
