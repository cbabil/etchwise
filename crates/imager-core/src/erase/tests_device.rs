//! Device-level behaviour: progress, errors, cancellation, tiny devices.

use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

use super::tests::MIB;
use super::{EraseError, format_device};
use crate::pipeline::{Device, Progress};

#[test]
fn in_memory_device_and_progress() {
    let mut dev = Cursor::new(vec![0xEE; 8 * 1024 * 1024]);
    let progress = Progress::default();
    format_device(&mut dev, &progress).unwrap();
    let snap = progress.snapshot();
    assert_eq!(snap.bytes_written, snap.write_total);
    let spf = super::layout::Fat32Layout::new(8192)
        .unwrap()
        .sectors_per_fat;
    assert_eq!(snap.write_total, (4 + 2 * u64::from(spf) + 1) * 512);
    assert_eq!(&dev.get_ref()[510..512], &[0x55, 0xAA]);
}

#[test]
fn too_small_device_writes_nothing() {
    let mut dev = Cursor::new(vec![0xEE; 4 * 1024 * 1024]);
    let err = format_device(&mut dev, &Progress::default()).unwrap_err();
    assert!(matches!(err, EraseError::InsufficientSpace { size } if size == 4 * MIB));
    assert_eq!(err.to_string(), "Insufficient space on device");
    assert!(dev.get_ref().iter().all(|&b| b == 0xEE));
}

#[test]
fn cancelled_before_start_writes_nothing() {
    let mut dev = Cursor::new(vec![0xEE; 8 * 1024 * 1024]);
    let progress = Progress::default();
    progress.cancel();
    let err = format_device(&mut dev, &progress).unwrap_err();
    assert!(matches!(err, EraseError::Cancelled));
    assert!(dev.get_ref().iter().all(|&b| b == 0xEE));
}

/// A device whose size and writes can be made to fail.
struct Broken {
    inner: Cursor<Vec<u8>>,
    fail_size: bool,
}

impl Read for Broken {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Write for Broken {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("write-protected"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Seek for Broken {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        self.inner.seek(to)
    }
}

impl Device for Broken {
    fn size(&mut self) -> io::Result<u64> {
        if self.fail_size {
            return Err(io::Error::other("no size"));
        }
        Ok(self.inner.get_ref().len() as u64)
    }

    fn force_sync(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn errors_carry_upstream_messages() {
    let inner = Cursor::new(vec![0; 8 * 1024 * 1024]);
    let mut dev = Broken {
        inner,
        fail_size: false,
    };
    let err = format_device(&mut dev, &Progress::default()).unwrap_err();
    assert_eq!(err.to_string(), "Error writing to device during formatting");
    dev.fail_size = true;
    let err = format_device(&mut dev, &Progress::default()).unwrap_err();
    assert_eq!(err.to_string(), "Error opening device for formatting");
    let seek = EraseError::Seek(io::Error::other("x"));
    assert_eq!(
        seek.to_string(),
        "Error seeking on device during formatting"
    );
    assert_eq!(
        EraseError::Cancelled.to_string(),
        "Unknown formatting error"
    );
}
