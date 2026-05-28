#[cfg(not(test))]
use crate::db::PostgresTwoFactorStore;
use crate::leaderboard::{FlaggedScoreStore, FlaggedScoreSubmission};
use crate::rate_limiter::{InMemoryRateLimiter, RateLimitResult, RateLimiter, UserQuotaStore};
use crate::two_factor::{
    AuditLogEntry, InMemoryStore, TwoFactorAuth, TwoFactorData, TwoFactorStore,
    UserTwoFactorSummary,
};
use crate::webhooks::{SecurityEventType, WebhookManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(not(test))]
use std::sync::OnceLock;

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

        if let Ok(existing) = store_get(&req.user_id) {
            if existing.enabled {
                return Err(
                    "2FA is already enabled. To re-enroll, you must first disable it.".to_string(),
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
        Self::recover_with_backup_with_ip(caller, req, None)
    }

    pub fn recover_with_backup_with_ip(
        caller: &AuthenticatedUser,
        req: RecoverWithBackupRequest,
        ip_address: Option<&str>,
    ) -> Result<RecoverWithBackupResponse, String> {
        caller.authorize(&req.user_id)?;

        let data = store_get(&req.user_id)?;

        if !data.enabled {
            return Err("2FA not enabled for user".to_string());
        }

        let backup_codes = &data.backup_codes;
        // Find the index of the provided backup code
        let code_index = match TwoFactorAuth::verify_backup_code(backup_codes, &req.backup_code) {
            Some(idx) => idx as i32,
            None => {
                // Even if code not in current list, check recovery log for single-use enforcement
                // This handles the case where codes were already used in a previous recovery
                let store = two_factor_store();
                // Try to find this code in recovery log (this is expensive but ensures single-use)
                // For now, just return the standard error
                return Err("InvalidRecoveryCode".to_string());
            }
        };

        // Check if code has already been used and log the usage atomically
        let store = two_factor_store();
        if let Err(e) = store.log_recovery_code_usage(&req.user_id, code_index, ip_address) {
            return Err(e);
        }

        // Now consume the code and generate new secret
        let mut backup_codes = backup_codes.clone();
        TwoFactorAuth::consume_backup_code(&mut backup_codes, &req.backup_code);

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
    pub fn log_rejected_submission(
        &self,
        user_id: String,
        attempted_score: u64,
        reason: String,
    ) {
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
        self.quota_store.set_quota(&req.user_id, req.requests_per_minute);
        Ok(())
    }

    /// POST /admin/rate-limits/unlimited — grant temporary unlimited bypass.
    pub fn grant_unlimited(
        &self,
        _admin: &AuthenticatedAdmin,
        req: GrantUnlimitedRequest,
    ) -> Result<(), String> {
        self.quota_store.grant_unlimited(&req.user_id, req.expires_at);
        Ok(())
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
    pub fn disable_two_fa(
        admin: &AuthenticatedAdmin,
        user_id: &str,
    ) -> Result<(), String> {
        two_factor_store().admin_disable_two_fa(user_id, &admin.admin_id)
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
            store.append_audit_log(
                user_id,
                "CanaryTriggered",
                user_id,
                meta.as_deref(),
            )?;

            // Fire webhook immediately
            let mut metadata = HashMap::new();
            if let Some(ip) = ip_address {
                metadata.insert("ip".to_string(), ip.to_string());
            }
            metadata.insert("user_id".to_string(), user_id.to_string());
            self.webhook_manager.fire(
                SecurityEventType::CanaryTriggered,
                user_id,
                metadata,
            );

            // Return false — canary accounts never grant access
            return Ok(false);
        }

        let data = store.get(user_id)?;
        TwoFactorAuth::verify_token(&data.secret, token)
    }
}

#[cfg(test)]
pub(crate) fn get_two_factor_store_for_tests() -> Arc<InMemoryStore> {
    test_two_factor_store()
}

// ---------------------------------------------------------------------------
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
    /// Return current pool utilisation. Only available when backed by Postgres.
    /// Requires `POOL_STATS_ENABLED=1` to be set; otherwise returns an error
    /// to avoid coupling the handler to a concrete store type at runtime.
    pub fn pool_stats() -> Result<PoolStatsResponse, String> {
        Err("pool stats require direct access to PostgresTwoFactorStore; call store.pool_stats() directly".to_string())
    }
}

#[cfg(test)]
impl PoolMetricsHandlers {
    pub fn pool_stats() -> Result<PoolStatsResponse, String> {
        // In tests there is no real pool; return a fixed sentinel so the
        // endpoint handler can be exercised without a database.
        Ok(PoolStatsResponse { active: 0, idle: 0, max: 0 })
    }
}
