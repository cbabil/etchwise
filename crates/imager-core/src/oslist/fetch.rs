//! Loading a repository (upstream `beginOSListFetch` and the sublist
//! fetches): the top-level list over http(s) or `file://`, one IPv4-only
//! retry when it fails, then every `subitems_url`, level by level in
//! parallel, spliced into the store.

use std::io;
use std::net::{IpAddr, Ipv4Addr};

use reqwest::Url;
use reqwest::blocking::Client;

use super::store::OsListStore;
use crate::pipeline::http;

/// A document body and its final URL.
type Fetched = io::Result<(Vec<u8>, String)>;

/// Upstream `OSLIST_URL`.
pub const DEFAULT_REPO: &str = "https://downloads.raspberrypi.com/os_list_imagingutility_v4.json";

/// Outcome of loading the top-level list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded {
    /// The URL that served the list after redirects (shown as the host).
    pub final_url: String,
    /// Sublists that failed; their groups keep `subitems_url`.
    pub failed_sublists: Vec<String>,
}

/// Fetches one document from `http(s)://` or `file://`.
///
/// # Errors
/// Fails on network or file errors.
pub fn fetch_document(client: &Client, url: &str) -> io::Result<(Vec<u8>, String)> {
    if !url.starts_with("file:") {
        return http::fetch(client, url);
    }
    let path = Url::parse(url).ok().and_then(|u| u.to_file_path().ok());
    let path = path.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("not a local file URL: {url}"),
        )
    })?;
    Ok((std::fs::read(path)?, url.to_owned()))
}

/// Loads `top_url` into an emptied `store`.
///
/// # Errors
/// Fails when the top-level list cannot be fetched (also over IPv4) or is
/// malformed; the store then stays unavailable (offline mode).
pub fn load_repository(store: &mut OsListStore, top_url: &str) -> io::Result<Loaded> {
    store.clear();
    let client = http::client().map_err(io::Error::other)?;
    let (data, final_url) = match fetch_document(&client, top_url) {
        Ok(fetched) => fetched,
        Err(_) => fetch_document(&ipv4_client()?, top_url)?,
    };
    let result = store.on_fetch_complete(&data, top_url, true);
    if result.malformed {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Incorrectly formatted OS list",
        ));
    }
    let mut pending = result.sublist_urls;
    let mut failed_sublists = Vec::new();
    while !pending.is_empty() {
        let fetched: Vec<(String, Fetched)> = std::thread::scope(|scope| {
            let handles: Vec<_> = pending
                .iter()
                .map(|url| (url.clone(), scope.spawn(|| fetch_document(&client, url))))
                .collect();
            handles
                .into_iter()
                .map(|(url, h)| {
                    (
                        url,
                        h.join()
                            .unwrap_or_else(|_| Err(io::Error::other("fetch panicked"))),
                    )
                })
                .collect()
        });
        pending = Vec::new();
        for (url, outcome) in fetched {
            match outcome {
                Ok((data, _)) => {
                    pending.extend(store.on_fetch_complete(&data, &url, false).sublist_urls)
                }
                Err(_) => failed_sublists.push(url),
            }
        }
    }
    Ok(Loaded {
        final_url,
        failed_sublists,
    })
}

fn ipv4_client() -> io::Result<Client> {
    http::builder()
        .local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
        .build()
        .map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::oslist::store::HwFilter;

    #[test]
    fn loads_local_repository_with_nested_sublists() {
        let dir = tempfile::tempdir().unwrap();
        let path = |name: &str| dir.path().join(name);
        let url = |name: &str| format!("file://{}", path(name).display()).replace(' ', "%20");
        std::fs::write(
            path("deep list.json"),
            json!({"os_list": [{"name": "Deep"}]}).to_string(),
        )
        .unwrap();
        std::fs::write(
            path("sub.json"),
            json!({"os_list": [{"name": "Sub", "subitems_url": url("deep list.json")}]})
                .to_string(),
        )
        .unwrap();
        let top = json!({"imager": {}, "os_list": [
            {"name": "Group", "subitems_url": url("sub.json")},
            {"name": "Broken", "subitems_url": url("missing.json")}
        ]});
        std::fs::write(path("top.json"), top.to_string()).unwrap();
        let mut store = OsListStore::default();
        let loaded = load_repository(&mut store, &url("top.json")).unwrap();
        assert_eq!(loaded.failed_sublists, [url("missing.json")]);
        let doc = store.filtered_document(&HwFilter::default());
        assert_eq!(
            doc["os_list"][0]["subitems"][0]["subitems"][0]["name"],
            "Deep"
        );
    }

    #[test]
    fn malformed_or_missing_top_level_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let bad = dir.path().join("bad.json");
        std::fs::write(&bad, b"{}").unwrap();
        let mut store = OsListStore::default();
        assert!(load_repository(&mut store, &format!("file://{}", bad.display())).is_err());
        assert!(!store.is_available());
        assert!(load_repository(&mut store, "file:///nonexistent/list.json").is_err());
    }

    #[test]
    fn rejects_remote_file_urls() {
        assert!(fetch_document(&http::client().unwrap(), "file://server/share/list.json").is_err());
    }
}
