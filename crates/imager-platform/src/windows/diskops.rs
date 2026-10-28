//! Taking a disk away from Windows for the write and giving it back
//! (upstream `processDriveLetter`, `winfile.cpp`, `cleanDiskFast`): each
//! volume on the disk locked and dismounted, its handle kept until the
//! write ends; the partition table removed so nothing remounts; eject
//! through `IOCTL_STORAGE_EJECT_MEDIA`.

use std::fs::File;
use std::io::{self, Seek, SeekFrom, Write};
use std::time::Duration;

use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_INVALID_FUNCTION};
use windows_sys::Win32::System::Ioctl::{
    FSCTL_ALLOW_EXTENDED_DASD_IO, FSCTL_DISMOUNT_VOLUME, FSCTL_LOCK_VOLUME,
    IOCTL_DISK_DELETE_DRIVE_LAYOUT, IOCTL_STORAGE_EJECT_MEDIA, IOCTL_STORAGE_MEDIA_REMOVAL,
    PREVENT_MEDIA_REMOVAL,
};

use super::drivelist::letters_of;
use super::ioctl::{bytes_of, command, ioctl, rw_handle};
use super::physical::{device_number, device_path, volume_path};
use crate::DiskResult;

/// Lock attempts per volume, 500 ms apart, as upstream `lockVolume`
/// (Explorer or an indexer may hold a file open for a while).
const LOCK_ATTEMPTS: usize = 20;
const LOCK_PAUSE: Duration = Duration::from_millis(500);

/// Locks and dismounts every volume on disk `number`; the handles keep
/// the volumes locked until dropped. A volume that cannot be opened (a
/// card slot without media) is skipped; one that stays in use is
/// dismounted anyway, which closes the other apps' handles.
///
/// # Errors
/// What went wrong, as a [`DiskResult`].
pub fn lock_volumes(number: u32) -> Result<Vec<File>, DiskResult> {
    let mut volumes = Vec::new();
    for letter in letters_of(number) {
        let volume = match rw_handle(&volume_path(letter)) {
            Ok(v) => v,
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                return Err(DiskResult::AccessDenied);
            }
            Err(_) => continue,
        };
        let _locked = lock(&volume);
        command(&volume, FSCTL_DISMOUNT_VOLUME).map_err(|_| DiskResult::Busy)?;
        volumes.push(volume);
    }
    Ok(volumes)
}

fn lock(volume: &File) -> bool {
    for attempt in 0..LOCK_ATTEMPTS {
        if command(volume, FSCTL_LOCK_VOLUME).is_ok() {
            return true;
        }
        if attempt + 1 < LOCK_ATTEMPTS {
            std::thread::sleep(LOCK_PAUSE);
        }
    }
    false
}

/// Removes the partition table (upstream `cleanDiskFast`), zeroing the
/// first sector when the IOCTL is refused, so Windows does not remount
/// the old volumes while the image is written.
pub fn clear_layout(disk: &mut File) -> io::Result<()> {
    let _ = command(disk, FSCTL_ALLOW_EXTENDED_DASD_IO);
    match command(disk, IOCTL_DISK_DELETE_DRIVE_LAYOUT) {
        Ok(()) => Ok(()),
        Err(e)
            if [ERROR_INVALID_FUNCTION, ERROR_FILE_NOT_FOUND]
                .map(|c| c as i32)
                .contains(&e.raw_os_error().unwrap_or(0)) =>
        {
            Ok(())
        }
        Err(_) => {
            disk.seek(SeekFrom::Start(0))?;
            disk.write_all(&[0u8; 4096])
        }
    }
}

/// Unlocks the media and ejects it: each volume (they reappear after the
/// write), or the whole disk when it has none.
pub fn eject_disk(device: &str) -> DiskResult {
    let Some(number) = device_number(device) else {
        return DiskResult::InvalidDrive;
    };
    let volumes = match lock_volumes(number) {
        Ok(v) if !v.is_empty() => v,
        Ok(_) => match rw_handle(&device_path(number)) {
            Ok(disk) => vec![disk],
            Err(e) => return result_of(&e),
        },
        Err(result) => return result,
    };
    let allow = PREVENT_MEDIA_REMOVAL {
        PreventMediaRemoval: false,
    };
    let ejected = volumes.iter().all(|v| {
        let _ = ioctl(v, IOCTL_STORAGE_MEDIA_REMOVAL, bytes_of(&allow), &mut []);
        command(v, IOCTL_STORAGE_EJECT_MEDIA).is_ok()
    });
    if ejected {
        DiskResult::Success
    } else {
        DiskResult::Error
    }
}

fn result_of(error: &io::Error) -> DiskResult {
    match error.kind() {
        io::ErrorKind::PermissionDenied => DiskResult::AccessDenied,
        io::ErrorKind::NotFound => DiskResult::InvalidDrive,
        _ => DiskResult::Error,
    }
}
