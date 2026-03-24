use serde::{Deserialize, Serialize};
use crate::two_factor::{TwoFactorAuth, TwoFactorStorage, TwoFactorData};

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
    pub fn enable_two_factor<S: TwoFactorStorage>(storage: &mut S, req: EnableTwoFactorRequest) -> Result<EnableTwoFactorResponse, String> {
        let setup = TwoFactorAuth::setup(&req.email, "PetChain")?;
        
        let data = TwoFactorData {
            secret: setup.secret.clone(),
            backup_codes: setup.backup_codes.clone(),
            enabled: false,
        };
        
        storage.save_two_factor_data(&req.user_id, data)?;
        
        Ok(EnableTwoFactorResponse {
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
            backup_codes: setup.backup_codes,
        })
    }

    // POST /api/2fa/verify - Verify token to complete 2FA setup
    pub fn verify_and_activate<S: TwoFactorStorage>(storage: &mut S, req: VerifyTwoFactorRequest) -> Result<bool, String> {
        let mut data = storage.get_two_factor_data(&req.user_id)?
            .ok_or_else(|| "2FA setup not found. Call enable first.".to_string())?;
        
        if TwoFactorAuth::verify_token(&data.secret, &req.token)? {
            data.enabled = true;
            storage.save_two_factor_data(&req.user_id, data)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // POST /api/auth/login/2fa - Verify 2FA token during login
    pub fn verify_login_token<S: TwoFactorStorage>(storage: &S, req: LoginWithTwoFactorRequest) -> Result<bool, String> {
        let data = storage.get_two_factor_data(&req.user_id)?
            .ok_or_else(|| "2FA not found for user".to_string())?;
        
        if !data.enabled {
            return Err("2FA is not enabled for this user".to_string());
        }
        
        TwoFactorAuth::verify_token(&data.secret, &req.token)
    }

    // POST /api/2fa/disable - Disable 2FA
    pub fn disable_two_factor<S: TwoFactorStorage>(storage: &mut S, req: DisableTwoFactorRequest) -> Result<bool, String> {
        let data = storage.get_two_factor_data(&req.user_id)?
            .ok_or_else(|| "2FA not found for user".to_string())?;

        if !data.enabled {
            return Err("2FA is not enabled".to_string());
        }

        if TwoFactorAuth::verify_token(&data.secret, &req.token)? {
            // Persistently disable: delete the stored secret and backup codes
            storage.delete_two_factor_data(&req.user_id)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // POST /api/2fa/recover - Use backup code for recovery
    pub fn recover_with_backup<S: TwoFactorStorage>(storage: &mut S, req: RecoverWithBackupRequest) -> Result<bool, String> {
        let mut data = storage.get_two_factor_data(&req.user_id)?
            .ok_or_else(|| "2FA not found for user".to_string())?;
        
        if !data.enabled {
            return Err("2FA is not enabled".to_string());
        }
        
        match TwoFactorAuth::verify_backup_code(&data.backup_codes, &req.backup_code)? {
            Some(index) => {
                // Remove the used backup code
                data.backup_codes.remove(index);
                storage.save_two_factor_data(&req.user_id, data)?;
                Ok(true)
            },
            None => Ok(false),
        }
    }
}
