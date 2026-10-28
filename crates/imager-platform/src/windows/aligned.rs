//! Whole-sector I/O over a raw disk. Windows accepts only sector-aligned
//! offsets and lengths on `\\.\PhysicalDriveN`, while the customisation
//! step writes small files through a FAT driver; unaligned reads and
//! writes become read-modify-write of the sectors they touch (sectors past
//! the end read as zeros). Pure,
//! tested on every host.

use std::io::{self, Read, Seek, SeekFrom, Write};

use imager_core::pipeline::Device;

/// Wraps a device whose I/O must be whole sectors.
pub struct SectorAligned<D> {
    inner: D,
    sector: u64,
    pos: u64,
}

impl<D: Device> SectorAligned<D> {
    /// `sector` is the disk's sector size (512 or 4096); 0 means 512.
    pub fn new(inner: D, sector: u32) -> Self {
        let sector = if sector == 0 { 512 } else { u64::from(sector) };
        Self {
            inner,
            sector,
            pos: 0,
        }
    }

    pub fn get_mut(&mut self) -> &mut D {
        &mut self.inner
    }

    fn aligned(&self, len: usize) -> bool {
        self.pos.is_multiple_of(self.sector) && (len as u64).is_multiple_of(self.sector)
    }

    /// The sectors covering `[pos, pos + len)`, read whole.
    fn read_span(&mut self, len: usize) -> io::Result<(u64, Vec<u8>)> {
        let start = self.pos - self.pos % self.sector;
        let end = (self.pos + len as u64).div_ceil(self.sector) * self.sector;
        let mut span = vec![0u8; usize::try_from(end - start).map_err(io::Error::other)?];
        self.inner.seek(SeekFrom::Start(start))?;
        let mut filled = 0;
        while filled < span.len() {
            match self.inner.read(&mut span[filled..])? {
                0 => break,
                n => filled += n,
            }
        }
        span.truncate(filled);
        Ok((start, span))
    }
}

impl<D: Device> Read for SectorAligned<D> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.aligned(buf.len()) {
            self.inner.seek(SeekFrom::Start(self.pos))?;
            let n = self.inner.read(buf)?;
            self.pos += n as u64;
            return Ok(n);
        }
        let (start, span) = self.read_span(buf.len())?;
        let from = usize::try_from(self.pos - start).map_err(io::Error::other)?;
        if from >= span.len() {
            return Ok(0);
        }
        let n = (span.len() - from).min(buf.len());
        buf[..n].copy_from_slice(&span[from..from + n]);
        self.pos += n as u64;
        Ok(n)
    }
}

impl<D: Device> Write for SectorAligned<D> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.aligned(buf.len()) {
            self.inner.seek(SeekFrom::Start(self.pos))?;
            let n = self.inner.write(buf)?;
            self.pos += n as u64;
            return Ok(n);
        }
        let (start, mut span) = self.read_span(buf.len())?;
        let from = usize::try_from(self.pos - start).map_err(io::Error::other)?;
        let whole = usize::try_from(
            (self.pos + buf.len() as u64).div_ceil(self.sector) * self.sector - start,
        )
        .map_err(io::Error::other)?;
        // Past the end: an image file grows, a disk refuses the write.
        span.resize(whole, 0);
        span[from..from + buf.len()].copy_from_slice(buf);
        self.inner.seek(SeekFrom::Start(start))?;
        self.inner.write_all(&span)?;
        self.pos += buf.len() as u64;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<D: Device> Seek for SectorAligned<D> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let target = match pos {
            SeekFrom::Start(p) => Some(p),
            SeekFrom::Current(d) => self.pos.checked_add_signed(d),
            SeekFrom::End(d) => self.inner.size()?.checked_add_signed(d),
        };
        self.pos = target
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek before the start"))?;
        Ok(self.pos)
    }
}

impl<D: Device> Device for SectorAligned<D> {
    fn size(&mut self) -> io::Result<u64> {
        self.inner.size()
    }

    fn force_sync(&mut self) -> io::Result<()> {
        self.inner.force_sync()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    /// A disk that refuses anything but whole 512-byte sectors.
    struct StrictDisk(Cursor<Vec<u8>>);

    impl StrictDisk {
        fn check(&self, len: usize) -> io::Result<()> {
            if !self.0.position().is_multiple_of(512) || !len.is_multiple_of(512) {
                return Err(io::Error::other(format!(
                    "unaligned: {} + {len}",
                    self.0.position()
                )));
            }
            Ok(())
        }
    }

    impl Read for StrictDisk {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.check(buf.len())?;
            self.0.read(buf)
        }
    }

    impl Write for StrictDisk {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.check(buf.len())?;
            self.0.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Seek for StrictDisk {
        fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
            self.0.seek(pos)
        }
    }

    impl Device for StrictDisk {
        fn size(&mut self) -> io::Result<u64> {
            Ok(self.0.get_ref().len() as u64)
        }

        fn force_sync(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn disk() -> SectorAligned<StrictDisk> {
        SectorAligned::new(StrictDisk(Cursor::new(vec![0xAA; 4096])), 512)
    }

    #[test]
    fn unaligned_writes_change_only_their_bytes() {
        let mut d = disk();
        d.seek(SeekFrom::Start(510)).unwrap();
        d.write_all(b"hello").unwrap();
        assert_eq!(d.stream_position().unwrap(), 515);
        let data = d.get_mut().0.get_ref().clone();
        assert_eq!(&data[508..517], b"\xAA\xAAhello\xAA\xAA");
        assert!(data[..508].iter().chain(&data[517..]).all(|b| *b == 0xAA));
    }

    #[test]
    fn unaligned_reads_return_the_right_bytes() {
        let mut d = disk();
        d.get_mut().0.get_mut()[1000..1003].copy_from_slice(b"abc");
        d.seek(SeekFrom::Start(999)).unwrap();
        let mut buf = [0u8; 5];
        d.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"\xAAabc\xAA");
        d.seek(SeekFrom::End(-2)).unwrap();
        let mut tail = [0u8; 8];
        assert_eq!(d.read(&mut tail).unwrap(), 2);
        assert_eq!(d.read(&mut tail).unwrap(), 0);
    }

    #[test]
    fn aligned_io_goes_straight_through() {
        let mut d = disk();
        d.seek(SeekFrom::Start(1024)).unwrap();
        d.write_all(&[1u8; 1024]).unwrap();
        d.seek(SeekFrom::Current(-1024)).unwrap();
        let mut back = [0u8; 512];
        d.read_exact(&mut back).unwrap();
        assert_eq!(back, [1u8; 512]);
        assert_eq!(d.size().unwrap(), 4096);
        d.force_sync().unwrap();
        d.flush().unwrap();
    }

    #[test]
    fn stays_inside_the_start_and_reads_nothing_past_the_end() {
        let mut d = disk();
        assert!(d.seek(SeekFrom::Current(-1)).is_err());
        d.seek(SeekFrom::Start(5000)).unwrap();
        let mut buf = [0u8; 3];
        assert_eq!(d.read(&mut buf).unwrap(), 0);
        d.seek(SeekFrom::Start(4094)).unwrap();
        d.write_all(b"xyz").unwrap();
        let data = d.get_mut().0.get_ref();
        assert_eq!(data.len(), 4608);
        assert_eq!(&data[4094..4097], b"xyz");
        assert_eq!(data[4097], 0);
        assert_eq!(
            SectorAligned::new(StrictDisk(Cursor::default()), 0).sector,
            512
        );
    }
}
