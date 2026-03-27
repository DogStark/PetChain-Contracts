use totp_rs::{Algorithm, Secret, TOTP};
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Configuration for TOTP parameters to ensure cryptographic agility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpConfig {
    /// Hash algorithm (SHA1, SHA256, SHA512)
    pub algorithm: Algorithm,
    /// Number of digits in the token (typically 6 or 8)
    pub digits: usize,
    /// Time window in seconds (typically 30)
    pub period: u64,
    /// Number of time windows to check (for clock skew tolerance)
    pub window: u8,
}

impl Default for TotpConfig {
    /// Default configuration using secure modern standards
    /// - SHA256 (more secure than SHA1)
    /// - 6 digits (standard)
    /// - 30 second period (standard)
    /// - 1 window tolerance (minimal clock skew)
    fn default() -> Self {
        Self {
            algorithm: Algorithm::SHA256,
            digits: 6,
            period: 30,
            window: 1,
        }
    }
}

impl TotpConfig {
    /// Legacy SHA1 configuration for backward compatibility
    pub fn legacy_sha1() -> Self {
        Self {
            algorithm: Algorithm::SHA1,
            digits: 6,
            period: 30,
            window: 1,
        }
    }

    /// High security configuration with SHA512 and 8 digits
    pub fn high_security() -> Self {
        Self {
            algorithm: Algorithm::SHA512,
            digits: 8,
            period: 30,
            window: 1,
        }
    }
}
use subtle::ConstantTimeEq;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TwoFactorSetup {
    pub secret: String,
    pub qr_code_base64: String,
    pub backup_codes: Vec<String>,
    pub config: TotpConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TwoFactorData {
    pub secret: String,
    pub backup_codes: Vec<String>,
    pub enabled: bool,
    pub config: TotpConfig,
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

    /// Setup 2FA with default configuration (SHA256)
    pub fn setup(user_email: &str, issuer: &str) -> Result<TwoFactorSetup, String> {
        Self::setup_with_config(user_email, issuer, TotpConfig::default())
    }

    /// Setup 2FA with custom configuration for cryptographic agility
    pub fn setup_with_config(
        user_email: &str,
        issuer: &str,
        config: TotpConfig,
    ) -> Result<TwoFactorSetup, String> {
        let secret = Self::generate_secret();
        let totp = TOTP::new(
            config.algorithm,
            config.digits,
            config.window,
            config.period,
            Secret::Encoded(secret.clone()).to_bytes().map_err(|e| e.to_string())?,
    pub fn setup(user_email: &str, issuer: &str) -> Result<TwoFactorSetup, TwoFactorError> {
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
            config,
        })
    }

    /// Verify token with default configuration (SHA256) - for backward compatibility
    pub fn verify_token(secret: &str, token: &str) -> Result<bool, String> {
        Self::verify_token_with_config(secret, token, TotpConfig::default())
    }

    /// Verify token with custom configuration
    pub fn verify_token_with_config(
        secret: &str,
        token: &str,
        config: TotpConfig,
    ) -> Result<bool, String> {
        let totp = TOTP::new(
            config.algorithm,
            config.digits,
            config.window,
            config.period,
            Secret::Encoded(secret.to_string()).to_bytes().map_err(|e| e.to_string())?,
        let token = match Self::validate_token_format(token) {
            Ok(t) => t,
            Err(_) => return Ok(false),
        };
        
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

        Ok(totp.check_current(token).map_err(|e| e.to_string())?)
    }

    pub fn generate_backup_codes(count: usize) -> Vec<String> {
        let mut rng = rand::thread_rng();
        let mut codes = std::collections::HashSet::new();
        while codes.len() < count {
            codes.insert(format!("{:04}-{:04}", rng.gen_range(0..10000), rng.gen_range(0..10000)));
        }
        codes.into_iter().collect()
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

/// Persistence abstraction for 2FA state
pub trait TwoFactorStore: Send + Sync {
    fn save(&self, user_id: &str, data: TwoFactorData) -> Result<(), String>;
    fn get(&self, user_id: &str) -> Result<TwoFactorData, String>;
    fn delete(&self, user_id: &str) -> Result<(), String>;
    fn update_enabled(&self, user_id: &str, enabled: bool) -> Result<(), String>;
    fn update_backup_codes(&self, user_id: &str, codes: Vec<String>) -> Result<(), String>;
}

/// In-memory implementation of TwoFactorStore for testing
#[derive(Default, Clone)]
pub struct InMemoryStore {
    data: Arc<Mutex<HashMap<String, TwoFactorData>>>,
}

impl TwoFactorStore for InMemoryStore {
    fn save(&self, user_id: &str, data: TwoFactorData) -> Result<(), String> {
        self.data.lock().unwrap().insert(user_id.to_string(), data);
        Ok(())
    }

    fn get(&self, user_id: &str) -> Result<TwoFactorData, String> {
        self.data
            .lock()
            .unwrap()
            .get(user_id)
            .map(|d| TwoFactorData {
                secret: d.secret.clone(),
                backup_codes: d.backup_codes.clone(),
                enabled: d.enabled,
                config: d.config.clone(),
            })
            .ok_or_else(|| format!("No 2FA data found for user: {}", user_id))
    }

    fn delete(&self, user_id: &str) -> Result<(), String> {
        self.data
            .lock()
            .unwrap()
            .remove(user_id)
            .ok_or_else(|| format!("No 2FA data found for user: {}", user_id))?;
        Ok(())
    }

    fn update_enabled(&self, user_id: &str, enabled: bool) -> Result<(), String> {
        let mut store = self.data.lock().unwrap();
        store
            .get_mut(user_id)
            .ok_or_else(|| format!("No 2FA data found for user: {}", user_id))
            .map(|d| d.enabled = enabled)
    }

    fn update_backup_codes(&self, user_id: &str, codes: Vec<String>) -> Result<(), String> {
        let mut store = self.data.lock().unwrap();
        store
            .get_mut(user_id)
            .ok_or_else(|| format!("No 2FA data found for user: {}", user_id))
            .map(|d| d.backup_codes = codes)
    }
}
