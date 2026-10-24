//! GPT editing for the seed partition: validate the image's primary GPT,
//! add one entry, and rewrite both copies with fresh CRCs, the backup moved
//! from the image's end to the device's last sector (what `sgdisk -e` does).

use std::io::{self, Read, Seek, SeekFrom, Write};

use super::mbr::slots;
use super::{BASIC_DATA_TYPE, SECTOR, read_sector, write_sector};
use crate::le::{invalid, put_u32, put_u64, read_u32, read_u64};

/// The largest entry array accepted (the spec minimum is 16 KiB).
const MAX_ENTRIES_BYTES: u64 = 1 << 20;

/// Header CRC over `header_size` bytes with the CRC field zeroed.
fn header_crc(header: &[u8; 512]) -> u32 {
    let size = (read_u32(header, 12) as usize).clamp(92, 512);
    let mut zeroed = *header;
    zeroed[16..20].fill(0);
    crc32fast::hash(&zeroed[..size])
}

fn seal(header: &mut [u8; 512]) {
    let crc = header_crc(header);
    put_u32(header, 16, crc);
}

/// The primary GPT of a written image.
pub(super) struct Gpt {
    header: [u8; 512],
    entries: Vec<u8>,
}

impl Gpt {
    /// Reads and validates the GPT at LBA 1.
    pub(super) fn read<D: Read + Seek>(device: &mut D) -> io::Result<Self> {
        let header = read_sector(device, 1)?;
        if header_crc(&header) != read_u32(&header, 16) {
            return Err(invalid("The image's GPT header CRC does not match"));
        }
        let (count, size) = (
            u64::from(read_u32(&header, 80)),
            u64::from(read_u32(&header, 84)),
        );
        if size < 128 || count * size > MAX_ENTRIES_BYTES {
            return Err(invalid("The image's GPT entry table is not supported"));
        }
        let mut entries = vec![0u8; (count * size) as usize];
        device.seek(SeekFrom::Start(read_u64(&header, 72) * SECTOR))?;
        device.read_exact(&mut entries)?;
        if crc32fast::hash(&entries) != read_u32(&header, 88) {
            return Err(invalid("The image's GPT entries CRC does not match"));
        }
        Ok(Self { header, entries })
    }

    fn entry_size(&self) -> usize {
        read_u32(&self.header, 84) as usize
    }

    fn entries_sectors(&self) -> u64 {
        (self.entries.len() as u64).div_ceil(SECTOR)
    }

    /// Last usable LBA once the backup sits at the end of a device.
    pub(super) fn last_usable_on(&self, sectors: u64) -> u64 {
        sectors - 2 - self.entries_sectors()
    }

    /// One past the last LBA any partition uses.
    pub(super) fn used_end(&self) -> u64 {
        self.entries
            .chunks(self.entry_size())
            .filter(|e| e[..16] != [0; 16])
            .map(|e| read_u64(e, 40) + 1)
            .max()
            .unwrap_or(0)
    }

    /// Adds the seed entry and writes backup, primary, then sector 0.
    pub(super) fn add<D: Read + Write + Seek>(
        mut self,
        device: &mut D,
        sectors: u64,
        first: u64,
        last: u64,
    ) -> io::Result<()> {
        self.add_entry(first, last)?;
        let old_backup = read_u64(&self.header, 32);
        let backup_lba = sectors - 1;
        put_u64(&mut self.header, 32, backup_lba);
        let last_usable = self.last_usable_on(sectors);
        put_u64(&mut self.header, 48, last_usable);
        put_u32(&mut self.header, 88, crc32fast::hash(&self.entries));
        seal(&mut self.header);
        let mut backup = self.header;
        let backup_entries = backup_lba - self.entries_sectors();
        put_u64(&mut backup, 24, backup_lba);
        put_u64(&mut backup, 32, 1);
        put_u64(&mut backup, 72, backup_entries);
        seal(&mut backup);
        write_sector(device, backup_entries, &self.entries)?;
        write_sector(device, backup_lba, &backup)?;
        write_sector(device, read_u64(&self.header, 72), &self.entries)?;
        if old_backup != backup_lba && old_backup < first {
            write_sector(device, old_backup, &[0u8; 512])?;
        }
        write_sector(device, 1, &self.header)?;
        protect_whole_device(device, sectors)
    }

    fn add_entry(&mut self, first: u64, last: u64) -> io::Result<()> {
        let size = self.entry_size();
        let entry = self
            .entries
            .chunks_mut(size)
            .find(|e| e[..16] == [0; 16])
            .ok_or_else(|| invalid("The image's GPT has no free partition entry"))?;
        entry.fill(0);
        entry[..16].copy_from_slice(&BASIC_DATA_TYPE);
        let mut guid: [u8; 16] = rand::random();
        guid[7] = (guid[7] & 0x0F) | 0x40;
        guid[8] = (guid[8] & 0x3F) | 0x80;
        entry[16..32].copy_from_slice(&guid);
        put_u64(entry, 32, first);
        put_u64(entry, 40, last);
        for (i, unit) in super::SEED_NAME.encode_utf16().enumerate() {
            entry[56 + 2 * i..58 + 2 * i].copy_from_slice(&unit.to_le_bytes());
        }
        Ok(())
    }
}

/// Grows the protective MBR's 0xEE entry to the whole device.
fn protect_whole_device<D: Read + Write + Seek>(device: &mut D, sectors: u64) -> io::Result<()> {
    let mut sector0 = read_sector(device, 0)?;
    let length = u32::try_from(sectors - 1).unwrap_or(u32::MAX);
    if let Some(slot) = slots().find(|&at| sector0[at + 4] == 0xEE) {
        put_u32(&mut sector0, slot + 12, length);
        write_sector(device, 0, &sector0)?;
    }
    Ok(())
}
