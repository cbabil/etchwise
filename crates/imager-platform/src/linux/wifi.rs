//! Wi-Fi prefill from NetworkManager (upstream `networkmanagerapi.cpp`):
//! the SSID of the active Wi-Fi connection and, when NetworkManager lets
//! this user read it, its saved key.

use std::collections::HashMap;

use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

const NM: &str = "org.freedesktop.NetworkManager";
const WIRELESS: &str = "802-11-wireless";
const SECURITY: &str = "802-11-wireless-security";

type Settings = HashMap<String, HashMap<String, OwnedValue>>;

/// SSID of the Wi-Fi network this computer is on, if any.
pub fn current_ssid() -> Option<String> {
    active_wifi(&Connection::system().ok()?).map(|(ssid, _)| ssid)
}

/// The saved key of `ssid` when it is the active network.
pub fn saved_password(ssid: &str) -> Option<String> {
    let bus = Connection::system().ok()?;
    let (current, settings) = active_wifi(&bus)?;
    if current != ssid {
        return None;
    }
    let secrets: Settings = settings_proxy(&bus, settings)?
        .call("GetSecrets", &(SECURITY,))
        .ok()?;
    let psk = secrets.get(SECURITY)?.get("psk")?.try_clone().ok()?;
    String::try_from(psk).ok().filter(|p| !p.is_empty())
}

/// The active Wi-Fi connection's SSID and settings object.
fn active_wifi(bus: &Connection) -> Option<(String, OwnedObjectPath)> {
    let nm = Proxy::new(bus, NM, "/org/freedesktop/NetworkManager", NM).ok()?;
    let active: Vec<OwnedObjectPath> = nm.get_property("ActiveConnections").ok()?;
    active.into_iter().find_map(|path| {
        let connection = Proxy::new(
            bus,
            NM,
            path,
            "org.freedesktop.NetworkManager.Connection.Active",
        )
        .ok()?;
        if connection.get_property::<String>("Type").ok()? != WIRELESS {
            return None;
        }
        let settings: OwnedObjectPath = connection.get_property("Connection").ok()?;
        let all: Settings = settings_proxy(bus, settings.clone())?
            .call("GetSettings", &())
            .ok()?;
        let ssid = all.get(WIRELESS)?.get("ssid")?.try_clone().ok()?;
        let bytes = Vec::<u8>::try_from(ssid).ok()?;
        Some((super::ssid::ssid_text(&bytes)?, settings))
    })
}

fn settings_proxy(bus: &Connection, path: OwnedObjectPath) -> Option<Proxy<'_>> {
    Proxy::new(
        bus,
        NM,
        path,
        "org.freedesktop.NetworkManager.Settings.Connection",
    )
    .ok()
}
