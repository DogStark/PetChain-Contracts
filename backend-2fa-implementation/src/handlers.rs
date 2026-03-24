use serde::{Deserialize, Serialize};
use crate::two_factor::{TwoFactorAuth, TwoFactorError};

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
    // POST /api/2fa/enable
    pub fn enable_two_factor(req: EnableTwoFactorRequest) -> Result<EnableTwoFactorResponse, TwoFactorError> {
        let setup = TwoFactorAuth::setup(&req.email, "PetChain")?;
        // Store in database: user_id -> TwoFactorData { secret, backup_codes, enabled: false }
        Ok(EnableTwoFactorResponse {
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
            backup_codes: setup.backup_codes,
        })
    }

    // POST /api/2fa/verify
    pub fn verify_and_activate(req: VerifyTwoFactorRequest) -> Result<(), TwoFactorError> {
        // let two_factor_data = db.get_two_factor_data(&req.user_id).ok_or(TwoFactorError::NotFound)?;
        let secret = "PLACEHOLDER_SECRET";
        TwoFactorAuth::verify_token(secret, &req.token)?;
        // db.update_two_factor_enabled(&req.user_id, true)?;
        Ok(())
    }

    // POST /api/auth/login/2fa
    pub fn verify_login_token(req: LoginWithTwoFactorRequest) -> Result<(), TwoFactorError> {
        // let two_factor_data = db.get_two_factor_data(&req.user_id).ok_or(TwoFactorError::NotFound)?;
        let secret = "PLACEHOLDER_SECRET";
        TwoFactorAuth::verify_token(secret, &req.token)
    }

    // POST /api/2fa/disable
    pub fn disable_two_factor(req: DisableTwoFactorRequest) -> Result<(), TwoFactorError> {
        // let two_factor_data = db.get_two_factor_data(&req.user_id).ok_or(TwoFactorError::NotFound)?;
        let secret = "PLACEHOLDER_SECRET";
        TwoFactorAuth::verify_token(secret, &req.token)?;
        // db.delete_two_factor_data(&req.user_id)?;
        Ok(())
    }

    // POST /api/2fa/recover
    pub fn recover_with_backup(req: RecoverWithBackupRequest) -> Result<(), TwoFactorError> {
        // let mut two_factor_data = db.get_two_factor_data(&req.user_id).ok_or(TwoFactorError::NotFound)?;
        let backup_codes = vec!["1234-5678".to_string()];
        TwoFactorAuth::verify_backup_code(&backup_codes, &req.backup_code)
            .map(|_| ())
            .ok_or(TwoFactorError::InvalidBackupCode)
    }
}
