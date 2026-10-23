//! Keeps the Mac awake while writing (upstream `MacSuspendInhibitor`):
//! IOPM assertions against idle system sleep and display sleep.

use core_foundation::base::TCFType;
use core_foundation::string::CFString;

use super::ffi::*;

/// Holds the assertions until dropped.
pub struct SleepInhibitor {
    ids: Vec<IOPMAssertionID>,
}

impl SleepInhibitor {
    /// Takes both assertions named "Raspberry Pi Imager".
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let name = CFString::from_static_string("Raspberry Pi Imager");
        let ids = ["PreventUserIdleSystemSleep", "PreventUserIdleDisplaySleep"]
            .iter()
            .filter_map(|kind| {
                let kind = CFString::from_static_string(kind);
                let mut id = 0;
                // SAFETY: valid CFStrings and an out-parameter.
                let ok = unsafe {
                    IOPMAssertionCreateWithName(
                        kind.as_concrete_TypeRef(),
                        kIOPMAssertionLevelOn,
                        name.as_concrete_TypeRef(),
                        &mut id,
                    )
                } == 0;
                ok.then_some(id)
            })
            .collect();
        Self { ids }
    }

    /// Number of assertions held.
    #[cfg(test)]
    pub fn held(&self) -> usize {
        self.ids.len()
    }
}

impl Drop for SleepInhibitor {
    fn drop(&mut self) {
        for id in &self.ids {
            // SAFETY: we own these assertion ids.
            unsafe { IOPMAssertionRelease(*id) };
        }
    }
}

/// `NSBeep`, the "Play sound when finished" option.
pub fn beep() {
    // SAFETY: AppKit function without arguments.
    unsafe { NSBeep() };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_and_releases_assertions() {
        let inhibitor = SleepInhibitor::new();
        assert_eq!(inhibitor.held(), 2);
    }
}
