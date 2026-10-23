//! Upstream `TestSystemToolValidation`, ported to the `fatfs` crate (the
//! upstream test shells out to fdisk/mount/fsck.fat), plus stale-data
//! overwriting.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

use fatfs::{FileSystem, FsOptions};

use super::format_device;
use super::tests::{MIB, formatted, read_at};
use crate::pipeline::Progress;

/// The partition at sector 8192 as its own seekable stream.
struct Window<'a> {
    file: &'a mut File,
    base: u64,
    len: u64,
    pos: u64,
}

impl<'a> Window<'a> {
    fn partition(file: &'a mut File) -> Self {
        let len = file.metadata().unwrap().len() - 8192 * 512;
        Self {
            file,
            base: 8192 * 512,
            len,
            pos: 0,
        }
    }
}

impl Read for Window<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = buf
            .len()
            .min(usize::try_from(self.len - self.pos).unwrap_or(usize::MAX));
        self.file.seek(SeekFrom::Start(self.base + self.pos))?;
        let n = self.file.read(&mut buf[..n])?;
        self.pos += n as u64;
        Ok(n)
    }
}

impl Write for Window<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.seek(SeekFrom::Start(self.base + self.pos))?;
        let n = self.file.write(buf)?;
        self.pos += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl Seek for Window<'_> {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        let pos = match to {
            SeekFrom::Start(p) => Some(p),
            SeekFrom::Current(d) => self.pos.checked_add_signed(d),
            SeekFrom::End(d) => self.len.checked_add_signed(d),
        };
        self.pos = pos.ok_or_else(|| io::Error::other("seek before start"))?;
        Ok(self.pos)
    }
}

fn mount(file: &mut File) -> io::Result<FileSystem<Window<'_>>> {
    FileSystem::new(Window::partition(file), FsOptions::new())
}

#[test]
fn two_gib_mounts_empty_with_label_and_accepts_a_file() {
    let mut file = formatted(2048 * MIB);
    let fs = mount(&mut file).unwrap();
    assert_eq!(fs.fat_type(), fatfs::FatType::Fat32);
    assert_eq!(fs.volume_label(), "BOOT");
    assert_eq!(fs.volume_id(), 0x1234_5678);
    assert_eq!(fs.root_dir().iter().count(), 0);
    let stats = fs.stats().unwrap();
    assert_eq!(
        (stats.cluster_size(), stats.total_clusters()),
        (4096, 522_240)
    );
    // Upstream's FAT is 2 entries short of the cluster count (see report),
    // so the last two clusters read as whatever follows FAT 1: FAT 2's head.
    assert!(stats.free_clusters() >= 522_236);
    fs.root_dir()
        .create_file("test.txt")
        .unwrap()
        .write_all(b"x")
        .unwrap();
    assert_eq!(fs.root_dir().iter().count(), 1);
}

/// Upstream's 64 MB image has 60 947 clusters, a FAT16-sized count in a
/// FAT32 BPB. Linux and macOS key off the BPB and mount it; `fatfs`
/// follows the spec's cluster-count rule and refuses, so the structure
/// is verified by hand instead.
#[test]
fn sixty_four_mib_is_fat32_by_bpb_but_fat16_by_cluster_count() {
    let mut file = formatted(64 * MIB);
    let err = mount(&mut file).err().unwrap();
    assert!(err.to_string().contains("FAT32 determination"), "{err}");
    let root = read_at(&mut file, (8192 + 32 + 2 * 477) * 512, 1024);
    assert!(root.iter().all(|&b| b == 0));
}

#[test]
fn root_cluster_overwrites_stale_data() {
    let mut file = tempfile::tempfile().unwrap();
    file.set_len(64 * MIB).unwrap();
    let root = (8192 + 32 + 2 * 477) * 512;
    file.seek(SeekFrom::Start(root)).unwrap();
    file.write_all(&[0xAA; 2048]).unwrap();
    format_device(&mut file, &Progress::default()).unwrap();
    let after = read_at(&mut file, root, 2048);
    assert!(after[..1024].iter().all(|&b| b == 0));
    assert!(after[1024..].iter().all(|&b| b == 0xAA), "only one cluster");
}
