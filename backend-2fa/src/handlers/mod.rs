//! HTTP handler collection for the backend-2fa service, split by endpoint domain.
//!
//! ## Module layout
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`auth`] | Core 2FA user flows: enroll, verify, login, disable, recover, upgrade |
//! | [`admin`] | Admin dashboard, recovery log, score moderation, rate-limit quotas, IP access, webhooks |
//! | [`canary`] | Canary token creation and detection (Issue #713) |
//! | [`tenant`] | Multi-tenant scoped handlers and super-admin provisioning |
//! | [`pool`] | Connection-pool metrics and real-time leaderboard WebSocket |
//!
//! All public items are re-exported from this module so existing code that
//! imports from `crate::handlers::*` continues to compile unchanged.

pub mod admin;
pub mod auth;
pub mod canary;
pub mod pool;
pub mod tenant;

// ---------------------------------------------------------------------------
// Re-exports — preserve the public API that callers depend on
// ---------------------------------------------------------------------------

pub use admin::{
    AddIpRuleRequest, AdminDashboardHandlers, AdminIpAccessHandlers, AdminRateLimitHandlers,
    AdminRecoveryHandlers, AdminScoreHandlers, AdminWebhookHandlers,
    ConfigureWebhookRequest, GrantUnlimitedRequest, SetUserQuotaRequest, WebhookConfigEntry,
};
pub use auth::{
    AuthenticatedAdmin, AuthenticatedUser, DisableTwoFactorRequest, EnableTwoFactorRequest,
    EnableTwoFactorResponse, GetRecoveryLogQuery, LoginWithTwoFactorRequest,
    RecoverWithBackupRequest, RecoverWithBackupResponse, RecoveryLogCaller, RecoveryUsageLogEntry,
    RevokeSessionRequest, TwoFactorHandlers, UpgradeAlgorithmRequest, UpgradeAlgorithmResponse,
    VerifyTwoFactorRequest,
};
pub use canary::{CanaryHandlers, CreateCanaryRequest, CreateCanaryResponse};
pub use pool::{leaderboard_ws, PoolMetricsHandlers, PoolStatsResponse};
pub use tenant::{
    MultiTenantHandlers, ProvisionTenantRequest, ProvisionTenantResponse,
    TenantProvisioningHandlers,
};

// ---------------------------------------------------------------------------
// Test helpers — forwarded from submodules
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) use auth::{
    clear_idempotency_store_for_tests, clear_two_factor_store_for_tests,
    get_two_factor_data_for_tests, get_two_factor_store_for_tests, overwrite_two_factor_data_for_tests,
    test_two_factor_store,
};
