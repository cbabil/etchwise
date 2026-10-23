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
