//! Input side of the sparse encoder: block classification, run coalescing,
//! segment splitting, partial-block buffering (upstream `processBlock`,
//! `feed`, `finish`).

use super::super::sparse::{
    BLK, CHUNK_HDR, CHUNK_TYPE_DONT_CARE, CHUNK_TYPE_FILL, CHUNK_TYPE_RAW, is_block_fill,
    payload_size,
};
use super::{DONT_CARE, FILL, RAW, SparseEncoder};

impl SparseEncoder {
    /// Unmapped (with a block map) → DONT_CARE, uniform → FILL, else RAW.
    fn classify(&mut self, block: &[u8]) -> (u16, u32) {
        let index = self.processed_blocks;
        if self
            .block_map
            .as_mut()
            .is_some_and(|m| !m.is_mapped_sequential(index))
        {
            self.stats[DONT_CARE] += 1;
            (CHUNK_TYPE_DONT_CARE, 0)
        } else if let Some(value) = is_block_fill(block) {
            self.stats[FILL] += 1;
            (CHUNK_TYPE_FILL, value)
        } else {
            self.stats[RAW] += 1;
            (CHUNK_TYPE_RAW, 0)
        }
    }

    fn process_block(&mut self, block: &[u8]) {
        let (kind, fill_value) = self.classify(block);
        let mut continues = self.run_blocks > 0 && kind == self.run_type;
        if continues && kind == CHUNK_TYPE_FILL && fill_value != self.run_fill_value {
            continues = false;
        }
        // Reserve room for the trailing DONT_CARE finalise appends.
        let trailer = if self.total_image_blocks > 0 {
            CHUNK_HDR
        } else {
            0
        };
        let cost = if continues {
            if kind == CHUNK_TYPE_RAW { BLK } else { 0 }
        } else {
            CHUNK_HDR + payload_size(kind)
        };
        if self.out.len() + cost + trailer > self.max_segment_size {
            self.finalise_segment();
            continues = false;
        }
        // Counted after a split, against the segment the block lands in.
        self.seg[match kind {
            CHUNK_TYPE_DONT_CARE => DONT_CARE,
            CHUNK_TYPE_FILL => FILL,
            _ => RAW,
        }] += 1;
        if !continues {
            self.flush_run();
            self.run_type = kind;
            self.run_blocks = 0;
            self.run_fill_value = fill_value;
            if kind == CHUNK_TYPE_RAW {
                self.out.resize(self.out.len() + CHUNK_HDR, 0);
                self.run_raw_start = self.out.len();
            }
        }
        if kind == CHUNK_TYPE_RAW {
            self.out.extend_from_slice(block);
        }
        self.run_blocks += 1;
        self.processed_blocks += 1;
    }

    /// Classifies the buffered partial block once it is full.
    fn process_partial(&mut self) {
        let block = std::mem::take(&mut self.partial);
        self.process_block(&block);
        self.partial = block;
        self.partial.clear();
    }

    /// Feeds raw image bytes; returns how many were consumed. Fewer than
    /// `data.len()` means a segment is ready: take it, then feed the rest.
    pub fn feed(&mut self, data: &[u8]) -> usize {
        self.ready.clear(); // the caller consumed the previous segment
        let mut src = 0;
        if !self.partial.is_empty() {
            let take = (BLK - self.partial.len()).min(data.len());
            self.partial.extend_from_slice(&data[..take]);
            src = take;
            if self.partial.len() < BLK {
                return data.len();
            }
            self.process_partial();
            if !self.ready.is_empty() {
                return src;
            }
        }
        while data.len() - src >= BLK {
            self.process_block(&data[src..src + BLK]);
            src += BLK;
            if !self.ready.is_empty() {
                return src;
            }
        }
        self.partial.extend_from_slice(&data[src..]);
        data.len()
    }

    /// Ends the input: zero-pads a partial block and finalises the last
    /// segment. Call again while [`Self::take_segment`] returns one.
    pub fn finish(&mut self) {
        self.ready.clear();
        if !self.partial.is_empty() {
            self.partial.resize(BLK, 0);
            self.process_partial();
            if !self.ready.is_empty() {
                return;
            }
        }
        self.finalise_segment();
    }
}
