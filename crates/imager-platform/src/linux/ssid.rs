//! NetworkManager stores an SSID as raw bytes. Pure, tested on every host.

/// The SSID as text: trailing NULs dropped, empty or non-UTF-8 is none
/// (it could not be typed into the Wi-Fi form anyway).
pub fn ssid_text(bytes: &[u8]) -> Option<String> {
    let end = bytes.iter().rposition(|b| *b != 0).map_or(0, |i| i + 1);
    std::str::from_utf8(&bytes[..end])
        .ok()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_ssid_bytes_as_text() {
        assert_eq!(ssid_text(b"Home Wi-Fi").as_deref(), Some("Home Wi-Fi"));
        assert_eq!(ssid_text(b"cafe\0\0").as_deref(), Some("cafe"));
        assert_eq!(
            ssid_text("caf\u{e9}".as_bytes()).as_deref(),
            Some("caf\u{e9}")
        );
        assert_eq!(ssid_text(b""), None);
        assert_eq!(ssid_text(b"\0"), None);
        assert_eq!(ssid_text(&[0xff, 0xfe]), None);
    }
}
