//! The write target: a raw device on macOS, a regular file or memory in
//! tests. Upstream `FileOperations` (Seek, WriteSequential, ForceSync,
//! GetSize).

use std::fs::File;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

/// A seekable byte target that can report its size and force a sync.
pub trait Device: Read + Write + Seek {
    /// Capacity in bytes.
    ///
    /// # Errors
    /// Fails when the size cannot be determined.
    fn size(&mut self) -> io::Result<u64>;

    /// Flushes and forces data to stable storage (`ForceSync`).
    ///
    /// # Errors
    /// Fails when the device rejects the sync.
    fn force_sync(&mut self) -> io::Result<()>;
}

impl Device for File {
    fn size(&mut self) -> io::Result<u64> {
        let here = self.stream_position()?;
        let end = self.seek(SeekFrom::End(0))?;
        self.seek(SeekFrom::Start(here))?;
        Ok(end)
    }

    fn force_sync(&mut self) -> io::Result<()> {
        self.flush()?;
        self.sync_all()
    }
}

impl Device for Cursor<Vec<u8>> {
    fn size(&mut self) -> io::Result<u64> {
        Ok(self.get_ref().len() as u64)
    }

    fn force_sync(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<D: Device + ?Sized> Device for &mut D {
    fn size(&mut self) -> io::Result<u64> {
        (**self).size()
    }

    fn force_sync(&mut self) -> io::Result<()> {
        (**self).force_sync()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_size_keeps_position() {
        let mut file = tempfile::tempfile().unwrap();
        file.write_all(&[1; 100]).unwrap();
        file.seek(SeekFrom::Start(10)).unwrap();
        assert_eq!(Device::size(&mut file).unwrap(), 100);
        assert_eq!(file.stream_position().unwrap(), 10);
        file.force_sync().unwrap();
        let mut cursor = Cursor::new(vec![0; 8]);
        let mut by_ref = &mut cursor;
        assert_eq!(Device::size(&mut by_ref).unwrap(), 8);
        by_ref.force_sync().unwrap();
    }
}
