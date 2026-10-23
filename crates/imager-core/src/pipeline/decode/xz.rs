//! Threaded xz decoding across concatenated streams.

use std::io::{self, BufRead, Read};

use liblzma::bufread::XzDecoder;
use liblzma::stream::MtStreamBuilder;

/// Multithreaded xz decoding (upstream sets libarchive's xz `threads`
/// option) across concatenated streams: liblzma's threaded decoder stops
/// at each stream end, so a new one is started while input remains.
pub(super) struct MultiStreamXz<R: BufRead> {
    current: Option<XzDecoder<R>>,
}

impl<R: BufRead> MultiStreamXz<R> {
    pub(super) fn new(reader: R) -> Self {
        Self {
            current: Some(Self::stream(reader)),
        }
    }

    fn stream(reader: R) -> XzDecoder<R> {
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get()) as u32;
        match MtStreamBuilder::new()
            .threads(threads)
            .memlimit_stop(u64::MAX)
            .decoder()
        {
            Ok(stream) => XzDecoder::new_stream(reader, stream),
            Err(_) => XzDecoder::new(reader),
        }
    }
}

impl<R: BufRead> Read for MultiStreamXz<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let Some(decoder) = self.current.as_mut() else {
                return Ok(0);
            };
            let n = decoder.read(buf)?;
            if n > 0 || buf.is_empty() {
                return Ok(n);
            }
            let Some(mut reader) = self.current.take().map(XzDecoder::into_inner) else {
                return Ok(0);
            };
            if skip_stream_padding(&mut reader)? {
                self.current = Some(Self::stream(reader));
            }
        }
    }
}

/// Skips the zero padding allowed between xz streams; true if another
/// stream follows.
fn skip_stream_padding<R: BufRead>(reader: &mut R) -> io::Result<bool> {
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Ok(false);
        }
        let zeros = available.iter().take_while(|b| **b == 0).count();
        if zeros < available.len() {
            reader.consume(zeros);
            return Ok(true);
        }
        let len = available.len();
        reader.consume(len);
    }
}
