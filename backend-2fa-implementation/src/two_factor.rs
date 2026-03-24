use totp_rs::{Algorithm, Secret, TOTP};
use rand::Rng;
use base32::{Alphabet, encode as b32encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub enum TwoFactorError {
    /// TOTP library / secret encoding failure
    SetupFailure(String),
    /// Token format is invalid (wrong length, non-numeric, etc.)
    InvalidToken,
    /// Token is well-formed but does not match the current window
    TokenMismatch,
    /// No 2FA record found for the user
    NotFound,
    /// Backup code was not found in the stored list
    InvalidBackupCode,
    /// Underlying storage / database error
    StorageError(String),
}

impl TwoFactorError {
    /// Map to an HTTP status code for use in the API layer.
    pub fn http_status(&self) -> u16 {
        match self {
            TwoFactorError::SetupFailure(_) => 500,
            TwoFactorError::InvalidToken => 422,
            // TokenMismatch and InvalidBackupCode both return 401;
            // callers must NOT distinguish "secret exists" from "token wrong".
            TwoFactorError::TokenMismatch => 401,
            TwoFactorError::NotFound => 401,
            TwoFactorError::InvalidBackupCode => 401,
            TwoFactorError::StorageError(_) => 500,
        }
    }
}

impl std::fmt::Display for TwoFactorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TwoFactorError::SetupFailure(_) => write!(f, "2FA setup failed"),
            TwoFactorError::InvalidToken => write!(f, "Invalid token format"),
            TwoFactorError::TokenMismatch => write!(f, "Authentication failed"),
            TwoFactorError::NotFound => write!(f, "Authentication failed"),
            TwoFactorError::InvalidBackupCode => write!(f, "Authentication failed"),
            TwoFactorError::StorageError(_) => write!(f, "Internal error"),
        }
    }
}

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

pub struct TwoFactorAuth;

impl TwoFactorAuth {
    pub fn generate_secret() -> String {
        let mut rng = rand::thread_rng();
        let raw: Vec<u8> = (0..20).map(|_| rng.gen::<u8>()).collect();
        b32encode(Alphabet::Rfc4648 { padding: false }, &raw)
    }

    pub fn setup(user_email: &str, issuer: &str) -> Result<TwoFactorSetup, TwoFactorError> {
        let secret = Self::generate_secret();
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.clone()).to_bytes().map_err(|e| TwoFactorError::SetupFailure(e.to_string()))?,
            Some(issuer.to_string()),
            user_email.to_string(),
        ).map_err(|e| TwoFactorError::SetupFailure(e.to_string()))?;

        let qr_url = totp.get_qr_base64().map_err(|e| TwoFactorError::SetupFailure(e.to_string()))?;
        let backup_codes = Self::generate_backup_codes(8);

        Ok(TwoFactorSetup {
            secret,
            qr_code_base64: qr_url,
            backup_codes,
        })
    }

    /// Verifies a TOTP token against the stored secret.
    /// Returns `Err(InvalidToken)` when the token is malformed and
    /// `Err(TokenMismatch)` when it is well-formed but incorrect.
    pub fn verify_token(secret: &str, token: &str) -> Result<(), TwoFactorError> {
        // Basic format guard: must be exactly 6 ASCII digits
        if token.len() != 6 || !token.chars().all(|c| c.is_ascii_digit()) {
            return Err(TwoFactorError::InvalidToken);
        }

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.to_string()).to_bytes().map_err(|e| TwoFactorError::SetupFailure(e.to_string()))?,
            None,
            String::new(),
        ).map_err(|e| TwoFactorError::SetupFailure(e.to_string()))?;

        let valid = totp.check_current(token).map_err(|e| TwoFactorError::SetupFailure(e.to_string()))?;
        if valid { Ok(()) } else { Err(TwoFactorError::TokenMismatch) }
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
