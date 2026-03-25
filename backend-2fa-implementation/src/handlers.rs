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

#[cfg(test)]
pub(crate) fn get_two_factor_data_for_tests(user_id: &str) -> Option<TwoFactorData> {
    two_factor_store()
        .lock()
        .ok()
        .and_then(|store| store.get(user_id).cloned())
}

#[cfg(test)]
pub(crate) fn overwrite_two_factor_data_for_tests(user_id: &str, data: TwoFactorData) {
    if let Ok(mut store) = two_factor_store().lock() {
        store.insert(user_id.to_string(), data);
    }
}

#[cfg(test)]
pub(crate) fn clear_two_factor_store_for_tests() {
    if let Ok(mut store) = two_factor_store().lock() {
        store.clear();
    }
}

#[cfg(test)]
pub(crate) fn overwrite_two_factor_data_for_tests(user_id: &str, data: TwoFactorData) {
    if let Ok(mut store) = two_factor_store().lock() {
        store.insert(user_id.to_string(), data);
    }
}

#[cfg(test)]
pub(crate) fn clear_two_factor_store_for_tests() {
    if let Ok(mut store) = two_factor_store().lock() {
        store.clear();
    }
}

impl Default for TwoFactorHandlers {
    fn default() -> Self {
        Self::new()
    }
}
