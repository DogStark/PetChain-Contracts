pub mod db;
pub mod handlers;
pub mod leaderboard;
pub mod rate_limiter;
pub mod tracing_middleware;
pub mod two_factor;
pub mod webhooks;

#[cfg(test)]
mod tests;

pub use db::PostgresTwoFactorStore;
pub use db::{select_secret_provider, AwsSecretsManagerProvider, EnvSecretProvider, SecretProvider};
pub use handlers::{
    AdminDashboardHandlers, AdminScoreHandlers, AuthenticatedAdmin, AuthenticatedUser,
    CanaryHandlers, CreateCanaryRequest, CreateCanaryResponse, TwoFactorHandlers,
};
pub use leaderboard::{
    FlaggedScoreStore, FlaggedScoreSubmission, ScoreSubmissionError, ScoreValidationConfig,
};
pub use rate_limiter::{
    DistributedRateLimiter, EndpointConfig, InMemoryRateLimiter, LiveRedisBackend,
    MockRedisBackend, RateLimitResult, RateLimiter, RedisBackend, RedisRateLimiter,
    SlidingWindowRateLimiter,
};
pub use tracing_middleware::sanitize_json_body;
pub use two_factor::{
    AuditLogEntry, InMemoryStore, RecoveryResult, TotpConfig, TwoFactorAuth, TwoFactorData,
    TwoFactorSetup, TwoFactorStore, UserTwoFactorSummary,
};
pub use webhooks::{
    DefaultHttpClient, HttpClient, SecurityEventType, WebhookDeliveryLog, WebhookManager,
    WebhookPayload,
};
