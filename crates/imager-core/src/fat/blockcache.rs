//! 4 KiB write-back block cache over a raw device (upstream
//! `DeviceWrapper`). Raw macOS devices need block-aligned I/O; the cache
//! turns byte-granular FAT access into whole-block reads and writes, and
//! `sync` writes block 0 (the MBR) last so an interrupted edit never leaves
//! a half-updated partition table.

use std::collections::{BTreeMap, btree_map};
use std::io::{self, Read, Seek, SeekFrom, Write};

/// Cache block size.
pub const BLOCK: u64 = 4096;

/// A cached block and whether it differs from the device.
struct Entry {
    data: Box<[u8; BLOCK as usize]>,
    dirty: bool,
}

/// Byte-addressable view of `D` backed by cached 4 KiB blocks.
pub struct BlockCache<D> {
    device: D,
    blocks: BTreeMap<u64, Entry>,
}

impl<D: Read + Write + Seek> BlockCache<D> {
    /// Wraps a device positioned anywhere.
    pub fn new(device: D) -> Self {
        Self {
            device,
            blocks: BTreeMap::new(),
        }
    }

    /// Reads `buf.len()` bytes at `offset`.
    ///
    /// # Errors
    /// Fails when a needed block cannot be read in full.
    pub fn pread(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        for Span {
            block,
            within,
            done,
            count,
        } in spans(offset, buf.len())
        {
            let entry = self.load(block)?;
            buf[done..done + count].copy_from_slice(&entry.data[within..within + count]);
        }
        Ok(())
    }

    /// Writes `buf` at `offset`; whole blocks are not read first.
    ///
    /// # Errors
    /// Fails when a partly overwritten block cannot be read.
    pub fn pwrite(&mut self, buf: &[u8], offset: u64) -> io::Result<()> {
        for Span {
            block,
            within,
            done,
            count,
        } in spans(offset, buf.len())
        {
            let entry = if count == BLOCK as usize {
                self.blocks.entry(block).or_insert_with(|| Entry {
                    data: Box::new([0; BLOCK as usize]),
                    dirty: false,
                })
            } else {
                self.load(block)?
            };
            entry.data[within..within + count].copy_from_slice(&buf[done..done + count]);
            entry.dirty = true;
        }
        Ok(())
    }

    /// Writes dirty blocks in ascending order, block 0 last, then flushes.
    ///
    /// # Errors
    /// Fails when the device rejects a seek, write or flush.
    pub fn sync(&mut self) -> io::Result<()> {
        let mut dirty: Vec<u64> = self
            .blocks
            .iter()
            .filter(|(_, e)| e.dirty)
            .map(|(n, _)| *n)
            .collect();
        dirty.sort_by_key(|n| *n == 0);
        for number in dirty {
            if let Some(entry) = self.blocks.get_mut(&number) {
                self.device.seek(SeekFrom::Start(number * BLOCK))?;
                self.device.write_all(&entry.data[..])?;
                entry.dirty = false;
            }
        }
        self.device.flush()
    }

    /// Whether any cached block still has to reach the device.
    #[cfg(test)]
    pub fn is_dirty(&self) -> bool {
        self.blocks.values().any(|e| e.dirty)
    }

    /// Syncs and returns the device.
    ///
    /// # Errors
    /// Fails when syncing fails.
    pub fn into_inner(mut self) -> io::Result<D> {
        self.sync()?;
        Ok(self.device)
    }

    fn load(&mut self, block: u64) -> io::Result<&mut Entry> {
        match self.blocks.entry(block) {
            btree_map::Entry::Occupied(entry) => Ok(entry.into_mut()),
            btree_map::Entry::Vacant(slot) => {
                let mut data = Box::new([0u8; BLOCK as usize]);
                self.device.seek(SeekFrom::Start(block * BLOCK))?;
                self.device.read_exact(&mut data[..])?;
                Ok(slot.insert(Entry { data, dirty: false }))
            }
        }
    }
}

/// The part of one cache block covered by a byte range.
struct Span {
    block: u64,
    within: usize,
    done: usize,
    count: usize,
}

/// Splits `len` bytes at `offset` into per-block spans.
fn spans(offset: u64, len: usize) -> impl Iterator<Item = Span> {
    let mut done = 0;
    std::iter::from_fn(move || {
        (done < len).then(|| {
            let at = offset + done as u64;
            let within = (at % BLOCK) as usize;
            let span = Span {
                block: at / BLOCK,
                within,
                done,
                count: (BLOCK as usize - within).min(len - done),
            };
            done += span.count;
            span
        })
    })
}
