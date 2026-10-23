//! Size and version helpers shown in the UI, ported from upstream
//! `ImageWriter::formatSize` and `ImageWriter::isVersionNewer`.

/// Formats a byte count with binary units and up to `decimals` rounded
/// fractional digits; a zero fraction is omitted ("1.5 KB", "6 GB"). Unit
/// names are upstream's English source strings; the UI formats its own.
pub fn format_size(bytes: u64, decimals: u32) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let (unit, name) = (1..=4u32)
        .rev()
        .map(|power| (1024u64.pow(power), UNITS[power as usize]))
        .find(|(size, _)| bytes >= *size)
        .unwrap_or((1, UNITS[0]));
    let (unit, scale) = (u128::from(unit), 10u128.pow(decimals));
    let bytes = u128::from(bytes);
    let mut whole = bytes / unit;
    let mut frac = ((bytes % unit) * scale + unit / 2) / unit;
    if frac >= scale {
        whole += 1;
        frac -= scale;
    }
    if frac == 0 {
        return format!("{whole} {name}");
    }
    format!("{whole}.{frac:0width$} {name}", width = decimals as usize)
}

/// True when `version` (optionally `v`-prefixed) is newer than `current`,
/// comparing numeric segments like `QVersionNumber`.
pub fn is_version_newer(version: &str, current: &str) -> bool {
    parse_version(version) > parse_version(current)
}

/// `QVersionNumber::fromString`: dot-separated numbers; a segment with a
/// non-digit suffix contributes its digits and ends parsing. Trailing zero
/// segments are dropped so `2.0` equals `2.0.0`.
fn parse_version(v: &str) -> Vec<u64> {
    let v = v.strip_prefix(['v', 'V']).unwrap_or(v);
    let mut segments = Vec::new();
    for part in v.split('.') {
        let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
        let Ok(number) = digits.parse() else { break };
        segments.push(number);
        if digits.len() != part.len() {
            break;
        }
    }
    while segments.last() == Some(&0) {
        segments.pop();
    }
    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt(bytes: u64, decimals: u32) -> String {
        format_size(bytes, decimals)
    }

    #[test]
    fn formats_sizes_like_upstream() {
        assert_eq!(fmt(512, 1), "512 B");
        assert_eq!(fmt(1536, 1), "1.5 KB");
        assert_eq!(fmt(1024 * 1024, 1), "1 MB");
        assert_eq!(fmt(1_364_720_984, 1), "1.3 GB");
        assert_eq!(fmt(6_744_440_832, 0), "6 GB");
        assert_eq!(fmt(1024 * 1024 * 1024 - 1, 1), "1024 MB");
        assert_eq!(fmt(32_017_047_552, 2), "29.82 GB");
        assert_eq!(fmt(1u64 << 41, 1), "2 TB");
        assert_eq!(fmt(1075, 2), "1.05 KB");
        assert_eq!(fmt(1023 * 1024 + 600, 0), "1024 KB");
        assert_eq!(fmt(0, 0), "0 B");
    }

    #[test]
    fn compares_versions() {
        assert!(is_version_newer("2.0.12", "2.0.7"));
        assert!(is_version_newer("v2.1", "v2.0.7"));
        assert!(!is_version_newer("2.0.7", "v2.0.7"));
        assert!(!is_version_newer("2.0", "2.0.0"));
        assert!(!is_version_newer("1.9.99", "2.0.7"));
        assert!(is_version_newer("3.0rc1", "2.0.7"));
        assert!(!is_version_newer("garbage", "2.0.7"));
    }
}
