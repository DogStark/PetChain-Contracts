use totp_rs::{Algorithm, Secret, TOTP};
use rand::Rng;
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

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
        Secret::generate_secret().to_encoded().to_string()
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

        let qr_url = format!("data:image/png;base64,{}", totp.get_qr_base64().map_err(|e| e.to_string())?);
        let backup_codes = Self::generate_backup_codes(8);

        Ok(TwoFactorSetup {
            secret,
            qr_code_base64,
            backup_codes,
        })
    }

    pub fn validate_token_format(token: &str) -> Result<String, String> {
        let trimmed = token.trim();
        if trimmed.len() != 6 {
            return Err("Token must be exactly 6 digits".to_string());
        }
        if !trimmed.chars().all(|c| c.is_ascii_digit()) {
            return Err("Token must contain only digits".to_string());
        }
        Ok(trimmed.to_string())
    }

    pub fn validate_backup_code_format(code: &str) -> Result<String, String> {
        let trimmed = code.trim();
        // Format: dddd-dddd (9 characters)
        if trimmed.len() != 9 {
            return Err("Backup code must be in dddd-dddd format".to_string());
        }
        let parts: Vec<&str> = trimmed.split('-').collect();
        if parts.len() != 2 || parts[0].len() != 4 || parts[1].len() != 4 {
            return Err("Backup code must be in dddd-dddd format".to_string());
        }
        if !parts[0].chars().all(|c| c.is_ascii_digit()) || !parts[1].chars().all(|c| c.is_ascii_digit()) {
            return Err("Backup code must contain only digits and a hyphen".to_string());
        }
        Ok(trimmed.to_string())
    }

    pub fn verify_token(secret: &str, token: &str) -> Result<bool, String> {
        let token = Self::validate_token_format(token)?;
        
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

        let mut is_valid = 0u8;
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        for i in -1..=1 {
            let t = (time as i64 + i * 30) as u64;
            let expected = totp.generate(t);
            let expected_bytes = expected.as_bytes();
            let token_bytes = token.as_bytes();
            
            if expected_bytes.len() == token_bytes.len() {
                is_valid |= expected_bytes.ct_eq(token_bytes).unwrap_u8();
            }
        }

        Ok(is_valid == 1)
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

    pub fn verify_backup_code(stored_codes: &[String], provided_code: &str) -> Result<Option<usize>, String> {
        let provided_code = Self::validate_backup_code_format(provided_code)?;
        let provided_bytes = provided_code.as_bytes();
        let mut found_index = None;

        for (i, code) in stored_codes.iter().enumerate() {
            let code_bytes = code.as_bytes();
            if code_bytes.len() == provided_bytes.len() {
                if code_bytes.ct_eq(provided_bytes).unwrap_u8() == 1 {
                    found_index = Some(i);
                }
            }
        }
        Ok(found_index)
    }
}
