//! Block map parsing, lookup, wire serialisation and DONT_CARE encoding
//! (no upstream test file covers `bmap.cpp`; these follow its behaviour).

use super::SparseEncoder;
use super::bmap::{BMAP_WIRE_HEADER_SIZE, BMAP_WIRE_MAGIC, BMAP_WIRE_RANGE_SIZE, BlockMap};
use super::sparse::{BLK, CHUNK_TYPE_DONT_CARE, SparseChunkHeader};
use super::tests_sparse::{decode_segments, feed_and_collect, random_bytes};

const SUM: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

fn sample() -> String {
    format!(
        r#"<?xml version="1.0" ?>
<!-- comment -->
<bmap version="2.0">
    <ImageSize> 40960 </ImageSize>
    <BlockSize> 4096 </BlockSize>
    <BlocksCount> 10 </BlocksCount>
    <MappedBlocksCount> 4 </MappedBlocksCount>
    <ChecksumType> sha256 </ChecksumType>
    <BlockMap>
        <Range chksum="{SUM}"> 6-7 </Range>
        <Range> 0-1 </Range>
        <Range chksum="bad"> 9 </Range>
    </BlockMap>
</bmap>"#
    )
}

#[test]
fn parses_counters_and_sorted_ranges() {
    let map = BlockMap::parse(sample().as_bytes()).unwrap();
    assert_eq!(
        (
            map.block_size(),
            map.block_count(),
            map.mapped_block_count()
        ),
        (4096, 10, 4)
    );
    let spans: Vec<_> = map
        .ranges()
        .iter()
        .map(|r| (r.begin, r.end, r.has_sha256))
        .collect();
    assert_eq!(spans, [(0, 2, false), (6, 8, true), (9, 10, false)]);
    assert!(map.has_checksums() && !map.is_empty());
    assert_eq!(hex::encode(map.ranges()[1].sha256), SUM);
}

#[test]
fn lookups_match_ranges() {
    let mut map = BlockMap::parse(sample().as_bytes()).unwrap();
    let mapped = [
        true, true, false, false, false, false, true, true, false, true, false,
    ];
    for (i, want) in mapped.iter().enumerate() {
        assert_eq!(map.is_mapped(i as u64), *want, "block {i}");
    }
    for (i, want) in mapped.iter().enumerate() {
        assert_eq!(map.is_mapped_sequential(i as u64), *want, "block {i}");
    }
}

#[test]
fn serializes_wire_format() {
    let map = BlockMap::parse(sample().as_bytes()).unwrap();
    let wire = map.serialize();
    assert_eq!(wire.len(), BMAP_WIRE_HEADER_SIZE + 3 * BMAP_WIRE_RANGE_SIZE);
    assert_eq!(wire[..4], BMAP_WIRE_MAGIC.to_le_bytes());
    assert_eq!(wire[4..8], 4096u32.to_le_bytes());
    assert_eq!(wire[8..12], 3u32.to_le_bytes());
    let second = &wire[16 + 40..16 + 80];
    assert_eq!(second[..4], 6u32.to_le_bytes());
    assert_eq!(second[4..8], 7u32.to_le_bytes()); // inclusive end
    assert_eq!(hex::encode(&second[8..]), SUM);
}

#[test]
fn rejects_malformed_xml_and_accepts_empty() {
    assert!(BlockMap::parse(b"<bmap><BlockMap>").is_err());
    assert!(BlockMap::parse(b"").unwrap().is_empty());
    assert!(BlockMap::parse(&[0xFF, 0xFE]).is_err());
}

#[test]
fn unmapped_blocks_become_dont_care() {
    let image = random_bytes(10 * BLK, 3);
    let mut enc = SparseEncoder::new(1024 * 1024, image.len() as u64);
    enc.set_block_map(BlockMap::parse(sample().as_bytes()).unwrap());
    let segments = feed_and_collect(&mut enc, &image, 65536);
    assert_eq!((enc.raw_block_count(), enc.dont_care_block_count()), (5, 5));
    let decoded = decode_segments(&segments);
    for block in [0, 1, 6, 7, 9] {
        let r = block * BLK..(block + 1) * BLK;
        assert_eq!(decoded[r.clone()], image[r]);
    }
    let second = SparseChunkHeader::parse(&segments[0][28 + 12 + 2 * BLK..]).unwrap();
    assert_eq!(
        (second.chunk_type, second.chunk_sz),
        (CHUNK_TYPE_DONT_CARE, 4)
    );
}
