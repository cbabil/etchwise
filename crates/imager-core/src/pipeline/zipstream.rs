//! Streaming reader for the first file of a zip archive, enough for
//! single-image zips: deflate (self-terminating) or stored with a known
//! size, Zip64 sizes included. Directory entries are skipped.

use std::io::{self, BufRead, Read, Take};

use flate2::bufread::DeflateDecoder;

const LOCAL_HEADER: u32 = 0x0403_4b50;

/// Decompressed bytes of the archive's first file.
pub enum ZipEntryReader<R: BufRead> {
    Stored(Take<R>),
    Deflated(DeflateDecoder<R>),
}

impl<R: BufRead> ZipEntryReader<R> {
    /// Parses local headers up to the first non-directory entry.
    ///
    /// # Errors
    /// Fails on a malformed header or an unsupported compression method.
    pub fn new(mut reader: R) -> io::Result<Self> {
        loop {
            let header = Header::read(&mut reader)?;
            if header.is_directory() {
                io::copy(&mut (&mut reader).take(header.compressed), &mut io::sink())?;
                continue;
            }
            return match header.method {
                8 => Ok(Self::Deflated(DeflateDecoder::new(reader))),
                0 if !header.sizes_deferred() => Ok(Self::Stored(reader.take(header.compressed))),
                method => Err(invalid(&format!("unsupported zip entry (method {method})"))),
            };
        }
    }
}

impl<R: BufRead> Read for ZipEntryReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stored(r) => r.read(buf),
            Self::Deflated(r) => r.read(buf),
        }
    }
}

struct Header {
    flags: u16,
    method: u16,
    compressed: u64,
    name: Vec<u8>,
}

impl Header {
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut fixed = [0u8; 30];
        reader.read_exact(&mut fixed)?;
        let u16_at = |i: usize| u16::from_le_bytes([fixed[i], fixed[i + 1]]);
        let u32_at =
            |i: usize| u32::from_le_bytes([fixed[i], fixed[i + 1], fixed[i + 2], fixed[i + 3]]);
        if u32_at(0) != LOCAL_HEADER {
            return Err(invalid("not a zip local file header"));
        }
        let mut name = vec![0u8; usize::from(u16_at(26))];
        reader.read_exact(&mut name)?;
        let mut extra = vec![0u8; usize::from(u16_at(28))];
        reader.read_exact(&mut extra)?;
        let mut compressed = u64::from(u32_at(18));
        if compressed == u64::from(u32::MAX) {
            compressed =
                zip64_compressed_size(&extra).ok_or_else(|| invalid("missing Zip64 size"))?;
        }
        Ok(Self {
            flags: u16_at(6),
            method: u16_at(8),
            compressed,
            name,
        })
    }

    fn is_directory(&self) -> bool {
        self.name.ends_with(b"/")
    }

    /// Bit 3: sizes follow the data in a descriptor.
    fn sizes_deferred(&self) -> bool {
        self.flags & 0x0008 != 0
    }
}

/// The compressed size from a Zip64 extra field (`0x0001`: uncompressed
/// then compressed, both u64).
fn zip64_compressed_size(mut extra: &[u8]) -> Option<u64> {
    while extra.len() >= 4 {
        let (id, len) = (
            u16::from_le_bytes([extra[0], extra[1]]),
            usize::from(u16::from_le_bytes([extra[2], extra[3]])),
        );
        let body = extra.get(4..4 + len)?;
        if id == 1 && body.len() >= 16 {
            return body[8..16].try_into().ok().map(u64::from_le_bytes);
        }
        extra = &extra[4 + len..];
    }
    None
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_owned())
}

#[cfg(test)]
mod tests;
