use std::io::{Cursor, Seek, SeekFrom};

use fatfs::FatType;

use super::*;
use crate::pipeline::writer::{WriteOptions, prepare, write_stream};
use crate::testutil::{MIB, mbr_image};

fn options(buffer: usize) -> WriteOptions {
    WriteOptions {
        buffer_size: buffer,
        ..WriteOptions::default()
    }
}

/// A device pre-filled with 0xAA so untouched bytes are visible.
fn device(len: usize) -> Cursor<Vec<u8>> {
    Cursor::new(vec![0xAA; len])
}

#[test]
fn customises_through_the_held_first_block() {
    let image = mbr_image(FatType::Fat16, 16);
    let mut dev = device(image.len() + 2 * MIB);
    let progress = Progress::default();
    prepare(&mut dev, &options(MIB), &Progress::default()).unwrap();
    let written = write_stream(&mut dev, Cursor::new(&image), &options(MIB), &progress).unwrap();
    verify(&mut dev, &written, &progress).unwrap();
    let plan = CustomisationPlan {
        init_format: "systemd".into(),
        firstrun: "#!/bin/sh\n".into(),
        ..CustomisationPlan::default()
    };
    finalize(&mut dev, written, Some(&plan), "", &progress).unwrap();
    assert_eq!(dev.get_ref()[..512], image[..512], "MBR from the image");
    dev.seek(SeekFrom::Start(0)).unwrap();
    let mut boot = BootPartition::open(dev).unwrap();
    assert_eq!(
        boot.read_file("firstrun.sh").unwrap().unwrap(),
        b"#!/bin/sh\n"
    );
}

#[test]
fn network_only_plan_does_not_customise() {
    let image = mbr_image(FatType::Fat16, 16);
    let mut dev = device(image.len());
    let progress = Progress::default();
    let written = write_stream(&mut dev, Cursor::new(&image), &options(MIB), &progress).unwrap();
    let plan = CustomisationPlan {
        init_format: "cloudinit".into(),
        network: "network:\n".into(),
        ..CustomisationPlan::default()
    };
    finalize(&mut dev, written, Some(&plan), "id", &progress).unwrap();
    let mut boot = BootPartition::open(dev).unwrap();
    assert!(!boot.exists("network-config").unwrap());
}
