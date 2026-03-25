use serde::{Deserialize, Serialize};
use crate::two_factor::{TwoFactorAuth, TwoFactorData, TwoFactorStore};

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

pub struct TwoFactorHandlers<S: TwoFactorStore> {
    store: S,
}

impl<S: TwoFactorStore> TwoFactorHandlers<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn store_ref(&self) -> &S {
        &self.store
    }

    /// POST /api/2fa/enable - Generate QR code and backup codes, persist as disabled until verified
    pub fn enable_two_factor(&self, req: EnableTwoFactorRequest) -> Result<EnableTwoFactorResponse, String> {
        let setup = TwoFactorAuth::setup(&req.email, "PetChain")?;

        self.store.save(&req.user_id, TwoFactorData {
            secret: setup.secret.clone(),
            backup_codes: setup.backup_codes.clone(),
            enabled: false,
            config: setup.config.clone(),
        })?;

        Ok(EnableTwoFactorResponse {
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
            backup_codes: setup.backup_codes,
        })
    }

    /// POST /api/2fa/verify - Verify token to activate 2FA
    pub fn verify_and_activate(&self, req: VerifyTwoFactorRequest) -> Result<bool, String> {
        let data = self.store.get(&req.user_id)?;

        if data.enabled {
            return Err("2FA is already enabled".to_string());
        }

        let is_valid = TwoFactorAuth::verify_token_with_config(&data.secret, &req.token, data.config)?;

        if is_valid {
            self.store.update_enabled(&req.user_id, true)?;
        }

        Ok(is_valid)
    }

    /// POST /api/auth/login/2fa - Verify 2FA token during login
    pub fn verify_login_token(&self, req: LoginWithTwoFactorRequest) -> Result<bool, String> {
        let data = self.store.get(&req.user_id)?;

        if !data.enabled {
            return Err("2FA is not enabled for this user".to_string());
        }

        TwoFactorAuth::verify_token_with_config(&data.secret, &req.token, data.config)
    }

    /// POST /api/2fa/disable - Disable 2FA after verifying token
    pub fn disable_two_factor(&self, req: DisableTwoFactorRequest) -> Result<bool, String> {
        let data = self.store.get(&req.user_id)?;

        if !data.enabled {
            return Err("2FA is not enabled for this user".to_string());
        }

        let is_valid = TwoFactorAuth::verify_token_with_config(&data.secret, &req.token, data.config)?;

        if is_valid {
            self.store.delete(&req.user_id)?;
        }

        Ok(is_valid)
    }

    /// POST /api/2fa/recover - Use backup code for recovery, invalidates used code
    pub fn recover_with_backup(&self, req: RecoverWithBackupRequest) -> Result<bool, String> {
        let data = self.store.get(&req.user_id)?;

        if !data.enabled {
            return Err("2FA is not enabled for this user".to_string());
        }

        if let Some(index) = TwoFactorAuth::verify_backup_code(&data.backup_codes, &req.backup_code) {
            let mut updated_codes = data.backup_codes.clone();
            updated_codes.remove(index);
            self.store.update_backup_codes(&req.user_id, updated_codes)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
