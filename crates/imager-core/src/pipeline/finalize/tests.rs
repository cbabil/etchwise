use std::io::{Cursor, Seek, SeekFrom};

use fatfs::FatType;

use super::*;
use crate::pipeline::writer::{WriteOptions, prepare, write_stream};
use crate::testutil::{MIB, autoinstall_plan, mbr_image, read_text, seed_volume};

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

#[test]
fn autoinstall_adds_a_seed_partition_and_leaves_the_image() {
    let image = mbr_image(FatType::Fat16, 16);
    let mut dev = device(image.len() + 2 * MIB + crate::seed::SEED_SIZE as usize);
    let progress = Progress::default();
    prepare(&mut dev, &options(MIB), &Progress::default()).unwrap();
    let written = write_stream(&mut dev, Cursor::new(&image), &options(MIB), &progress).unwrap();
    let plan = autoinstall_plan();
    finalize(&mut dev, written, Some(&plan), "etchwise-1", &progress).unwrap();
    let bytes = dev.into_inner();
    assert_eq!(
        bytes[512..image.len()],
        image[512..],
        "image untouched after the MBR"
    );
    assert_eq!(
        bytes[446 + 16 + 4],
        0x0C,
        "seed partition added as MBR slot 2"
    );
    let fs = seed_volume(&bytes, 2);
    assert_eq!(
        read_text(&fs, "user-data"),
        format!("#cloud-config\n{}", plan.cloudinit)
    );
    assert_eq!(read_text(&fs, "meta-data"), "instance-id: etchwise-1\n");
}

#[test]
fn kura_setup_lands_on_the_boot_partition_after_verify() {
    let image = mbr_image(FatType::Fat16, 16);
    let mut dev = device(image.len() + 2 * MIB);
    let progress = Progress::default();
    prepare(&mut dev, &options(MIB), &Progress::default()).unwrap();
    let written = write_stream(&mut dev, Cursor::new(&image), &options(MIB), &progress).unwrap();
    verify(&mut dev, &written, &progress).unwrap();
    let settings = serde_json::json!({"hostname": "kura-01", "sshUserName": "chris"});
    let plan = crate::customise::plan::plan_from_settings(
        crate::customise::Settings(settings.as_object().unwrap()),
        crate::customise::plan::PlanInputs {
            init_format: crate::customise::plan::KURA,
            ..Default::default()
        },
    );
    finalize(&mut dev, written, Some(&plan), "", &progress).unwrap();
    assert_eq!(dev.get_ref()[..512], image[..512], "MBR from the image");
    dev.seek(SeekFrom::Start(0)).unwrap();
    let mut boot = BootPartition::open(dev).unwrap();
    let setup: serde_json::Value =
        serde_json::from_slice(&boot.read_file("kura-setup.json").unwrap().unwrap()).unwrap();
    assert_eq!(
        setup,
        serde_json::json!({"version": 1, "hostname": "kura-01", "user": {"name": "chris"}})
    );
    assert!(
        boot.read_file("cmdline.txt").unwrap().is_none(),
        "cmdline untouched"
    );
}
