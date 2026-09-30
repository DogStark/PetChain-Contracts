//! Admin endpoint handlers.
//!
//! Contains [`AdminDashboardHandlers`], [`AdminRecoveryHandlers`],
//! [`AdminScoreHandlers`], [`AdminRateLimitHandlers`],
//! [`AdminIpAccessHandlers`], and [`AdminWebhookHandlers`].
//!
//! [`AuthenticatedAdmin`] is defined in [`super::auth`] and re-exported here
//! for convenience.
use crate::dead_letter::DlqEntry;
use crate::ip_access::{IpAccessEntry, IpAccessStore, IpListType};
use crate::leaderboard::{FlaggedScoreStore, FlaggedScoreSubmission, InMemoryFlaggedScoreStore};
use crate::rate_limiter::UserQuotaStore;
use crate::two_factor::{AuditLogEntry, LockedUserSummary, UserTwoFactorSummary};
use crate::webhooks::{SecurityEventType, WebhookManager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::auth::{two_factor_store, RecoveryUsageLogEntry};

// Re-export AuthenticatedAdmin so callers can import it from either module.
pub use super::auth::AuthenticatedAdmin;

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

    /// GET /admin/users/{user_id}/2fa-summary — returns UserTwoFactorSummary.
    pub fn get_user_two_factor_summary(
        _admin: &AuthenticatedAdmin,
        user_id: &str,
    ) -> Result<UserTwoFactorSummary, String> {
        // Validate user_id
        if user_id.is_empty() {
            return Err("user_id must not be empty".to_string());
        }
        if user_id.len() > 64 {
            return Err("user_id must not exceed 64 characters".to_string());
        }

        let store = two_factor_store();
        let data = store.get(user_id)?;
        let is_canary = store.is_canary(user_id);
        Ok(UserTwoFactorSummary {
            user_id: user_id.to_string(),
            enabled: data.enabled,
            is_canary,
        })
    }
}

// ---------------------------------------------------------------------------
// Admin handlers for recovery code audit log
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Admin handlers for managing flagged leaderboard scores
// ---------------------------------------------------------------------------

/// Admin handlers for managing flagged leaderboard scores
pub struct AdminScoreHandlers {
    flagged_store: Arc<dyn FlaggedScoreStore>,
}

impl AdminScoreHandlers {
    pub fn new() -> Self {
        Self {
            flagged_store: Arc::new(InMemoryFlaggedScoreStore::new()),
        }
    }

    pub fn with_store(flagged_store: Arc<dyn FlaggedScoreStore>) -> Self {
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
        self.store.add_entry(
            &req.cidr,
            IpListType::Allow,
            req.note.as_deref(),
            &admin.admin_id,
        )
    }

    /// POST /admin/ip/block
    pub fn block_ip(
        &self,
        admin: &AuthenticatedAdmin,
        req: AddIpRuleRequest,
    ) -> Result<IpAccessEntry, String> {
        self.store.add_entry(
            &req.cidr,
            IpListType::Block,
            req.note.as_deref(),
            &admin.admin_id,
        )
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

// ---------------------------------------------------------------------------
// Issue #907 — Admin Webhook Configuration Handlers
// ---------------------------------------------------------------------------

/// Request body for `POST /admin/webhooks/configure`.
#[derive(Debug, Deserialize, Clone)]
pub struct ConfigureWebhookRequest {
    pub event_type: SecurityEventType,
    pub url: String,
}

/// A single entry in the webhook configuration list.
#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct WebhookConfigEntry {
    pub event_type: String,
    pub urls: Vec<String>,
}

/// Admin handlers for managing webhook subscriptions.
pub struct AdminWebhookHandlers {
    webhook_manager: Arc<WebhookManager>,
}

impl AdminWebhookHandlers {
    pub fn new(webhook_manager: Arc<WebhookManager>) -> Self {
        Self { webhook_manager }
    }

    /// POST /admin/webhooks/configure — register a URL for a security event type.
    pub fn configure(
        &self,
        _admin: &AuthenticatedAdmin,
        req: ConfigureWebhookRequest,
    ) -> Result<(), String> {
        self.webhook_manager
            .configure(req.event_type, req.url)
            .map_err(|e| e.to_string())
    }

    /// DELETE /admin/webhooks/{event_type} — remove all URLs for an event type.
    pub fn remove_config(
        &self,
        _admin: &AuthenticatedAdmin,
        event_type: &SecurityEventType,
    ) -> Result<(), String> {
        self.webhook_manager.remove_config(event_type);
        Ok(())
    }

    /// GET /admin/webhooks — list all configured event→URL mappings.
    pub fn list_configured_events(&self, _admin: &AuthenticatedAdmin) -> Vec<WebhookConfigEntry> {
        let mut entries: Vec<WebhookConfigEntry> = self
            .webhook_manager
            .list_configs()
            .into_iter()
            .map(|(event_type, urls)| WebhookConfigEntry { event_type, urls })
            .collect();
        entries.sort_by(|a, b| a.event_type.cmp(&b.event_type));
        entries
    }

    /// GET /admin/webhooks/dead-letter — return all DLQ entries (newest first).
    ///
    /// Each entry represents a webhook delivery that exhausted all retry
    /// attempts. The original payload and failure reason are included so
    /// operators can diagnose what went wrong.
    pub fn get_dead_letter_queue(&self, _admin: &AuthenticatedAdmin) -> Vec<DlqEntry> {
        self.webhook_manager.get_dead_letter_queue()
    }

    /// POST /admin/webhooks/dead-letter/replay — retry all DLQ entries.
    ///
    /// Each entry is re-delivered through the normal retry path. Entries that
    /// succeed are removed from the DLQ; entries that still fail remain with
    /// an incremented `replay_attempts` counter.
    ///
    /// Returns `(succeeded, failed)` counts.
    pub fn replay_dead_letter_queue(&self, _admin: &AuthenticatedAdmin) -> (usize, usize) {
        self.webhook_manager.replay_dead_letter_queue()
    }
}
