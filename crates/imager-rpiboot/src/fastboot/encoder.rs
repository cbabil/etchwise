//! Streaming raw-image to sparse-segment encoder (upstream
//! `sparse_encoder.cpp`). Each segment fits the device's
//! max-download-size and covers the whole image: a leading DONT_CARE skips
//! blocks earlier segments wrote, a trailing DONT_CARE pads to the end.
//! Uniform blocks (zeros included) become FILL chunks; with a block map,
//! unmapped blocks become DONT_CARE.

use super::bmap::BlockMap;
use super::sparse::{
    BLK, CHUNK_TYPE_DONT_CARE, FILE_HDR, SPARSE_BLK_SZ, SPARSE_CHUNK_HDR_SZ, SPARSE_FILE_HDR_SZ,
    Segment, SegmentStats, push_chunk,
};

/// Smallest usable segment: file header, leading and trailing DONT_CARE,
/// one chunk header and one block.
pub const MIN_SEGMENT_SIZE: u32 =
    SPARSE_FILE_HDR_SZ + 2 * SPARSE_CHUNK_HDR_SZ + SPARSE_CHUNK_HDR_SZ + SPARSE_BLK_SZ;

/// Upstream `SparseEncoder`.
#[derive(Debug)]
pub struct SparseEncoder {
    max_segment_size: usize,
    total_image_blocks: u64,
    block_map: Option<BlockMap>,
    out: Vec<u8>,
    chunk_count: u32,
    segment_blocks: u32,
    run_type: u16,
    run_blocks: u32,
    run_fill_value: u32,
    run_raw_start: usize,
    partial: Vec<u8>,
    processed_blocks: u64,
    seg: [u32; 3],
    ready_stats: SegmentStats,
    ready: Vec<u8>,
    stats: [u64; 3],
}

/// Indices into the raw/fill/dont-care counters.
const RAW: usize = 0;
const FILL: usize = 1;
const DONT_CARE: usize = 2;

impl SparseEncoder {
    /// `max_segment_size` is the device's max-download-size;
    /// `total_image_size` the uncompressed size in bytes (0: unknown).
    ///
    /// # Panics
    /// When `max_segment_size < MIN_SEGMENT_SIZE` (upstream asserts).
    pub fn new(max_segment_size: u32, total_image_size: u64) -> Self {
        assert!(
            max_segment_size >= MIN_SEGMENT_SIZE,
            "segment size too small"
        );
        let max = max_segment_size as usize;
        let mut encoder = Self {
            max_segment_size: max,
            total_image_blocks: total_image_size.div_ceil(u64::from(SPARSE_BLK_SZ)),
            block_map: None,
            out: Vec::with_capacity(max),
            chunk_count: 0,
            segment_blocks: 0,
            run_type: 0,
            run_blocks: 0,
            run_fill_value: 0,
            run_raw_start: 0,
            partial: Vec::with_capacity(BLK),
            processed_blocks: 0,
            seg: [0; 3],
            ready_stats: SegmentStats::default(),
            ready: Vec::with_capacity(max),
            stats: [0; 3],
        };
        encoder.begin_segment();
        encoder
    }

    /// Unmapped blocks become DONT_CARE. Call before the first feed.
    pub fn set_block_map(&mut self, map: BlockMap) {
        debug_assert_eq!(self.processed_blocks, 0);
        self.block_map = Some(map);
    }

    /// Blocks sent as RAW so far.
    pub fn raw_block_count(&self) -> u64 {
        self.stats[RAW]
    }

    /// Blocks sent as FILL so far.
    pub fn fill_block_count(&self) -> u64 {
        self.stats[FILL]
    }

    /// Blocks skipped as DONT_CARE so far.
    pub fn dont_care_block_count(&self) -> u64 {
        self.stats[DONT_CARE]
    }

    /// Blocks classified so far.
    pub fn total_blocks_processed(&self) -> u64 {
        self.processed_blocks
    }

    /// The segment completed by the last feed/finish, if any.
    pub fn take_segment(&self) -> Option<Segment<'_>> {
        (!self.ready.is_empty()).then_some(Segment {
            data: &self.ready,
            stats: self.ready_stats,
        })
    }

    fn begin_segment(&mut self) {
        self.out.clear();
        self.out.resize(FILE_HDR, 0);
        self.chunk_count = 0;
        self.segment_blocks = 0;
        self.run_type = 0;
        self.run_blocks = 0;
        self.seg = [0; 3];
        if self.processed_blocks > 0 {
            let skip = self.processed_blocks as u32;
            push_chunk(
                &mut self.out,
                CHUNK_TYPE_DONT_CARE,
                skip,
                SPARSE_CHUNK_HDR_SZ,
            );
            self.segment_blocks = skip;
            self.chunk_count += 1;
        }
    }
}

/// `feed`, `finish` and block classification.
#[path = "encoder_feed.rs"]
mod feed;
/// `flush_run` and `finalise_segment`.
#[path = "encoder_segment.rs"]
mod segment;
