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
// Issue #1460 — Adoption case audit export
// ---------------------------------------------------------------------------

/// Maximum number of audit entries returned in a single page.
pub const ADOPTION_AUDIT_MAX_PAGE_SIZE: u32 = 100;

/// A single adoption case transition record exposed to authorized auditors.
///
/// Applicant private data (names, contact details, documents) is deliberately
/// excluded; only the transition metadata, actor, and event id are surfaced.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdoptionCaseAuditEntry {
    /// Monotonic event id assigned when the transition was recorded.
    pub event_id: u64,
    /// Identifier of the adoption case the transition belongs to.
    pub case_id: String,
    /// Transition label, e.g. `submitted`, `under_review`, `approved`.
    pub transition: String,
    /// Actor (admin id) that performed the transition.
    pub actor: String,
    /// Unix timestamp (seconds) when the transition was recorded.
    pub timestamp: u64,
}

/// Paginated response for the adoption case audit export.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdoptionCaseAuditPage {
    pub entries: Vec<AdoptionCaseAuditEntry>,
    pub page: u32,
    pub page_size: u32,
    /// Total number of transitions recorded for the case.
    pub total: u64,
}

/// In-memory store for adoption case audit transitions.
///
/// Kept intentionally small and append-only so persistent state growth is
/// bounded by the number of recorded transitions.
#[derive(Debug, Default)]
pub struct AdoptionCaseAuditStore {
    entries: Vec<AdoptionCaseAuditEntry>,
    next_event_id: u64,
}

impl AdoptionCaseAuditStore {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_event_id: 1,
        }
    }

    /// Record a transition. Returns the assigned event id.
    pub fn record(
        &mut self,
        case_id: &str,
        transition: &str,
        actor: &str,
        timestamp: u64,
    ) -> u64 {
        let event_id = self.next_event_id;
        self.next_event_id = self.next_event_id.saturating_add(1);
        self.entries.push(AdoptionCaseAuditEntry {
            event_id,
            case_id: case_id.to_string(),
            transition: transition.to_string(),
            actor: actor.to_string(),
            timestamp,
        });
        event_id
    }

    /// Paginated audit view for a single case, ordered by event id ascending.
    pub fn page(
        &self,
        case_id: &str,
        page: u32,
        page_size: u32,
    ) -> AdoptionCaseAuditPage {
        let filtered: Vec<&AdoptionCaseAuditEntry> = self
            .entries
            .iter()
            .filter(|e| e.case_id == case_id)
            .collect();
        let total = filtered.len() as u64;
        let start = (page as usize).saturating_mul(page_size as usize);
        let entries = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .cloned()
            .collect();
        AdoptionCaseAuditPage {
            entries,
            page,
            page_size,
            total,
        }
    }
}

/// Admin handlers for the adoption case audit export (Issue #1460).
pub struct AdminAdoptionAuditHandlers {
    store: Arc<std::sync::Mutex<AdoptionCaseAuditStore>>,
}

impl AdminAdoptionAuditHandlers {
    pub fn new() -> Self {
        Self {
            store: Arc::new(std::sync::Mutex::new(AdoptionCaseAuditStore::new())),
        }
    }

    pub fn with_store(store: Arc<std::sync::Mutex<AdoptionCaseAuditStore>>) -> Self {
        Self { store }
    }

    /// Record a case transition. Used by the intake/transfer flows.
    pub fn record_transition(
        &self,
        case_id: &str,
        transition: &str,
        actor: &str,
        timestamp: u64,
    ) -> Result<u64, String> {
        if case_id.is_empty() || case_id.len() > 64 {
            return Err("case_id must be 1..=64 characters".to_string());
        }
        if transition.is_empty() || transition.len() > 64 {
            return Err("transition must be 1..=64 characters".to_string());
        }
        if actor.is_empty() || actor.len() > 64 {
            return Err("actor must be 1..=64 characters".to_string());
        }
        let mut store = self
            .store
            .lock()
            .map_err(|_| "adoption audit store poisoned".to_string())?;
        Ok(store.record(case_id, transition, actor, timestamp))
    }

    /// GET /admin/adoption/cases/{case_id}/audit — paginated audit export.
    ///
    /// Requires an authenticated admin. Applicant private data is never
    /// included in the returned entries.
    pub fn get_case_audit(
        &self,
        _admin: &AuthenticatedAdmin,
        case_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<AdoptionCaseAuditPage, String> {
        if case_id.is_empty() || case_id.len() > 64 {
            return Err("case_id must be 1..=64 characters".to_string());
        }
        if page_size == 0 || page_size > ADOPTION_AUDIT_MAX_PAGE_SIZE {
            return Err(format!(
                "page_size must be 1..={}",
                ADOPTION_AUDIT_MAX_PAGE_SIZE
            ));
        }
        let store = self
            .store
            .lock()
            .map_err(|_| "adoption audit store poisoned".to_string())?;
        Ok(store.page(case_id, page, page_size))
    }
}

impl Default for AdminAdoptionAuditHandlers {
    fn default() -> Self {
        Self::new()
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
pub struct AddIpRuleReques

/* … truncated 4458 chars — edit only what you need near the top … */
