//! Sector images, byte for byte as upstream's packed structs serialise
//! (`MbrPartitionEntry`, `Fat32BootSector`, `Fat32FsInfo`, the first FAT
//! sector). Everything not set here is zero, as upstream's `{}` init and
//! zeroed `AlignedBuffer`s are.

use super::layout::{
    BACKUP_BOOT_SECTOR, FAT32_PARTITION_TYPE, FS_INFO_SECTOR, Fat32Layout, PARTITION_START_SECTOR,
    SECTOR_SIZE,
};

/// One 512-byte sector.
pub(crate) type Sector = [u8; SECTOR_SIZE as usize];

/// Offset of the first MBR partition entry.
pub(crate) const MBR_ENTRY_OFFSET: usize = 446;

fn put_u16(buf: &mut Sector, at: usize, value: u16) {
    buf[at..at + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(buf: &mut Sector, at: usize, value: u32) {
    buf[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

/// Upstream's simple CHS triple `(head, sector, cylinder)` for an LBA, the
/// cylinder clamped to 1023 (upstream clamps only the end address; the
/// start, LBA 8192, is cylinder 0 either way).
pub(crate) fn chs(lba: u32) -> [u8; 3] {
    let cylinder = (lba / (63 * 255)).min(1023);
    let head = (lba / 63) % 255;
    let sector = (lba % 63) + 1;
    // Masked to the CHS bit fields, as upstream's implicit narrowing does.
    #[allow(clippy::cast_possible_truncation)]
    [
        head as u8,
        (((cylinder >> 2) & 0xC0) | (sector & 0x3F)) as u8,
        (cylinder & 0xFF) as u8,
    ]
}

/// `WriteMbr`: one bootable FAT32-LBA entry at 8192 and the 55 AA mark.
pub(crate) fn mbr(partition_sectors: u32) -> Sector {
    let mut buf = [0; SECTOR_SIZE as usize];
    let entry = MBR_ENTRY_OFFSET;
    buf[entry] = 0x80;
    buf[entry + 1..entry + 4].copy_from_slice(&chs(PARTITION_START_SECTOR));
    buf[entry + 4] = FAT32_PARTITION_TYPE;
    let end = PARTITION_START_SECTOR
        .wrapping_add(partition_sectors)
        .wrapping_sub(1);
    buf[entry + 5..entry + 8].copy_from_slice(&chs(end));
    put_u32(&mut buf, entry + 8, PARTITION_START_SECTOR);
    put_u32(&mut buf, entry + 12, partition_sectors);
    buf[510] = 0x55;
    buf[511] = 0xAA;
    buf
}

/// `WriteBootSector`; `hidden_sectors` is the absolute sector the copy is
/// written to (upstream passes 8192 for the primary, 8198 for the backup).
pub(crate) fn boot_sector(layout: &Fat32Layout, hidden_sectors: u32) -> Sector {
    let config = &layout.config;
    let mut buf = [0; SECTOR_SIZE as usize];
    buf[..3].copy_from_slice(&[0xEB, 0x58, 0x90]);
    buf[3..11].copy_from_slice(b"mkfs.fat");
    put_u16(&mut buf, 11, SECTOR_SIZE as u16);
    buf[13] = config.sectors_per_cluster;
    put_u16(&mut buf, 14, config.reserved_sectors);
    buf[16] = config.num_fats;
    buf[21] = 0xF8;
    put_u16(&mut buf, 24, 63);
    put_u16(&mut buf, 26, 255);
    put_u32(&mut buf, 28, hidden_sectors);
    put_u32(&mut buf, 32, config.total_sectors);
    put_u32(&mut buf, 36, layout.sectors_per_fat);
    put_u32(&mut buf, 44, 2);
    put_u16(&mut buf, 48, FS_INFO_SECTOR as u16);
    put_u16(&mut buf, 50, BACKUP_BOOT_SECTOR as u16);
    buf[64] = 0x80;
    buf[66] = 0x29;
    put_u32(&mut buf, 67, config.volume_id);
    buf[71..82].copy_from_slice(&padded_label(config.volume_label));
    buf[82..90].copy_from_slice(b"FAT32   ");
    put_u16(&mut buf, 510, 0xAA55);
    buf
}

/// The 11-byte space-padded label (truncated beyond 11 bytes).
pub(crate) fn padded_label(label: &str) -> [u8; 11] {
    let mut out = [b' '; 11];
    let bytes = label.as_bytes();
    let n = bytes.len().min(out.len());
    out[..n].copy_from_slice(&bytes[..n]);
    out
}

/// `WriteFsInfo`: signatures, free count, next free cluster 3.
pub(crate) fn fs_info(layout: &Fat32Layout) -> Sector {
    let mut buf = [0; SECTOR_SIZE as usize];
    put_u32(&mut buf, 0, 0x4161_5252);
    put_u32(&mut buf, 484, 0x6141_7272);
    put_u32(&mut buf, 488, layout.free_clusters());
    put_u32(&mut buf, 492, 3);
    put_u32(&mut buf, 508, 0xAA55_0000);
    buf
}

/// First sector of each FAT copy: media/EOC entries 0-1, root chain end at
/// entry 2. The rest of the FAT is zero.
pub(crate) fn fat_head() -> Sector {
    let mut buf = [0; SECTOR_SIZE as usize];
    put_u32(&mut buf, 0, 0x0FFF_FFF8);
    put_u32(&mut buf, 4, 0x0FFF_FFFF);
    put_u32(&mut buf, 8, 0x0FFF_FFFF);
    buf
}
