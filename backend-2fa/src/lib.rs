pub mod db;
pub mod error;
pub mod handlers;
pub mod health;
pub mod ip_access;
pub mod leaderboard;
pub mod metrics;
pub mod rate_limit_middleware;
pub mod rate_limiter;
pub mod tracing_middleware;
pub mod two_factor;
pub mod webhooks;
pub mod migrations;

#[cfg(test)]
mod tests;

pub use db::PostgresTwoFactorStore;
pub use db::{
    select_secret_provider, AwsSecretsManagerProvider, EnvSecretProvider, PoolStats,
    PostgresIpAccessStore, SecretProvider,
};
pub use error::{ApiError, ErrorResponseMiddleware, NoCacheMiddleware};
pub use handlers::{
    leaderboard_ws, AddIpRuleRequest, AdminDashboardHandlers, AdminIpAccessHandlers,
    AdminRateLimitHandlers, AdminScoreHandlers, AuthenticatedAdmin, AuthenticatedUser,
    CanaryHandlers, CreateCanaryRequest, CreateCanaryResponse, GrantUnlimitedRequest,
    PoolMetricsHandlers, PoolStatsResponse, SetUserQuotaRequest, TwoFactorHandlers,
};
pub use ip_access::{
    CidrBlock, InMemoryIpAccessStore, IpAccessDecision, IpAccessEntry, IpAccessMiddleware,
    IpAccessStore, IpListType,
};
pub use leaderboard::{
    broadcast_score_update, FlaggedScoreStore, FlaggedScoreSubmission, LeaderboardEntry,
    LeaderboardScoreUpdate, LeaderboardWsHub, LeaderboardWsSession, ScoreSubmissionError,
    ScoreValidationConfig,
};
pub use metrics::{
    dec_leaderboard_ws_connections, inc_leaderboard_ws_connections, metrics, record_rate_limit_hit,
    record_recovery_code_use, record_redis_fallback, record_totp_verification, record_webhook_delivery,
    record_webhook_retry, render_metrics, set_db_pool_stats, start_request_timer,
};
pub use rate_limit_middleware::{
    RateLimitMiddleware, HEADER_LIMIT, HEADER_REMAINING, HEADER_RESET, HEADER_RETRY_AFTER,
};
pub use rate_limiter::{
    progressive_delay_secs, DistributedRateLimiter, EndpointConfig, InMemoryRateLimiter,
    LiveRedisBackend, MockRedisBackend, RateLimitResult, RateLimiter, RedisBackend,
    RedisRateLimiter, RedisTwoFactorFailureCounter, SlidingWindowRateLimiter,
    TenantRateLimitKey,
};
pub use health::{
    HealthAggregator, HealthCheck, HealthReport, PostgresHealthCheck, RedisHealthCheck,
    SubsystemStatus, WebhookHealthCheck,
};
pub use tracing_middleware::sanitize_json_body;
pub use two_factor::{
    AuditLogEntry, InMemoryStore, RecoveryResult, TenantConfig, TenantRegistry, TenantScopedStore,
    TotpConfig, TwoFactorAuth, TwoFactorData, TwoFactorLockoutState, TwoFactorSetup,
    TwoFactorStore, UserTwoFactorSummary,
};
pub use webhooks::{
    DefaultHttpClient, HttpClient, RetryPolicy, SecurityEventType, WebhookDeliveryLog,
    WebhookManager, WebhookPayload, WebhookUrlError, validate_webhook_url,
};
