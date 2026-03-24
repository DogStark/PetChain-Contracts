use totp_rs::{Algorithm, Secret, TOTP};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorSetup {
    pub secret: String,
    pub qr_code_base64: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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
        let mut rng = rand::thread_rng();
        let raw: [u8; 20] = rng.gen();
        // Encode as RFC 4648 base32 (no padding) — the format totp-rs expects
        base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &raw)
    }

    pub fn setup(user_email: &str, issuer: &str) -> Result<TwoFactorSetup, String> {
        let secret = Self::generate_secret();
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.clone()).to_bytes().map_err(|e| e.to_string())?,
            Some(issuer.to_string()),
            user_email.to_string(),
        ).map_err(|e| e.to_string())?;

        let qr_url = totp.get_qr_base64().map_err(|e| e.to_string())?;
        let backup_codes = Self::generate_backup_codes(8);

        Ok(TwoFactorSetup {
            secret,
            qr_code_base64: qr_url,
            backup_codes,
        })
    }

    pub fn verify_token(secret: &str, token: &str) -> Result<bool, String> {
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.to_string()).to_bytes().map_err(|e| e.to_string())?,
            None,
            String::new(),
        ).map_err(|e| e.to_string())?;

        Ok(totp.check_current(token).map_err(|e| e.to_string())?)
    }

    pub fn generate_backup_codes(count: usize) -> Vec<String> {
        let mut rng = rand::thread_rng();
        (0..count)
            .map(|_| {
                format!("{:04}-{:04}", rng.gen_range(0..10000), rng.gen_range(0..10000))
            })
            .collect()
    }

    pub fn verify_backup_code(stored_codes: &[String], provided_code: &str) -> Option<usize> {
        stored_codes.iter().position(|code| code == provided_code)
    }

    /// Executes the recovery policy after a valid backup code is consumed:
    /// - Rotates the TOTP secret (old secret is immediately invalid)
    /// - Invalidates ALL remaining backup codes and issues a fresh set
    /// - Keeps 2FA enabled so the account is not left unprotected
    ///
    /// Callers MUST persist the returned `RecoveryResult` to the database,
    /// replacing the previous `TwoFactorData` entirely.
    pub fn rotate_after_recovery() -> RecoveryResult {
        let new_secret = Self::generate_secret();
        let new_backup_codes = Self::generate_backup_codes(8);
        RecoveryResult {
            new_secret,
            new_backup_codes,
            enabled: true,
        }
    }
}
