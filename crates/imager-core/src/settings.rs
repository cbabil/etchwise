//! Persistent settings with upstream's keys and defaults (QSettings
//! "Raspberry Pi Imager"). Stored as one JSON file; the groups
//! `imagecustomization` and `caching` are nested objects.

use std::fs;
use std::path::PathBuf;

use serde_json::{Map, Value};

use self::variant::dedupe_keys;
pub use self::variant::variant_to_bool;
pub(crate) use self::variant::variant_to_string;

mod error;
mod variant;

pub use self::error::SettingsError;

/// Root keys with a default, as in upstream `getBoolSetting`. Upstream's
/// `telemetry` default is gone: Etchwise sends no statistics.
pub const DEFAULTS: [(&str, bool); 2] = [("eject", true), ("check_version", true)];

/// Group holding the saved OS customisation.
pub const CUSTOMISATION_GROUP: &str = "imagecustomization";

/// Settings store; writes through to disk when backed by a file.
#[derive(Debug, Clone, Default)]
pub struct Settings {
    path: Option<PathBuf>,
    root: Map<String, Value>,
}

impl Settings {
    /// In-memory settings (tests, CLI without persistence).
    pub fn in_memory() -> Self {
        Self::default()
    }

    /// Loads `path`; a missing file means empty settings.
    ///
    /// # Errors
    /// Fails when the file exists but cannot be read or parsed.
    pub fn load(path: PathBuf) -> Result<Self, SettingsError> {
        let root = match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|source| SettingsError::Parse {
                path: path.clone(),
                source,
            })?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Map::new(),
            Err(source) => return Err(SettingsError::Io { path, source }),
        };
        Ok(Self {
            path: Some(path),
            root,
        })
    }

    /// `getBoolSetting`; see [`DEFAULTS`].
    pub fn bool(&self, key: &str) -> bool {
        self.value(key).is_some_and(|v| variant_to_bool(&v))
    }

    /// `getStringSetting`; non-strings are rendered, missing keys are empty.
    pub fn string(&self, key: &str) -> String {
        self.value(key)
            .map(|v| variant_to_string(&v))
            .unwrap_or_default()
    }

    /// A root key's value, or its default from [`DEFAULTS`].
    pub fn value(&self, key: &str) -> Option<Value> {
        self.root.get(key).cloned().or_else(|| {
            DEFAULTS
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| Value::Bool(*v))
        })
    }

    /// `setSetting`.
    ///
    /// # Errors
    /// Fails when the settings file cannot be written.
    pub fn set(&mut self, key: &str, value: Value) -> Result<(), SettingsError> {
        self.root.insert(key.to_owned(), value);
        self.save()
    }

    /// Reads `group/key`.
    pub fn group_value(&self, group: &str, key: &str) -> Option<&Value> {
        self.root.get(group)?.get(key)
    }

    /// Writes `group/key`.
    ///
    /// # Errors
    /// Fails when the settings file cannot be written.
    pub fn set_group_value(
        &mut self,
        group: &str,
        key: &str,
        value: Value,
    ) -> Result<(), SettingsError> {
        self.group_mut(group).insert(key.to_owned(), value);
        self.save()
    }

    /// Removes `group/key`.
    ///
    /// # Errors
    /// Fails when the settings file cannot be written.
    pub fn remove_group_value(&mut self, group: &str, key: &str) -> Result<(), SettingsError> {
        self.group_mut(group).remove(key);
        self.save()
    }

    /// `getSavedCustomisationSettings`, with `sshAuthorizedKeys` de-duplicated.
    pub fn customisation(&self) -> Map<String, Value> {
        let mut map = self
            .root
            .get(CUSTOMISATION_GROUP)
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        if let Some(keys) = map.get("sshAuthorizedKeys").map(variant_to_string) {
            map.insert(
                "sshAuthorizedKeys".into(),
                Value::String(dedupe_keys(&keys)),
            );
        }
        map
    }

    /// `setSavedCustomisationSettings`: replaces the whole group.
    ///
    /// # Errors
    /// Fails when the settings file cannot be written.
    pub fn set_customisation(&mut self, map: Map<String, Value>) -> Result<(), SettingsError> {
        self.root
            .insert(CUSTOMISATION_GROUP.into(), Value::Object(map));
        self.save()
    }

    /// `clearSavedCustomisationSettings`.
    ///
    /// # Errors
    /// Fails when the settings file cannot be written.
    pub fn clear_customisation(&mut self) -> Result<(), SettingsError> {
        self.set_customisation(Map::new())
    }

    fn group_mut(&mut self, group: &str) -> &mut Map<String, Value> {
        let entry = self
            .root
            .entry(group.to_owned())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }
        entry
            .as_object_mut()
            .unwrap_or_else(|| unreachable!("group was just made an object"))
    }

    fn save(&self) -> Result<(), SettingsError> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let io = |source| SettingsError::Io {
            path: path.clone(),
            source,
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(io)?;
        }
        let json =
            serde_json::to_vec_pretty(&self.root).map_err(|source| SettingsError::Parse {
                path: path.clone(),
                source,
            })?;
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, json).map_err(io)?;
        fs::rename(&temp, path).map_err(io)
    }
}

#[cfg(test)]
mod tests;
