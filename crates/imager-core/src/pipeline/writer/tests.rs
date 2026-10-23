use std::io::{Cursor, Read};
use std::sync::atomic::Ordering;

use sha2::{Digest, Sha256};

use super::*;
use crate::pipeline::finalize::{finalize, verify};
use crate::testutil::MIB;

fn options(buffer: usize) -> WriteOptions {
    WriteOptions {
        buffer_size: buffer,
        ..WriteOptions::default()
    }
}

fn sha(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

/// A device pre-filled with 0xAA so untouched bytes are visible.
fn device(len: usize) -> Cursor<Vec<u8>> {
    Cursor::new(vec![0xAA; len])
}

#[test]
fn prepare_zeroes_both_ends() {
    let mut dev = device(4 * MIB);
    prepare(&mut dev, &options(4096), &Progress::default()).unwrap();
    let data = dev.get_ref();
    assert!(data[..MIB].iter().all(|b| *b == 0));
    assert!(data[3 * MIB..].iter().all(|b| *b == 0));
    assert_eq!(data[2 * MIB], 0xAA);
    assert_eq!(dev.position(), 0);
    let mut skipped = device(4 * MIB);
    prepare(
        &mut skipped,
        &WriteOptions {
            skip_end_of_device: true,
            ..options(4096)
        },
        &Progress::default(),
    )
    .unwrap();
    assert_eq!(skipped.get_ref()[4 * MIB - 1], 0xAA);
}

#[test]
fn holds_back_first_block_and_hashes_everything() {
    let image: Vec<u8> = (0..20_000u32).map(|i| (i % 7) as u8 + 1).collect();
    let mut dev = device(64 * 1024);
    let progress = Progress::default();
    let opts = WriteOptions {
        expected_sha256: Some(sha(&image).to_uppercase()),
        ..options(4096)
    };
    let written = write_stream(&mut dev, Cursor::new(&image), &opts, &progress).unwrap();
    assert_eq!(written.first_block, image[..4096]);
    assert_eq!(written.length, 20_000);
    assert_eq!(
        dev.get_ref()[..4096],
        [0xAA; 4096],
        "first block not written yet"
    );
    assert_eq!(dev.get_ref()[4096..20_000], image[4096..]);
    assert_eq!(progress.bytes_written.load(Ordering::Relaxed), 20_000);
    verify(&mut dev, &written, &progress).unwrap();
    finalize(&mut dev, written, None, "", &progress).unwrap();
    assert_eq!(dev.get_ref()[..20_000], image[..]);
    assert_eq!(progress.state(), WriteState::Finalizing);
}

#[test]
fn hash_mismatch_is_reported() {
    let mut dev = device(64 * 1024);
    let opts = WriteOptions {
        expected_sha256: Some("00".repeat(32)),
        ..options(4096)
    };
    let err = write_stream(
        &mut dev,
        Cursor::new(vec![1u8; 9000]),
        &opts,
        &Progress::default(),
    )
    .unwrap_err();
    assert!(matches!(err, WriteError::HashMismatch { .. }));
    assert!(
        err.to_string()
            .starts_with("Download appears to be corrupt")
    );
}

#[test]
fn verify_detects_changed_contents_unless_skipped() {
    let image = vec![5u8; 12_288];
    let mut dev = device(64 * 1024);
    let progress = Progress::default();
    let written = write_stream(&mut dev, Cursor::new(&image), &options(4096), &progress).unwrap();
    dev.get_mut()[8000] = 9;
    assert!(matches!(
        verify(&mut dev, &written, &progress),
        Err(WriteError::VerifyMismatch)
    ));
    progress.skip_verification();
    verify(&mut dev, &written, &progress).unwrap();
}

#[test]
fn verify_fails_on_short_device() {
    let progress = Progress::default();
    let written = Written {
        first_block: vec![0; 10],
        length: 100,
        sha256: [0; 32],
    };
    assert!(matches!(
        verify(&mut Cursor::new(vec![0u8; 50]), &written, &progress),
        Err(WriteError::VerifyRead)
    ));
}

#[test]
fn zero_skip_seeks_over_zero_blocks_and_excludes_them_from_the_hash() {
    let mut second = vec![2u8; 100];
    second.resize(4096, 0);
    let image = [
        vec![1u8; 8192],
        second.clone(),
        vec![0u8; 8192],
        vec![3u8; 4096],
    ]
    .concat();
    let mut dev = device(64 * 1024);
    let opts = WriteOptions {
        zero_skip: true,
        ..options(8192)
    };
    let written = write_stream(&mut dev, Cursor::new(&image), &opts, &Progress::default()).unwrap();
    let data = dev.get_ref();
    assert_eq!(data[8192..12_288], second[..]);
    assert!(
        data[12_288..20_480].iter().all(|b| *b == 0xAA),
        "zero blocks were seeked over"
    );
    assert_eq!(data[20_480..24_576], [3u8; 4096][..]);
    assert_eq!(written.length, image.len() as u64);
    let expected: [u8; 32] = Sha256::new()
        .chain_update(&image[..12_288])
        .chain_update(&image[20_480..])
        .finalize()
        .into();
    assert_eq!(written.sha256, expected);
}

#[test]
fn cancellation_stops_the_loop() {
    let progress = Progress::default();
    progress.cancel();
    let err = write_stream(
        &mut device(8192),
        Cursor::new(vec![1u8; 100]),
        &options(4096),
        &progress,
    )
    .unwrap_err();
    assert!(matches!(err, WriteError::Cancelled));
}

#[test]
fn empty_source_writes_nothing() {
    let mut dev = device(8192);
    let written = write_stream(
        &mut dev,
        Cursor::new(Vec::<u8>::new()),
        &options(4096),
        &Progress::default(),
    )
    .unwrap();
    assert!(written.first_block.is_empty() && written.length == 0);
}

#[test]
fn source_errors_surface_as_decompress_errors() {
    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("bad xz"))
        }
    }
    let err = write_stream(
        &mut device(8192),
        Broken,
        &options(4096),
        &Progress::default(),
    )
    .unwrap_err();
    assert_eq!(err.to_string(), "Error extracting archive: bad xz");
}
