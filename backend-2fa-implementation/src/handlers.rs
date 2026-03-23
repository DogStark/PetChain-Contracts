use serde::{Deserialize, Serialize};
use crate::two_factor::TwoFactorAuth;

#[derive(Debug, Deserialize)]
pub struct EnableTwoFactorRequest {
    pub user_id: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct EnableTwoFactorResponse {
    pub secret: String,
    pub qr_code: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyTwoFactorRequest {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginWithTwoFactorRequest {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct DisableTwoFactorRequest {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct RecoverWithBackupRequest {
    pub user_id: String,
    pub backup_code: String,
}

pub struct TwoFactorHandlers;

impl TwoFactorHandlers {
    // POST /api/2fa/enable - Generate QR code and backup codes
    pub fn enable_two_factor(req: EnableTwoFactorRequest) -> Result<EnableTwoFactorResponse, String> {
        let setup = TwoFactorAuth::setup(&req.email, "PetChain")?;
        
        Ok(EnableTwoFactorResponse {
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
            backup_codes: setup.backup_codes,
        })
    }

    // POST /api/2fa/verify - Verify token to complete 2FA setup
    pub fn verify_and_activate(req: VerifyTwoFactorRequest) -> Result<bool, String> {
        let secret = "PLACEHOLDER_SECRET"; // Get from DB
        TwoFactorAuth::verify_token(secret, &req.token)
    }

    // POST /api/auth/login/2fa - Verify 2FA token during login
    pub fn verify_login_token(req: LoginWithTwoFactorRequest) -> Result<bool, String> {
        let secret = "PLACEHOLDER_SECRET"; // Get from DB
        TwoFactorAuth::verify_token(secret, &req.token)
    }

    // POST /api/2fa/disable - Disable 2FA
    pub fn disable_two_factor(req: DisableTwoFactorRequest) -> Result<bool, String> {
        let secret = "PLACEHOLDER_SECRET"; // Get from DB
        TwoFactorAuth::verify_token(secret, &req.token)
    }

    // POST /api/2fa/recover - Use backup code for recovery
    pub fn recover_with_backup(req: RecoverWithBackupRequest) -> Result<bool, String> {
        let backup_codes = vec!["1234-5678".to_string()]; // Get from DB
        
        match TwoFactorAuth::verify_backup_code(&backup_codes, &req.backup_code)? {
            Some(_index) => Ok(true),
            None => Ok(false),
        }
    }
}
