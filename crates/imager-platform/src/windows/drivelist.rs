//! Physical disk enumeration (upstream `drivelist_windows.cpp`): every
//! `\\.\PhysicalDriveN` that answers, described from its storage
//! descriptor and geometry, with the drive letters whose volumes live on
//! it. Works without administrator rights.

use std::io;

use imager_core::drives::DeviceDescriptor;
use windows_sys::Win32::Storage::FileSystem::GetLogicalDrives;
use windows_sys::Win32::System::Ioctl::{
    IOCTL_DISK_GET_DRIVE_GEOMETRY_EX, IOCTL_DISK_IS_WRITABLE, IOCTL_STORAGE_GET_DEVICE_NUMBER,
    IOCTL_STORAGE_QUERY_PROPERTY, PropertyStandardQuery, STORAGE_DEVICE_NUMBER,
    STORAGE_PROPERTY_QUERY, StorageDeviceProperty,
};
use windows_sys::Win32::System::SystemInformation::GetSystemWindowsDirectoryW;

use super::describe::{DiskFacts, describe, parse_geometry, parse_storage_descriptor};
use super::ioctl::{bytes_of, ioctl, query_handle, read_struct};
use super::physical::{device_path, volume_path};

/// Disk numbers probed; Windows numbers disks from 0 without gaps in
/// practice, but a removed disk can leave one.
const MAX_DISKS: u32 = 64;
/// `FILE_DEVICE_DISK`.
const FILE_DEVICE_DISK: u32 = 7;

/// Every physical disk with its drive letters.
///
/// # Errors
/// Never fails as a whole; disks that do not answer are left out.
pub fn list_drives() -> io::Result<Vec<DeviceDescriptor>> {
    let letters = volume_letters();
    let windows = windows_letter();
    Ok((0..MAX_DISKS)
        .filter_map(|number| probe(number, &letters, windows))
        .map(|facts| describe(&facts))
        .collect())
}

/// The drive letters on disk `number`.
pub fn letters_of(number: u32) -> Vec<char> {
    volume_letters()
        .into_iter()
        .filter(|(_, disk)| *disk == number)
        .map(|(letter, _)| letter)
        .collect()
}

fn probe(number: u32, letters: &[(char, u32)], windows: Option<char>) -> Option<DiskFacts> {
    let disk = query_handle(&device_path(number)).ok()?;
    let query = STORAGE_PROPERTY_QUERY {
        PropertyId: StorageDeviceProperty,
        QueryType: PropertyStandardQuery,
        AdditionalParameters: [0],
    };
    let mut descriptor = vec![0u8; 1024];
    let filled = ioctl(
        &disk,
        IOCTL_STORAGE_QUERY_PROPERTY,
        bytes_of(&query),
        &mut descriptor,
    )
    .ok()?;
    let storage = parse_storage_descriptor(&descriptor[..filled])?;
    let mut geometry = vec![0u8; 256];
    let filled = ioctl(&disk, IOCTL_DISK_GET_DRIVE_GEOMETRY_EX, &[], &mut geometry).unwrap_or(0);
    let (size, sector_size) = parse_geometry(&geometry[..filled]).unwrap_or((0, 512));
    let letters: Vec<char> = letters
        .iter()
        .filter(|(_, disk)| *disk == number)
        .map(|(letter, _)| *letter)
        .collect();
    Some(DiskFacts {
        number,
        storage,
        size,
        sector_size,
        writable: ioctl(&disk, IOCTL_DISK_IS_WRITABLE, &[], &mut []).is_ok(),
        holds_windows: windows.is_some_and(|w| letters.contains(&w)),
        letters,
    })
}

/// `(letter, disk number)` for every lettered volume on a disk.
fn volume_letters() -> Vec<(char, u32)> {
    // SAFETY: no arguments.
    let mask = unsafe { GetLogicalDrives() };
    ('A'..='Z')
        .enumerate()
        .filter(|(bit, _)| mask & (1 << bit) != 0)
        .filter_map(|(_, letter)| {
            let volume = query_handle(&volume_path(letter)).ok()?;
            let number: STORAGE_DEVICE_NUMBER =
                read_struct(&volume, IOCTL_STORAGE_GET_DEVICE_NUMBER).ok()?;
            (number.DeviceType == FILE_DEVICE_DISK).then_some((letter, number.DeviceNumber))
        })
        .collect()
}

/// The letter of the volume holding Windows (usually C).
fn windows_letter() -> Option<char> {
    let mut buffer = [0u16; 260];
    // SAFETY: the buffer and its length in characters.
    let len = unsafe { GetSystemWindowsDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) };
    char::from_u32(u32::from(*buffer.first().filter(|_| len > 0)?)).map(|c| c.to_ascii_uppercase())
}
