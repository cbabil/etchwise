//! Editing the boot (first) partition in place on the raw device, the way
//! upstream does after writing an image: no mount, a block-cached FAT
//! driver over the open device (`DeviceWrapperFatPartition`), MBR last.

mod blockcache;
mod extract;
mod partition;

use std::io::{self, Read, Seek, Write};

use fatfs::{FileSystem, FsOptions};

pub use self::blockcache::BlockCache;
use self::partition::Window;
pub use self::partition::{Extent, locate};

/// The first partition of a device, opened for file edits.
pub struct BootPartition<D> {
    cache: BlockCache<D>,
    extent: Extent,
}

impl<D: Read + Write + Seek> BootPartition<D> {
    /// Locates partition 1 (GPT or MBR) on `device`.
    ///
    /// # Errors
    /// Fails when the partition table is missing or invalid.
    pub fn open(device: D) -> io::Result<Self> {
        let mut cache = BlockCache::new(device);
        let extent = locate(&mut cache, 1)?;
        Ok(Self { cache, extent })
    }

    /// Like [`open`](Self::open), with `prefix` (the held-back first chunk
    /// of a fresh write) laid over the start of the device in the cache, so
    /// it reaches the device in the final sync, MBR block last.
    ///
    /// # Errors
    /// Fails when the partition table in `prefix` is missing or invalid.
    pub fn open_with_prefix(device: D, prefix: &[u8]) -> io::Result<Self> {
        let mut cache = BlockCache::new(device);
        cache.pwrite(prefix, 0)?;
        let extent = locate(&mut cache, 1)?;
        Ok(Self { cache, extent })
    }

    /// Formats `extent` as a new, empty FAT volume named `label` (padded to
    /// 11 bytes) and opens it; fatfs picks FAT12/16/32 by size.
    ///
    /// # Errors
    /// Fails when the volume cannot be written.
    pub fn format_new(device: D, extent: Extent, label: [u8; 11]) -> io::Result<Self> {
        let mut cache = BlockCache::new(device);
        let options = fatfs::FormatVolumeOptions::new().volume_label(label);
        fatfs::format_volume(Window::new(&mut cache, extent), options)?;
        Ok(Self { cache, extent })
    }

    /// The partition's byte range.
    pub fn extent(&self) -> Extent {
        self.extent
    }

    /// Reads a file (`name` or `dir/name`); `None` when it does not exist.
    ///
    /// # Errors
    /// Fails when the filesystem cannot be read.
    pub fn read_file(&mut self, path: &str) -> io::Result<Option<Vec<u8>>> {
        self.with_fs(|fs| match fs.root_dir().open_file(path) {
            Ok(mut file) => {
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;
                Ok(Some(data))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        })
    }

    /// Creates or replaces a file; its directory must already exist.
    ///
    /// # Errors
    /// Fails when the directory is missing or the filesystem is full.
    pub fn write_file(&mut self, path: &str, contents: &[u8]) -> io::Result<()> {
        self.with_fs(|fs| {
            let mut file = fs.root_dir().create_file(path)?;
            file.truncate()?;
            file.write_all(contents)?;
            file.flush()
        })
    }

    /// Whether `path` names an existing file or directory.
    ///
    /// # Errors
    /// Fails when the filesystem cannot be read.
    pub fn exists(&mut self, path: &str) -> io::Result<bool> {
        self.with_fs(|fs| {
            let root = fs.root_dir();
            Ok(root.open_file(path).is_ok() || root.open_dir(path).is_ok())
        })
    }

    /// Deletes a file; `false` when it did not exist.
    ///
    /// # Errors
    /// Fails when the filesystem cannot be updated.
    pub fn delete_file(&mut self, path: &str) -> io::Result<bool> {
        self.with_fs(|fs| match fs.root_dir().remove(path) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        })
    }

    /// Every file path, recursively (`dir/name`), skipping `.` and `..`.
    ///
    /// # Errors
    /// Fails when a directory cannot be read.
    pub fn list_files(&mut self) -> io::Result<Vec<String>> {
        self.with_fs(|fs| {
            let mut files = Vec::new();
            let mut pending = vec![(String::new(), fs.root_dir())];
            while let Some((prefix, dir)) = pending.pop() {
                for entry in dir.iter() {
                    let entry = entry?;
                    let name = entry.file_name();
                    if name == "." || name == ".." {
                        continue;
                    }
                    let path = format!("{prefix}{name}");
                    if entry.is_dir() {
                        pending.push((format!("{path}/"), entry.to_dir()));
                    } else {
                        files.push(path);
                    }
                }
            }
            Ok(files)
        })
    }

    /// Writes every change (MBR block last) and returns the device.
    ///
    /// # Errors
    /// Fails when the device rejects the writes.
    pub fn finish(self) -> io::Result<D> {
        self.cache.into_inner()
    }

    fn with_fs<R>(
        &mut self,
        op: impl FnOnce(&FileSystem<Window<'_, D>>) -> io::Result<R>,
    ) -> io::Result<R> {
        let fs = FileSystem::new(
            Window::new(&mut self.cache, self.extent),
            FsOptions::new().update_accessed_date(false),
        )?;
        let result = op(&fs)?;
        fs.unmount()?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests;
