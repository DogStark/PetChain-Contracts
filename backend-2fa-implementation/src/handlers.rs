use crate::two_factor::{TwoFactorAuth, TwoFactorData, TwoFactorSetup};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

fn two_factor_store() -> &'static Mutex<HashMap<String, TwoFactorData>> {
    static STORE: OnceLock<Mutex<HashMap<String, TwoFactorData>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn load_two_factor_data(user_id: &str) -> Result<TwoFactorData, String> {
    let store = two_factor_store()
        .lock()
        .map_err(|_| "2FA storage lock poisoned".to_string())?;

    store
        .get(user_id)
        .cloned()
        .ok_or_else(|| format!("2FA not configured for user {}", user_id))
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthenticatedUser {
    pub user_id: String,
}

impl AuthenticatedUser {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self { user_id: user_id.into() }
    }

    pub fn authorize(&self, requested_user_id: &str) -> Result<(), String> {
        if self.user_id != requested_user_id {
            return Err("Forbidden: you can only manage your own 2FA".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct VerifyTwoFactorRequest {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoginWithTwoFactorRequest {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DisableTwoFactorRequest {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RecoverWithBackupRequest {
    pub user_id: String,
    pub backup_code: String,
}

#[derive(Debug, Serialize)]
pub struct RecoverWithBackupResponse {
    pub new_secret: String,
    pub new_backup_codes: Vec<String>,
    pub enabled: bool,
}

pub struct TwoFactorHandlers;

impl TwoFactorHandlers {
    // POST /api/2fa/enable - Generate QR code and backup codes
    pub fn enable_two_factor(
        caller: &AuthenticatedUser,
        req: EnableTwoFactorRequest,
    ) -> Result<EnableTwoFactorResponse, String> {
        caller.authorize(&req.user_id)?;

        let mut store = two_factor_store()
            .lock()
            .map_err(|_| "2FA storage lock poisoned".to_string())?;

        // ISSUE #294: Don't re-disclose secrets if 2FA is already enabled
        if let Some(existing) = store.get(&req.user_id) {
            if existing.enabled {
                return Err("2FA is already enabled. To re-enroll, you must first disable it.".to_string());
            }
        }

        let setup = TwoFactorAuth::setup(&req.email, "PetChain")?;

        let data = TwoFactorData {
            secret: setup.secret.clone(),
            backup_codes: setup.backup_codes.clone(),
            enabled: false,
        };

        store.insert(req.user_id, data);

        Ok(EnableTwoFactorResponse {
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
            backup_codes: setup.backup_codes,
        })
    }

    // POST /api/2fa/verify - Verify token to complete 2FA setup
    pub fn verify_and_activate(
        caller: &AuthenticatedUser,
        req: VerifyTwoFactorRequest,
    ) -> Result<bool, String> {
        caller.authorize(&req.user_id)?;

        let mut store = two_factor_store()
            .lock()
            .map_err(|_| "2FA storage lock poisoned".to_string())?;

        let two_factor_data = store
            .get_mut(&req.user_id)
            .ok_or_else(|| format!("2FA not configured for user {}", req.user_id))?;

        let is_valid = TwoFactorAuth::verify_token(&two_factor_data.secret, &req.token)?;

        if is_valid {
            two_factor_data.enabled = true;
        }

        Ok(is_valid)
    }

    // POST /api/auth/login/2fa - Verify 2FA token during login
    pub fn verify_login_token(
        caller: &AuthenticatedUser,
        req: LoginWithTwoFactorRequest,
    ) -> Result<bool, String> {
        caller.authorize(&req.user_id)?;

        let two_factor_data = load_two_factor_data(&req.user_id)?;
        if !two_factor_data.enabled {
            return Err("2FA not enabled for user".to_string());
        }

        TwoFactorAuth::verify_token(&two_factor_data.secret, &req.token)
    }

    // POST /api/2fa/disable - Disable 2FA
    pub fn disable_two_factor(
        caller: &AuthenticatedUser,
        req: DisableTwoFactorRequest,
    ) -> Result<bool, String> {
        caller.authorize(&req.user_id)?;

        let mut store = two_factor_store()
            .lock()
            .map_err(|_| "2FA storage lock poisoned".to_string())?;

        let two_factor_data = store
            .get_mut(&req.user_id)
            .ok_or_else(|| format!("2FA not configured for user {}", req.user_id))?;

        if !two_factor_data.enabled {
            return Ok(false);
        }

        let is_valid = TwoFactorAuth::verify_token(&two_factor_data.secret, &req.token)?;

        if is_valid {
            two_factor_data.enabled = false;
        }

        Ok(is_valid)
    }

    // POST /api/2fa/recover - Use backup code for recovery
    pub fn recover_with_backup(
        caller: &AuthenticatedUser,
        req: RecoverWithBackupRequest,
    ) -> Result<RecoverWithBackupResponse, String> {
        caller.authorize(&req.user_id)?;

        let mut store = two_factor_store()
            .lock()
            .map_err(|_| "2FA storage lock poisoned".to_string())?;

        let two_factor_data = store
            .get_mut(&req.user_id)
            .ok_or_else(|| format!("2FA not configured for user {}", req.user_id))?;

        if !two_factor_data.enabled {
            return Err("2FA not enabled for user".to_string());
        }

        if TwoFactorAuth::consume_backup_code(&mut two_factor_data.backup_codes, &req.backup_code) {
            // Success: rotate secret and backup codes
            let setup = TwoFactorAuth::setup("RECOVERY_SESSION", "PetChain")?;
            
            two_factor_data.secret = setup.secret.clone();
            two_factor_data.backup_codes = setup.backup_codes.clone();
            two_factor_data.enabled = true; // Still enabled

            Ok(RecoverWithBackupResponse {
                new_secret: setup.secret,
                new_backup_codes: setup.backup_codes,
                enabled: true,
            })
        } else {
            Err("Invalid backup code".to_string())
        }
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
