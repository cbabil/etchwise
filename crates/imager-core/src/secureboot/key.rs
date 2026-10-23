//! RSA-2048 key handling: `boot.sig` generation (upstream
//! `SecureBoot::generateBootSig`/`rsaSign`) and the key fingerprint shown in
//! the UI (upstream `getRsaKeyFingerprint`).

use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::traits::PublicKeyParts;
use rsa::{Pkcs1v15Sign, RsaPrivateKey};
use sha2::{Digest, Sha256};

use super::SecureBootError;

const MODULUS_BYTES: usize = 256;

/// Parses a PKCS#1 (`RSA PRIVATE KEY`) or PKCS#8 (`PRIVATE KEY`) PEM key and
/// checks that it is RSA-2048.
fn load_key(pem: &str) -> Result<RsaPrivateKey, SecureBootError> {
    let pem = pem.trim();
    let key = RsaPrivateKey::from_pkcs1_pem(pem)
        .or_else(|_| RsaPrivateKey::from_pkcs8_pem(pem))
        .map_err(|e| SecureBootError::InvalidKey(e.to_string()))?;
    if key.size() != MODULUS_BYTES {
        return Err(SecureBootError::KeySize(key.size() * 8));
    }
    Ok(key)
}

/// The `boot.sig` text for `boot_img`:
/// `<sha256 hex>\nts: <timestamp>\nrsa2048: <signature hex>\n`, where the
/// signature is RSA PKCS#1 v1.5 over the SHA-256 digest of `boot_img`
/// (DigestInfo-wrapped, i.e. a standard RSA-SHA256 signature). Hex is
/// lower-case; the output is deterministic for a given key, image and time.
///
/// # Errors
/// Fails on an invalid or non-2048-bit key.
pub fn boot_sig(
    boot_img: &[u8],
    private_key_pem: &str,
    timestamp: u64,
) -> Result<String, SecureBootError> {
    let key = load_key(private_key_pem)?;
    let digest = Sha256::digest(boot_img);
    let signature = key.sign(Pkcs1v15Sign::new::<Sha256>(), &digest)?;
    Ok(format!(
        "{}\nts: {timestamp}\nrsa2048: {}\n",
        hex::encode(digest),
        hex::encode(signature)
    ))
}

/// The fingerprint upstream shows for the key: SHA-256 of the bootloader's
/// public-key format (modulus as 256 little-endian bytes, then the exponent
/// as 8 little-endian bytes), first 16 bytes as upper-case hex with a `:`
/// after the 8th, e.g. `0123456789ABCDEF:0123456789ABCDEF`.
///
/// # Errors
/// Fails on an invalid or non-2048-bit key.
pub fn key_fingerprint(private_key_pem: &str) -> Result<String, SecureBootError> {
    let key = load_key(private_key_pem)?;
    let mut blob = key.n().to_bytes_le();
    blob.resize(MODULUS_BYTES, 0);
    let mut exponent = key.e().to_bytes_le();
    if exponent.len() > 8 {
        return Err(SecureBootError::InvalidKey(
            "exponent exceeds 64 bits".into(),
        ));
    }
    exponent.resize(8, 0);
    blob.extend_from_slice(&exponent);
    let hash = Sha256::digest(&blob);
    Ok(format!(
        "{}:{}",
        hex::encode_upper(&hash[..8]),
        hex::encode_upper(&hash[8..16])
    ))
}
