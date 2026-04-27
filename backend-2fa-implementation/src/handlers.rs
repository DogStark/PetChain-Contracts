#[cfg(not(test))]
use crate::db::PostgresTwoFactorStore;
use crate::rate_limiter::{InMemoryRateLimiter, RateLimitResult, RateLimiter};
use crate::two_factor::{InMemoryStore, TwoFactorAuth, TwoFactorData, TwoFactorStore};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

#[cfg(test)]
fn test_two_factor_store() -> &'static Arc<InMemoryStore> {
    static STORE: OnceLock<Arc<InMemoryStore>> = OnceLock::new();
    STORE.get_or_init(|| Arc::new(InMemoryStore::default()))
}

#[cfg(test)]
fn two_factor_store() -> Arc<dyn TwoFactorStore> {
    test_two_factor_store().clone()
}

#[cfg(not(test))]
fn two_factor_store() -> Arc<dyn TwoFactorStore> {
    static STORE: OnceLock<Arc<dyn TwoFactorStore>> = OnceLock::new();
    STORE
        .get_or_init(|| match std::env::var("DATABASE_URL") {
            Ok(database_url) => match PostgresTwoFactorStore::connect(&database_url) {
                Ok(store) => Arc::new(store),
                Err(_) => Arc::new(InMemoryStore::default()),
            },
            Err(_) => Arc::new(InMemoryStore::default()),
        })
        .clone()
}

fn store_insert(user_id: &str, data: TwoFactorData) -> Result<(), String> {
    two_factor_store().save(user_id, data)
}

fn store_get(user_id: &str) -> Result<TwoFactorData, String> {
    two_factor_store()
        .get(user_id)
        .map_err(|_| format!("2FA not configured for user {}", user_id))
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthenticatedUser {
    pub user_id: String,
}

impl AuthenticatedUser {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
        }
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

pub struct TwoFactorHandlers {
    limiter: Arc<dyn RateLimiter>,
}

impl TwoFactorHandlers {
    pub fn new() -> Self {
        Self {
            limiter: Arc::new(InMemoryRateLimiter::default()),
        }
    }

    pub fn with_limiter(limiter: Arc<dyn RateLimiter>) -> Self {
        Self { limiter }
    }

    pub fn enable_two_factor(
        caller: &AuthenticatedUser,
        req: EnableTwoFactorRequest,
    ) -> Result<EnableTwoFactorResponse, String> {
        caller.authorize(&req.user_id)?;

        {
            let store = two_factor_store();
            if let Ok(existing) = store.get(&req.user_id) {
                if existing.enabled {
                    return Err(
                        "2FA is already enabled. To re-enroll, you must first disable it."
                            .to_string(),
                    );
                }
        if let Ok(existing) = store_get(&req.user_id) {
            if existing.enabled {
                return Err(
                    "2FA is already enabled. To re-enroll, you must first disable it."
                        .to_string(),
                );
            }
        }

        let setup = TwoFactorAuth::setup(&req.email, "PetChain")?;

        store_insert(
            &req.user_id,
            TwoFactorData {
                secret: setup.secret.clone(),
                backup_codes: setup.backup_codes.clone(),
                enabled: false,
            },
        )?;

        Ok(EnableTwoFactorResponse {
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
            backup_codes: setup.backup_codes,
        })
    }

    pub fn verify_and_activate(
        &self,
        caller: &AuthenticatedUser,
        req: VerifyTwoFactorRequest,
    ) -> Result<bool, String> {
        caller.authorize(&req.user_id)?;

        let key = format!("verify:{}", req.user_id);
        if let RateLimitResult::Blocked { retry_after_secs } = self.limiter.record_failure(&key) {
            return Err(format!(
                "Too many failed attempts. Retry after {} seconds.",
                retry_after_secs
            ));
        }

        let data = store_get(&req.user_id)?;
        let result = TwoFactorAuth::verify_token(&data.secret, &req.token)?;
        if result {
            two_factor_store().update_enabled(&req.user_id, true)?;
        }

        if result {
            self.limiter.record_success(&key);
        }

        Ok(result)
    }

    pub fn verify_login_token(
        &self,
        caller: &AuthenticatedUser,
        req: LoginWithTwoFactorRequest,
    ) -> Result<bool, String> {
        caller.authorize(&req.user_id)?;

        let key = format!("login:{}", req.user_id);
        if let RateLimitResult::Blocked { retry_after_secs } = self.limiter.record_failure(&key) {
            return Err(format!(
                "Too many failed attempts. Retry after {} seconds.",
                retry_after_secs
            ));
        }

        let data = store_get(&req.user_id)?;
        if !data.enabled {
            return Ok(false);
        }

        let is_valid = TwoFactorAuth::verify_token(&data.secret, &req.token)?;

        if is_valid {
            self.limiter.record_success(&key);
        }

        Ok(is_valid)
    }

    pub fn disable_two_factor(
        &self,
        caller: &AuthenticatedUser,
        req: DisableTwoFactorRequest,
    ) -> Result<bool, String> {
        caller.authorize(&req.user_id)?;

        let key = format!("disable:{}", req.user_id);
        if let RateLimitResult::Blocked { retry_after_secs } = self.limiter.record_failure(&key) {
            return Err(format!(
                "Too many failed attempts. Retry after {} seconds.",
                retry_after_secs
            ));
        }

        let data = store_get(&req.user_id)?;
        if !data.enabled {
            return Ok(false);
        }

        let result = TwoFactorAuth::verify_token(&data.secret, &req.token)?;
        if result {
            two_factor_store().update_enabled(&req.user_id, false)?;
        }

        if result {
            self.limiter.record_success(&key);
        }

        Ok(result)
    }

    pub fn recover_with_backup(
        caller: &AuthenticatedUser,
        req: RecoverWithBackupRequest,
    ) -> Result<RecoverWithBackupResponse, String> {
        caller.authorize(&req.user_id)?;

        let data = store_get(&req.user_id)?;

        if !data.enabled {
            return Err("2FA not enabled for user".to_string());
        }

        let mut backup_codes = data.backup_codes.clone();
        if !TwoFactorAuth::consume_backup_code(&mut backup_codes, &req.backup_code) {
            return Err("Invalid backup code".to_string());
        }

        let setup = TwoFactorAuth::setup("recovery", "PetChain")?;

        store_insert(
            &req.user_id,
            TwoFactorData {
                secret: setup.secret.clone(),
                backup_codes: setup.backup_codes.clone(),
                enabled: true,
            },
        )?;

        Ok(RecoverWithBackupResponse {
            new_secret: setup.secret,
            new_backup_codes: setup.backup_codes,
            enabled: true,
        })
    }
}

impl Default for TwoFactorHandlers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub(crate) fn get_two_factor_data_for_tests(user_id: &str) -> Option<TwoFactorData> {
    two_factor_store().get(user_id).ok()
}

#[cfg(test)]
pub(crate) fn overwrite_two_factor_data_for_tests(user_id: &str, data: TwoFactorData) {
    let _ = two_factor_store().save(user_id, data);
}

#[cfg(test)]
pub(crate) fn clear_two_factor_store_for_tests() {
    test_two_factor_store().clear();
}
