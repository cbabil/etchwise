//! `repos/etchwise.json` loads as a repository: one Intel NUC device whose
//! filter shows the x86-64 images (Ubuntu Server with autoinstall setup), and
//! a Kura folder for the Raspberry Pi 5 whose sublist comes from Kura's
//! release (here a local copy, so the test needs no network).

use std::path::{Path, PathBuf};

use serde_json::Value;

use super::dispatch;
use crate::state::State;
use imager_core::oslist::fetch::load_repository;
use imager_core::settings::Settings;

/// Kura's published sublist URL, as in the repository file.
const KURA_LIST: &str = "https://github.com/cbabil/kura/releases/latest/download/etchwise.json";

/// What Kura's `scripts/release/make-etchwise-list.sh` publishes.
fn kura_fragment() -> Value {
    serde_json::json!({"os_list": [{
        "name": "Kura 0.2.0",
        "description": "Lightweight home-server OS",
        "icon": "data:image/svg+xml;base64,PHN2Zz48L3N2Zz4=",
        "url": "https://github.com/cbabil/kura/releases/download/v0.2.0/kura-0.2.0-rpi5-arm64.img.zst",
        "extract_size": 2_147_483_648_u64,
        "extract_sha256": "b".repeat(64),
        "image_download_size": 400_000_000,
        "image_download_sha256": "a".repeat(64),
        "release_date": "2026-10-25",
        "init_format": "kura",
        "devices": ["pi5-64bit"],
        "capabilities": []
    }]})
}

fn repo_file() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../repos/etchwise.json")
        .canonicalize()
        .expect("repos/etchwise.json exists")
}

/// The repository with Kura's sublist URL pointed at a local fragment.
fn loaded_state() -> State {
    let dir = tempfile::tempdir().expect("temp dir").keep();
    let fragment = dir.join("kura.json");
    std::fs::write(&fragment, kura_fragment().to_string()).expect("fragment");
    let text = std::fs::read_to_string(repo_file()).expect("repo");
    assert!(
        text.contains(KURA_LIST),
        "the Kura folder points at Kura's release"
    );
    let local = text.replace(KURA_LIST, &format!("file://{}", fragment.display()));
    let repo = dir.join("etchwise.json");
    std::fs::write(&repo, local).expect("repo copy");
    let mut state = State::new(Settings::in_memory(), std::env::temp_dir());
    load_repository(&mut state.store, &format!("file://{}", repo.display()))
        .expect("repository loads");
    state
}

fn call(state: &mut State, method: &str, args: &[Value]) -> Value {
    dispatch(state, method, args).expect(method)
}

#[test]
fn lists_the_nuc_as_the_default_device() {
    let mut state = loaded_state();
    let list = call(&mut state, "getHWList", &[]);
    let names: Vec<&str> = list["devices"]
        .as_array()
        .expect("devices")
        .iter()
        .filter_map(|d| d["name"].as_str())
        .collect();
    assert_eq!(names, ["Intel NUC"]);
    assert_eq!(list["currentIndex"], 0);
    let icon = list["devices"][0]["icon"].as_str().unwrap_or_default();
    assert!(
        icon.starts_with("data:image/svg+xml;base64,"),
        "device icon: {icon:.40}"
    );
}

#[test]
fn shows_the_x86_images_with_setup_for_ubuntu_server_only() {
    let mut state = loaded_state();
    call(&mut state, "getHWList", &[]);
    call(&mut state, "setHWIndex", &[0.into()]);
    let os_list = call(&mut state, "getOSList", &[]);
    let images: Vec<Value> = os_list
        .as_array()
        .expect("os list")
        .iter()
        .filter(|o| o["url"].as_str().is_some_and(|u| u.starts_with("https://")))
        .cloned()
        .collect();
    let names: Vec<&str> = images.iter().filter_map(|o| o["name"].as_str()).collect();
    assert_eq!(
        names,
        [
            "Ubuntu Server 26.04.1 LTS",
            "Ubuntu Desktop 26.04.1 LTS",
            "Debian 13.7 (netinst)",
            "Proxmox VE 9.2",
        ]
    );
    for image in &images {
        let name = &image["name"];
        // The payload the UI sends (selectImage.ts setSource).
        let source = serde_json::json!({
            "url": image["url"],
            "initFormat": image["init_format"].as_str().unwrap_or_default(),
            "releaseDate": image["release_date"],
        });
        call(&mut state, "setSrc", &[source]);
        // Only Ubuntu Server has first-boot setup (autoinstall) so far.
        let setup = name
            .as_str()
            .is_some_and(|n| n.starts_with("Ubuntu Server"));
        assert_eq!(
            call(&mut state, "imageSupportsCustomization", &[]),
            setup,
            "{name} customisable"
        );
        assert_eq!(
            call(&mut state, "imageUsesAutoinstall", &[]),
            setup,
            "{name} autoinstall"
        );
        let icon = image["icon"].as_str().unwrap_or_default();
        assert!(icon.starts_with("https://"), "{name} icon: {icon}");
        let sha = image["extract_sha256"].as_str().unwrap_or_default();
        assert!(
            sha.len() == 64 && sha.chars().all(|c| c.is_ascii_hexdigit()),
            "{name}"
        );
        assert_eq!(
            image["extract_size"], image["image_download_size"],
            "{name}"
        );
        assert_eq!(
            image["extract_size"].as_u64().unwrap_or(0) % 512,
            0,
            "{name}"
        );
    }
}

/// The Kura folder as a Raspberry Pi 5 sees it (the Pi 5 device comes from
/// the official list this repository is merged onto).
fn kura_for_pi5(state: &State) -> Value {
    let filter = imager_core::oslist::store::HwFilter {
        tags: vec!["pi5-64bit".into()],
        inclusive: false,
    };
    let doc = state.store.filtered_document(&filter);
    doc["os_list"]
        .as_array()
        .expect("os list")
        .iter()
        .find(|o| o["name"] == "Kura")
        .cloned()
        .expect("Kura folder for the Pi 5")
}

#[test]
fn lists_kura_for_the_pi5_only() {
    let mut state = loaded_state();
    let kura = kura_for_pi5(&state);
    assert_eq!(kura["subitems"][0]["name"], "Kura 0.2.0");
    assert!(
        kura["icon"]
            .as_str()
            .unwrap_or_default()
            .starts_with("data:image/svg+xml;base64,")
    );
    call(&mut state, "getHWList", &[]);
    call(&mut state, "setHWIndex", &[0.into()]);
    let nuc = call(&mut state, "getOSList", &[]);
    let names: Vec<&str> = nuc
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|o| o["name"].as_str())
        .collect();
    assert!(!names.iter().any(|n| n.starts_with("Kura")), "{names:?}");
}

#[test]
fn kura_offers_first_boot_setup_without_autoinstall() {
    let mut state = loaded_state();
    let image = kura_for_pi5(&state)["subitems"][0].clone();
    let source = serde_json::json!({
        "url": image["url"],
        "initFormat": image["init_format"],
        "releaseDate": image["release_date"],
    });
    call(&mut state, "setSrc", &[source]);
    assert_eq!(call(&mut state, "imageSupportsCustomization", &[]), true);
    assert_eq!(call(&mut state, "imageUsesAutoinstall", &[]), false);
}
