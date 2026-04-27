pub mod db;
pub mod handlers;
pub mod rate_limiter;
pub mod two_factor;

#[cfg(test)]
mod tests;

pub use db::PostgresTwoFactorStore;
pub use handlers::{AuthenticatedUser, TwoFactorHandlers};
pub use rate_limiter::{InMemoryRateLimiter, RateLimitResult, RateLimiter, RedisRateLimiter};
pub use two_factor::{
    InMemoryStore, RecoveryResult, TotpConfig, TwoFactorAuth, TwoFactorData, TwoFactorSetup,
    TwoFactorStore,
};
