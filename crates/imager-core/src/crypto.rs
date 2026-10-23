//! Password hashing for the user account (upstream `ImageWriter::crypt`):
//! yescrypt for OS releases from 2023-01-01, sha256crypt (`$5$`) before.
//! Both use upstream's bundled C code. Salts come from a CSPRNG instead of
//! upstream's time-seeded `mt19937`; the salt alphabet and lengths match.

use std::ffi::{CStr, CString, c_char};
use std::sync::Mutex;

use rand::Rng;

const SALT_CHARS: &[u8] = b"./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

unsafe extern "C" {
    fn yescrypt_crypt(key: *const c_char, salt: *const c_char) -> *mut c_char;
    fn sha256_crypt(key: *const c_char, salt: *const c_char) -> *mut c_char;
}

/// Both C functions return a pointer into a static buffer.
static C_HASHERS: Mutex<()> = Mutex::new(());

/// Hashes `password` for an OS released on `release_date` (`YYYY-MM-DD`).
/// Returns `None` if the password contains a NUL byte or hashing fails.
pub fn crypt_password<R: Rng + ?Sized>(
    password: &str,
    release_date: &str,
    rng: &mut R,
) -> Option<String> {
    if uses_yescrypt(release_date) {
        yescrypt(password, &salt(rng, 16))
    } else {
        sha256crypt(password, &format!("$5${}", salt(rng, 10)))
    }
}

/// yescrypt via upstream's wrapper. A plain `salt` is used as raw salt
/// bytes with upstream's parameters (`$y$jB5$...`); a `$y$` setting is used as-is.
pub fn yescrypt(password: &str, salt: &str) -> Option<String> {
    call(yescrypt_crypt, password, salt)
}

/// sha256crypt; `salt` includes the `$5$` prefix.
pub fn sha256crypt(password: &str, salt: &str) -> Option<String> {
    call(sha256_crypt, password, salt)
}

fn call(
    f: unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_char,
    password: &str,
    salt: &str,
) -> Option<String> {
    let (key, salt) = (CString::new(password).ok()?, CString::new(salt).ok()?);
    let _guard = C_HASHERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // SAFETY: both pointers are valid NUL-terminated strings for the call;
    // the result is null or a NUL-terminated static buffer, copied out while
    // the lock keeps other callers from overwriting it.
    let result = unsafe { f(key.as_ptr(), salt.as_ptr()) };
    if result.is_null() {
        return None;
    }
    // SAFETY: non-null result points at a NUL-terminated buffer (see above).
    let hash = unsafe { CStr::from_ptr(result) }.to_str().ok()?.to_owned();
    (!hash.is_empty()).then_some(hash)
}

fn salt<R: Rng + ?Sized>(rng: &mut R, len: usize) -> String {
    (0..len)
        .map(|_| char::from(SALT_CHARS[rng.random_range(0..SALT_CHARS.len())]))
        .collect()
}

/// `QDate::fromString(date, "yyyy-MM-dd")` is valid and on/after 2023-01-01.
pub(crate) fn uses_yescrypt(release_date: &str) -> bool {
    parse_iso_date(release_date).is_some_and(|date| date >= (2023, 1, 1))
}

fn parse_iso_date(s: &str) -> Option<(u32, u32, u32)> {
    let bytes = s.as_bytes();
    let shape_ok = bytes.len() == 10 && bytes[4] == b'-' && bytes[7] == b'-';
    let digits_ok = bytes
        .iter()
        .enumerate()
        .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit());
    if !shape_ok || !digits_ok {
        return None;
    }
    let (year, month, day) = (
        s[0..4].parse().ok()?,
        s[5..7].parse().ok()?,
        s[8..10].parse().ok()?,
    );
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    (1..=days).contains(&day).then_some((year, month, day))
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;

    #[test]
    fn yescrypt_matches_openwall_vector() {
        assert_eq!(
            yescrypt("pleaseletmein", "$y$jD5.7$LdJMENpBABJJ3hIHjB1Bi.").as_deref(),
            Some("$y$jD5.7$LdJMENpBABJJ3hIHjB1Bi.$HboGM6qPrsK.StKYGt6KErmUYtioHreJd98oIugoNB6")
        );
    }

    #[test]
    fn sha256crypt_matches_drepper_vector() {
        assert_eq!(
            sha256crypt("Hello world!", "$5$saltstring").as_deref(),
            Some("$5$saltstring$5B8vYYiY.CVt1RlTTf8KbXBH3hsxY/GNooZaBBGWEc5")
        );
    }

    #[test]
    fn picks_algorithm_by_release_date() {
        let mut rng = StdRng::seed_from_u64(3);
        let new = crypt_password("raspberry", "2026-10-06", &mut rng).unwrap();
        // Upstream's wrapper parameters (N=16384, r=8) encode as `jB5`.
        assert!(new.starts_with("$y$jB5$"), "{new}");
        assert_eq!(yescrypt("raspberry", &new).as_deref(), Some(new.as_str()));
        let old = crypt_password("raspberry", "2022-12-31", &mut rng).unwrap();
        assert!(old.starts_with("$5$") && old.len() == 3 + 10 + 1 + 43);
        assert_eq!(
            sha256crypt("raspberry", &old).as_deref(),
            Some(old.as_str())
        );
        assert!(
            crypt_password("raspberry", "", &mut rng)
                .unwrap()
                .starts_with("$5$")
        );
    }

    #[test]
    fn rejects_nul_bytes() {
        assert!(sha256crypt("a\0b", "$5$x").is_none());
    }

    #[test]
    fn validates_dates_like_qdate() {
        assert!(uses_yescrypt("2023-01-01"));
        assert!(uses_yescrypt("2024-02-29"));
        assert!(!uses_yescrypt("2023-02-29"));
        assert!(!uses_yescrypt("2023-1-01"));
        assert!(!uses_yescrypt("2023-13-01"));
        assert!(!uses_yescrypt("2023-04-31"));
        assert!(!uses_yescrypt("2023/01/01"));
        assert!(uses_yescrypt("2100-12-31") && !uses_yescrypt("2100-02-29"));
        assert_eq!(parse_iso_date("2000-02-29"), Some((2000, 2, 29)));
    }
}
