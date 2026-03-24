use serde::{Deserialize, Serialize};
use crate::two_factor::{TwoFactorAuth, TwoFactorData, TwoFactorSetup, RecoveryResult};

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

/// Response returned after a successful backup-code recovery.
/// The caller must re-enroll their authenticator app with `new_secret`
/// and store the `new_backup_codes` — all previous material is revoked.
#[derive(Debug, Serialize)]
pub struct RecoverWithBackupResponse {
    pub new_secret: String,
    pub new_backup_codes: Vec<String>,
    pub enabled: bool,
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
    //
    // Recovery policy:
    //  1. Validate the provided backup code against stored codes.
    //  2. On success, rotate the TOTP secret — the old secret is immediately invalid.
    //  3. Invalidate ALL remaining backup codes and issue a fresh set.
    //  4. Keep 2FA enabled; the user must re-enroll their authenticator app.
    //  5. Persist the new TwoFactorData to the database before returning.
    pub fn recover_with_backup(req: RecoverWithBackupRequest) -> Result<RecoverWithBackupResponse, String> {
        // Fetch from database
        // let mut two_factor_data = db.get_two_factor_data(&req.user_id)?;
        
        let backup_codes = vec!["1234-5678".to_string()]; // Get from DB

        if TwoFactorAuth::verify_backup_code(&backup_codes, &req.backup_code).is_none() {
            return Err("Invalid backup code".to_string());
        }

        // Rotate secret and invalidate all old codes (including remaining backup codes)
        let recovery = TwoFactorAuth::rotate_after_recovery();

        // Persist new state to database — replaces old secret and all old backup codes
        // db.update_two_factor_data(&req.user_id, &TwoFactorData {
        //     secret: recovery.new_secret.clone(),
        //     backup_codes: recovery.new_backup_codes.clone(),
        //     enabled: recovery.enabled,
        // })?;

        Ok(RecoverWithBackupResponse {
            new_secret: recovery.new_secret,
            new_backup_codes: recovery.new_backup_codes,
            enabled: recovery.enabled,
        })
    }
}
