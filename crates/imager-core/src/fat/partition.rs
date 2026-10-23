//! Partition lookup (MBR or GPT, upstream `DeviceWrapper::fatPartition`)
//! and a bounded window onto one partition (`DeviceWrapperPartition`).

use std::io::{self, Read, Seek, SeekFrom, Write};

use super::blockcache::BlockCache;

/// Byte range of a partition on the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extent {
    pub start: u64,
    pub len: u64,
}

fn u32_at(b: &[u8], at: usize) -> u64 {
    u64::from(u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]]))
}

fn u64_at(b: &[u8], at: usize) -> u64 {
    let mut bytes = [0; 8];
    bytes.copy_from_slice(&b[at..at + 8]);
    u64::from_le_bytes(bytes)
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_owned())
}

/// Finds primary partition `nr` (1-4), preferring a GPT whose header sits
/// at LBA 1.
///
/// # Errors
/// Fails for numbers outside 1-4, a bad MBR signature or an empty slot.
pub fn locate<D: Read + Write + Seek>(cache: &mut BlockCache<D>, nr: u64) -> io::Result<Extent> {
    if !(1..=4).contains(&nr) {
        return Err(invalid("Only basic partitions 1-4 supported"));
    }
    let mut gpt = [0u8; 92];
    cache.pread(&mut gpt, 512)?;
    if &gpt[0..8] == b"EFI PART" && u64_at(&gpt, 24) == 1 {
        if nr > u32_at(&gpt, 80) {
            return Err(invalid("Partition does not exist"));
        }
        let mut entry = [0u8; 48];
        cache.pread(
            &mut entry,
            u64_at(&gpt, 72) * 512 + u32_at(&gpt, 84) * (nr - 1),
        )?;
        let (first, last) = (u64_at(&entry, 32), u64_at(&entry, 40));
        return Ok(Extent {
            start: first * 512,
            len: (last.wrapping_sub(first).wrapping_add(1)) * 512,
        });
    }
    let mut mbr = [0u8; 512];
    cache.pread(&mut mbr, 0)?;
    if mbr[510] != 0x55 || mbr[511] != 0xAA {
        return Err(invalid("MBR does not have valid signature"));
    }
    let slot = 446 + 16 * (nr as usize - 1);
    let (start, sectors) = (u32_at(&mbr, slot + 8), u32_at(&mbr, slot + 12));
    if start == 0 || sectors == 0 {
        return Err(invalid("Partition does not exist"));
    }
    Ok(Extent {
        start: start * 512,
        len: sectors * 512,
    })
}

/// Read/Write/Seek confined to one partition.
pub struct Window<'a, D> {
    cache: &'a mut BlockCache<D>,
    extent: Extent,
    pos: u64,
}

impl<'a, D> Window<'a, D> {
    /// A window starting at the partition's first byte.
    pub fn new(cache: &'a mut BlockCache<D>, extent: Extent) -> Self {
        Self {
            cache,
            extent,
            pos: 0,
        }
    }

    fn check(&self, size: usize, what: &str) -> io::Result<u64> {
        if self.pos + size as u64 > self.extent.len {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("trying to {what} beyond partition"),
            ));
        }
        Ok(self.extent.start + self.pos)
    }
}

impl<D: Read + Write + Seek> Read for Window<'_, D> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let at = self.check(buf.len(), "read")?;
        self.cache.pread(buf, at)?;
        self.pos += buf.len() as u64;
        Ok(buf.len())
    }
}

impl<D: Read + Write + Seek> Write for Window<'_, D> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let at = self.check(buf.len(), "write")?;
        self.cache.pwrite(buf, at)?;
        self.pos += buf.len() as u64;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<D> Seek for Window<'_, D> {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        let target = match to {
            SeekFrom::Start(p) => Some(p),
            SeekFrom::Current(d) => self.pos.checked_add_signed(d),
            SeekFrom::End(d) => self.extent.len.checked_add_signed(d),
        };
        match target {
            Some(p) if p <= self.extent.len => {
                self.pos = p;
                Ok(p)
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "trying to seek beyond partition",
            )),
        }
    }
}
