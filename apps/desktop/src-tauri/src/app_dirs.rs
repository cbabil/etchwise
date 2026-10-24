//! Where Etchwise keeps its settings and cache. Builds before the rename
//! used the `pi-imager` identifier; their folders move over once so saved
//! settings (such as the extra repository) carry across.

use std::path::{Path, PathBuf};

/// The app identifier, also the folder name under config and cache dirs.
pub const APP_DIR: &str = "me.digitalminimalist.etchwise";
/// The identifier used before the rename.
const OLD_APP_DIR: &str = "me.digitalminimalist.pi-imager";

/// `base/APP_DIR`, first moving `base/OLD_APP_DIR` there when only the old
/// folder exists. A failed move leaves the old folder alone (fresh start).
pub fn app_dir(base: &Path) -> PathBuf {
    let new = base.join(APP_DIR);
    let old = base.join(OLD_APP_DIR);
    if !new.exists() && old.is_dir() {
        let _ = std::fs::rename(&old, &new);
    }
    new
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_the_old_folder_once() {
        let base = tempfile::tempdir().unwrap();
        std::fs::create_dir(base.path().join(OLD_APP_DIR)).unwrap();
        std::fs::write(base.path().join(OLD_APP_DIR).join("settings.json"), "{}").unwrap();
        let dir = app_dir(base.path());
        assert_eq!(dir, base.path().join(APP_DIR));
        assert!(dir.join("settings.json").is_file());
        assert!(!base.path().join(OLD_APP_DIR).exists());
    }

    #[test]
    fn keeps_an_existing_new_folder() {
        let base = tempfile::tempdir().unwrap();
        for name in [APP_DIR, OLD_APP_DIR] {
            std::fs::create_dir(base.path().join(name)).unwrap();
            std::fs::write(base.path().join(name).join("settings.json"), name).unwrap();
        }
        let dir = app_dir(base.path());
        assert_eq!(
            std::fs::read_to_string(dir.join("settings.json")).unwrap(),
            APP_DIR
        );
        assert!(base.path().join(OLD_APP_DIR).exists());
    }

    #[test]
    fn works_without_any_folder() {
        let base = tempfile::tempdir().unwrap();
        assert_eq!(app_dir(base.path()), base.path().join(APP_DIR));
    }
}
