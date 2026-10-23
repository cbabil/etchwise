//! Content-based decompression, as upstream's libarchive setup (all
//! filters, raw format): xz, gzip, bzip2 and zstd streams (multi-stream
//! aware), the first file of a zip, or the bytes as-is.

use std::io::{self, BufRead, BufReader, Read};

use bzip2::bufread::MultiBzDecoder;
use flate2::bufread::MultiGzDecoder;

use self::xz::MultiStreamXz;
use super::buffers::MIB;
use super::zipstream::ZipEntryReader;

/// Container or compression detected from the first bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Format {
    Xz,
    Gzip,
    Bzip2,
    Zstd,
    Zip,
    SevenZip,
    Raw,
}

/// Classifies a stream by its magic bytes.
pub(crate) fn detect(magic: &[u8]) -> Format {
    let starts = |m: &[u8]| magic.starts_with(m);
    if starts(&[0xFD, b'7', b'z', b'X', b'Z', 0]) {
        Format::Xz
    } else if starts(&[0x1F, 0x8B]) {
        Format::Gzip
    } else if starts(b"BZh") {
        Format::Bzip2
    } else if starts(&[0x28, 0xB5, 0x2F, 0xFD]) {
        Format::Zstd
    } else if starts(b"PK\x03\x04") {
        Format::Zip
    } else if starts(&[b'7', b'z', 0xBC, 0xAF, 0x27, 0x1C]) {
        Format::SevenZip
    } else {
        Format::Raw
    }
}

/// Wraps `reader` in the decoder its content needs.
///
/// # Errors
/// Fails when the start cannot be read or the format is unsupported.
pub fn decoder<'a, R: Read + Send + 'a>(reader: R) -> io::Result<Box<dyn Read + Send + 'a>> {
    let mut buffered = BufReader::with_capacity(MIB, reader);
    Ok(match detect(buffered.fill_buf()?) {
        Format::Xz => Box::new(MultiStreamXz::new(buffered)),
        Format::Gzip => Box::new(MultiGzDecoder::new(buffered)),
        Format::Bzip2 => Box::new(MultiBzDecoder::new(buffered)),
        Format::Zstd => Box::new(zstd::stream::read::Decoder::with_buffer(buffered)?),
        Format::Zip => Box::new(ZipEntryReader::new(buffered)?),
        Format::SevenZip => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "7z archives are not supported",
            ));
        }
        Format::Raw => Box::new(buffered),
    })
}

mod xz;

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use super::*;

    fn payload() -> Vec<u8> {
        (0..200_000u32)
            .flat_map(|i| (i % 251).to_le_bytes())
            .collect()
    }

    fn roundtrip(compressed: Vec<u8>, expected: Format) {
        assert_eq!(detect(&compressed), expected);
        let mut reader = decoder(Cursor::new(compressed)).unwrap();
        let mut out = Vec::new();
        reader.read_to_end(&mut out).unwrap();
        assert_eq!(out, payload());
    }

    #[test]
    fn decodes_every_supported_format() {
        let data = payload();
        roundtrip(xz(&data), Format::Xz);
        let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        gz.write_all(&data).unwrap();
        roundtrip(gz.finish().unwrap(), Format::Gzip);
        let mut bz = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::fast());
        bz.write_all(&data).unwrap();
        roundtrip(bz.finish().unwrap(), Format::Bzip2);
        roundtrip(
            zstd::encode_all(Cursor::new(&data), 1).unwrap(),
            Format::Zstd,
        );
        roundtrip(data.clone(), Format::Raw);
    }

    fn xz(data: &[u8]) -> Vec<u8> {
        let mut enc = liblzma::write::XzEncoder::new(Vec::new(), 1);
        enc.write_all(data).unwrap();
        enc.finish().unwrap()
    }

    #[test]
    fn concatenated_xz_streams_with_padding_are_all_decoded() {
        let data = payload();
        let (a, b) = data.split_at(5000);
        let mut joined = xz(a);
        joined.extend([0u8; 8]);
        joined.extend(xz(b));
        roundtrip(joined.clone(), Format::Xz);
        joined.extend([0u8; 4]);
        roundtrip(joined, Format::Xz);
    }

    #[test]
    fn concatenated_gzip_members_are_all_decoded() {
        let data = payload();
        let (a, b) = data.split_at(1000);
        let mut joined = Vec::new();
        for part in [a, b] {
            let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
            gz.write_all(part).unwrap();
            joined.extend(gz.finish().unwrap());
        }
        roundtrip(joined, Format::Gzip);
    }

    #[test]
    fn rejects_7z_and_detects_short_input() {
        let seven = vec![b'7', b'z', 0xBC, 0xAF, 0x27, 0x1C, 0, 4];
        assert!(decoder(Cursor::new(seven)).is_err());
        assert_eq!(detect(&[]), Format::Raw);
        assert_eq!(detect(&[0x1F]), Format::Raw);
    }
}
