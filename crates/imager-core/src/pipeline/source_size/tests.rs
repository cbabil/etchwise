use std::io::Write;

use super::*;

fn file(dir: &Path, name: &str, data: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, data).unwrap();
    path
}

#[test]
fn raw_images_use_the_file_size() {
    let dir = tempfile::tempdir().unwrap();
    let size = local_source_size(&file(dir.path(), "x.IMG", &[0; 1024]));
    assert_eq!(
        size,
        SourceSize {
            download: 1024,
            extract: 1024,
            multiple_files: false,
            extract_known: true
        }
    );
}

#[test]
fn xz_size_comes_from_the_index() {
    let dir = tempfile::tempdir().unwrap();
    let mut enc = liblzma::write::XzEncoder::new(Vec::new(), 1);
    enc.write_all(&vec![3u8; 123_456]).unwrap();
    let size = local_source_size(&file(dir.path(), "x.img.xz", &enc.finish().unwrap()));
    assert_eq!(size.extract, 123_456);
    assert_eq!(
        local_source_size(&file(dir.path(), "bad.xz", b"not an xz file at all")).extract,
        0
    );
    assert_eq!(
        local_source_size(&file(dir.path(), "tiny.xz", b"YZ")).extract,
        0
    );
}

#[test]
fn gz_size_comes_from_isize() {
    let dir = tempfile::tempdir().unwrap();
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    enc.write_all(&vec![1u8; 70_000]).unwrap();
    let size = local_source_size(&file(dir.path(), "x.img.gz", &enc.finish().unwrap()));
    assert_eq!((size.extract, size.extract_known), (70_000, true));
    let unknown = local_source_size(&file(dir.path(), "short.gz", b"1234"));
    assert_eq!((unknown.extract, unknown.extract_known), (0, false));
    let mut wrapped = vec![0u8; 100];
    wrapped.extend(10u32.to_le_bytes());
    assert_eq!(
        local_source_size(&file(dir.path(), "wrap.gz", &wrapped)).extract,
        10 + (1 << 32)
    );
}

#[test]
fn zip_sums_entries_and_flags_multiple_files() {
    let dir = tempfile::tempdir().unwrap();
    let make = |entries: &[(&str, usize)]| {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, len) in entries {
            zip.start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(&vec![9u8; *len]).unwrap();
        }
        zip.finish().unwrap().into_inner()
    };
    let one = local_source_size(&file(
        dir.path(),
        "a.zip",
        &make(&[("a.img", 4096), ("empty", 0)]),
    ));
    assert_eq!((one.extract, one.multiple_files), (4096, false));
    let two = local_source_size(&file(
        dir.path(),
        "b.zip",
        &make(&[("a", 512), ("b", 1024)]),
    ));
    assert_eq!((two.extract, two.multiple_files), (1536, true));
    assert_eq!(
        local_source_size(&file(dir.path(), "c.zst", b"zstd")).extract,
        0
    );
    assert_eq!(local_source_size(Path::new("/nonexistent.img")).download, 0);
}

#[test]
fn rejects_malformed_indexes() {
    assert_eq!(parse_xz_index(&[1, 0]), None);
    assert_eq!(parse_xz_index(&[]), None);
    assert_eq!(parse_xz_index(&[0, 1, 0x80]), None);
    assert_eq!(varint(&mut &[0xFFu8; 10][..]), None);
    assert_eq!(parse_xz_index(&[0, 2, 5, 10, 5, 20]), Some(30));
}
