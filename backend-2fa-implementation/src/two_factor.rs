use totp_rs::{Algorithm, Secret, TOTP};
use rand::Rng;
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

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
        Secret::generate_secret().to_encoded().to_string()
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

        let qr_url = format!("data:image/png;base64,{}", totp.get_qr_base64().map_err(|e| e.to_string())?);
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

        // We check current, previous, and next windows to match totp-rs behavior
        // but we do it in constant-time.
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
            
            // Constant-time check of each generated token
            if expected_bytes.len() == token_bytes.len() {
                is_valid |= expected_bytes.ct_eq(token_bytes).unwrap_u8();
            }
        }

        Ok(is_valid == 1)
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
        let mut found_index = None;
        let provided_bytes = provided_code.as_bytes();

        for (i, code) in stored_codes.iter().enumerate() {
            let code_bytes = code.as_bytes();
            // Constant-time check of each code and iterate through ALL codes to avoid timing leaks
            if code_bytes.len() == provided_bytes.len() {
                if code_bytes.ct_eq(provided_bytes).unwrap_u8() == 1 {
                    found_index = Some(i);
                }
            }
        }
        found_index
    }
}
