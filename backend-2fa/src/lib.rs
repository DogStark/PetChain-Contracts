pub mod db;
pub mod handlers;
pub mod leaderboard;
pub mod metrics;
pub mod rate_limiter;
pub mod tracing_middleware;
pub mod two_factor;
pub mod webhooks;

#[cfg(test)]
mod tests;

pub use db::PostgresTwoFactorStore;
pub use db::{select_secret_provider, AwsSecretsManagerProvider, EnvSecretProvider, SecretProvider, PoolStats};
pub use handlers::{
    AdminDashboardHandlers, AdminRateLimitHandlers, AdminScoreHandlers, AuthenticatedAdmin,
    AuthenticatedUser, CanaryHandlers, CreateCanaryRequest, CreateCanaryResponse,
    GrantUnlimitedRequest, PoolMetricsHandlers, PoolStatsResponse, SetUserQuotaRequest,
    TwoFactorHandlers, leaderboard_ws,
};
pub use leaderboard::{
    broadcast_score_update, FlaggedScoreStore, FlaggedScoreSubmission, LeaderboardEntry,
    LeaderboardScoreUpdate, LeaderboardWsHub, ScoreSubmissionError, ScoreValidationConfig,
};
pub use rate_limiter::{
    DistributedRateLimiter, EndpointConfig, InMemoryRateLimiter, LiveRedisBackend,
    MockRedisBackend, RateLimitResult, RateLimiter, RedisBackend, RedisRateLimiter,
    SlidingWindowRateLimiter,
};
pub use metrics::{
    metrics, record_rate_limit_hit, record_recovery_code_use, record_totp_verification,
    render_metrics, set_db_pool_stats, start_request_timer,
};
pub use tracing_middleware::sanitize_json_body;
pub use two_factor::{
    AuditLogEntry, InMemoryStore, RecoveryResult, TotpConfig, TwoFactorAuth, TwoFactorData,
    TwoFactorSetup, TwoFactorStore, UserTwoFactorSummary,
    TenantConfig, TenantRegistry, TenantScopedStore,
};
pub use webhooks::{
    DefaultHttpClient, HttpClient, SecurityEventType, WebhookDeliveryLog, WebhookManager,
    WebhookPayload,
};
