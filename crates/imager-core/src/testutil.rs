//! Test fixtures shared across modules.

use std::io::Cursor;

use fatfs::{FatType, FormatVolumeOptions};

pub const MIB: usize = 1024 * 1024;

/// A disk image with an MBR and one FAT partition at 1 MiB.
pub fn mbr_image(fat_type: FatType, size_mib: usize) -> Vec<u8> {
    let start = MIB;
    let mut image = vec![0u8; start + size_mib * MIB];
    let mut volume = Cursor::new(&mut image[start..]);
    fatfs::format_volume(
        &mut volume,
        FormatVolumeOptions::new()
            .fat_type(fat_type)
            .volume_label(*b"bootfs     "),
    )
    .unwrap();
    let slot = 446;
    image[slot + 4] = 0x0c;
    image[slot + 8..slot + 12].copy_from_slice(&((start / 512) as u32).to_le_bytes());
    image[slot + 12..slot + 16].copy_from_slice(&((size_mib * MIB / 512) as u32).to_le_bytes());
    image[510] = 0x55;
    image[511] = 0xAA;
    image
}

/// An Ubuntu autoinstall plan with something to apply.
pub fn autoinstall_plan() -> crate::customise::plan::CustomisationPlan {
    crate::customise::plan::CustomisationPlan {
        init_format: crate::customise::plan::AUTOINSTALL_UBUNTU.into(),
        cloudinit: "autoinstall:\n  version: 1\n".into(),
        ..Default::default()
    }
}

/// The seed volume in MBR slot `slot` (1-4) of `device`, checked to be
/// labelled `CIDATA`.
pub fn seed_volume(device: &[u8], slot: usize) -> fatfs::FileSystem<Cursor<Vec<u8>>> {
    let entry = 446 + 16 * (slot - 1);
    let start = u32::from_le_bytes(device[entry + 8..entry + 12].try_into().unwrap()) as usize;
    seed_volume_at(device, start * 512)
}

/// The seed volume starting at byte `start`, checked to be labelled `CIDATA`.
pub fn seed_volume_at(device: &[u8], start: usize) -> fatfs::FileSystem<Cursor<Vec<u8>>> {
    let size = crate::seed::SEED_SIZE as usize;
    let volume = Cursor::new(device[start..start + size].to_vec());
    let fs = fatfs::FileSystem::new(volume, fatfs::FsOptions::new()).unwrap();
    assert_eq!(fs.volume_label(), "CIDATA");
    fs
}

/// A root file of `fs` as text.
pub fn read_text(fs: &fatfs::FileSystem<Cursor<Vec<u8>>>, name: &str) -> String {
    let mut out = String::new();
    std::io::Read::read_to_string(&mut fs.root_dir().open_file(name).unwrap(), &mut out).unwrap();
    out
}
