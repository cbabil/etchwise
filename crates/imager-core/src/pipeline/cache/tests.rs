use std::io::{Cursor, Read};

use serde_json::json;

use super::*;

fn record(dir: &Path, contents: &[u8]) -> CacheRecord {
    let file = dir.join(CACHE_FILE_NAME);
    fs::write(&file, contents).unwrap();
    CacheRecord {
        file,
        image_sha256: "aa".into(),
        file_sha256: hex::encode(Sha256::digest(contents)),
    }
}

#[test]
fn tee_copies_and_hashes_the_stream() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(CACHE_FILE_NAME);
    let done = Arc::new(OnceLock::new());
    let mut tee = CacheTee::new(
        Cursor::new(b"compressed bytes".to_vec()),
        &path,
        Arc::clone(&done),
    );
    let mut out = Vec::new();
    tee.read_to_end(&mut out).unwrap();
    let hash = done.get().unwrap().clone();
    assert_eq!(out, b"compressed bytes");
    assert_eq!(fs::read(&path).unwrap(), b"compressed bytes");
    assert_eq!(hash, hex::encode(Sha256::digest(b"compressed bytes")));
    let failed = Arc::new(OnceLock::new());
    let mut unwritable = CacheTee::new(
        Cursor::new(vec![1]),
        &dir.path().join("missing/dir/file"),
        Arc::clone(&failed),
    );
    unwritable.read_to_end(&mut Vec::new()).unwrap();
    assert!(failed.get().is_none());
}

#[test]
fn records_round_trip_and_invalidate() {
    let dir = tempfile::tempdir().unwrap();
    let mut settings = Settings::in_memory();
    assert!(caching_enabled(&settings));
    assert!(load_record(&settings).is_none());
    let rec = record(dir.path(), b"data");
    save_record(&mut settings, &rec).unwrap();
    assert_eq!(load_record(&settings), Some(rec.clone()));
    assert!(verify_file(&rec).unwrap());
    assert!(is_cached(Some(&rec), true, "aa"));
    assert!(!is_cached(Some(&rec), false, "aa"));
    assert!(!is_cached(Some(&rec), true, "bb"));
    assert!(!is_cached(None, true, ""));
    invalidate(&mut settings, Some(&rec.file)).unwrap();
    assert!(load_record(&settings).is_none());
    assert!(!rec.file.exists());
}

#[test]
fn stale_or_empty_files_are_not_records() {
    let dir = tempfile::tempdir().unwrap();
    let mut settings = Settings::in_memory();
    let rec = record(dir.path(), b"");
    save_record(&mut settings, &rec).unwrap();
    assert!(load_record(&settings).is_none());
    let tampered = CacheRecord {
        file_sha256: "00".into(),
        ..record(dir.path(), b"x")
    };
    assert!(!verify_file(&tampered).unwrap());
    settings
        .set_group_value("caching", "enabled", json!(false))
        .unwrap();
    assert!(!caching_enabled(&settings));
}

#[test]
fn needs_five_gib_free_after_the_download() {
    let gib = 1024 * 1024 * 1024;
    assert!(has_room(7 * gib, 2 * gib));
    assert!(!has_room(7 * gib, 3 * gib));
    assert!(!has_room(gib, 2 * gib));
}
