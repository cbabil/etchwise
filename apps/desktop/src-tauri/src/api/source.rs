//! The selected image (`setSrc`, custom files, cache lookups).

use std::path::Path;

use imager_core::pipeline::cache::is_cached;
use imager_core::pipeline::source_size::local_source_size;
use serde_json::Value;

use super::args::{Args, Handled, done, ok};
use crate::state::{Source, State};
use imager_core::customise::plan::AUTOINSTALL_UBUNTU;

pub fn handle(state: &mut State, method: &str, args: &Args<'_>) -> Handled {
    match method {
        "setSrc" => {
            let source: Source = serde_json::from_value(args.value(0)).unwrap_or_default();
            state.src = (!source.url.is_empty()).then(|| complete(source));
            ok(Value::Null)
        }
        "isCached" => ok(is_cached(
            state.cache_record.as_ref(),
            state.cache_verified,
            &args.str(0),
        )),
        "srcFileName" => ok(state
            .src
            .as_ref()
            .map(|s| s.url.rsplit('/').next().unwrap_or_default().to_owned())
            .unwrap_or_default()),
        "getSelectedSourceSize" => ok(state.src.as_ref().map_or(0, |s| {
            if s.extract_len > 0 {
                s.extract_len
            } else {
                s.download_len
            }
        })),
        "isExtractSizeKnown" => ok(state.src.as_ref().is_none_or(|s| s.extract_size_known)),
        "imageSupportsCustomization" => ok(state
            .src
            .as_ref()
            .is_some_and(|s| !s.init_format.is_empty())),
        "imageSupportsCcRpi" => ok(state
            .src
            .as_ref()
            .is_some_and(|s| s.init_format == "cloudinit-rpi")),
        // Etchwise: Ubuntu Server images set themselves up through autoinstall.
        "imageUsesAutoinstall" => ok(state
            .src
            .as_ref()
            .is_some_and(|s| s.init_format == AUTOINSTALL_UBUNTU)),
        "localSourceInfo" => ok(serde_json::json!(local_info(&args.str(0)))),
        "readFileContents" => done(std::fs::read_to_string(args.str(0))),
        _ => None,
    }
}

/// `ImageWriter::setSrc`: `none` means no customisation; local files get
/// their sizes from the file itself.
fn complete(mut source: Source) -> Source {
    if source.init_format == "none" {
        source.init_format.clear();
    }
    let path = source.url.strip_prefix("file://").map(str::to_owned);
    source.extract_size_known =
        !(source.extract_len == 0 && source.url.to_lowercase().ends_with(".gz"));
    if let Some(path) = path {
        let size = local_source_size(Path::new(&path));
        if source.download_len == 0 {
            source.download_len = size.download;
        }
        if source.extract_len == 0 {
            source.extract_len = size.extract;
            source.multiple_files = size.multiple_files;
            source.extract_size_known = size.extract_known;
        }
    }
    source
}

/// Sizes for a custom image picked in the file dialog.
fn local_info(path: &str) -> serde_json::Map<String, Value> {
    let size = local_source_size(Path::new(path));
    let mut map = serde_json::Map::new();
    map.insert("downloadLen".into(), size.download.into());
    map.insert("extractLen".into(), size.extract.into());
    map.insert("multipleFiles".into(), size.multiple_files.into());
    map
}
