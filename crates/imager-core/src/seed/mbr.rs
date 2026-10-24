//! MBR editing for the seed partition on images without a GPT.

use std::io;

use crate::erase::{FAT32_PARTITION_TYPE, MBR_ENTRY_OFFSET};
use crate::le::read_u32;

/// CHS bytes meaning "use LBA" for partitions past the CHS limit.
const CHS_MAX: [u8; 3] = [0xFE, 0xFF, 0xFF];

/// Byte offsets of the four MBR partition entries.
pub(super) fn slots() -> impl Iterator<Item = usize> {
    (0..4).map(|i| MBR_ENTRY_OFFSET + 16 * i)
}

/// One past the last sector any MBR partition uses.
pub(super) fn used_end(sector0: &[u8; 512]) -> u64 {
    slots()
        .filter(|&at| sector0[at + 4] != 0)
        .map(|at| u64::from(read_u32(sector0, at + 8)) + u64::from(read_u32(sector0, at + 12)))
        .max()
        .unwrap_or(0)
}

/// Fills the first free slot with a FAT32 partition of `count` sectors.
pub(super) fn add(sector0: &mut [u8; 512], first: u64, count: u64) -> io::Result<()> {
    if sector0[510..512] != [0x55, 0xAA] {
        return Err(io::Error::other("The image has no partition table"));
    }
    let slot = slots()
        .find(|&at| sector0[at + 4] == 0)
        .ok_or_else(|| io::Error::other("The image's MBR has no free partition slot"))?;
    let too_far = || io::Error::other("The setup partition is beyond the MBR limit");
    let first = u32::try_from(first).map_err(|_| too_far())?;
    let count = u32::try_from(count).map_err(|_| too_far())?;
    let entry = &mut sector0[slot..slot + 16];
    entry[0] = 0;
    entry[1..4].copy_from_slice(&CHS_MAX);
    entry[4] = FAT32_PARTITION_TYPE;
    entry[5..8].copy_from_slice(&CHS_MAX);
    entry[8..12].copy_from_slice(&first.to_le_bytes());
    entry[12..16].copy_from_slice(&count.to_le_bytes());
    Ok(())
}
