//! The Android sparse image wire format and block classification
//! (upstream `sparse_encoder.h`). All header fields are little-endian.

/// File header magic.
pub const SPARSE_MAGIC: u32 = 0xED26_FF3A;
/// Format version written.
pub const SPARSE_MAJOR_VER: u16 = 1;
pub const SPARSE_MINOR_VER: u16 = 0;
/// Size of [`SparseFileHeader`] on the wire.
pub const SPARSE_FILE_HDR_SZ: u32 = 28;
/// Size of [`SparseChunkHeader`] on the wire.
pub const SPARSE_CHUNK_HDR_SZ: u32 = 12;
/// Block size used for every image.
pub const SPARSE_BLK_SZ: u32 = 4096;

/// Chunk carrying `chunk_sz` blocks of data.
pub const CHUNK_TYPE_RAW: u16 = 0xCAC1;
/// Chunk repeating one 32-bit value.
pub const CHUNK_TYPE_FILL: u16 = 0xCAC2;
/// Chunk skipping blocks (nothing written).
pub const CHUNK_TYPE_DONT_CARE: u16 = 0xCAC3;

pub(crate) const BLK: usize = SPARSE_BLK_SZ as usize;
pub(crate) const FILE_HDR: usize = SPARSE_FILE_HDR_SZ as usize;
pub(crate) const CHUNK_HDR: usize = SPARSE_CHUNK_HDR_SZ as usize;

/// The 28-byte sparse file header.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SparseFileHeader {
    pub magic: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub file_hdr_sz: u16,
    pub chunk_hdr_sz: u16,
    pub blk_sz: u32,
    pub total_blks: u32,
    pub total_chunks: u32,
    pub image_checksum: u32,
}

impl SparseFileHeader {
    /// Wire encoding.
    pub fn to_bytes(&self) -> [u8; FILE_HDR] {
        let mut b = [0u8; FILE_HDR];
        b[0..4].copy_from_slice(&self.magic.to_le_bytes());
        b[4..6].copy_from_slice(&self.major_version.to_le_bytes());
        b[6..8].copy_from_slice(&self.minor_version.to_le_bytes());
        b[8..10].copy_from_slice(&self.file_hdr_sz.to_le_bytes());
        b[10..12].copy_from_slice(&self.chunk_hdr_sz.to_le_bytes());
        b[12..16].copy_from_slice(&self.blk_sz.to_le_bytes());
        b[16..20].copy_from_slice(&self.total_blks.to_le_bytes());
        b[20..24].copy_from_slice(&self.total_chunks.to_le_bytes());
        b[24..28].copy_from_slice(&self.image_checksum.to_le_bytes());
        b
    }

    /// Decodes the first 28 bytes; `None` when shorter.
    pub fn parse(b: &[u8]) -> Option<Self> {
        let b = b.get(..FILE_HDR)?;
        Some(Self {
            magic: le32(&b[0..4]),
            major_version: le16(&b[4..6]),
            minor_version: le16(&b[6..8]),
            file_hdr_sz: le16(&b[8..10]),
            chunk_hdr_sz: le16(&b[10..12]),
            blk_sz: le32(&b[12..16]),
            total_blks: le32(&b[16..20]),
            total_chunks: le32(&b[20..24]),
            image_checksum: le32(&b[24..28]),
        })
    }
}

/// The 12-byte chunk header: `chunk_sz` in blocks, `total_sz` in bytes
/// including this header.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SparseChunkHeader {
    pub chunk_type: u16,
    pub reserved1: u16,
    pub chunk_sz: u32,
    pub total_sz: u32,
}

impl SparseChunkHeader {
    /// Wire encoding.
    pub fn to_bytes(&self) -> [u8; CHUNK_HDR] {
        let mut b = [0u8; CHUNK_HDR];
        b[0..2].copy_from_slice(&self.chunk_type.to_le_bytes());
        b[2..4].copy_from_slice(&self.reserved1.to_le_bytes());
        b[4..8].copy_from_slice(&self.chunk_sz.to_le_bytes());
        b[8..12].copy_from_slice(&self.total_sz.to_le_bytes());
        b
    }

    /// Decodes the first 12 bytes; `None` when shorter.
    pub fn parse(b: &[u8]) -> Option<Self> {
        let b = b.get(..CHUNK_HDR)?;
        Some(Self {
            chunk_type: le16(&b[0..2]),
            reserved1: le16(&b[2..4]),
            chunk_sz: le32(&b[4..8]),
            total_sz: le32(&b[8..12]),
        })
    }
}

fn le16(b: &[u8]) -> u16 {
    u16::from_le_bytes([b[0], b[1]])
}

fn le32(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

const ZERO_BLOCK: [u8; BLK] = [0; BLK];

/// True when all bytes of the block are zero (`isBlockZero`).
pub fn is_block_zero(block: &[u8]) -> bool {
    block == ZERO_BLOCK.as_slice()
}

/// The repeated 32-bit value when the block is one value 1024 times
/// (`isBlockFill`); read as little-endian, so it round-trips the bytes.
pub fn is_block_fill(block: &[u8]) -> Option<u32> {
    if block.len() != BLK {
        return None;
    }
    // Period-4 check: every byte equals the one four bytes later.
    (block[..BLK - 4] == block[4..]).then(|| le32(&block[..4]))
}

/// Per-segment statistics (`SegmentStats`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SegmentStats {
    /// Blocks covered, including the leading/trailing DONT_CARE padding.
    pub blocks: u32,
    pub chunks: u32,
    pub raw_blocks: u32,
    pub fill_blocks: u32,
    pub dont_care_blocks: u32,
    /// Segment size on the wire.
    pub wire_bytes: usize,
}

/// A completed segment from [`super::SparseEncoder::take_segment`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment<'a> {
    pub data: &'a [u8],
    pub stats: SegmentStats,
}

/// Appends a chunk header.
pub(crate) fn push_chunk(out: &mut Vec<u8>, chunk_type: u16, blocks: u32, total_sz: u32) {
    let hdr = SparseChunkHeader {
        chunk_type,
        reserved1: 0,
        chunk_sz: blocks,
        total_sz,
    };
    out.extend_from_slice(&hdr.to_bytes());
}

/// Bytes a chunk of `chunk_type` carries after its header.
pub(crate) fn payload_size(chunk_type: u16) -> usize {
    match chunk_type {
        CHUNK_TYPE_RAW => BLK,
        CHUNK_TYPE_FILL => 4,
        _ => 0,
    }
}
