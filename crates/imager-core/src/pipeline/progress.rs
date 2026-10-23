//! Progress shared between the write thread and the UI (upstream polls the
//! atomic counters every 100 ms), plus cancellation and verify skipping.

use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};

/// Upstream `WriteState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[repr(u8)]
pub enum WriteState {
    Idle,
    Preparing,
    Writing,
    Verifying,
    Finalizing,
    Succeeded,
    Failed,
    Cancelling,
    Cancelled,
}

impl WriteState {
    const ALL: [Self; 9] = [
        Self::Idle,
        Self::Preparing,
        Self::Writing,
        Self::Verifying,
        Self::Finalizing,
        Self::Succeeded,
        Self::Failed,
        Self::Cancelling,
        Self::Cancelled,
    ];
}

/// Counters and flags; all methods take `&self`.
#[derive(Debug, Default)]
pub struct Progress {
    pub download_now: AtomicU64,
    pub download_total: AtomicU64,
    pub bytes_written: AtomicU64,
    pub write_total: AtomicU64,
    pub verify_now: AtomicU64,
    pub verify_total: AtomicU64,
    state: AtomicU8,
    cancelled: AtomicBool,
    skip_verify: AtomicBool,
    status: Mutex<String>,
}

/// A point-in-time copy for the UI.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressSnapshot {
    pub state: WriteState,
    pub download_now: u64,
    pub download_total: u64,
    pub bytes_written: u64,
    pub write_total: u64,
    pub verify_now: u64,
    pub verify_total: u64,
    /// Upstream `preparationStatusUpdate` text, e.g. "Opening drive...".
    pub status: String,
}

impl Progress {
    pub fn state(&self) -> WriteState {
        WriteState::ALL
            [usize::from(self.state.load(Ordering::Relaxed)).min(WriteState::ALL.len() - 1)]
    }

    pub fn set_state(&self, state: WriteState) {
        self.state.store(state as u8, Ordering::Relaxed);
    }

    /// `cancelDownload`.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
        self.set_state(WriteState::Cancelling);
    }

    /// The cancellation flag, for APIs that poll an `AtomicBool`.
    pub fn cancel_flag(&self) -> &AtomicBool {
        &self.cancelled
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// `skipCurrentVerification`.
    pub fn skip_verification(&self) {
        self.skip_verify.store(true, Ordering::Relaxed);
    }

    pub fn verification_skipped(&self) -> bool {
        self.skip_verify.load(Ordering::Relaxed)
    }

    pub fn add(counter: &AtomicU64, n: u64) {
        counter.fetch_add(n, Ordering::Relaxed);
    }

    pub fn set(counter: &AtomicU64, n: u64) {
        counter.store(n, Ordering::Relaxed);
    }

    /// Sets the preparation status line (upstream's English text).
    pub fn set_status(&self, status: &str) {
        status.clone_into(&mut self.status.lock().unwrap_or_else(PoisonError::into_inner));
    }

    pub fn status(&self) -> String {
        self.status
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn snapshot(&self) -> ProgressSnapshot {
        let get = |c: &AtomicU64| c.load(Ordering::Relaxed);
        ProgressSnapshot {
            state: self.state(),
            download_now: get(&self.download_now),
            download_total: get(&self.download_total),
            bytes_written: get(&self.bytes_written),
            write_total: get(&self.write_total),
            verify_now: get(&self.verify_now),
            verify_total: get(&self.verify_total),
            status: self.status(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_state_flags_and_counters() {
        let p = Progress::default();
        assert_eq!(p.state(), WriteState::Idle);
        p.set_state(WriteState::Verifying);
        Progress::add(&p.bytes_written, 7);
        p.skip_verification();
        assert!(p.verification_skipped() && !p.is_cancelled());
        p.cancel();
        let s = p.snapshot();
        assert_eq!((s.state, s.bytes_written), (WriteState::Cancelling, 7));
        assert!(p.is_cancelled());
        p.set_status("Opening drive...");
        assert_eq!(p.snapshot().status, "Opening drive...");
        for state in WriteState::ALL {
            p.set_state(state);
            assert_eq!(p.state(), state);
        }
    }
}
