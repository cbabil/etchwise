//! In-memory extraction of `bootfiles.bin` (upstream `bootfiles.cpp`):
//! the sideload firmware ships its boot files as a TAR archive whose
//! regular entries are held in a name-to-bytes map for the file server.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

/// Extracted archive entries.
#[derive(Debug, Default, Clone)]
pub struct Bootfiles {
    files: BTreeMap<String, Vec<u8>>,
}

impl Bootfiles {
    #[cfg(test)]
    /// Replaces the contents with the entries of an in-memory TAR.
    ///
    /// # Errors
    /// `Empty archive data`, or a failure reading an entry's data.
    pub fn extract_from_memory(&mut self, tar: &[u8]) -> Result<(), String> {
        self.files.clear();
        if tar.is_empty() {
            return Err("Empty archive data".into());
        }
        self.extract_from_archive(tar)
    }

    /// Replaces the contents with the entries of a TAR file.
    ///
    /// # Errors
    /// `Failed to open tar file: ...`, or a failure reading an entry.
    pub fn extract_from_file(&mut self, path: &Path) -> Result<(), String> {
        self.files.clear();
        let file =
            std::fs::File::open(path).map_err(|e| format!("Failed to open tar file: {e}"))?;
        self.extract_from_archive(std::io::BufReader::with_capacity(16384, file))
    }

    /// Like upstream's `archive_read_next_header` loop, reading stops
    /// quietly at the first unreadable header; only data errors fail.
    fn extract_from_archive(&mut self, reader: impl Read) -> Result<(), String> {
        let mut archive = tar::Archive::new(reader);
        let Ok(entries) = archive.entries() else {
            return Ok(());
        };
        for entry in entries {
            let Ok(mut entry) = entry else { break };
            if !entry.header().entry_type().is_file() {
                continue;
            }
            let raw_name = String::from_utf8_lossy(&entry.path_bytes()).into_owned();
            let mut data = Vec::with_capacity(entry.size() as usize);
            entry
                .read_to_end(&mut data)
                .map_err(|e| format!("Error reading archive entry '{raw_name}': {e}"))?;
            let name = raw_name
                .strip_prefix("./")
                .filter(|rest| !rest.is_empty())
                .map_or(raw_name.clone(), str::to_owned);
            self.files.insert(name, data);
        }
        Ok(())
    }

    /// Looks up `name`, then without / with a leading `./`, then (for a
    /// bare name) as `<chip_prefix>/<name>` (e.g. `2712/mcb.bin`).
    pub fn find(&self, name: &str, chip_prefix: &str) -> Option<&[u8]> {
        let get = |key: &str| self.files.get(key).map(Vec::as_slice);
        if let Some(data) = get(name) {
            return Some(data);
        }
        if let Some(rest) = name.strip_prefix("./").filter(|r| !r.is_empty())
            && let Some(data) = get(rest)
        {
            return Some(data);
        }
        if let Some(data) = get(&format!("./{name}")) {
            return Some(data);
        }
        if !chip_prefix.is_empty() && !name.contains('/') {
            return get(&format!("{chip_prefix}/{name}"));
        }
        None
    }

    #[cfg(test)]
    /// Number of extracted entries.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    #[cfg(test)]
    /// All entries by name.
    pub fn files(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.files
    }
}
