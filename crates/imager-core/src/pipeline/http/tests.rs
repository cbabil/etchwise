use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use super::*;

/// Serves `body` over HTTP/1.1 with Range support. The first `drops`
/// requests send only half the body and then close the connection.
fn serve(body: Vec<u8>, drops: usize, ignore_range: bool) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/image.img", listener.local_addr().unwrap());
    let requests = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&requests);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut start = 0usize;
            let mut user_agent = String::new();
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                    break;
                }
                let lower = line.to_ascii_lowercase();
                if let Some(r) = lower.strip_prefix("range: bytes=") {
                    start = r.trim().trim_end_matches('-').parse().unwrap();
                }
                if lower.starts_with("user-agent:") {
                    user_agent = line.clone();
                }
            }
            assert!(user_agent.contains("Etchwise/v"), "{user_agent}");
            let n = seen.fetch_add(1, Ordering::SeqCst);
            let start = if ignore_range { 0 } else { start };
            let rest = &body[start..];
            let status = if start > 0 {
                "206 Partial Content"
            } else {
                "200 OK"
            };
            let mut head = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n",
                rest.len()
            );
            if start > 0 {
                head += &format!(
                    "Content-Range: bytes {start}-{}/{}\r\n",
                    body.len() - 1,
                    body.len()
                );
            }
            stream.write_all(format!("{head}\r\n").as_bytes()).unwrap();
            let send = if n < drops {
                &rest[..rest.len() / 2]
            } else {
                rest
            };
            let _ = stream.write_all(send);
        }
    });
    (url, requests)
}

fn body() -> Vec<u8> {
    (0..300_000u32).map(|i| (i % 253) as u8).collect()
}

#[test]
fn downloads_and_counts_progress() {
    let (url, _) = serve(body(), 0, false);
    let progress = Arc::new(Progress::default());
    let mut dl = ResumableDownload::open(client().unwrap(), &url, Arc::clone(&progress)).unwrap();
    let mut out = Vec::new();
    dl.read_to_end(&mut out).unwrap();
    assert_eq!(out, body());
    let snap = progress.snapshot();
    assert_eq!((snap.download_now, snap.download_total), (300_000, 300_000));
}

#[test]
fn resumes_after_a_dropped_connection() {
    let (url, requests) = serve(body(), 1, false);
    let progress = Arc::new(Progress::default());
    let mut dl = ResumableDownload::open(client().unwrap(), &url, progress)
        .unwrap()
        .with_backoff(Duration::from_millis(1));
    let mut out = Vec::new();
    dl.read_to_end(&mut out).unwrap();
    assert_eq!(out, body());
    assert_eq!(requests.load(Ordering::SeqCst), 2);
}

#[test]
fn fails_when_the_server_cannot_resume() {
    let (url, _) = serve(body(), 1, true);
    let mut dl = ResumableDownload::open(client().unwrap(), &url, Arc::new(Progress::default()))
        .unwrap()
        .with_backoff(Duration::from_millis(1));
    let err = dl.read_to_end(&mut Vec::new()).unwrap_err();
    assert!(err.to_string().contains("cannot resume"), "{err}");
}

#[test]
fn gives_up_after_max_resumes_or_cancel() {
    let (url, _) = serve(body(), usize::MAX, false);
    let progress = Arc::new(Progress::default());
    let mut dl = ResumableDownload::open(client().unwrap(), &url, Arc::clone(&progress))
        .unwrap()
        .with_backoff(Duration::from_millis(1));
    assert!(dl.read_to_end(&mut Vec::new()).is_err());
    let (url, _) = serve(body(), usize::MAX, false);
    let mut dl = ResumableDownload::open(client().unwrap(), &url, Arc::clone(&progress)).unwrap();
    progress.cancel();
    assert!(dl.read_to_end(&mut Vec::new()).is_err());
}

#[test]
fn fetches_whole_documents() {
    let (url, _) = serve(b"{\"os_list\": []}".to_vec(), 0, false);
    let (data, final_url) = fetch(&client().unwrap(), &url).unwrap();
    assert_eq!(data, b"{\"os_list\": []}");
    assert_eq!(final_url, url);
    assert!(fetch(&client().unwrap(), "http://127.0.0.1:1/none").is_err());
}

#[test]
fn parses_content_range_and_user_agent() {
    assert_eq!(range_start("bytes 100-199/200"), Some(100));
    assert_eq!(range_start("items 1-2"), None);
    assert_eq!(
        user_agent(),
        format!("Mozilla/5.0 Etchwise/{IMAGER_VERSION}")
    );
}
