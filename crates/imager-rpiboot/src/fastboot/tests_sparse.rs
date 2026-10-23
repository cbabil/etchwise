//! Upstream `sparse_encoder_test.cpp`: decoding helpers and the
//! single-segment cases. Upstream seeds `std::mt19937`; a small xorshift
//! generator stands in (any non-uniform data exercises the same paths).

use super::SparseEncoder;
use super::sparse::{
    BLK, CHUNK_HDR, CHUNK_TYPE_DONT_CARE, CHUNK_TYPE_FILL, CHUNK_TYPE_RAW, FILE_HDR, SPARSE_BLK_SZ,
    SPARSE_CHUNK_HDR_SZ, SPARSE_FILE_HDR_SZ, SPARSE_MAGIC, SPARSE_MAJOR_VER, SparseChunkHeader,
    SparseFileHeader, is_block_fill, is_block_zero,
};

/// Bytes in `1..=255` (never all-zero or uniform blocks).
pub fn random_bytes(len: usize, seed: u64) -> Vec<u8> {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state % 255) as u8 + 1
        })
        .collect()
}

/// Overlays one segment onto `out` (RAW/FILL write, DONT_CARE skips).
pub fn apply_sparse(sparse: &[u8], out: &mut [u8]) {
    let fhdr = SparseFileHeader::parse(sparse).unwrap();
    assert_eq!(fhdr.magic, SPARSE_MAGIC);
    assert_eq!(fhdr.major_version, SPARSE_MAJOR_VER);
    assert_eq!(u32::from(fhdr.file_hdr_sz), SPARSE_FILE_HDR_SZ);
    assert_eq!(u32::from(fhdr.chunk_hdr_sz), SPARSE_CHUNK_HDR_SZ);
    assert_eq!(fhdr.blk_sz, SPARSE_BLK_SZ);
    assert!(out.len() >= fhdr.total_blks as usize * BLK);
    let (mut pos, mut off) = (FILE_HDR, 0);
    for _ in 0..fhdr.total_chunks {
        let chdr = SparseChunkHeader::parse(&sparse[pos..]).unwrap();
        pos += CHUNK_HDR;
        let bytes = chdr.chunk_sz as usize * BLK;
        match chdr.chunk_type {
            CHUNK_TYPE_RAW => {
                out[off..off + bytes].copy_from_slice(&sparse[pos..pos + bytes]);
                pos += bytes;
            }
            CHUNK_TYPE_FILL => {
                let fill = &sparse[pos..pos + 4];
                pos += 4;
                for word in out[off..off + bytes].as_chunks_mut::<4>().0 {
                    word.copy_from_slice(fill);
                }
            }
            CHUNK_TYPE_DONT_CARE => {}
            other => panic!("Unknown chunk type: {other:#x}"),
        }
        off += bytes;
    }
    assert_eq!(off, fhdr.total_blks as usize * BLK);
    assert_eq!(pos, sparse.len());
}

/// Decodes one segment onto zeros.
pub fn decode_sparse(sparse: &[u8]) -> Vec<u8> {
    let total = SparseFileHeader::parse(sparse).unwrap().total_blks as usize;
    let mut out = vec![0; total * BLK];
    apply_sparse(sparse, &mut out);
    out
}

/// Overlays self-positioning segments onto one image.
pub fn decode_segments(segments: &[Vec<u8>]) -> Vec<u8> {
    let Some(first) = segments.first() else {
        return Vec::new();
    };
    let total = SparseFileHeader::parse(first).unwrap().total_blks as usize;
    let mut out = vec![0; total * BLK];
    for seg in segments {
        apply_sparse(seg, &mut out);
    }
    out
}

/// Feeds `data` in `step`-byte pieces, collecting every segment.
pub fn feed_and_collect(enc: &mut SparseEncoder, data: &[u8], step: usize) -> Vec<Vec<u8>> {
    let mut segments = Vec::new();
    let mut off = 0;
    while off < data.len() {
        let end = (off + step).min(data.len());
        off += enc.feed(&data[off..end]);
        if let Some(seg) = enc.take_segment() {
            segments.push(seg.data.to_vec());
        }
    }
    loop {
        enc.finish();
        let Some(seg) = enc.take_segment() else {
            break;
        };
        segments.push(seg.data.to_vec());
    }
    segments
}

const MIB: u32 = 1024 * 1024;
const ONE_FILL: usize = FILE_HDR + CHUNK_HDR + 4;

#[test]
fn all_zero_image_produces_fill_zero_chunks() {
    let image = vec![0u8; 64 * BLK];
    let mut enc = SparseEncoder::new(MIB, image.len() as u64);
    let segments = feed_and_collect(&mut enc, &image, 65536);
    assert_eq!(segments.len(), 1);
    assert_eq!(decode_sparse(&segments[0]), image);
    assert_eq!(enc.fill_block_count(), 64);
    assert_eq!((enc.raw_block_count(), enc.dont_care_block_count()), (0, 0));
    assert_eq!(segments[0].len(), ONE_FILL);
}

#[test]
fn fill_pattern_image_produces_fill_chunks() {
    let image: Vec<u8> = (0..16 * BLK / 4)
        .flat_map(|_| 0xDEAD_BEEFu32.to_le_bytes())
        .collect();
    let mut enc = SparseEncoder::new(MIB, image.len() as u64);
    let segments = feed_and_collect(&mut enc, &image, 65536);
    assert_eq!(segments.len(), 1);
    assert_eq!(decode_sparse(&segments[0]), image);
    assert_eq!(enc.fill_block_count(), 16);
    assert_eq!((enc.raw_block_count(), enc.dont_care_block_count()), (0, 0));
    assert_eq!(segments[0].len(), ONE_FILL);
}

#[test]
fn random_data_produces_raw_chunks() {
    let image = random_bytes(8 * BLK, 42);
    let mut enc = SparseEncoder::new(MIB, image.len() as u64);
    let segments = feed_and_collect(&mut enc, &image, 65536);
    assert_eq!(segments.len(), 1);
    assert_eq!(decode_sparse(&segments[0]), image);
    assert_eq!(enc.raw_block_count(), 8);
}

#[test]
fn mixed_image_raw_zero_fill_raw() {
    let mut image = vec![0u8; 4 * BLK];
    image[..BLK].copy_from_slice(&random_bytes(BLK, 123));
    for word in image[2 * BLK..3 * BLK].as_chunks_mut::<4>().0 {
        word.copy_from_slice(&0xAAAA_AAAAu32.to_le_bytes());
    }
    image[3 * BLK..].copy_from_slice(&random_bytes(BLK, 124));
    let mut enc = SparseEncoder::new(MIB, image.len() as u64);
    let segments = feed_and_collect(&mut enc, &image, 65536);
    assert_eq!(segments.len(), 1);
    assert_eq!(decode_sparse(&segments[0]), image);
    assert_eq!(enc.raw_block_count(), 2);
    assert_eq!(enc.fill_block_count(), 2);
    assert_eq!(enc.dont_care_block_count(), 0);
}

#[test]
fn is_block_zero_identifies_zero_blocks() {
    let mut block = vec![0u8; BLK];
    assert!(is_block_zero(&block));
    block[0] = 1;
    assert!(!is_block_zero(&block));
    block[0] = 0;
    block[BLK - 1] = 1;
    assert!(!is_block_zero(&block));
}

#[test]
fn is_block_fill_identifies_fill_blocks() {
    let pattern = 0x1234_5678u32;
    let mut block: Vec<u8> = (0..BLK / 4).flat_map(|_| pattern.to_le_bytes()).collect();
    assert_eq!(is_block_fill(&block), Some(pattern));
    block[BLK - 1] ^= 0xFF;
    assert_eq!(is_block_fill(&block), None);
}
