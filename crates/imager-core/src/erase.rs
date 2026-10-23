//! Erase: MBR + FAT32 formatting, a port of upstream v2.0.7
//! `DiskFormatter` as driven by `DriveFormatThread` ("Erase" in the OS
//! list). One bootable FAT32-LBA partition from sector 8192 to the end of
//! the device, labelled `BOOT`, with an empty root directory.
//!
//! The write sequence is upstream's: MBR, boot sector, FSInfo, backup boot
//! sector, both FATs, root cluster. Upstream issues no explicit sync (it
//! just closes the descriptor), so neither does this; it only flushes.

mod layout;
mod sectors;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_device;
#[cfg(test)]
mod tests_mount;

use std::io::{self, SeekFrom};

use crate::pipeline::{Device, Progress, WriteState};

use self::layout::{
    BACKUP_BOOT_SECTOR, DeviceLayout, FS_INFO_SECTOR, PARTITION_START_SECTOR, SECTOR_SIZE,
};
use self::sectors::{boot_sector, fat_head, fs_info, mbr};

/// Why formatting failed, with upstream's `formatErrorToString` messages.
#[derive(Debug, thiserror::Error)]
pub enum EraseError {
    /// The device size could not be read (`kFileOpenError`).
    #[error("Error opening device for formatting")]
    Open(#[source] io::Error),
    /// A write or the final flush failed (`kFileWriteError`).
    #[error("Error writing to device during formatting")]
    Write(#[source] io::Error),
    /// Seeking to a structure failed (`kFileSeekError`).
    #[error("Error seeking on device during formatting")]
    Seek(#[source] io::Error),
    /// The device cannot hold the layout (`kInsufficientSpace`; upstream
    /// never detects this and would underflow instead).
    #[error("Insufficient space on device")]
    InsufficientSpace {
        /// Device size in bytes.
        size: u64,
    },
    /// Cancelled between writes (`kCancelled`, which upstream's message
    /// switch falls through to its default text for).
    #[error("Unknown formatting error")]
    Cancelled,
}

/// One write: `head` bytes at `offset`, then `zeros` zero bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Region {
    pub(crate) offset: u64,
    pub(crate) head: Vec<u8>,
    pub(crate) zeros: u64,
}

impl Region {
    fn sector(sector: u32, head: &[u8]) -> Self {
        Self::at(sector, head, 0)
    }

    fn at(sector: u32, head: &[u8], zeros: u64) -> Self {
        let offset = u64::from(sector) * u64::from(SECTOR_SIZE);
        Self {
            offset,
            head: head.to_vec(),
            zeros,
        }
    }

    /// Bytes this region writes.
    pub(crate) fn len(&self) -> u64 {
        self.head.len() as u64 + self.zeros
    }
}

/// Every write upstream makes for a device of `size_bytes`, in order;
/// `None` when the device is too small.
pub(crate) fn plan(size_bytes: u64) -> Option<Vec<Region>> {
    let layout = DeviceLayout::for_device(size_bytes)?;
    let fat = layout.fat;
    let start = PARTITION_START_SECTOR;
    let sector = u64::from(SECTOR_SIZE);
    let fat_tail = (u64::from(fat.sectors_per_fat) - 1) * sector;
    let mut regions = vec![
        Region::sector(0, &mbr(layout.mbr_partition_sectors)),
        Region::sector(start, &boot_sector(&fat, start)),
        Region::sector(start + FS_INFO_SECTOR, &fs_info(&fat)),
        Region::sector(
            start + BACKUP_BOOT_SECTOR,
            &boot_sector(&fat, start + BACKUP_BOOT_SECTOR),
        ),
    ];
    for copy in 0..u32::from(fat.config.num_fats) {
        let at = start + fat.fat_start() + copy * fat.sectors_per_fat;
        regions.push(Region::at(at, &fat_head(), fat_tail));
    }
    let root_bytes = u64::from(fat.config.sectors_per_cluster) * sector;
    regions.push(Region::at(start + fat.root_start(), &[], root_bytes));
    Some(regions)
}

/// Formats `device` as upstream's Erase does once the device is open.
/// Sets `progress.write_total` to the bytes to write and advances
/// `progress.bytes_written`; checks cancellation before each write.
///
/// # Errors
/// [`EraseError`] when the size cannot be read, the device is too small,
/// a seek or write fails, or `progress` is cancelled.
pub fn format_device<D: Device>(device: &mut D, progress: &Progress) -> Result<(), EraseError> {
    let size = device.size().map_err(EraseError::Open)?;
    let regions = plan(size).ok_or(EraseError::InsufficientSpace { size })?;
    progress.set_state(WriteState::Writing);
    Progress::set(&progress.write_total, regions.iter().map(Region::len).sum());
    Progress::set(&progress.bytes_written, 0);
    for region in &regions {
        if progress.is_cancelled() {
            return Err(EraseError::Cancelled);
        }
        write_region(device, region, progress)?;
    }
    device.flush().map_err(EraseError::Write)
}

/// Largest zero run written per call (upstream allocates the whole FAT).
const ZERO_CHUNK: usize = 1 << 20;
static ZEROS: [u8; ZERO_CHUNK] = [0; ZERO_CHUNK];

fn write_region<D: Device>(
    device: &mut D,
    region: &Region,
    progress: &Progress,
) -> Result<(), EraseError> {
    device
        .seek(SeekFrom::Start(region.offset))
        .map_err(EraseError::Seek)?;
    device.write_all(&region.head).map_err(EraseError::Write)?;
    Progress::add(&progress.bytes_written, region.head.len() as u64);
    let mut left = region.zeros;
    while left > 0 {
        let n = usize::try_from(left).map_or(ZERO_CHUNK, |l| l.min(ZERO_CHUNK));
        device.write_all(&ZEROS[..n]).map_err(EraseError::Write)?;
        Progress::add(&progress.bytes_written, n as u64);
        left -= n as u64;
    }
    Ok(())
}
