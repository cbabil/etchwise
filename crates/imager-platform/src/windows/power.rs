//! Keeps Windows awake while writing (upstream
//! `windows_suspend_inhibitor.cpp`): `SetThreadExecutionState` for system
//! and display, restored when dropped. The finish sound is `MessageBeep`.

use windows_sys::Win32::System::Diagnostics::Debug::MessageBeep;
use windows_sys::Win32::System::Power::{
    ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED, EXECUTION_STATE,
    SetThreadExecutionState,
};

/// `MB_OK`: the default system sound.
const MB_OK: u32 = 0;

/// Holds the execution state for this thread until dropped.
pub struct SleepInhibitor {
    previous: EXECUTION_STATE,
}

impl SleepInhibitor {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        // SAFETY: plain flags.
        let previous = unsafe {
            SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED)
        };
        Self { previous }
    }
}

impl Drop for SleepInhibitor {
    fn drop(&mut self) {
        // SAFETY: restores the state returned above (0 when that failed,
        // which clears ours).
        unsafe { SetThreadExecutionState(self.previous | ES_CONTINUOUS) };
    }
}

/// The "Play sound when finished" option.
pub fn beep() {
    // SAFETY: plain flag.
    unsafe { MessageBeep(MB_OK) };
}
