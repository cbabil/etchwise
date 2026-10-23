//! `repos/intel-nuc.json` loads as a repository: one Intel NUC device whose
//! filter shows the x86-64 images, none of which offer customisation.

use std::path::Path;

use serde_json::Value;

use super::dispatch;
use crate::state::State;
use imager_core::oslist::fetch::load_repository;
use imager_core::settings::Settings;

fn loaded_state() -> State {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../repos/intel-nuc.json")
        .canonicalize()
        .expect("repos/intel-nuc.json exists");
    let mut state = State::new(Settings::in_memory(), std::env::temp_dir());
    load_repository(&mut state.store, &format!("file://{}", file.display()))
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
}

#[test]
fn shows_the_x86_images_without_customisation() {
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
        call(&mut state, "setSrc", std::slice::from_ref(image));
        assert_eq!(
            call(&mut state, "imageSupportsCustomization", &[]),
            false,
            "{name} customisable"
        );
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
