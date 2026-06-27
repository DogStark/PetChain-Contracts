#[cfg(not(test))]
use crate::db::PostgresTwoFactorStore;
use crate::error::ApiError;
use crate::leaderboard::{leaderboard_ws_endpoint, FlaggedScoreStore, FlaggedScoreSubmission};
use crate::rate_limiter::{
    InMemoryRateLimiter, RateLimitResult, RateLimiter, TenantRateLimitKey, UserQuotaStore,
};
use crate::two_factor::{
    AuditLogEntry, HmacAlgorithm, InMemoryStore, LockedUserSummary, TenantConfig, TenantRegistry,
    TenantScopedStore, TotpConfig, TwoFactorAuth, TwoFactorData, TwoFactorStore,
    UserTwoFactorSummary,
};
use crate::webhooks::{SecurityEventType, WebhookManager};
use actix_web::{web::Payload, Error, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(not(test))]
use std::sync::OnceLock;

fn verification_config(algorithm: HmacAlgorithm) -> TotpConfig {
match algorithm {
HmacAlgorithm::SHA512 => TotpConfig::high_security(),
HmacAlgorithm::SHA256 => TotpConfig::high_security(),
_ => TotpConfig::legacy_sha1(),
}
}

/// Verify a TOTP token with replay protection.
fn verify_token_with_replay_protection(
secret: &str,
token: &str,
config: TotpConfig,
last_used_step: Option<u64>,
) -> Result<bool, String> {
TwoFactorAuth::verify_token_with_config(secret, token, config, last_used_step)
}
(algorithm: HmacAlgorithm) -> TotpConfig {
    match algorithm {
        HmacAlgorithm::SHA512 => TotpConfig::high_security(),
        HmacAlgorithm::SHA256 => TotpConfig::high_security(),
        _ => TotpConfig::legacy_sha1(),
    }
}

#[cfg(test)]
fn test_two_factor_store() -> Arc<InMemoryStore> {
    std::thread_local! {
        static STORE: Arc<InMemoryStore> = Arc::new(InMemoryStore::default());
    }

    STORE.with(|store| store.clone())
}

#[cfg(test)]
fn two_factor_store() -> Arc<dyn TwoFactorStore> {
    test_two_factor_store()
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


const IDEMPOTENCY_TTL_SECS: u64 = 300; // 5 minutes

#[derive(Clone)]
struct IdempotencyEntry {
    response: EnableTwoFactorResponse,
    stored_at: u64,
}

#[cfg(test)]
fn test_idempotency_store() -> Arc<std::sync::Mutex<HashMap<String, IdempotencyEntry>>> {
    std::thread_local! {
        static STORE: Arc<std::sync::Mutex<HashMap<String, IdempotencyEntry>>> =
            Arc::new(std::sync::Mutex::new(HashMap::new()));
    }
    STORE.with(|store| store.clone())
}

#[cfg(test)]
fn idempotency_store() -> Arc<std::sync::Mutex<HashMap<String, IdempotencyEntry>>> {
    test_idempotency_store()
}

#[cfg(not(test))]
fn idempotency_store() -> Arc<std::sync::Mutex<HashMap<String, IdempotencyEntry>>> {
    static STORE: OnceLock<Arc<std::sync::Mutex<HashMap<String, IdempotencyEntry>>>> = OnceLock::new();
    STORE
        .get_or_init(|| Arc::new(std::sync::Mutex::new(HashMap::new())))
        .clone()
}

fn idempotency_key(user_id: &str, key: &str) -> String {
    format!("{}::{}", user_id, key)
}

fn current_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
pub(crate) fn clear_idempotency_store_for_tests() {
    test_idempotency_store().lock().unwrap().clear();
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

    pub fn authorize(&self, requested_user_id: &str) -> Result<(), ApiError> {
        if self.user_id != requested_user_id {
            return Err(ApiError::forbidden(
                "Forbidden: you can only manage your own 2FA",
                None,
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct EnableTwoFactorRequest {
    pub user_id: String,
    pub email: String,
    #[serde(default)]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct EnableTwoFactorResponse {
    pub secret: String,
    pub otpauth_uri: String,
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
    pub new_otpauth_uri: String,
    pub new_backup_codes: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct RecoveryUsageLogEntry {
    pub id: i32,
    pub user_id: String,
    pub code_index: i32,
    pub used_at: String,
    pub ip_address: Option<String>,
}

pub struct TwoFactorHandlers {
    limiter: Arc<dyn RateLimiter>,
    store: Arc<dyn TwoFactorStore>,
    issuer: String,
}

impl TwoFactorHandlers {
    const DEFAULT_LOCKOUT_THRESHOLD: u32 = 10;

    pub fn new() -> Self {
        Self {
            limiter: Arc::new(InMemoryRateLimiter::default()),
            store: two_factor_store(),
            issuer: "PetChain".to_string(),
        }
    }

    pub fn with_limiter(limiter: Arc<dyn RateLimiter>) -> Self {
        Self {
            limiter,
            store: two_factor_store(),
            issuer: "PetChain".to_string(),
        }
    }

    pub fn with_store(store: Arc<dyn TwoFactorStore>) -> Self {
        Self {
            limiter: Arc::new(InMemoryRateLimiter::default()),
            store,
            issuer: "PetChain".to_string(),
        }
    }

    pub fn with_store_and_limiter(
        store: Arc<dyn TwoFactorStore>,
        limiter: Arc<dyn RateLimiter>,
    ) -> Self {
        Self {
            limiter,
            store,
            issuer: "PetChain".to_string(),
        }
    }

    pub fn with_store_and_issuer(
        store: Arc<dyn TwoFactorStore>,
        issuer: impl Into<String>,
    ) -> Self {
        Self {
            limiter: Arc::new(InMemoryRateLimiter::default()),
            store,
            issuer: issuer.into(),
        }
    }

    fn rate_limit_key(prefix: &str, user_id: &str) -> String {
        format!("{}:{}", prefix, user_id)
    }

    fn store_get(&self, user_id: &str) -> Result<TwoFactorData, ApiError> {
        self.store.get(user_id).map_err(|_| {
            ApiError::not_found(format!("2FA not configured for user {}", user_id), None)
        })
    }

    fn ensure_not_locked(&self, user_id: &str) -> Result<(), ApiError> {
        let state = self
            .store
            .get_lockout_state(user_id)
            .map_err(|e| ApiError::internal_error(e, None))?;
        if state.locked {
            return Err(ApiError::locked(
                "2FA account locked after 10 failed attempts. Use admin unlock or a recovery code.",
                None,
            ));
        }
        Ok(())
    }

    fn record_failed_verification(&self, user_id: &str) -> Result<(), ApiError> {
        let state = self
            .store
            .record_failed_two_fa_attempt(user_id, Self::DEFAULT_LOCKOUT_THRESHOLD)
            .map_err(|e| ApiError::internal_error(e, None))?;
        if state.locked {
            return Err(ApiError::locked(
                "2FA account locked due to too many failed attempts. Use admin unlock or a recovery code.",
                None,
            ));
        }
        Ok(())
    }

    pub fn enable_two_factor(
        caller: &AuthenticatedUser,
        req: EnableTwoFactorRequest,
    ) -> Result<EnableTwoFactorResponse, ApiError> {
        Self::new().enroll(caller, req)
    }

    pub fn enroll(
        &self,
        caller: &AuthenticatedUser,
        req: EnableTwoFactorRequest,
    ) -> Result<EnableTwoFactorResponse, ApiError> {
        caller.authorize(&req.user_id)?;

        if let Some(key) = req.idempotency_key.as_deref() {
            let lookup = idempotency_key(&req.user_id, key);
            let store = idempotency_store();
            let guard = store.lock().unwrap();
            if let Some(entry) = guard.get(&lookup) {
                if current_unix_secs().saturating_sub(entry.stored_at) < IDEMPOTENCY_TTL_SECS {
                    return Ok(entry.response.clone());
                }
            }
        }

        if let Ok(existing) = self.store_get(&req.user_id) {
            if existing.enabled {
                return Err(ApiError::conflict(
                    "2FA is already enabled. To re-enroll, you must first disable it.",
                    None,
                ));
            }
        }

        let setup = TwoFactorAuth::setup(&req.email, &self.issuer)
            .map_err(|e| ApiError::internal_error(e, None))?;

        self.store
            .save(
                &req.user_id,
                TwoFactorData {
                    secret: setup.secret.clone(),
                    backup_codes: setup.backup_codes.clone(),
                    enabled: false,
                    algorithm: setup.config.algorithm,
                },
            )
            .map_err(|e| ApiError::internal_error(e, None))?;

        let response = EnableTwoFactorResponse {
            secret: setup.secret,
            otpauth_uri: setup.otpauth_uri,
            qr_code: setup.qr_code_base64,
            backup_codes: setup.backup_codes,
        };

        if let Some(key) = req.idempotency_key.as_deref() {
            let lookup = idempotency_key(&req.user_id, key);
            idempotency_store().lock().unwrap().insert(
                lookup,
                IdempotencyEntry {
                    response: response.clone(),
                    stored_at: current_unix_secs(),
                },
            );
        }

        Ok(response)
    }

    pub fn verify_and_activate(
        &self,
        caller: &AuthenticatedUser,
        req: VerifyTwoFactorRequest,
    ) -> Result<bool, ApiError> {
        caller.authorize(&req.user_id)?;

        self.ensure_not_locked(&req.user_id)?;
        let key = Self::rate_limit_key("verify", &req.user_id);
        let rate_result = self.limiter.record_failure(&key);
        if rate_result.is_blocked() {
            return Err(ApiError::too_many_requests(
                format!(
                    "Too many failed attempts. Retry after {} seconds.",
                    rate_result.retry_after_secs()
                ),
                None,
            ));
        }

        let data = self.store_get(&req.user_id)?;
        let result = TwoFactorAuth::verify_token_with_config(
            &data.secret,
            &req.token,
            verification_config(data.algorithm),
        )
        .map_err(|e| ApiError::internal_error(e, None))?;
        if result {
            self.store
                .update_enabled(&req.user_id, true)
                .map_err(|e| ApiError::internal_error(e, None))?;
            self.store
                .reset_two_fa_failures(&req.user_id)
                .map_err(|e| ApiError::internal_error(e, None))?;
            self.limiter.record_success(&key);
            return Ok(true);
        }

        self.record_failed_verification(&req.user_id)?;
        Ok(false)
    }

    pub fn verify_login_token(
        &self,
        caller: &AuthenticatedUser,
        req: LoginWithTwoFactorRequest,
    ) -> Result<bool, ApiError> {
        caller.authorize(&req.user_id)?;

        self.ensure_not_locked(&req.user_id)?;
        let key = Self::rate_limit_key("login", &req.user_id);
        let rate_result = self.limiter.record_failure(&key);
        if rate_result.is_blocked() {
            return Err(ApiError::too_many_requests(
                format!(
                    "Too many failed attempts. Retry after {} seconds.",
                    rate_result.retry_after_secs()
                ),
                None,
            ));
        }

        let data = self.store_get(&req.user_id)?;
        if !data.enabled {
            return Ok(false);
        }

        let is_valid = TwoFactorAuth::verify_token_with_config(
            &data.secret,
            &req.token,
            verification_config(data.algorithm),
        )
        .map_err(|e| ApiError::internal_error(e, None))?;

        if is_valid {
            self.store
                .reset_two_fa_failures(&req.user_id)
                .map_err(|e| ApiError::internal_error(e, None))?;
            self.limiter.record_success(&key);
            return Ok(true);
        }

        self.record_failed_verification(&req.user_id)?;
        Ok(false)
    }

    pub fn disable_two_factor(
        &self,
        caller: &AuthenticatedUser,
        req: DisableTwoFactorRequest,
    ) -> Result<bool, ApiError> {
        caller.authorize(&req.user_id)?;

        self.ensure_not_locked(&req.user_id)?;
        let key = Self::rate_limit_key("disable", &req.user_id);
        let rate_result = self.limiter.record_failure(&key);
        if rate_result.is_blocked() {
            return Err(ApiError::too_many_requests(
                format!(
                    "Too many failed attempts. Retry after {} seconds.",
                    rate_result.retry_after_secs()
                ),
                None,
            ));
        }

        let data = self.store_get(&req.user_id)?;
        if !data.enabled {
            return Ok(false);
        }

        let result = TwoFactorAuth::verify_token_with_config(
            &data.secret,
            &req.token,
            verification_config(data.algorithm),
        )
        .map_err(|e| ApiError::internal_error(e, None))?;
        if result {
            self.store
                .update_enabled(&req.user_id, false)
                .map_err(|e| ApiError::internal_error(e, None))?;
            self.store
                .reset_two_fa_failures(&req.user_id)
                .map_err(|e| ApiError::internal_error(e, None))?;
            self.limiter.record_success(&key);
            return Ok(true);
        }

        self.record_failed_verification(&req.user_id)?;
        Ok(false)
    }

    pub fn recover_with_backup(
        caller: &AuthenticatedUser,
        req: RecoverWithBackupRequest,
    ) -> Result<RecoverWithBackupResponse, ApiError> {
        Self::new().recover(caller, req, None)
    }

    pub fn recover_with_backup_with_ip(
        caller: &AuthenticatedUser,
        req: RecoverWithBackupRequest,
        ip_address: Option<&str>,
    ) -> Result<RecoverWithBackupResponse, ApiError> {
        Self::new().recover(caller, req, ip_address)
    }

    pub fn recover(
        &self,
        caller: &AuthenticatedUser,
        req: RecoverWithBackupRequest,
        ip_address: Option<&str>,
    ) -> Result<RecoverWithBackupResponse, ApiError> {
        caller.authorize(&req.user_id)?;

        let data = self.store_get(&req.user_id)?;

        if !data.enabled {
            return Err(ApiError::bad_request("2FA not enabled for user", None));
        }

        let backup_codes = &data.backup_codes;
        // Find the index of the provided backup code
        let code_index = match TwoFactorAuth::verify_backup_code(backup_codes, &req.backup_code) {
            Some(idx) => idx as i32,
            None => {
                return Err(ApiError::bad_request("InvalidRecoveryCode", None));
            }
        };

        // Check if code has already been used and log the usage atomically
        self.store
            .log_recovery_code_usage(&req.user_id, code_index, ip_address)
            .map_err(|e| {
                if e.contains("InvalidRecoveryCode") {
                    ApiError::bad_request("InvalidRecoveryCode", None)
                } else {
                    ApiError::internal_error(e, None)
                }
            })?;

        // Now consume the code and generate new secret
        let mut backup_codes = backup_codes.clone();
        TwoFactorAuth::consume_backup_code(&mut backup_codes, &req.backup_code);

        let setup = TwoFactorAuth::setup("recovery", &self.issuer)
            .map_err(|e| ApiError::internal_error(e, None))?;

        self.store
            .save(
                &req.user_id,
                TwoFactorData {
                    secret: setup.secret.clone(),
                    backup_codes: setup.backup_codes.clone(),
                    enabled: true,
                    algorithm: setup.config.algorithm,
                },
            )
            .map_err(|e| ApiError::internal_error(e, None))?;
        self.store
            .unlock_two_fa_account(&req.user_id, "recovery_code")
            .map_err(|e| ApiError::internal_error(e, None))?;

        Ok(RecoverWithBackupResponse {
            new_secret: setup.secret,
            new_otpauth_uri: setup.otpauth_uri,
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

/// Admin handlers for recovery code audit log
pub struct AdminRecoveryHandlers;

impl AdminRecoveryHandlers {
    /// Get recovery code usage log (admin-only endpoint would check authorization externally)
    pub fn get_recovery_log(
        page: u32,
        page_size: u32,
    ) -> Result<Vec<RecoveryUsageLogEntry>, String> {
        let entries = two_factor_store().get_recovery_usage_log(page, page_size)?;
        Ok(entries
            .into_iter()
            .map(|e| RecoveryUsageLogEntry {
                id: e.id as i32,
                user_id: e.user_id,
                code_index: e.code_index,
                used_at: e.used_at,
                ip_address: e.ip_address,
            })
            .collect())
    }
}

/// Admin handlers for managing flagged leaderboard scores
pub struct AdminScoreHandlers {
    flagged_store: Arc<FlaggedScoreStore>,
}

impl AdminScoreHandlers {
    pub fn new() -> Self {
        Self {
            flagged_store: Arc::new(FlaggedScoreStore::new()),
        }
    }

    pub fn with_store(flagged_store: Arc<FlaggedScoreStore>) -> Self {
        Self { flagged_store }
    }

    /// Get all flagged submissions
    pub fn get_all_flagged(&self) -> Vec<FlaggedScoreSubmission> {
        self.flagged_store.get_all_flagged()
    }

    /// Get flagged submissions for a specific user
    pub fn get_flagged_by_user(&self, user_id: &str) -> Vec<FlaggedScoreSubmission> {
        self.flagged_store.get_flagged_by_user(user_id)
    }

    /// Log a rejected score submission
    pub fn log_rejected_submission(&self, user_id: String, attempted_score: u64, reason: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let flagged = FlaggedScoreSubmission {
            user_id,
            attempted_score,
            timestamp,
            reason,
        };

        self.flagged_store.add_flagged(flagged);
    }

    /// Clear all flagged submissions (for testing)
    #[cfg(test)]
    pub fn clear_flagged(&self) {
        self.flagged_store.clear();
    }
}

impl Default for AdminScoreHandlers {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Admin rate-limit quota management
// ---------------------------------------------------------------------------

/// Request / response types for quota admin endpoints.
#[derive(Debug, Deserialize, Clone)]
pub struct SetUserQuotaRequest {
    pub user_id: String,
    pub requests_per_minute: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GrantUnlimitedRequest {
    pub user_id: String,
    /// Unix timestamp (seconds) until which the bypass is active.
    pub expires_at: u64,
}

/// Admin handlers for per-user rate-limit quota management.
pub struct AdminRateLimitHandlers {
    pub quota_store: Arc<UserQuotaStore>,
}

impl AdminRateLimitHandlers {
    pub fn new(quota_store: Arc<UserQuotaStore>) -> Self {
        Self { quota_store }
    }

    /// POST /admin/rate-limits/quota — set per-user requests-per-minute limit.
    /// Takes effect on the user's next request window.
    pub fn set_user_quota(
        &self,
        _admin: &AuthenticatedAdmin,
        req: SetUserQuotaRequest,
    ) -> Result<(), String> {
        self.quota_store
            .set_quota(&req.user_id, req.requests_per_minute);
        Ok(())
    }

    /// POST /admin/rate-limits/unlimited — grant temporary unlimited bypass.
    pub fn grant_unlimited(
        &self,
        _admin: &AuthenticatedAdmin,
        req: GrantUnlimitedRequest,
    ) -> Result<(), String> {
        self.quota_store
            .grant_unlimited(&req.user_id, req.expires_at);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Admin IP allowlist / blocklist management (Issue #701)
// ---------------------------------------------------------------------------

use crate::ip_access::{IpAccessEntry, IpAccessStore, IpListType};

/// Request body for `POST /admin/ip/allow` and `POST /admin/ip/block`.
#[derive(Debug, Deserialize, Clone)]
pub struct AddIpRuleRequest {
    pub cidr: String,
    pub note: Option<String>,
}

/// Admin handlers for managing the IP allowlist and blocklist consulted by
/// [`crate::ip_access::IpAccessMiddleware`] on every request.
pub struct AdminIpAccessHandlers {
    store: Arc<dyn IpAccessStore>,
}

impl AdminIpAccessHandlers {
    pub fn new(store: Arc<dyn IpAccessStore>) -> Self {
        Self { store }
    }

    /// POST /admin/ip/allow
    pub fn allow_ip(
        &self,
        admin: &AuthenticatedAdmin,
        req: AddIpRuleRequest,
    ) -> Result<IpAccessEntry, String> {
        self.store
            .add_entry(&req.cidr, IpListType::Allow, req.note.as_deref(), &admin.admin_id)
    }

    /// POST /admin/ip/block
    pub fn block_ip(
        &self,
        admin: &AuthenticatedAdmin,
        req: AddIpRuleRequest,
    ) -> Result<IpAccessEntry, String> {
        self.store
            .add_entry(&req.cidr, IpListType::Block, req.note.as_deref(), &admin.admin_id)
    }

    /// DELETE /admin/ip/{entry_id} — removes an entry from whichever list it's on.
    pub fn remove_entry(&self, _admin: &AuthenticatedAdmin, entry_id: i64) -> Result<(), String> {
        self.store.remove_entry(entry_id)
    }

    pub fn list_allow(&self) -> Vec<IpAccessEntry> {
        self.store.list_entries(IpListType::Allow)
    }

    pub fn list_block(&self) -> Vec<IpAccessEntry> {
        self.store.list_entries(IpListType::Block)
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

// ---------------------------------------------------------------------------
// Admin JWT scope check helper
// ---------------------------------------------------------------------------

/// Represents an authenticated admin caller (must have `admin` scope in JWT).
/// In a real HTTP layer the JWT would be validated by middleware; here we model
/// the scope as a field so handlers can enforce it without depending on a web
/// framework.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthenticatedAdmin {
    pub admin_id: String,
}

impl AuthenticatedAdmin {
    pub fn new(admin_id: impl Into<String>) -> Self {
        Self {
            admin_id: admin_id.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Issue #688 — Admin Dashboard Endpoint Suite
// ---------------------------------------------------------------------------

pub struct AdminDashboardHandlers;

impl AdminDashboardHandlers {
    /// GET /admin/users — paginated list of users with 2FA status.
    /// Canary accounts are excluded from this listing.
    pub fn list_users(
        _admin: &AuthenticatedAdmin,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<UserTwoFactorSummary>, String> {
        two_factor_store().list_users(page, page_size)
    }

    /// POST /admin/users/{id}/disable-2fa — force-disable with audit log entry.
    pub fn disable_two_fa(admin: &AuthenticatedAdmin, user_id: &str) -> Result<(), String> {
        two_factor_store().admin_disable_two_fa(user_id, &admin.admin_id)
    }

    /// POST /admin/users/{id}/unlock-2fa — clear persistent lockout state.
    pub fn unlock_two_fa(admin: &AuthenticatedAdmin, user_id: &str) -> Result<(), String> {
        two_factor_store().unlock_two_fa_account(user_id, &admin.admin_id)
    }

    /// GET /admin/locked-users — list all accounts currently in a locked state.
    pub fn list_locked_users(
        _admin: &AuthenticatedAdmin,
    ) -> Result<Vec<LockedUserSummary>, String> {
        two_factor_store().list_locked_users()
    }

    /// GET /admin/users/{id}/audit-log — full 2FA event history (paginated).
    pub fn get_audit_log(
        _admin: &AuthenticatedAdmin,
        user_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<AuditLogEntry>, String> {
        two_factor_store().get_audit_log(user_id, page, page_size)
    }
}

// ---------------------------------------------------------------------------
// Issue #713 — Canary Token Detection
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct CreateCanaryRequest {
    pub user_id: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct CreateCanaryResponse {
    pub user_id: String,
    pub secret: String,
    pub qr_code: String,
}

pub struct CanaryHandlers {
    webhook_manager: Arc<WebhookManager>,
}

impl CanaryHandlers {
    pub fn new(webhook_manager: Arc<WebhookManager>) -> Self {
        Self { webhook_manager }
    }

    /// Admin: create a canary TOTP account that looks real but triggers an
    /// alert when any verification is attempted.
    pub fn create_canary(
        admin: &AuthenticatedAdmin,
        req: CreateCanaryRequest,
    ) -> Result<CreateCanaryResponse, String> {
        let setup = TwoFactorAuth::setup(&req.email, "PetChain")?;

        two_factor_store().save(
            &req.user_id,
            TwoFactorData {
                secret: setup.secret.clone(),
                backup_codes: setup.backup_codes.clone(),
                enabled: true,
                algorithm: setup.config.algorithm,
            },
        )?;

        two_factor_store().set_canary(&req.user_id, true)?;

        two_factor_store().append_audit_log(
            &req.user_id,
            "canary_created",
            &admin.admin_id,
            None,
        )?;

        Ok(CreateCanaryResponse {
            user_id: req.user_id,
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
        })
    }

    /// Verify a TOTP token for a user. If the account is a canary, log a
    /// `CanaryTriggered` audit event and fire the webhook immediately.
    /// The canary account always returns `false` for the verification result
    /// so the attacker gets no useful feedback.
    pub fn verify_with_canary_check(
        &self,
        user_id: &str,
        token: &str,
        ip_address: Option<&str>,
    ) -> Result<bool, String> {
        let store = two_factor_store();

        if store.is_canary(user_id) {
            // Log the trigger event
            let meta = ip_address.map(|ip| format!("ip={}", ip));
            store.append_audit_log(user_id, "CanaryTriggered", user_id, meta.as_deref())?;

            // Fire webhook immediately
            let mut metadata = HashMap::new();
            if let Some(ip) = ip_address {
                metadata.insert("ip".to_string(), ip.to_string());
            }
            metadata.insert("user_id".to_string(), user_id.to_string());
            self.webhook_manager
                .fire(SecurityEventType::CanaryTriggered, user_id, metadata);

            // Return false — canary accounts never grant access
            return Ok(false);
        }

        let data = store.get(user_id)?;
        TwoFactorAuth::verify_token_with_config(
            &data.secret,
            token,
            verification_config(data.algorithm),
        )
    }
}

#[cfg(test)]
pub(crate) fn get_two_factor_store_for_tests() -> Arc<InMemoryStore> {
    test_two_factor_store()
}

// ---------------------------------------------------------------------------
// Multi-tenant support (Issue: multi-tenant 2FA)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct ProvisionTenantRequest {
    pub tenant_id: String,
    pub totp_issuer: String,
    pub rate_limit_max_failures: u32,
}

#[derive(Debug, Serialize)]
pub struct ProvisionTenantResponse {
    pub tenant_id: String,
    pub totp_issuer: String,
    pub rate_limit_max_failures: u32,
    /// `true` if `tenant_id` already existed and this call returned the
    /// existing tenant's config instead of creating a new one. Lets
    /// infrastructure automation safely retry `POST /tenant/provision`
    /// without erroring or creating duplicates.
    pub already_existed: bool,
}

/// Handlers that operate within a single tenant's namespace.
/// All user data is scoped to the tenant; cross-tenant access is rejected
/// at the `TenantScopedStore` level.
pub struct MultiTenantHandlers {
    store: TenantScopedStore,
    limiter: Arc<dyn RateLimiter>,
}

impl MultiTenantHandlers {
    pub fn new(store: TenantScopedStore) -> Self {
        Self {
            limiter: Arc::new(InMemoryRateLimiter::default()),
            store,
        }
    }

    pub fn with_limiter(store: TenantScopedStore, limiter: Arc<dyn RateLimiter>) -> Self {
        Self { store, limiter }
    }

    pub fn enable_two_factor(
        &self,
        caller: &AuthenticatedUser,
        user_id: &str,
        email: &str,
    ) -> Result<EnableTwoFactorResponse, String> {
        caller.authorize(user_id).map_err(|e| e.to_string())?;

        if let Ok(existing) = self.store.get(user_id) {
            if existing.enabled {
                return Err(
                    "2FA is already enabled. To re-enroll, you must first disable it.".to_string(),
                );
            }
        }

        let setup = TwoFactorAuth::setup(email, self.store.issuer())?;

        self.store.save(
            user_id,
            TwoFactorData {
                secret: setup.secret.clone(),
                backup_codes: setup.backup_codes.clone(),
                enabled: false,
                algorithm: setup.config.algorithm,
            },
        )?;

        Ok(EnableTwoFactorResponse {
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
            backup_codes: setup.backup_codes,
            otpauth_uri: setup.otpauth_uri,
        })
    }

    pub fn verify_and_activate(
        &self,
        caller: &AuthenticatedUser,
        user_id: &str,
        token: &str,
    ) -> Result<bool, String> {
        caller.authorize(user_id).map_err(|e| e.to_string())?;

        let max_failures = self.store.config.rate_limit_max_failures;
        let key = TenantRateLimitKey::new(
            &self.store.config.tenant_id,
            "verify",
            user_id,
        );
        if let RateLimitResult::Blocked { retry_after_secs, .. } = self.limiter.record_failure(key.as_str()) {
            return Err(format!(
                "Too many failed attempts. Retry after {} seconds.",
                retry_after_secs
            ));
        }
        let _ = max_failures; // per-tenant config available for custom limiter wiring

        let data = self.store.get(user_id)?;
        let result = TwoFactorAuth::verify_token_with_config(
            &data.secret,
            token,
            verification_config(data.algorithm),
        )?;
        if result {
            self.store.update_enabled(user_id, true)?;
            self.limiter.record_success(key.as_str());
        }
        Ok(result)
    }

    pub fn disable_two_factor(
        &self,
        caller: &AuthenticatedUser,
        user_id: &str,
        token: &str,
    ) -> Result<bool, String> {
        caller.authorize(user_id).map_err(|e| e.to_string())?;

        let key = TenantRateLimitKey::new(
            &self.store.config.tenant_id,
            "disable",
            user_id,
        );
        if let RateLimitResult::Blocked { retry_after_secs, .. } = self.limiter.record_failure(key.as_str()) {
            return Err(format!(
                "Too many failed attempts. Retry after {} seconds.",
                retry_after_secs
            ));
        }

        let data = self.store.get(user_id)?;
        if !data.enabled {
            return Ok(false);
        }
        let result = TwoFactorAuth::verify_token_with_config(
            &data.secret,
            token,
            verification_config(data.algorithm),
        )?;
        if result {
            self.store.update_enabled(user_id, false)?;
            self.limiter.record_success(key.as_str());
        }
        Ok(result)
    }
}

/// Super-admin handler for tenant provisioning.
pub struct TenantProvisioningHandlers {
    registry: Arc<TenantRegistry>,
}

impl TenantProvisioningHandlers {
    pub fn new(registry: Arc<TenantRegistry>) -> Self {
        Self { registry }
    }

    /// Provision a tenant (super-admin only — caller must be verified externally).
    ///
    /// Idempotent: calling this repeatedly with the same `tenant_id` never
    /// errors or creates a duplicate. The first call creates the tenant and
    /// returns `already_existed: false`; subsequent calls return the
    /// existing tenant's config with `already_existed: true`. This lets
    /// infrastructure automation safely retry provisioning on failure.
    pub fn provision_tenant(
        &self,
        _super_admin: &AuthenticatedAdmin,
        req: ProvisionTenantRequest,
    ) -> Result<ProvisionTenantResponse, String> {
        let config = TenantConfig {
            tenant_id: req.tenant_id.clone(),
            totp_issuer: req.totp_issuer.clone(),
            rate_limit_max_failures: req.rate_limit_max_failures,
        };
        let (existing_or_new, already_existed) = self.registry.provision(config)?;
        Ok(ProvisionTenantResponse {
            tenant_id: existing_or_new.tenant_id,
            totp_issuer: existing_or_new.totp_issuer,
            rate_limit_max_failures: existing_or_new.rate_limit_max_failures,
            already_existed,
        })
    }

    pub fn get_tenant_config(&self, tenant_id: &str) -> Option<TenantConfig> {
        self.registry.get_config(tenant_id)
    }
}
// Pool metrics endpoint
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct PoolStatsResponse {
    pub active: u32,
    pub idle: u32,
    pub max: u32,
}

pub struct PoolMetricsHandlers;

#[cfg(not(test))]
impl PoolMetricsHandlers {
    /// Return current pool utilisation. Only available when backed by Postgres
    /// and `POOL_STATS_ENABLED=1` is set in the environment.
    pub fn pool_stats() -> Result<PoolStatsResponse, String> {
        if std::env::var("POOL_STATS_ENABLED").as_deref() != Ok("1") {
            return Err("pool stats require direct access to PostgresTwoFactorStore; call store.pool_stats() directly".to_string());
        }
        match two_factor_store().try_pool_stats() {
            Some(stats) => Ok(PoolStatsResponse {
                active: stats.active,
                idle: stats.idle,
                max: stats.max,
            }),
            None => Err("pool stats require direct access to PostgresTwoFactorStore; call store.pool_stats() directly".to_string()),
        }
    }
}

#[cfg(test)]
impl PoolMetricsHandlers {
    pub fn pool_stats() -> Result<PoolStatsResponse, String> {
        // In tests there is no real pool; return a fixed sentinel so the
        // endpoint handler can be exercised without a database.
        Ok(PoolStatsResponse {
            active: 0,
            idle: 0,
            max: 0,
        })
    }
}

/// WebSocket endpoint for real-time leaderboard updates.
///
/// Mount this at `GET /leaderboard/ws`.
pub async fn leaderboard_ws(req: HttpRequest, stream: Payload) -> Result<HttpResponse, Error> {
    leaderboard_ws_endpoint(req, stream).await
}
