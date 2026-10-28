//! Opening a disk for writing: volumes locked and dismounted, partition
//! table cleared, then `\\.\PhysicalDriveN` opened read-write with its
//! length from `IOCTL_DISK_GET_LENGTH_INFO`, behind [`SectorAligned`].
//! Dropping the device rereads the disk's layout so the new partitions
//! appear (upstream `refreshDiskView`).

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

use imager_core::pipeline::{Device, Progress};
use windows_sys::Win32::System::Ioctl::{
    FSCTL_ALLOW_EXTENDED_DASD_IO, GET_LENGTH_INFORMATION, IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
    IOCTL_DISK_GET_LENGTH_INFO, IOCTL_DISK_UPDATE_PROPERTIES,
};

use super::aligned::SectorAligned;
use super::describe::parse_geometry;
use super::diskops::{clear_layout, lock_volumes};
use super::ioctl::{command, ioctl, read_struct, rw_handle};
use super::physical::device_number;

/// A disk handle that knows its length (seeking to the end does not work
/// on physical drives), or an image file.
pub struct Disk {
    file: File,
    length: u64,
    physical: bool,
}

/// The write target: a disk in whole sectors, holding its volumes locked.
pub struct RawDevice {
    disk: SectorAligned<Disk>,
    _volumes: Vec<File>,
}

/// Prepares and opens `device`; a path that is not `\\.\PhysicalDriveN`
/// is a plain image file.
///
/// # Errors
/// The message for the user when the disk cannot be prepared or opened.
pub fn open_for_write(device: &str, progress: &Progress) -> Result<RawDevice, String> {
    let Some(number) = device_number(device) else {
        let file = crate::open_plain_file(device)?;
        return Ok(RawDevice::new(
            Disk {
                file,
                length: 0,
                physical: false,
            },
            512,
            Vec::new(),
        ));
    };
    progress.set_status("Unmounting drive...");
    let volumes = lock_volumes(number).map_err(|r| crate::unmount_failed(device, r))?;
    progress.set_status("Opening drive...");
    let mut file = rw_handle(device).map_err(|e| opening_failed(device, &e))?;
    clear_layout(&mut file).map_err(|e| opening_failed(device, &e))?;
    let _ = command(&file, FSCTL_ALLOW_EXTENDED_DASD_IO);
    let length = read_struct::<GET_LENGTH_INFORMATION>(&file, IOCTL_DISK_GET_LENGTH_INFO)
        .map_err(|e| opening_failed(device, &e))?
        .Length;
    let mut geometry = vec![0u8; 256];
    let filled = ioctl(&file, IOCTL_DISK_GET_DRIVE_GEOMETRY_EX, &[], &mut geometry).unwrap_or(0);
    let sector = parse_geometry(&geometry[..filled]).map_or(512, |(_, s)| s);
    let disk = Disk {
        file,
        length: u64::try_from(length).unwrap_or(0),
        physical: true,
    };
    Ok(RawDevice::new(disk, sector, volumes))
}

fn opening_failed(device: &str, error: &io::Error) -> String {
    let hint = if error.kind() == io::ErrorKind::PermissionDenied {
        format!("<br>Please run {} as administrator", imager_core::APP_NAME)
    } else {
        String::new()
    };
    format!("Error opening disk device '{device}'{hint} ({error}).")
}

impl RawDevice {
    fn new(disk: Disk, sector: u32, volumes: Vec<File>) -> Self {
        Self {
            disk: SectorAligned::new(disk, sector),
            _volumes: volumes,
        }
    }
}

impl Drop for RawDevice {
    fn drop(&mut self) {
        let disk = self.disk.get_mut();
        if disk.physical {
            let _ = command(&disk.file, IOCTL_DISK_UPDATE_PROPERTIES);
        }
    }
}

impl Read for Disk {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl Write for Disk {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl Seek for Disk {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}

impl Device for Disk {
    /// The disk's length, or an image file's current length.
    fn size(&mut self) -> io::Result<u64> {
        if self.physical {
            Ok(self.length)
        } else {
            Ok(self.file.metadata()?.len())
        }
    }

    /// `FlushFileBuffers`.
    fn force_sync(&mut self) -> io::Result<()> {
        self.file.sync_all()
    }
}

impl Read for RawDevice {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.disk.read(buf)
    }
}

impl Write for RawDevice {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.disk.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.disk.flush()
    }
}

impl Seek for RawDevice {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.disk.seek(pos)
    }
}

impl Device for RawDevice {
    fn size(&mut self) -> io::Result<u64> {
        self.disk.size()
    }

    fn force_sync(&mut self) -> io::Result<()> {
        self.disk.force_sync()
    }
}
