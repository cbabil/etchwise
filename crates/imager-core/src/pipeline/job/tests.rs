use std::io::{BufRead, BufReader, Cursor, Write};
use std::net::TcpListener;

use fatfs::FatType;
use sha2::{Digest, Sha256};

use super::*;
use crate::fat::BootPartition;
use crate::testutil::{MIB, mbr_image};

fn xz(data: &[u8]) -> Vec<u8> {
    let mut enc = liblzma::write::XzEncoder::new(Vec::new(), 1);
    enc.write_all(data).unwrap();
    enc.finish().unwrap()
}

/// Serves `body` to every request (no Range support needed here).
fn serve(body: Vec<u8>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/os.img.xz", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap() > 2 {
                line.clear();
            }
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream
                .write_all(head.as_bytes())
                .and_then(|()| stream.write_all(&body));
        }
    });
    url
}

fn job(source: Source, image: &[u8]) -> Job {
    Job {
        source: Some(source),
        extract_size: image.len() as u64,
        write: WriteOptions {
            expected_sha256: Some(hex::encode(Sha256::digest(image))),
            buffer_size: MIB,
            ..WriteOptions::default()
        },
        verify: true,
        ..Job::default()
    }
}

#[test]
fn writes_a_local_xz_image_with_customisation() {
    let dir = tempfile::tempdir().unwrap();
    let image = mbr_image(FatType::Fat16, 16);
    let path = dir.path().join("os.img.xz");
    std::fs::write(&path, xz(&image)).unwrap();
    let mut device = Cursor::new(vec![0xAAu8; image.len() + 4 * MIB]);
    let plan = CustomisationPlan {
        init_format: "cloudinit".into(),
        cloudinit: "hostname: pi\n".into(),
        ..CustomisationPlan::default()
    };
    let progress = Arc::new(Progress::default());
    let record = run(
        &mut device,
        &Job {
            plan: Some(plan),
            ..job(Source::Local(path), &image)
        },
        &progress,
    )
    .unwrap();
    assert!(record.is_none(), "local files are never cached");
    assert_eq!(progress.state(), WriteState::Succeeded);
    assert_eq!(progress.snapshot().verify_now, image.len() as u64);
    let mut boot = BootPartition::open(device).unwrap();
    assert_eq!(
        boot.read_file("user-data").unwrap().unwrap(),
        b"#cloud-config\nhostname: pi\n"
    );
    let meta = String::from_utf8(boot.read_file("meta-data").unwrap().unwrap()).unwrap();
    assert!(meta.starts_with("instance-id: rpi-imager-"));
}

#[test]
fn caches_remote_downloads() {
    let dir = tempfile::tempdir().unwrap();
    let image = vec![7u8; 3 * MIB];
    let compressed = xz(&image);
    let url = serve(compressed.clone());
    let mut device = Cursor::new(vec![0u8; 8 * MIB]);
    let progress = Arc::new(Progress::default());
    let record = run(
        &mut device,
        &Job {
            cache_dir: Some(dir.path().to_path_buf()),
            ..job(Source::Remote(url), &image)
        },
        &progress,
    )
    .unwrap()
    .unwrap();
    assert_eq!(std::fs::read(&record.file).unwrap(), compressed);
    assert_eq!(record.file_sha256, hex::encode(Sha256::digest(&compressed)));
    assert_eq!(record.image_sha256, hex::encode(Sha256::digest(&image)));
    assert_eq!(device.get_ref()[..3 * MIB], image[..]);
}

#[test]
fn bad_hash_fails_and_drops_the_cache_file() {
    let dir = tempfile::tempdir().unwrap();
    let image = vec![7u8; 2 * MIB];
    let url = serve(xz(&image));
    let mut bad = job(Source::Remote(url), &image);
    bad.write.expected_sha256 = Some("00".repeat(32));
    bad.cache_dir = Some(dir.path().to_path_buf());
    let progress = Arc::new(Progress::default());
    let err = run(&mut Cursor::new(vec![0u8; 8 * MIB]), &bad, &progress).unwrap_err();
    assert!(matches!(err, WriteError::HashMismatch { .. }));
    assert_eq!(progress.state(), WriteState::Failed);
    assert!(!dir.path().join(CACHE_FILE_NAME).exists());
}

#[test]
fn size_checks_and_missing_sources() {
    assert!(matches!(
        check_sizes(10 * 512, 512),
        Err(WriteError::TooSmall { .. })
    ));
    assert!(matches!(
        check_sizes(1000, 0),
        Err(WriteError::NotSectorMultiple { size: 1000 })
    ));
    check_sizes(0, 0).unwrap();
    let progress = Arc::new(Progress::default());
    let err = run(
        &mut Cursor::new(vec![0u8; 4 * MIB]),
        &Job::default(),
        &progress,
    )
    .unwrap_err();
    assert!(matches!(err, WriteError::Decompress(_)), "{err}");
    let missing = Job {
        source: Some(Source::Local("/nonexistent/x.img".into())),
        ..Job::default()
    };
    assert!(run(&mut Cursor::new(vec![0u8; 4 * MIB]), &missing, &progress).is_err());
}

#[test]
fn cancelled_jobs_end_cancelled() {
    let progress = Arc::new(Progress::default());
    progress.cancel();
    let image = vec![1u8; MIB];
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("x.img");
    std::fs::write(&path, &image).unwrap();
    let err = run(
        &mut Cursor::new(vec![0u8; 4 * MIB]),
        &job(Source::Local(path), &image),
        &progress,
    )
    .unwrap_err();
    assert!(matches!(err, WriteError::Cancelled));
    assert_eq!(progress.state(), WriteState::Cancelled);
    assert!(instance_id().starts_with("rpi-imager-"));
}
