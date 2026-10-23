//! Pure geometry: upstream's constants, `CalculateFat32Config`,
//! `CalculateSectorsPerFat` and the partition sizes `FormatDrive` derives
//! from the device size. All arithmetic is upstream's 32-bit arithmetic;
//! only underflow (a device too small to hold the layout) is rejected.

/// Bytes per sector (`kSectorSize`).
pub(crate) const SECTOR_SIZE: u32 = 512;
/// First partition sector, 4 MiB in (`kPartitionStartSector`).
pub(crate) const PARTITION_START_SECTOR: u32 = 8192;
/// MBR partition type: FAT32 with LBA (`kFat32PartitionType`).
pub(crate) const FAT32_PARTITION_TYPE: u8 = 0x0C;
/// Backup boot sector, relative to the partition start.
pub(crate) const BACKUP_BOOT_SECTOR: u32 = 6;
/// FSInfo sector, relative to the partition start.
pub(crate) const FS_INFO_SECTOR: u32 = 1;
/// Reserved sectors before the first FAT.
pub(crate) const RESERVED_SECTORS: u16 = 32;
/// Number of FAT copies.
pub(crate) const NUM_FATS: u8 = 2;
/// Volume label written to the boot sector.
pub(crate) const VOLUME_LABEL: &str = "BOOT";
/// Fixed volume serial number.
pub(crate) const VOLUME_ID: u32 = 0x1234_5678;

/// Upstream `Fat32Config`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Fat32Config {
    pub(crate) total_sectors: u32,
    pub(crate) sectors_per_cluster: u8,
    pub(crate) reserved_sectors: u16,
    pub(crate) num_fats: u8,
    pub(crate) volume_label: &'static str,
    pub(crate) volume_id: u32,
}

impl Fat32Config {
    /// `CalculateFat32Config`: cluster size from the partition size.
    pub(crate) fn for_partition(partition_sectors: u32) -> Self {
        Self {
            total_sectors: partition_sectors,
            sectors_per_cluster: sectors_per_cluster_for(partition_sectors),
            reserved_sectors: RESERVED_SECTORS,
            num_fats: NUM_FATS,
            volume_label: VOLUME_LABEL,
            volume_id: VOLUME_ID,
        }
    }
}

/// Upstream's cluster size thresholds (in sectors of the partition).
pub(crate) fn sectors_per_cluster_for(partition_sectors: u32) -> u8 {
    match partition_sectors {
        0..66_600 => 1,
        66_600..532_480 => 2,
        532_480..16_777_216 => 8,
        16_777_216..33_554_432 => 16,
        _ => 32,
    }
}

fn fat_sectors_for(data_sectors: u32, sectors_per_cluster: u8) -> u32 {
    let clusters = data_sectors / u32::from(sectors_per_cluster);
    let fat_bytes = (clusters + 2) * 4;
    fat_bytes.div_ceil(SECTOR_SIZE)
}

/// `CalculateSectorsPerFat`: two-pass estimate; `None` on underflow.
pub(crate) fn sectors_per_fat(config: &Fat32Config) -> Option<u32> {
    let data = config
        .total_sectors
        .checked_sub(u32::from(config.reserved_sectors))?;
    let first = fat_sectors_for(data, config.sectors_per_cluster);
    let data = data.checked_sub(first.checked_mul(u32::from(config.num_fats))?)?;
    Some(fat_sectors_for(data, config.sectors_per_cluster))
}

/// The filesystem geometry `WriteFat32` lays down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Fat32Layout {
    pub(crate) config: Fat32Config,
    pub(crate) sectors_per_fat: u32,
    /// Data clusters (`total_clusters` in `WriteFsInfo`).
    pub(crate) total_clusters: u32,
}

impl Fat32Layout {
    /// Geometry for a partition; `None` when the root cluster cannot fit.
    pub(crate) fn new(partition_sectors: u32) -> Option<Self> {
        let config = Fat32Config::for_partition(partition_sectors);
        let sectors_per_fat = sectors_per_fat(&config)?;
        let fats = sectors_per_fat.checked_mul(u32::from(config.num_fats))?;
        let data = partition_sectors
            .checked_sub(u32::from(config.reserved_sectors))?
            .checked_sub(fats)?;
        let total_clusters = data / u32::from(config.sectors_per_cluster);
        (total_clusters >= 1).then_some(Self {
            config,
            sectors_per_fat,
            total_clusters,
        })
    }

    /// FSInfo free count: every cluster but the root directory's.
    pub(crate) fn free_clusters(&self) -> u32 {
        self.total_clusters - 1
    }

    /// First FAT sector, relative to the partition start.
    pub(crate) fn fat_start(&self) -> u32 {
        u32::from(self.config.reserved_sectors)
    }

    /// Root directory (cluster 2) sector, relative to the partition start.
    pub(crate) fn root_start(&self) -> u32 {
        self.fat_start() + u32::from(self.config.num_fats) * self.sectors_per_fat
    }
}

/// Partition sizes `FormatDrive` derives from the device size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeviceLayout {
    /// Length recorded in the MBR (sector count clamped to `u32::MAX`).
    pub(crate) mbr_partition_sectors: u32,
    /// The filesystem itself.
    pub(crate) fat: Fat32Layout,
}

impl DeviceLayout {
    /// Upstream's sizing: the MBR clamps the sector count to 32 bits, the
    /// filesystem truncates it (`std::uint32_t total_sectors = size / 512`),
    /// so beyond 2 TiB the two disagree exactly as upstream's do.
    pub(crate) fn for_device(size_bytes: u64) -> Option<Self> {
        let sectors = size_bytes / u64::from(SECTOR_SIZE);
        let clamped = u32::try_from(sectors).unwrap_or(u32::MAX);
        // Truncation is the upstream behaviour being reproduced.
        #[allow(clippy::cast_possible_truncation)]
        let truncated = sectors as u32;
        let mbr_partition_sectors = clamped.checked_sub(PARTITION_START_SECTOR)?;
        let fs_sectors = truncated.checked_sub(PARTITION_START_SECTOR)?;
        Some(Self {
            mbr_partition_sectors,
            fat: Fat32Layout::new(fs_sectors)?,
        })
    }
}
