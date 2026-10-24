//! Everything upstream's `ImageWriter` object holds between QML calls.

use std::path::PathBuf;
use std::sync::Arc;

use imager_core::customise::plan::CustomisationPlan;
use imager_core::oslist::fetch::DEFAULT_REPO;
use imager_core::oslist::hw::{Capabilities, HwDevice};
use imager_core::oslist::store::{HwFilter, OsListStore};
use imager_core::pipeline::Progress;
use imager_core::pipeline::cache::CacheRecord;
use imager_core::settings::Settings;

/// The OS chosen in the OS step (`ImageWriter::setSrc`).
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Source {
    pub url: String,
    pub download_len: u64,
    pub extract_len: u64,
    pub expected_hash: String,
    pub multiple_files: bool,
    pub parent_category: String,
    pub os_name: String,
    pub init_format: String,
    pub release_date: String,
    pub bmap_url: String,
    pub extract_size_known: bool,
}

/// Hidden debug options (Cmd+Option+S).
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DebugOptions {
    pub direct_io: bool,
    pub periodic_sync: bool,
    pub verbose_logging: bool,
    pub async_io: bool,
    pub async_queue_depth: u32,
    pub ipv4_only: bool,
    pub skip_end_of_device: bool,
    pub rpiboot: bool,
}

/// Backend state behind one mutex.
pub struct State {
    pub settings: Settings,
    pub cache_dir: PathBuf,
    pub store: OsListStore,
    pub repo: String,
    pub repo_display: String,
    pub hw_devices: Vec<HwDevice>,
    pub hw_index: Option<usize>,
    pub hw_filter: HwFilter,
    pub caps: Capabilities,
    pub src: Option<Source>,
    pub dst: Option<(String, u64)>,
    /// `rpiboot://…` id when the target is a board in USB boot mode.
    pub rpiboot_device: Option<String>,
    /// Chips of the boards in USB boot mode seen by the last drive poll
    /// (upstream `DriveListModel::connectedRpibootChipsChanged`).
    pub rpiboot_chips: Vec<&'static str>,
    /// Why the extra repository could not be merged; empty when it was.
    pub extra_repo_error: String,
    pub plan: Option<CustomisationPlan>,
    pub progress: Arc<Progress>,
    pub writing: bool,
    pub connect_token: String,
    pub language: String,
    pub debug: DebugOptions,
    pub cache_record: Option<CacheRecord>,
    pub cache_verified: bool,
    pub verify_enabled: bool,
}

impl State {
    pub fn new(settings: Settings, cache_dir: PathBuf) -> Self {
        let cache_record = imager_core::pipeline::cache::load_record(&settings);
        Self {
            settings,
            cache_dir,
            store: OsListStore::default(),
            repo: DEFAULT_REPO.to_owned(),
            repo_display: DEFAULT_REPO.to_owned(),
            hw_devices: Vec::new(),
            hw_index: None,
            hw_filter: HwFilter::default(),
            caps: Capabilities::default(),
            src: None,
            dst: None,
            rpiboot_device: None,
            rpiboot_chips: Vec::new(),
            extra_repo_error: String::new(),
            plan: None,
            progress: Arc::new(Progress::default()),
            writing: false,
            connect_token: String::new(),
            language: String::new(),
            debug: DebugOptions {
                async_queue_depth: 4,
                ..DebugOptions::default()
            },
            cache_record,
            cache_verified: false,
            verify_enabled: true,
        }
    }

    /// The selected device's architecture, for OS ordering.
    pub fn architecture(&self) -> &str {
        self.hw_index
            .and_then(|i| self.hw_devices.get(i))
            .map_or("", |d| d.architecture.as_str())
    }

    pub fn custom_repo(&self) -> bool {
        self.repo != DEFAULT_REPO
    }
}
