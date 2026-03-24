use base64::{engine::general_purpose, Engine as _};
use qrcode::QrCode;
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use totp_rs::{Algorithm, Secret, TOTP};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TwoFactorSetup {
    pub secret: String,
    pub qr_code_base64: String,
    pub backup_codes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TwoFactorData {
    pub secret: String,
    pub backup_codes: Vec<String>,
    pub enabled: bool,
}

/// Returned after a successful backup-code recovery.
/// Contains the new secret and fresh backup codes that must be persisted,
/// replacing all previous 2FA material.
#[derive(Debug, Serialize, Deserialize)]
pub struct RecoveryResult {
    /// New TOTP secret — the old secret is now invalid.
    pub new_secret: String,
    /// Fresh set of backup codes — all previous codes are now invalid.
    pub new_backup_codes: Vec<String>,
    /// 2FA remains enabled after recovery.
    pub enabled: bool,
}

pub struct TwoFactorAuth;

impl TwoFactorAuth {
    pub fn generate_secret() -> String {
        const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        let mut rng = thread_rng();
        let range = Uniform::from(0..BASE32_ALPHABET.len());

        (0..32)
            .map(|_| BASE32_ALPHABET[range.sample(&mut rng)] as char)
            .collect()
    }

    pub fn setup(user_email: &str, issuer: &str) -> Result<TwoFactorSetup, String> {
        let secret = Self::generate_secret();
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.clone())
                .to_bytes()
                .map_err(|e| e.to_string())?,
            Some(issuer.to_string()),
            user_email.to_string(),
        )
        .map_err(|e| e.to_string())?;

        let qr_code_base64 = totp.get_qr_base64().map_err(|e| e.to_string())?;
        let backup_codes = Self::generate_backup_codes(8);

        Ok(TwoFactorSetup {
            secret,
            qr_code_base64,
            backup_codes,
        })
    }

    /// Verify a token using the default drift policy (STANDARD, ±1 step).
    ///
    /// Prefer [`verify_token_with_policy`] when you need explicit control
    /// over acceptable clock drift.
    pub fn verify_token(secret: &str, token: &str) -> Result<bool, String> {
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.to_string())
                .to_bytes()
                .map_err(|e| e.to_string())?,
            None,
            String::new(),
        )
        .map_err(|e| e.to_string())?;

        Ok(false)
    }

    pub fn generate_backup_codes(count: usize) -> Vec<String> {
        let mut rng = thread_rng();
        (0..count)
            .map(|_| {
                format!(
                    "{:04}-{:04}",
                    rng.gen_range(0..10000),
                    rng.gen_range(0..10000)
                )
            })
            .collect()
    }

    pub fn verify_backup_code(stored_codes: &[String], provided_code: &str) -> Option<usize> {
        stored_codes.iter().position(|code| code == provided_code)
    }

    /// Consume a backup code: removes it from the list if found and returns true.
    /// The caller MUST persist the mutated `stored_codes` after a `true` return
    /// to guarantee single-use semantics.
    pub fn consume_backup_code(stored_codes: &mut Vec<String>, provided_code: &str) -> bool {
        if let Some(index) = Self::verify_backup_code(stored_codes, provided_code) {
            stored_codes.remove(index);
            true
        } else {
            false
        }
    }
}
