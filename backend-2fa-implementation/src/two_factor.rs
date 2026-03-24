use totp_rs::{Algorithm, Secret, TOTP};
use qrcode::QrCode;
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorSetup {
    pub secret: String,
    pub qr_code_base64: String,
    pub backup_codes: Vec<String>,
    pub config: TotpConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorData {
    pub secret: String,
    pub backup_codes: Vec<String>,
    pub enabled: bool,
    pub config: TotpConfig,
}

pub struct TwoFactorAuth;

impl TwoFactorAuth {
    pub fn generate_secret() -> String {
        Secret::generate_secret().to_string()
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
            Some(issuer.to_string()),
            user_email.to_string(),
        ).map_err(|e| e.to_string())?;

        let qr_url = totp.get_qr_base64().map_err(|e| e.to_string())?;
        let backup_codes = Self::generate_backup_codes(8);

        Ok(TwoFactorSetup {
            secret,
            qr_code_base64: qr_url,
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
}
