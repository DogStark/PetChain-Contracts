use serde::{Deserialize, Serialize};
use crate::two_factor::{TwoFactorAuth, TwoFactorData, TwoFactorSetup};

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
        
        // Store in database: user_id -> TwoFactorData { secret, backup_codes, enabled: false }
        // Database call here
        
        Ok(EnableTwoFactorResponse {
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
            backup_codes: setup.backup_codes,
        })
    }

    // POST /api/2fa/verify - Verify token to complete 2FA setup
    pub fn verify_and_activate(req: VerifyTwoFactorRequest) -> Result<bool, String> {
        // Fetch from database: user_id -> TwoFactorData
        // let two_factor_data = db.get_two_factor_data(&req.user_id)?;
        
        // Placeholder - replace with actual DB fetch
        let secret = "PLACEHOLDER_SECRET"; // Get from DB
        
        let is_valid = TwoFactorAuth::verify_token(secret, &req.token)?;
        
        if is_valid {
            // Update database: set enabled = true
            // db.update_two_factor_enabled(&req.user_id, true)?;
        }
        
        Ok(is_valid)
    }

    // POST /api/auth/login/2fa - Verify 2FA token during login
    pub fn verify_login_token(req: LoginWithTwoFactorRequest) -> Result<bool, String> {
        // Fetch from database
        // let two_factor_data = db.get_two_factor_data(&req.user_id)?;
        
        let secret = "PLACEHOLDER_SECRET"; // Get from DB
        
        TwoFactorAuth::verify_token(secret, &req.token)
    }

    // POST /api/2fa/disable - Disable 2FA
    pub fn disable_two_factor(req: DisableTwoFactorRequest) -> Result<bool, String> {
        // Fetch from database
        // let two_factor_data = db.get_two_factor_data(&req.user_id)?;
        
        let secret = "PLACEHOLDER_SECRET"; // Get from DB
        let is_valid = TwoFactorAuth::verify_token(secret, &req.token)?;
        
        if is_valid {
            // Delete from database or set enabled = false
            // db.delete_two_factor_data(&req.user_id)?;
        }
        
        Ok(is_valid)
    }

    // POST /api/2fa/recover - Use backup code for recovery
    pub fn recover_with_backup(req: RecoverWithBackupRequest) -> Result<bool, String> {
        // Fetch from database
        // let mut two_factor_data = db.get_two_factor_data(&req.user_id)?;
        
        let backup_codes = vec!["1234-5678".to_string()]; // Get from DB
        
        if let Some(index) = TwoFactorAuth::verify_backup_code(&backup_codes, &req.backup_code) {
            // Remove used backup code from database
            // two_factor_data.backup_codes.remove(index);
            // db.update_two_factor_data(&req.user_id, &two_factor_data)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
