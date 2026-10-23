//! Buffer sizes from upstream `SystemMemoryManager`.

pub(crate) const KIB: usize = 1024;
pub(crate) const MIB: usize = 1024 * KIB;
pub(crate) const GIB: u64 = 1024 * MIB as u64;
/// Upstream `IMAGEWRITER_BLOCKSIZE`, the default write chunk.
pub const DEFAULT_WRITE_BUFFER: usize = MIB;

/// `getOptimalWriteBufferSize`: write chunk by total RAM. The first chunk
/// (held back until the end) has this size too.
pub fn write_buffer_size(total_mem_mb: u64) -> usize {
    match total_mem_mb {
        0..1024 => 512 * KIB,
        1024..2048 => MIB,
        2048..4096 => 2 * MIB,
        4096..8192 => 4 * MIB,
        _ => 8 * MIB,
    }
}

/// `getAdaptiveVerifyBufferSize` base sizes by amount to verify.
pub fn verify_buffer_size(bytes: u64) -> usize {
    match bytes {
        b if b < 100 * MIB as u64 => 256 * KIB,
        b if b < GIB => 2 * MIB,
        b if b < 4 * GIB => 4 * MIB,
        _ => 8 * MIB,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_follow_upstream_tiers() {
        assert_eq!(write_buffer_size(512), 512 * KIB);
        assert_eq!(write_buffer_size(1024), MIB);
        assert_eq!(write_buffer_size(3000), 2 * MIB);
        assert_eq!(write_buffer_size(4096), 4 * MIB);
        assert_eq!(write_buffer_size(65536), 8 * MIB);
        assert_eq!(verify_buffer_size(10 * MIB as u64), 256 * KIB);
        assert_eq!(verify_buffer_size(500 * MIB as u64), 2 * MIB);
        assert_eq!(verify_buffer_size(2048 * MIB as u64), 4 * MIB);
        assert_eq!(verify_buffer_size(8192 * MIB as u64), 8 * MIB);
    }
}
