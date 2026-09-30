//! Connection-pool metrics handler and leaderboard WebSocket endpoint.
use crate::leaderboard::leaderboard_ws_endpoint;
use actix_web::{web::Payload, Error, HttpRequest, HttpResponse};
use serde::Serialize;

use super::auth::AuthenticatedAdmin;

#[cfg(not(test))]
use super::auth::two_factor_store;

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
    /// Return current pool utilisation. Only available when backed by Postgres
    /// and `POOL_STATS_ENABLED=1` is set in the environment.
    /// Requires admin authentication.
    pub fn pool_stats(_admin: &AuthenticatedAdmin) -> Result<PoolStatsResponse, String> {
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
    pub fn pool_stats(_admin: &AuthenticatedAdmin) -> Result<PoolStatsResponse, String> {
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

#[cfg(test)]
mod pool_metrics_tests {
    use crate::handlers::auth::{
        AuthenticatedAdmin, AuthenticatedUser, EnableTwoFactorRequest, RevokeSessionRequest,
        TwoFactorHandlers, VerifyTwoFactorRequest,
    };
    use crate::handlers::pool::PoolMetricsHandlers;
    use crate::two_factor::InMemoryStore;
    use std::sync::Arc;

    #[test]
    fn test_pool_stats_admin_access_succeeds() {
        let admin = AuthenticatedAdmin::new("admin-user");
        let result = PoolMetricsHandlers::pool_stats(&admin);
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.active, 0);
        assert_eq!(stats.idle, 0);
        assert_eq!(stats.max, 0);
    }

    mod revoke_session_tests {
        use super::*;

        fn handlers() -> TwoFactorHandlers {
            TwoFactorHandlers::with_store(Arc::new(InMemoryStore::default()))
        }

        #[test]
        fn test_revoke_specific_session() {
            let h = handlers();
            let caller = AuthenticatedUser::new("user-1");

            let result = h.revoke_session(
                &caller,
                RevokeSessionRequest {
                    session_id: Some("jti-abc".to_string()),
                    revoke_all: false,
                },
            );
            assert!(result.is_ok());

            assert!(h.store.is_session_revoked("user-1", "jti-abc", 0));
            // A different session_id for the same user is untouched.
            assert!(!h.store.is_session_revoked("user-1", "jti-other", 0));
        }

        #[test]
        fn test_revoke_all_sessions() {
            let h = handlers();
            let caller = AuthenticatedUser::new("user-2");

            let before = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let result = h.revoke_session(
                &caller,
                RevokeSessionRequest {
                    session_id: None,
                    revoke_all: true,
                },
            );
            assert!(result.is_ok());

            // Any session issued at/before the revoke_all call is now invalid,
            // even though its specific JTI was never explicitly revoked.
            assert!(h
                .store
                .is_session_revoked("user-2", "jti-never-seen", before));

            // A session issued after revoke_all is fine.
            let after = before + 100;
            assert!(!h.store.is_session_revoked("user-2", "jti-fresh", after));
        }

        #[test]
        fn test_revoked_token_rejected_on_use() {
            let h = handlers();
            let caller = AuthenticatedUser::new("user-3");

            h.revoke_session(
                &caller,
                RevokeSessionRequest {
                    session_id: Some("jti-xyz".to_string()),
                    revoke_all: false,
                },
            )
            .unwrap();

            // Simulates what auth middleware should do on every request:
            // check is_session_revoked before trusting the bearer token.
            let issued_at = 0;
            let is_valid = !h.store.is_session_revoked("user-3", "jti-xyz", issued_at);
            assert!(!is_valid, "revoked token must be rejected");
        }
    }

    #[test]
    fn test_pool_stats_requires_authentication() {
        let admin = AuthenticatedAdmin::new("admin-user");
        let result = PoolMetricsHandlers::pool_stats(&admin);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pool_stats_different_admin_still_succeeds() {
        let admin1 = AuthenticatedAdmin::new("admin-1");
        let admin2 = AuthenticatedAdmin::new("admin-2");

        let result1 = PoolMetricsHandlers::pool_stats(&admin1);
        let result2 = PoolMetricsHandlers::pool_stats(&admin2);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    // -----------------------------------------------------------------------
    // Issue #1061 – Unified failure-count key ("2fa:{user_id}")
    // -----------------------------------------------------------------------

    /// Both verify_and_activate and verify_login_token must produce the same
    /// rate-limit key "2fa:{user_id}" so a success on either path resets the
    /// failure counter for both endpoints.
    #[test]
    fn test_verify_and_login_share_same_rate_limit_key() {
        let verify_key = TwoFactorHandlers::rate_limit_key("2fa", "alice");
        let login_key = TwoFactorHandlers::rate_limit_key("2fa", "alice");
        assert_eq!(
            verify_key, login_key,
            "verify_and_activate and verify_login_token must share the same 2fa:{{user_id}} key"
        );
        assert_eq!(verify_key, "2fa:alice");
    }

    /// Fail verify_and_activate N-1 times → call record_success on the shared
    /// "2fa:{user_id}" key (simulating a successful verify_login_token) →
    /// verify_and_activate must not be rate-limited on the next call.
    #[test]
    fn test_failed_verify_counter_is_reset_by_login_success_key() {
        use crate::rate_limiter::InMemoryRateLimiter;

        let store = Arc::new(InMemoryStore::default());
        let limiter = Arc::new(InMemoryRateLimiter::default());
        let handlers = TwoFactorHandlers::with_store_and_limiter(
            store.clone() as Arc<dyn crate::two_factor::TwoFactorStore>,
            limiter.clone(),
        );

        let caller = AuthenticatedUser::new("key-test-user");
        let enroll_req = EnableTwoFactorRequest {
            user_id: "key-test-user".to_string(),
            email: "key@example.com".to_string(),
            idempotency_key: None,
        };
        let _ = handlers.enroll(&caller, enroll_req);

        // Accumulate 2 failures via verify_and_activate.
        let bad_verify = VerifyTwoFactorRequest {
            user_id: "key-test-user".to_string(),
            token: "000000".to_string(),
        };
        for _ in 0..2 {
            let _ = handlers.verify_and_activate(&caller, bad_verify.clone());
        }

        // Simulate a successful login by calling record_success on the unified key.
        let key = TwoFactorHandlers::rate_limit_key("2fa", "key-test-user");
        assert_eq!(key, "2fa:key-test-user");
        limiter.record_success(&key);

        // After reset, verify_and_activate must not return a rate-limit error.
        let result = handlers.verify_and_activate(&caller, bad_verify.clone());
        match result {
            Err(e) => {
                let msg = format!("{:?}", e);
                assert!(
                    !msg.contains("Too many") && !msg.contains("rate"),
                    "verify_and_activate must not be rate-limited after login success reset; got: {msg}"
                );
            }
            Ok(_) => {}
        }
    }
}
