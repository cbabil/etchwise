//! An offline fetcher for the firmware and sideload tests.

use std::collections::HashMap;
use std::io;
use std::sync::Mutex;

use super::fetcher::Fetcher;
use super::manifest::{PROVISIONER_RAW_BASE, USBBOOT_RAW_BASE};
use super::test_support::tar_of;

/// Serves fixed bodies by URL and records requests.
#[derive(Default)]
pub struct FakeFetcher {
    pub bodies: HashMap<String, Vec<u8>>,
    pub requested: Mutex<Vec<String>>,
}

impl Fetcher for FakeFetcher {
    fn fetch(&self, url: &str) -> io::Result<Vec<u8>> {
        self.requested.lock().expect("lock").push(url.to_owned());
        self.bodies
            .get(url)
            .cloned()
            .ok_or_else(|| io::Error::other("404 Not Found"))
    }
}

/// A fetcher with the BCM2711/BCM2712 files (bootcode in the TAR).
pub fn fetcher_for_cm4_cm5() -> FakeFetcher {
    let tar = tar_of(&[
        ("2711/bootcode4.bin", b"bc4"),
        ("2712/bootcode5.bin", b"bc5"),
    ]);
    let mut f = FakeFetcher::default();
    let usb = |p: &str| format!("{USBBOOT_RAW_BASE}{p}");
    f.bodies.insert(
        format!("{PROVISIONER_RAW_BASE}host-support/fastboot-gadget.img"),
        b"img".to_vec(),
    );
    f.bodies
        .insert(usb("mass-storage-gadget64/config.txt"), b"cfg".to_vec());
    f.bodies.insert(usb("firmware/bootfiles.bin"), tar);
    f
}
