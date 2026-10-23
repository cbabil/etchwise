//! Output side of the sparse encoder: closing runs into chunks and
//! finishing segments with the trailing DONT_CARE and file header
//! (upstream `flushRun`, `finaliseSegment`).

use super::super::sparse::{
    CHUNK_HDR, CHUNK_TYPE_DONT_CARE, CHUNK_TYPE_FILL, CHUNK_TYPE_RAW, FILE_HDR, SPARSE_BLK_SZ,
    SPARSE_CHUNK_HDR_SZ, SPARSE_FILE_HDR_SZ, SPARSE_MAGIC, SPARSE_MAJOR_VER, SPARSE_MINOR_VER,
    SegmentStats, SparseFileHeader, push_chunk,
};
use super::{DONT_CARE, FILL, RAW, SparseEncoder};

impl SparseEncoder {
    pub(super) fn flush_run(&mut self) {
        if self.run_blocks == 0 {
            return;
        }
        let blocks = self.run_blocks;
        match self.run_type {
            CHUNK_TYPE_RAW => {
                let mut hdr = Vec::with_capacity(CHUNK_HDR);
                let total = SPARSE_CHUNK_HDR_SZ + blocks * SPARSE_BLK_SZ;
                push_chunk(&mut hdr, CHUNK_TYPE_RAW, blocks, total);
                let at = self.run_raw_start - CHUNK_HDR;
                self.out[at..self.run_raw_start].copy_from_slice(&hdr);
            }
            CHUNK_TYPE_FILL => {
                push_chunk(
                    &mut self.out,
                    CHUNK_TYPE_FILL,
                    blocks,
                    SPARSE_CHUNK_HDR_SZ + 4,
                );
                self.out
                    .extend_from_slice(&self.run_fill_value.to_le_bytes());
            }
            _ => push_chunk(
                &mut self.out,
                CHUNK_TYPE_DONT_CARE,
                blocks,
                SPARSE_CHUNK_HDR_SZ,
            ),
        }
        self.segment_blocks += blocks;
        self.chunk_count += 1;
        self.run_blocks = 0;
        self.run_type = 0;
    }

    pub(super) fn finalise_segment(&mut self) {
        self.flush_run();
        if self.seg == [0; 3] {
            return; // only the leading DONT_CARE prefix: discard
        }
        let total = self.total_image_blocks as u32;
        if self.total_image_blocks > 0 && self.segment_blocks < total {
            let tail = total - self.segment_blocks;
            push_chunk(
                &mut self.out,
                CHUNK_TYPE_DONT_CARE,
                tail,
                SPARSE_CHUNK_HDR_SZ,
            );
            self.segment_blocks += tail;
            self.chunk_count += 1;
        }
        let header = SparseFileHeader {
            magic: SPARSE_MAGIC,
            major_version: SPARSE_MAJOR_VER,
            minor_version: SPARSE_MINOR_VER,
            file_hdr_sz: SPARSE_FILE_HDR_SZ as u16,
            chunk_hdr_sz: SPARSE_CHUNK_HDR_SZ as u16,
            blk_sz: SPARSE_BLK_SZ,
            total_blks: if total > 0 {
                total
            } else {
                self.segment_blocks
            },
            total_chunks: self.chunk_count,
            image_checksum: 0,
        };
        self.out[..FILE_HDR].copy_from_slice(&header.to_bytes());
        self.ready_stats = SegmentStats {
            blocks: self.segment_blocks,
            chunks: self.chunk_count,
            raw_blocks: self.seg[RAW],
            fill_blocks: self.seg[FILL],
            dont_care_blocks: self.seg[DONT_CARE],
            wire_bytes: self.out.len(),
        };
        // `ready` was cleared by the caller's feed/finish: reuse its buffer.
        std::mem::swap(&mut self.ready, &mut self.out);
        self.begin_segment();
    }
}
