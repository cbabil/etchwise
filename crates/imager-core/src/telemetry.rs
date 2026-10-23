//! Anonymous download statistics (upstream `DownloadStatsTelemetry`): one
//! form POST per remote download from the default repository, when the
//! `telemetry` setting is on. Fire and forget.

use std::fmt::Write as _;
use std::time::Duration;

use crate::IMAGER_VERSION;
use crate::pipeline::http;

/// Upstream `TELEMETRY_URL`.
pub const TELEMETRY_URL: &str = "https://rpi-imager-stats.raspberrypi.com/downloads";

/// What one report contains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report<'a> {
    /// Image URL.
    pub url: &'a str,
    /// Parent category name in the OS list.
    pub os: &'a str,
    /// OS entry name.
    pub image: &'a str,
    /// `QSysInfo::productType()`, e.g. `macos`.
    pub os_type: &'a str,
    /// `QSysInfo::productVersion()`, e.g. `26.0`.
    pub os_version: &'a str,
    /// `QSysInfo::currentCpuArchitecture()`, e.g. `arm64`.
    pub arch: &'a str,
    /// `QLocale().name()`, e.g. `en_GB`.
    pub locale: &'a str,
}

/// Upstream sends only for remote images while using the default repository.
pub fn should_send(
    telemetry_setting: bool,
    image_is_remote: bool,
    using_default_repo: bool,
) -> bool {
    telemetry_setting && image_is_remote && using_default_repo
}

/// The `x-www-form-urlencoded` body, field order as upstream.
pub fn body(report: &Report<'_>) -> String {
    let version = IMAGER_VERSION.strip_prefix('v').unwrap_or(IMAGER_VERSION);
    let fields = [
        ("url", report.url),
        ("os", report.os),
        ("image", report.image),
        ("imagerVersion", version),
        ("imagerOsType", report.os_type),
        ("imagerOsVersion", report.os_version),
        ("imagerOsArch", report.arch),
        ("imagerLocale", report.locale),
    ];
    fields
        .iter()
        .map(|(k, v)| format!("{k}={}", percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

/// Posts to [`TELEMETRY_URL`] on a background thread; failures are
/// ignored, as upstream.
pub fn send_in_background(body: String) -> std::thread::JoinHandle<()> {
    send_to(TELEMETRY_URL.to_owned(), body)
}

fn send_to(url: String, body: String) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let client = http::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build();
        if let Ok(client) = client {
            let _ = client
                .post(url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(body)
                .send();
        }
    })
}

/// `QUrl::toPercentEncoding`: everything but `A-Za-z0-9-._~` is escaped.
fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len() * 3);
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(b));
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_upstream_post_body() {
        let report = Report {
            url: "https://downloads.raspberrypi.com/a b.img.xz",
            os: "Raspberry Pi OS (other)",
            image: "Raspberry Pi OS Lite (64-bit)",
            os_type: "macos",
            os_version: "26.0",
            arch: "arm64",
            locale: "en_GB",
        };
        assert_eq!(
            body(&report),
            "url=https%3A%2F%2Fdownloads.raspberrypi.com%2Fa%20b.img.xz&os=Raspberry%20Pi%20OS%20%28other%29&image=Raspberry%20Pi%20OS%20Lite%20%2864-bit%29&imagerVersion=2.0.7&imagerOsType=macos&imagerOsVersion=26.0&imagerOsArch=arm64&imagerLocale=en_GB"
        );
        assert_eq!(percent_encode("é~"), "%C3%A9~");
    }

    #[test]
    fn posts_the_form_body() {
        use std::io::{BufRead, BufReader, Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/downloads", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let (mut length, mut line) = (0, String::new());
            while reader.read_line(&mut line).unwrap() > 2 {
                if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = v.trim().parse().unwrap();
                }
                line.clear();
            }
            let mut body = vec![0; length];
            reader.read_exact(&mut body).unwrap();
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .unwrap();
            String::from_utf8(body).unwrap()
        });
        send_to(url, "url=x&os=y".into()).join().unwrap();
        assert_eq!(server.join().unwrap(), "url=x&os=y");
    }

    #[test]
    fn sends_only_for_remote_default_repo_downloads() {
        assert!(should_send(true, true, true));
        assert!(!should_send(false, true, true));
        assert!(!should_send(true, false, true));
        assert!(!should_send(true, true, false));
    }
}
