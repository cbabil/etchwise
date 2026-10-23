//! Upstream `rpiboot_bootfiles_test.cpp`: TAR extraction and lookup.

use super::bootfiles::Bootfiles;
use super::test_support::{tar_of, temp_dir, write_file};

fn extracted(files: &[(&str, &[u8])]) -> Bootfiles {
    let mut bf = Bootfiles::default();
    assert_eq!(bf.extract_from_memory(&tar_of(files)), Ok(()));
    bf
}

#[test]
fn extracts_tar_from_memory() {
    let bf = extracted(&[("config.txt", b"Hello"), ("kernel.img", &[0, 1, 2, 3])]);
    assert_eq!(bf.len(), 2);
    assert_eq!(bf.find("config.txt", ""), Some(&b"Hello"[..]));
    assert_eq!(bf.find("kernel.img", ""), Some(&[0u8, 1, 2, 3][..]));
}

#[test]
fn handles_empty_archive() {
    assert!(Bootfiles::default().extract_from_memory(&[]).is_err());
}

#[test]
fn returns_none_for_missing_file() {
    let bf = extracted(&[("exists.txt", &[1, 2, 3])]);
    assert!(bf.find("exists.txt", "").is_some());
    assert!(bf.find("missing.txt", "").is_none());
}

#[test]
fn strips_leading_dot_slash_from_entry_names() {
    let bf = extracted(&[("./config.txt", &[1, 2, 3])]);
    assert!(bf.files().contains_key("config.txt"));
    assert!(bf.find("config.txt", "").is_some());
}

#[test]
fn handles_large_file() {
    let large = vec![0xAAu8; 1024 * 1024];
    let bf = extracted(&[("large.bin", &large)]);
    let data = bf.find("large.bin", "").expect("present");
    assert_eq!(data.len(), large.len());
    assert_eq!((data[0], data[data.len() - 1]), (0xAA, 0xAA));
}

#[test]
fn does_not_crash_on_corrupt_data() {
    let mut garbage = vec![0xDEu8; 512];
    garbage[..2].copy_from_slice(&[0xBA, 0xAD]);
    let mut bf = Bootfiles::default();
    if bf.extract_from_memory(&garbage).is_ok() {
        assert!(bf.find("config.txt", "").is_none());
        assert!(bf.find("kernel.img", "").is_none());
    }
}

#[test]
fn does_not_crash_on_truncated_data() {
    let mut truncated = vec![0u8; 100];
    truncated[0] = b'f';
    let mut bf = Bootfiles::default();
    if bf.extract_from_memory(&truncated).is_ok() {
        assert!(bf.find("config.txt", "").is_none());
    }
}

#[test]
fn reports_error_on_empty_archive() {
    let err = Bootfiles::default()
        .extract_from_memory(&[])
        .expect_err("empty");
    assert!(!err.is_empty());
}

#[test]
fn find_with_and_without_dot_slash_are_equivalent() {
    let bf = extracted(&[("kernel.img", &[1, 2])]);
    assert!(bf.find("kernel.img", "").is_some());
    assert!(bf.find("./kernel.img", "").is_some());
}

#[test]
fn extracts_multiple_files_and_iterates() {
    let bf = extracted(&[("a.txt", &[1]), ("b.txt", &[2]), ("c.txt", &[3])]);
    assert_eq!(bf.len(), 3);
    let names: Vec<_> = bf.files().keys().map(String::as_str).collect();
    assert_eq!(names, vec!["a.txt", "b.txt", "c.txt"]);
}

#[test]
fn find_resolves_chip_specific_subdirectory_via_prefix() {
    let mcb: &[u8] = &[0xDE, 0xAD];
    let bf = extracted(&[("2712/mcb.bin", mcb), ("config.txt", &[0xBE, 0xEF])]);
    assert!(bf.find("config.txt", "").is_some());
    assert!(bf.find("mcb.bin", "").is_none());
    assert_eq!(bf.find("mcb.bin", "2712"), Some(mcb));
    assert!(bf.find("mcb.bin", "2711").is_none());
    assert!(bf.find("2712/mcb.bin", "").is_some());
}

#[test]
fn extract_from_memory_clears_previous_state() {
    let mut bf = extracted(&[("first.txt", &[1])]);
    assert!(bf.find("first.txt", "").is_some());
    assert_eq!(
        bf.extract_from_memory(&tar_of(&[("second.txt", &[2])])),
        Ok(())
    );
    assert!(bf.find("first.txt", "").is_none());
    assert!(bf.find("second.txt", "").is_some());
    assert_eq!(bf.len(), 1);
}

#[test]
fn extracts_from_file_and_skips_directories() {
    let dir = temp_dir();
    let mut tar = tar_of(&[("2711/", &[]), ("2711/bootcode4.bin", b"bc4")]);
    tar[156] = b'5'; // first entry is a directory
    let sum: u32 = tar[..512]
        .iter()
        .enumerate()
        .map(|(i, b)| {
            if (148..156).contains(&i) {
                32
            } else {
                u32::from(*b)
            }
        })
        .sum();
    tar[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
    write_file(dir.path(), "bootfiles.bin", &tar);
    let mut bf = Bootfiles::default();
    assert_eq!(
        bf.extract_from_file(&dir.path().join("bootfiles.bin")),
        Ok(())
    );
    assert_eq!(bf.len(), 1);
    assert_eq!(bf.find("bootcode4.bin", "2711"), Some(&b"bc4"[..]));
    let missing = bf.extract_from_file(&dir.path().join("absent.bin"));
    assert!(
        missing
            .expect_err("absent")
            .starts_with("Failed to open tar file: ")
    );
}
