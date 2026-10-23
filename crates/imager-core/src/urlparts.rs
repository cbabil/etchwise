//! Minimal URL splitting with the QUrl semantics upstream relies on:
//! a scheme is `[A-Za-z][A-Za-z0-9+.-]*:` and the host sits between `//`
//! and the next `/`, `?` or `#`, without user info or port.

/// Lower-cased scheme, or `None` when the string has no valid scheme.
pub fn scheme(url: &str) -> Option<String> {
    let (head, _) = url.split_once(':')?;
    let mut chars = head.chars();
    let first_ok = chars.next().is_some_and(|c| c.is_ascii_alphabetic());
    let rest_ok = chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    (first_ok && rest_ok).then(|| head.to_ascii_lowercase())
}

/// Host part of an authority-based URL, empty when absent.
pub fn host(url: &str) -> &str {
    let Some((_, after)) = url.split_once("://") else {
        return "";
    };
    let authority = after.split(['/', '?', '#']).next().unwrap_or_default();
    let without_user = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    if without_user.starts_with('[') {
        return without_user.split_inclusive(']').next().unwrap_or_default();
    }
    without_user.split(':').next().unwrap_or_default()
}

/// True for http(s) URLs.
pub fn is_http(scheme: &str) -> bool {
    scheme == "http" || scheme == "https"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_scheme_and_host() {
        assert_eq!(scheme("HTTPS://h/x").as_deref(), Some("https"));
        assert_eq!(scheme("no-scheme"), None);
        assert_eq!(scheme("1http://h"), None);
        assert_eq!(scheme("a b:x"), None);
        assert_eq!(host("https://user@host.example:8080/p?q#f"), "host.example");
        assert_eq!(host("https:///p"), "");
        assert_eq!(host("http://[::1]:80/"), "[::1]");
        assert_eq!(host("mailto:x"), "");
        assert!(is_http("http") && !is_http("file"));
    }
}
