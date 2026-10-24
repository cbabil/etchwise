//! Whole write jobs with an Ubuntu autoinstall plan: room for the seed
//! partition is checked up front, and the seed lands after the image.

use std::io::Cursor;
use std::sync::Arc;

use fatfs::FatType;
use sha2::{Digest, Sha256};

use super::*;
use crate::customise::plan::{AUTOINSTALL_UBUNTU, CustomisationPlan};
use crate::seed::{SEED_ROOM, SEED_SIZE};
use crate::testutil::{MIB, autoinstall_plan, mbr_image, read_text, seed_volume};

fn job(path: std::path::PathBuf, image: &[u8]) -> Job {
    Job {
        source: Some(Source::Local(path)),
        extract_size: image.len() as u64,
        write: WriteOptions {
            expected_sha256: Some(hex::encode(Sha256::digest(image))),
            buffer_size: MIB,
            ..WriteOptions::default()
        },
        verify: true,
        plan: Some(autoinstall_plan()),
        ..Job::default()
    }
}

#[test]
fn needs_room_for_the_seed_partition() {
    let plan = |init_format: &str| CustomisationPlan {
        init_format: init_format.into(),
        ..autoinstall_plan()
    };
    assert_eq!(seed_room(None), 0);
    assert_eq!(seed_room(Some(&plan("cloudinit"))), 0);
    assert_eq!(seed_room(Some(&plan(AUTOINSTALL_UBUNTU))), SEED_ROOM);
    let nothing_to_apply = CustomisationPlan {
        cloudinit: String::new(),
        ..plan(AUTOINSTALL_UBUNTU)
    };
    assert_eq!(
        seed_room(Some(&nothing_to_apply)),
        0,
        "plain ISO, no false TooSmall"
    );
}

#[test]
fn writes_verifies_and_adds_the_seed() {
    let dir = tempfile::tempdir().unwrap();
    let image = mbr_image(FatType::Fat16, 16);
    let path = dir.path().join("server.iso");
    std::fs::write(&path, &image).unwrap();
    let progress = Arc::new(Progress::default());
    let mut small = Cursor::new(vec![0u8; image.len() + 4 * MIB]);
    let err = run(&mut small, &job(path.clone(), &image), &progress).unwrap_err();
    assert!(matches!(err, WriteError::TooSmall { .. }), "{err}");
    let mut device = Cursor::new(vec![0xAAu8; image.len() + SEED_SIZE as usize + 4 * MIB]);
    run(&mut device, &job(path, &image), &progress).unwrap();
    assert_eq!(progress.state(), WriteState::Succeeded);
    assert_eq!(progress.snapshot().verify_now, image.len() as u64);
    let bytes = device.into_inner();
    assert_eq!(bytes[512..image.len()], image[512..]);
    assert!(read_text(&seed_volume(&bytes, 2), "user-data").contains("autoinstall:"));
}
