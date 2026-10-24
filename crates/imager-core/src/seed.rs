//! The cloud-init NoCloud `CIDATA` partition that Ubuntu's installer reads
//! autoinstall `user-data` from (not in upstream). It is added after the
//! written image: a new GPT entry (moving the backup GPT from the image end to
//! the device end) or a new MBR entry, formatted FAT and filled. The volume
//! is written first and the partition tables last, sector 0 at the very end.

mod gpt;
mod mbr;

use std::io::{self, Read, Seek, SeekFrom, Write};

use crate::fat::{BootPartition, Extent};

/// Size of the seed partition.
pub const SEED_SIZE: u64 = 16 << 20;
/// Volume label Ubuntu's NoCloud data source looks for, also the GPT
/// partition name.
pub(crate) const SEED_NAME: &str = "CIDATA";
const SEED_LABEL: [u8; 11] = *b"CIDATA     ";
pub(crate) const SECTOR: u64 = 512;
/// The partition starts on a 1 MiB boundary.
const ALIGN_SECTORS: u64 = 2048;
/// Space a seed needs after the image: the partition, its alignment, and
/// the backup GPT moved to the device end.
pub const SEED_ROOM: u64 = SEED_SIZE + 2 * ALIGN_SECTORS * SECTOR;
/// GPT "Microsoft basic data" type EBD0A0A2-B9E5-4433-87C0-68B6B72699C7.
pub(crate) const BASIC_DATA_TYPE: [u8; 16] = [
    0xA2, 0xA0, 0xD0, 0xEB, 0xE5, 0xB9, 0x33, 0x44, 0x87, 0xC0, 0x68, 0xB6, 0xB7, 0x26, 0x99, 0xC7,
];

/// Adds the seed partition holding `files` after an image of `image_len`
/// bytes on a device of `device_len` bytes.
///
/// # Errors
/// No room after the image, no free partition slot, a damaged GPT, or an
/// I/O failure.
pub fn add_seed_partition<D: Read + Write + Seek>(
    device: &mut D,
    device_len: u64,
    image_len: u64,
    files: &[(&str, &[u8])],
) -> io::Result<()> {
    let sectors = device_len / SECTOR;
    if read_sector(device, 1)?.starts_with(b"EFI PART") {
        let table = gpt::Gpt::read(device)?;
        let first = placement(image_len, table.used_end(), table.last_usable_on(sectors))?;
        write_volume(device, first, files)?;
        table.add(device, sectors, first, first + SEED_SIZE / SECTOR - 1)
    } else {
        let mut sector0 = read_sector(device, 0)?;
        let limit = sectors.min(u64::from(u32::MAX)) - 1;
        let first = placement(image_len, mbr::used_end(&sector0), limit)?;
        write_volume(device, first, files)?;
        mbr::add(&mut sector0, first, SEED_SIZE / SECTOR)?;
        write_sector(device, 0, &sector0)
    }
}

/// First LBA of the seed: after the image and every partition, aligned, and
/// ending by `last_usable`.
fn placement(image_len: u64, used_end: u64, last_usable: u64) -> io::Result<u64> {
    let after = image_len.div_ceil(SECTOR).max(used_end);
    let first = after.next_multiple_of(ALIGN_SECTORS);
    if first + SEED_SIZE / SECTOR - 1 > last_usable {
        return Err(io::Error::other(
            "No room after the image for the setup partition: use a larger USB stick",
        ));
    }
    Ok(first)
}

fn write_volume<D: Read + Write + Seek>(
    device: &mut D,
    first: u64,
    files: &[(&str, &[u8])],
) -> io::Result<()> {
    let extent = Extent {
        start: first * SECTOR,
        len: SEED_SIZE,
    };
    let mut volume = BootPartition::format_new(&mut *device, extent, SEED_LABEL)?;
    for (name, contents) in files {
        volume.write_file(name, contents)?;
    }
    volume.finish().map(drop)
}

pub(crate) fn read_sector<D: Read + Seek>(
    device: &mut D,
    lba: u64,
) -> io::Result<[u8; SECTOR as usize]> {
    let mut sector = [0u8; SECTOR as usize];
    device.seek(SeekFrom::Start(lba * SECTOR))?;
    device.read_exact(&mut sector)?;
    Ok(sector)
}

pub(crate) fn write_sector<D: Write + Seek>(
    device: &mut D,
    lba: u64,
    data: &[u8],
) -> io::Result<()> {
    device.seek(SeekFrom::Start(lba * SECTOR))?;
    device.write_all(data)
}

#[cfg(test)]
mod tests;
