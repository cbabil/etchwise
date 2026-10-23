//! Deterministic RSA test keys.

use std::sync::OnceLock;

use rsa::RsaPrivateKey;
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::pkcs8::LineEnding;
use rsa::rand_core::{CryptoRng, Error as RandError, RngCore};

/// Deterministic SplitMix64 stream so the test key is reproducible.
pub struct TestRng(pub u64);

impl RngCore for TestRng {
    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(8) {
            chunk.copy_from_slice(&self.next_u64().to_le_bytes()[..chunk.len()]);
        }
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RandError> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl CryptoRng for TestRng {}

pub fn test_key() -> &'static RsaPrivateKey {
    static KEY: OnceLock<RsaPrivateKey> = OnceLock::new();
    KEY.get_or_init(|| RsaPrivateKey::new(&mut TestRng(42), 2048).unwrap())
}

pub fn pem() -> String {
    test_key().to_pkcs1_pem(LineEnding::LF).unwrap().to_string()
}
