//! Whether a download is reported to the official repository's statistics
//! (upstream `DownloadStatsTelemetry`): only remote images, only with the
//! official repository as the main one, only with telemetry on, and never
//! for images merged in from an extra repository.

use imager_core::oslist::fetch::DEFAULT_REPO;
use imager_core::telemetry;

use crate::state::State;

pub(crate) fn should_report(state: &State, remote: bool, url: &str) -> bool {
    !state.store.is_from_extra(url)
        && telemetry::should_send(
            state.settings.bool("telemetry"),
            remote,
            state.repo == DEFAULT_REPO,
        )
}

#[cfg(test)]
mod tests {
    use imager_core::oslist::store::OsListStore;
    use imager_core::settings::Settings;
    use serde_json::json;

    use super::*;

    const PI_IMAGE: &str = "https://downloads.raspberrypi.com/os.img.xz";
    const NUC_IMAGE: &str = "https://releases.ubuntu.com/s.iso";

    fn store(doc: &serde_json::Value) -> OsListStore {
        let mut store = OsListStore::default();
        store.on_fetch_complete(doc.to_string().as_bytes(), "top", true);
        store
    }

    fn merged_state() -> State {
        let mut state = State::new(Settings::in_memory(), std::env::temp_dir());
        state.store =
            store(&json!({"imager": {}, "os_list": [{"name": "Pi OS", "url": PI_IMAGE}]}));
        let extra =
            store(&json!({"imager": {}, "os_list": [{"name": "Ubuntu", "url": NUC_IMAGE}]}));
        state.store.merge_extra(&extra);
        state
    }

    #[test]
    fn reports_official_images_only() {
        let state = merged_state();
        assert!(should_report(&state, true, PI_IMAGE));
        assert!(!should_report(&state, true, NUC_IMAGE));
        assert!(!should_report(&state, false, PI_IMAGE));
    }

    #[test]
    fn respects_the_setting_and_custom_repositories() {
        let mut state = merged_state();
        state.settings.set("telemetry", json!(false)).unwrap();
        assert!(!should_report(&state, true, PI_IMAGE));
        let mut state = merged_state();
        state.repo = "file:///r/custom.json".into();
        assert!(!should_report(&state, true, PI_IMAGE));
    }
}
