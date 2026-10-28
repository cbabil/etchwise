//! Keeps the computer awake while writing (upstream
//! `linux_suspend_inhibitor.cpp`): a logind `sleep:idle` block inhibitor,
//! held as a descriptor until dropped. The finish sound plays the
//! freedesktop "complete" event through `canberra-gtk-play`.

use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::OwnedFd;

/// Holds the inhibitor lock until dropped.
pub struct SleepInhibitor {
    _lock: Option<OwnedFd>,
}

impl SleepInhibitor {
    /// Takes the lock; without logind the write simply goes on.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            _lock: inhibit().ok(),
        }
    }
}

fn inhibit() -> zbus::Result<OwnedFd> {
    let bus = Connection::system()?;
    Proxy::new(
        &bus,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )?
    .call(
        "Inhibit",
        &(
            "sleep:idle",
            imager_core::APP_NAME,
            "Writing an image",
            "block",
        ),
    )
}

/// The "Play sound when finished" option; silent where libcanberra is
/// missing.
pub fn beep() {
    let _ = std::process::Command::new("canberra-gtk-play")
        .arg("--id=complete")
        .spawn();
}
