//! An opened disk as a write target (upstream `MacOSFileOperations`):
//! size from `DKIOCGETBLOCKCOUNT` × `DKIOCGETBLOCKSIZE`, `fsync` for
//! `ForceSync`, optional `F_NOCACHE` (upstream's Direct I/O debug option,
//! off by default).

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::fd::AsRawFd;

use imager_core::pipeline::Device;

/// `_IOR('d', 24, uint32_t)`.
const DKIOCGETBLOCKSIZE: libc::c_ulong = 0x4004_6418;
/// `_IOR('d', 25, uint64_t)`.
const DKIOCGETBLOCKCOUNT: libc::c_ulong = 0x4008_6419;

/// A raw disk (or, for tests, a regular file).
pub struct RawDevice {
    file: File,
}

impl RawDevice {
    pub fn new(file: File) -> Self {
        Self { file }
    }

    /// Upstream's "Direct I/O" debug option: bypass the buffer cache.
    ///
    /// # Errors
    /// Fails when `fcntl` rejects the flag.
    pub fn set_direct_io(&self, enabled: bool) -> io::Result<()> {
        // SAFETY: valid descriptor; F_NOCACHE takes an int flag.
        if unsafe {
            libc::fcntl(
                self.file.as_raw_fd(),
                libc::F_NOCACHE,
                libc::c_int::from(enabled),
            )
        } == -1
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    fn ioctl_size(&self) -> Option<u64> {
        let (mut block_size, mut block_count) = (0u32, 0u64);
        let fd = self.file.as_raw_fd();
        // SAFETY: both ioctls write one integer of the declared size.
        let ok = unsafe {
            libc::ioctl(fd, DKIOCGETBLOCKSIZE, &mut block_size) == 0
                && libc::ioctl(fd, DKIOCGETBLOCKCOUNT, &mut block_count) == 0
        };
        ok.then(|| u64::from(block_size) * block_count)
    }
}

impl Device for RawDevice {
    fn size(&mut self) -> io::Result<u64> {
        match self.ioctl_size() {
            Some(size) => Ok(size),
            None => Ok(self.file.metadata()?.len()),
        }
    }

    /// Plain `fsync`, as upstream; `File::sync_all` would issue
    /// `F_FULLFSYNC`, which raw character devices can reject.
    fn force_sync(&mut self) -> io::Result<()> {
        // SAFETY: valid open descriptor.
        if unsafe { libc::fsync(self.file.as_raw_fd()) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

impl Read for RawDevice {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl Write for RawDevice {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl Seek for RawDevice {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regular_files_fall_back_to_their_length() {
        let mut file = tempfile::tempfile().unwrap();
        file.write_all(&[0u8; 4096]).unwrap();
        let mut device = RawDevice::new(file);
        assert_eq!(device.size().unwrap(), 4096);
        device.set_direct_io(true).unwrap();
        device.seek(SeekFrom::Start(0)).unwrap();
        device.write_all(&[1u8; 512]).unwrap();
        device.force_sync().unwrap();
        device.seek(SeekFrom::Start(0)).unwrap();
        let mut back = [0u8; 4];
        device.read_exact(&mut back).unwrap();
        assert_eq!(back, [1; 4]);
    }
}
