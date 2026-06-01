use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

pub const DEFAULT_EXPIRY_SECS: u64 = 30 * 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq)]
pub struct TrustedDevice {
    pub id:                 String,
    pub user_id:            String,
    pub device_fingerprint: String,
    pub token_hash:         String,
    pub issued_at:          u64,
    pub expires_at:         u64,
    pub label:              Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum DeviceVerificationResult {
    Trusted,
    Expired,
    NotFound,
    UserMismatch,
    InvalidSignature,
}

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Generates an HMAC-SHA256 device token bound to user + fingerprint + timestamp.
/// Uses the portable hmac construction with sha2 (already in Cargo.toml).
/// In production, swap for `hmac` crate for constant-time operations.
pub fn generate_device_token(secret: &[u8], user_id: &str, fingerprint: &str, issued_at: u64) -> String {
    let message = format!("{user_id}:{fingerprint}:{issued_at}");
    hmac_sha256_hex(secret, message.as_bytes())
}

/// Stores only the hash of the raw token — never the token itself.
pub fn hash_token(token: &str) -> String {
    let mut h = Sha256::new();
    h.update(token.as_bytes());
    hex_encode(h.finalize().as_slice())
}

pub fn verify_device_token(
    device:          &TrustedDevice,
    presented_token: &str,
    presenting_user: &str,
    _secret:         &[u8],
    now:             u64,
) -> DeviceVerificationResult {
    if device.user_id != presenting_user {
        return DeviceVerificationResult::UserMismatch;
    }
    if now >= device.expires_at {
        return DeviceVerificationResult::Expired;
    }
    if hash_token(presented_token) != device.token_hash {
        return DeviceVerificationResult::InvalidSignature;
    }
    DeviceVerificationResult::Trusted
}

// ─── Internal HMAC-SHA256 (ipad/opad construction via sha2) ──────────────────

fn hmac_sha256_hex(key: &[u8], message: &[u8]) -> String {
    const BLOCK: usize = 64;
    let mut k = [0u8; BLOCK];
    if key.len() <= BLOCK {
        k[..key.len()].copy_from_slice(key);
    } else {
        let mut h = Sha256::new();
        h.update(key);
        let digest = h.finalize();
        k[..32].copy_from_slice(&digest);
    }
    let mut ipad = [0u8; BLOCK];
    let mut opad = [0u8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] = k[i] ^ 0x36;
        opad[i] = k[i] ^ 0x5c;
    }
    let mut inner = Sha256::new();
    inner.update(&ipad);
    inner.update(message);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(&opad);
    outer.update(inner_hash);
    hex_encode(outer.finalize().as_slice())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &[u8] = b"test-secret-key";
    const USER:   &str  = "user_123";
    const FP:     &str  = "Mozilla/5.0|device-id-abc";

    fn make_device(token: &str, user: &str, issued: u64, expires: u64) -> TrustedDevice {
        TrustedDevice {
            id:                 "dev_1".into(),
            user_id:            user.into(),
            device_fingerprint: FP.into(),
            token_hash:         hash_token(token),
            issued_at:          issued,
            expires_at:         expires,
            label:              Some("Test Device".into()),
        }
    }

    #[test]
    fn token_generation_is_deterministic() {
        let t1 = generate_device_token(SECRET, USER, FP, 1_700_000_000);
        let t2 = generate_device_token(SECRET, USER, FP, 1_700_000_000);
        assert_eq!(t1, t2);
    }

    #[test]
    fn different_users_produce_different_tokens() {
        let t1 = generate_device_token(SECRET, "user_a", FP, 1_700_000_000);
        let t2 = generate_device_token(SECRET, "user_b", FP, 1_700_000_000);
        assert_ne!(t1, t2);
    }

    #[test]
    fn different_fingerprints_produce_different_tokens() {
        let t1 = generate_device_token(SECRET, USER, "fp_a", 1_700_000_000);
        let t2 = generate_device_token(SECRET, USER, "fp_b", 1_700_000_000);
        assert_ne!(t1, t2);
    }

    #[test]
    fn valid_token_returns_trusted() {
        let now    = 1_700_000_000u64;
        let token  = generate_device_token(SECRET, USER, FP, now);
        let device = make_device(&token, USER, now, now + DEFAULT_EXPIRY_SECS);
        assert_eq!(
            verify_device_token(&device, &token, USER, SECRET, now + 3600),
            DeviceVerificationResult::Trusted
        );
    }

    #[test]
    fn expired_token_returns_expired() {
        let issued  = 1_000_000u64;
        let expires = issued + DEFAULT_EXPIRY_SECS;
        let token   = generate_device_token(SECRET, USER, FP, issued);
        let device  = make_device(&token, USER, issued, expires);
        assert_eq!(
            verify_device_token(&device, &token, USER, SECRET, expires + 1),
            DeviceVerificationResult::Expired
        );
    }

    #[test]
    fn wrong_user_returns_user_mismatch() {
        let now    = 1_700_000_000u64;
        let token  = generate_device_token(SECRET, USER, FP, now);
        let device = make_device(&token, USER, now, now + DEFAULT_EXPIRY_SECS);
        assert_eq!(
            verify_device_token(&device, &token, "attacker", SECRET, now),
            DeviceVerificationResult::UserMismatch
        );
    }

    #[test]
    fn tampered_token_returns_invalid_signature() {
        let now    = 1_700_000_000u64;
        let token  = generate_device_token(SECRET, USER, FP, now);
        let device = make_device(&token, USER, now, now + DEFAULT_EXPIRY_SECS);
        assert_eq!(
            verify_device_token(&device, "tampered_token", USER, SECRET, now),
            DeviceVerificationResult::InvalidSignature
        );
    }

    #[test]
    fn hash_token_is_not_plaintext() {
        let token  = "my_raw_token";
        let hashed = hash_token(token);
        assert_ne!(hashed, token);
        assert_eq!(hashed.len(), 64);
    }
}
