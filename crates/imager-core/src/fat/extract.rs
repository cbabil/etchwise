//! Streaming writes into the partition, for extracting multi-file archives
//! (upstream writes them through the auto-mounted volume; this writes the
//! same files straight into the FAT).

use std::io::{self, Read, Seek, Write};

use super::BootPartition;

impl<D: Read + Write + Seek> BootPartition<D> {
    /// Creates `path` and every missing parent directory.
    ///
    /// # Errors
    /// Fails when a component exists as a file or the filesystem is full.
    pub fn create_dir_all(&mut self, path: &str) -> io::Result<()> {
        self.with_fs(|fs| {
            let mut dir = fs.root_dir();
            for part in path.split('/').filter(|p| !p.is_empty()) {
                dir = dir.create_dir(part)?;
            }
            Ok(())
        })
    }

    /// Streams `contents` into a new file, creating parent directories,
    /// in one mount. Refuses to replace an existing file or directory
    /// (libarchive's `NO_OVERWRITE`).
    ///
    /// # Errors
    /// `AlreadyExists` for an existing entry; I/O errors otherwise.
    pub fn write_new_file(&mut self, path: &str, contents: &mut dyn Read) -> io::Result<u64> {
        let (parent, name) = path.rsplit_once('/').unwrap_or(("", path));
        self.with_fs(|fs| {
            let mut dir = fs.root_dir();
            for part in parent.split('/').filter(|p| !p.is_empty()) {
                dir = dir.create_dir(part)?;
            }
            if dir.open_file(name).is_ok() || dir.open_dir(name).is_ok() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("{path} already exists"),
                ));
            }
            let mut file = dir.create_file(name)?;
            let written = io::copy(contents, &mut file)?;
            file.flush()?;
            Ok(written)
        })
    }
}
