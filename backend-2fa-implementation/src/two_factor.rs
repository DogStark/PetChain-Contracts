use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use totp_rs::{Algorithm, Secret, TOTP};

/// Configuration for TOTP parameters to ensure cryptographic agility
#[derive(Debug, Clone)]
pub struct TotpConfig {
    pub algorithm: Algorithm,
    pub digits: usize,
    pub period: u64,
    pub window: u8,
}

impl Default for TotpConfig {
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
    pub fn legacy_sha1() -> Self {
        Self {
            algorithm: Algorithm::SHA1,
            digits: 6,
            period: 30,
            window: 1,
        }
    }

    pub fn high_security() -> Self {
        Self {
            algorithm: Algorithm::SHA512,
            digits: 8,
            period: 30,
            window: 1,
        }
    }
}

#[derive(Clone, Debug)]
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
}

/// Returned after a successful backup-code recovery.
#[derive(Debug, Serialize, Deserialize)]
pub struct RecoveryResult {
    pub new_secret: String,
    pub new_backup_codes: Vec<String>,
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

    /// Setup 2FA with custom configuration
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
            Secret::Encoded(secret.clone())
                .to_bytes()
                .map_err(|e| e.to_string())?,
            Some(issuer.to_string()),
            user_email.to_string(),
        )
        .map_err(|e| e.to_string())?;

        let qr_code_base64 = format!(
            "data:image/png;base64,{}",
            totp.get_qr_base64().map_err(|e| e.to_string())?
        );
        let backup_codes = Self::generate_backup_codes(8);

        Ok(TwoFactorSetup {
            secret,
            qr_code_base64,
            backup_codes,
            config,
        })
    }

    /// Verify token with default configuration (SHA256)
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
            Secret::Encoded(secret.to_string())
                .to_bytes()
                .map_err(|e| e.to_string())?,
            None,
            String::new(),
        )
        .map_err(|e| e.to_string())?;

        totp.check_current(token).map_err(|e| e.to_string())
    }

    pub fn generate_backup_codes(count: usize) -> Vec<String> {
        let mut rng = thread_rng();
        let mut codes = HashSet::new();
        while codes.len() < count {
            codes.insert(format!(
                "{:04}-{:04}",
                rng.gen_range(0..10000),
                rng.gen_range(0..10000)
            ));
        }
        codes.into_iter().collect()
    }

    pub fn verify_backup_code(stored_codes: &[String], provided_code: &str) -> Option<usize> {
        stored_codes.iter().position(|code| code == provided_code)
    }

    /// Consume a backup code: removes it from the list if found and returns true.
    pub fn consume_backup_code(stored_codes: &mut Vec<String>, provided_code: &str) -> bool {
        if let Some(index) = Self::verify_backup_code(stored_codes, provided_code) {
            stored_codes.remove(index);
            true
        } else {
            false
        }
    }
}

/// Persistence abstraction for 2FA state (kept for compatibility)
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

impl InMemoryStore {
    pub fn clear(&self) {
        self.data.lock().unwrap().clear();
    }
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
            .cloned()
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
