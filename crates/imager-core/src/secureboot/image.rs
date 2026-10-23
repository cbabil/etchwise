//! `boot.img` creation (upstream `SecureBoot::createBootImg` and the
//! platform `BootImgCreator`s): a partitionless FAT32 volume holding the
//! boot files with their directory structure.

use std::collections::BTreeMap;
use std::io::{self, Cursor, Write};

use fatfs::{FatType, FileSystem, FormatVolumeOptions, FsOptions};

const MIB: u64 = 1024 * 1024;

/// Smallest image upstream creates (the FAT32 minimum it assumes).
const FAT32_MIN_SIZE: u64 = 33 * MIB;

/// Image size for `total` bytes of content: 150 %, rounded up to the next
/// whole MiB (always adding one, as upstream does), at least 33 MiB.
pub fn boot_img_size(total: u64) -> u64 {
    ((total * 3 / 2) / MIB + 1)
        .saturating_mul(MIB)
        .max(FAT32_MIN_SIZE)
}

/// Builds `boot.img` in memory from `path -> contents` (paths may contain
/// `/`-separated directories, which are created as needed).
///
/// # Errors
/// Fails when the image does not fit in memory or the files do not fit.
pub fn create_boot_img(files: &BTreeMap<String, Vec<u8>>) -> io::Result<Vec<u8>> {
    let total = files.values().map(|data| data.len() as u64).sum();
    let size = usize::try_from(boot_img_size(total))
        .map_err(|_| io::Error::other("boot.img too large for memory"))?;
    let mut image = vec![0u8; size];
    fatfs::format_volume(
        Cursor::new(&mut image[..]),
        FormatVolumeOptions::new().fat_type(FatType::Fat32),
    )?;
    let fs = FileSystem::new(Cursor::new(&mut image[..]), FsOptions::new())?;
    for (path, data) in files {
        let root = fs.root_dir();
        for (end, _) in path.match_indices('/') {
            root.create_dir(&path[..end])?;
        }
        let mut file = root.create_file(path)?;
        file.truncate()?;
        file.write_all(data)?;
        file.flush()?;
    }
    fs.unmount()?;
    Ok(image)
}
