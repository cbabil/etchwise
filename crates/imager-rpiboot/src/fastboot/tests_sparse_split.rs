//! Upstream `sparse_encoder_test.cpp`: segment splitting, partial blocks,
//! headers, empty and large images, odd feed sizes.

use super::SparseEncoder;
use super::sparse::{
    BLK, CHUNK_HDR, FILE_HDR, SPARSE_BLK_SZ, SPARSE_MAGIC, SPARSE_MAJOR_VER, SparseFileHeader,
};
use super::tests_sparse::{decode_segments, feed_and_collect, random_bytes};

#[test]
fn segment_splitting_respects_max_download_size() {
    let max = (FILE_HDR + CHUNK_HDR + 2 * BLK) as u32;
    let image = random_bytes(8 * BLK, 99);
    let mut enc = SparseEncoder::new(max, image.len() as u64);
    let segments = feed_and_collect(&mut enc, &image, 65536);
    assert!(segments.len() >= 4);
    assert!(segments.iter().all(|s| s.len() <= max as usize));
    assert_eq!(decode_segments(&segments), image);
}

#[test]
fn partial_final_block_is_zero_padded() {
    let image = random_bytes(2 * BLK + 1000, 77);
    let mut enc = SparseEncoder::new(1024 * 1024, image.len() as u64);
    let segments = feed_and_collect(&mut enc, &image, 65536);
    assert!(!segments.is_empty());
    let decoded = decode_segments(&segments);
    assert_eq!(decoded.len(), 3 * BLK);
    assert_eq!(decoded[..image.len()], image[..]);
    assert!(decoded[image.len()..].iter().all(|&b| b == 0));
}

#[test]
fn each_segment_has_a_valid_sparse_file_header() {
    let max = (FILE_HDR + CHUNK_HDR + 3 * BLK) as u32;
    let image = random_bytes(20 * BLK, 55);
    let mut enc = SparseEncoder::new(max, image.len() as u64);
    let segments = feed_and_collect(&mut enc, &image, 65536);
    assert!(segments.len() > 1);
    for seg in &segments {
        let h = SparseFileHeader::parse(seg).unwrap();
        assert_eq!(h.magic, SPARSE_MAGIC);
        assert_eq!(h.major_version, SPARSE_MAJOR_VER);
        assert_eq!(h.blk_sz, SPARSE_BLK_SZ);
        assert!(h.total_chunks > 0);
        assert_eq!(h.total_blks, 20);
    }
}

#[test]
fn empty_image_produces_no_segments() {
    let mut enc = SparseEncoder::new(1024 * 1024, 0);
    enc.finish();
    assert!(enc.take_segment().is_none());
}

#[test]
fn large_zero_regions_produce_compact_sparse_output() {
    const IMAGE: u64 = 1024 * 1024 * 1024;
    let mut enc = SparseEncoder::new(256 * 1024 * 1024, IMAGE);
    let zeros = vec![0u8; 1024 * 1024];
    for _ in 0..IMAGE / zeros.len() as u64 {
        let mut rest = &zeros[..];
        while !rest.is_empty() {
            rest = &rest[enc.feed(rest)..];
        }
    }
    enc.finish();
    let seg = enc.take_segment().unwrap();
    assert_eq!(seg.data.len(), FILE_HDR + CHUNK_HDR + 4);
    assert_eq!(enc.fill_block_count(), IMAGE / BLK as u64);
    assert_eq!(enc.raw_block_count(), 0);
}

#[test]
fn feeding_data_in_odd_chunk_sizes_works() {
    let image = random_bytes(10 * BLK, 42);
    let mut enc = SparseEncoder::new(1024 * 1024, image.len() as u64);
    let segments = feed_and_collect(&mut enc, &image, 997);
    assert_eq!(decode_segments(&segments), image);
}

#[test]
fn unknown_size_segments_cover_only_what_was_seen() {
    // Not upstream: totalImageSize 0 means no trailing DONT_CARE.
    let image = random_bytes(3 * BLK, 5);
    let mut enc = SparseEncoder::new(1024 * 1024, 0);
    let segments = feed_and_collect(&mut enc, &image, 4096);
    let h = SparseFileHeader::parse(&segments[0]).unwrap();
    assert_eq!((segments.len(), h.total_blks, h.total_chunks), (1, 3, 1));
    let stats = {
        let mut enc = SparseEncoder::new(1024 * 1024, 0);
        enc.feed(&image);
        enc.finish();
        enc.take_segment().unwrap().stats
    };
    assert_eq!((stats.blocks, stats.raw_blocks, stats.chunks), (3, 3, 1));
}
