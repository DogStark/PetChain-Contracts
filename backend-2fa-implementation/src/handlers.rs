use crate::rate_limiter::{InMemoryRateLimiter, RateLimitResult, RateLimiter};
use crate::two_factor::{TwoFactorAuth, TwoFactorData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

// ---------------------------------------------------------------------------
// In-memory persistence store (keyed by user_id)
// ---------------------------------------------------------------------------

fn two_factor_store() -> &'static Mutex<HashMap<String, TwoFactorData>> {
    static STORE: OnceLock<Mutex<HashMap<String, TwoFactorData>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn store_insert(user_id: &str, data: TwoFactorData) -> Result<(), String> {
    two_factor_store()
        .lock()
        .map_err(|_| "2FA storage lock poisoned".to_string())?
        .insert(user_id.to_string(), data);
    Ok(())
}

fn store_get(user_id: &str) -> Result<TwoFactorData, String> {
    two_factor_store()
        .lock()
        .map_err(|_| "2FA storage lock poisoned".to_string())?
        .get(user_id)
        .cloned()
        .ok_or_else(|| format!("2FA not configured for user {}", user_id))
}

fn store_get_mut_then<F, T>(user_id: &str, f: F) -> Result<T, String>
where
    F: FnOnce(&mut TwoFactorData) -> Result<T, String>,
{
    let mut guard = two_factor_store()
        .lock()
        .map_err(|_| "2FA storage lock poisoned".to_string())?;
    let data = guard
        .get_mut(user_id)
        .ok_or_else(|| format!("2FA not configured for user {}", user_id))?;
    f(data)
}

// ---------------------------------------------------------------------------
// AuthenticatedUser — constructed by middleware from a validated JWT/session
// ---------------------------------------------------------------------------

/// Represents a verified, authenticated caller — constructed by middleware
/// (e.g. from a validated JWT or session token) and passed into every handler.
///
/// Handlers must never trust `user_id` values that arrive in request bodies
/// directly; they must compare against this principal instead.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthenticatedUser {
    pub user_id: String,
}

impl AuthenticatedUser {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self { user_id: user_id.into() }
    }

    /// Returns `Err` if the request targets a different user than the caller.
    pub fn authorize(&self, requested_user_id: &str) -> Result<(), String> {
        if self.user_id != requested_user_id {
            return Err("Forbidden: you can only manage your own 2FA".to_string());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

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

/// Response returned after a successful backup-code recovery.
/// The caller must re-enroll their authenticator app with `new_secret`
/// and store the `new_backup_codes` — all previous material is revoked.
#[derive(Debug, Serialize)]
pub struct RecoverWithBackupResponse {
    pub new_secret: String,
    pub new_backup_codes: Vec<String>,
    pub enabled: bool,
}

// ---------------------------------------------------------------------------
// TwoFactorHandlers
// ---------------------------------------------------------------------------

pub struct TwoFactorHandlers {
    limiter: Arc<dyn RateLimiter>,
}

impl TwoFactorHandlers {
    pub fn new() -> Self {
        Self { limiter: Arc::new(InMemoryRateLimiter::default()) }
    }

    pub fn with_limiter(limiter: Arc<dyn RateLimiter>) -> Self {
        Self { limiter }
    }

    // POST /api/2fa/enable
    //
    // Generates a TOTP secret and backup codes, persists TwoFactorData
    // (enabled: false) keyed by user_id, and returns values consistent
    // with what was stored.
    pub fn enable_two_factor(
        caller: &AuthenticatedUser,
        req: EnableTwoFactorRequest,
    ) -> Result<EnableTwoFactorResponse, String> {
        caller.authorize(&req.user_id)?;

        // ISSUE #294: Don't re-disclose secrets if 2FA is already enabled
        {
            let store = two_factor_store()
                .lock()
                .map_err(|_| "2FA storage lock poisoned".to_string())?;
            if let Some(existing) = store.get(&req.user_id) {
                if existing.enabled {
                    return Err("2FA is already enabled. To re-enroll, you must first disable it.".to_string());
                }
            }
        }

        let setup = TwoFactorAuth::setup(&req.email, "PetChain")?;

        let data = TwoFactorData {
            secret: setup.secret.clone(),
            backup_codes: setup.backup_codes.clone(),
            enabled: false,
        };

        store_insert(&req.user_id, data)?;

        Ok(EnableTwoFactorResponse {
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
            backup_codes: setup.backup_codes,
        })
    }

    // POST /api/2fa/verify — verify token to complete 2FA setup
    pub fn verify_and_activate(
        &self,
        caller: &AuthenticatedUser,
        req: VerifyTwoFactorRequest,
    ) -> Result<bool, String> {
        caller.authorize(&req.user_id)?;

        let key = format!("verify:{}", req.user_id);
        if let RateLimitResult::Blocked { retry_after_secs } = self.limiter.record_failure(&key) {
            return Err(format!("Too many failed attempts. Retry after {} seconds.", retry_after_secs));
        }

        let result = store_get_mut_then(&req.user_id, |data| {
            let is_valid = TwoFactorAuth::verify_token(&data.secret, &req.token)?;
            if is_valid {
                data.enabled = true;
            }
            Ok(is_valid)
        })?;

        if result {
            self.limiter.record_success(&key);
        }

        Ok(result)
    }

    // POST /api/auth/login/2fa — verify 2FA token during login
    //
    // Note: login is a pre-auth flow — the caller has already passed password
    // verification and holds a short-lived pre-auth session token. Middleware
    // must still construct an AuthenticatedUser from that token so we can
    // confirm the user_id matches before accepting the TOTP token.
    pub fn verify_login_token(
        &self,
        caller: &AuthenticatedUser,
        req: LoginWithTwoFactorRequest,
    ) -> Result<bool, String> {
        caller.authorize(&req.user_id)?;

        let key = format!("login:{}", req.user_id);
        if let RateLimitResult::Blocked { retry_after_secs } = self.limiter.record_failure(&key) {
            return Err(format!("Too many failed attempts. Retry after {} seconds.", retry_after_secs));
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

    // POST /api/2fa/disable
    pub fn disable_two_factor(
        &self,
        caller: &AuthenticatedUser,
        req: DisableTwoFactorRequest,
    ) -> Result<bool, String> {
        caller.authorize(&req.user_id)?;

        let key = format!("disable:{}", req.user_id);
        if let RateLimitResult::Blocked { retry_after_secs } = self.limiter.record_failure(&key) {
            return Err(format!("Too many failed attempts. Retry after {} seconds.", retry_after_secs));
        }

        let result = store_get_mut_then(&req.user_id, |data| {
            if !data.enabled {
                return Ok(false);
            }
            let is_valid = TwoFactorAuth::verify_token(&data.secret, &req.token)?;
            if is_valid {
                data.enabled = false;
            }
            Ok(is_valid)
        })?;

        if result {
            self.limiter.record_success(&key);
        }

        Ok(result)
    }

    // POST /api/2fa/recover — use backup code for recovery
    //
    // Recovery policy:
    //  1. Validate the provided backup code against stored codes.
    //  2. On success, rotate the TOTP secret — the old secret is immediately invalid.
    //  3. Invalidate ALL remaining backup codes and issue a fresh set.
    //  4. Keep 2FA enabled; the user must re-enroll their authenticator app.
    //  5. Persist the new TwoFactorData to the store before returning.
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

// ---------------------------------------------------------------------------
// Test helpers (not part of the public API)
// ---------------------------------------------------------------------------

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
