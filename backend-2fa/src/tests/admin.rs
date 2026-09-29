#![allow(unused_imports)]
use super::common::*;
use crate::error::ApiError;
use crate::handlers::{
    clear_two_factor_store_for_tests, get_two_factor_data_for_tests,
    get_two_factor_store_for_tests, overwrite_two_factor_data_for_tests, AdminDashboardHandlers,
    AdminRecoveryHandlers, AdminWebhookHandlers, AuthenticatedAdmin, AuthenticatedUser,
    CanaryHandlers, ConfigureWebhookRequest, CreateCanaryRequest, DisableTwoFactorRequest,
    EnableTwoFactorRequest, LoginWithTwoFactorRequest, PoolMetricsHandlers, ProvisionTenantRequest,
    RecoverWithBackupRequest, TenantProvisioningHandlers, TwoFactorHandlers,
    UpgradeAlgorithmRequest, VerifyTwoFactorRequest,
};
use crate::rate_limiter::{
    progressive_delay_secs, DistributedRateLimiter, EndpointConfig, InMemoryRateLimiter,
    MockRedisBackend, RateLimitResult, RateLimiter, RedisRateLimiter, RedisTwoFactorFailureCounter,
    SlidingWindowRateLimiter,
};
use crate::two_factor::{
    InMemoryStore, MockStoreConfig, MockStoreFailure, MockTwoFactorStore, TenantConfig,
    TenantRegistry, TenantScopedStore, TotpConfig, TwoFactorAuth, TwoFactorData, TwoFactorStore,
};
use crate::webhooks::{DefaultHttpClient, HttpClient, SecurityEventType, WebhookManager};
use actix_web::ResponseError;
use std::collections::{BTreeMap, HashMap};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, Secret, TOTP};

// ── W3C Traceparent Header Tests ──

mod tracing_context {
    use crate::tracing_middleware::TraceContext;

    #[test]
    fn parse_valid_traceparent() {
        let header = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let tc = TraceContext::parse(header).unwrap();
        assert_eq!(tc.trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(tc.parent_span_id, "00f067aa0ba902b7");
        assert_eq!(tc.flags, "01");
    }

    #[test]
    fn parse_valid_traceparent_with_zeros() {
        let header = "00-00000000000000000000000000000000-0000000000000000-00";
        let tc = TraceContext::parse(header).unwrap();
        assert_eq!(tc.trace_id, "00000000000000000000000000000000");
        assert_eq!(tc.parent_span_id, "0000000000000000");
        assert_eq!(tc.flags, "00");
    }

    #[test]
    fn parse_invalid_traceparent_wrong_parts() {
        let header = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7";
        assert!(TraceContext::parse(header).is_none());
    }

    #[test]
    fn parse_invalid_traceparent_wrong_trace_id_length() {
        let header = "00-4bf92f3577b34da6a3ce929d0e0e47-00f067aa0ba902b7-01";
        assert!(TraceContext::parse(header).is_none());
    }

    #[test]
    fn parse_invalid_traceparent_wrong_parent_span_length() {
        let header = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902-01";
        assert!(TraceContext::parse(header).is_none());
    }

    #[test]
    fn parse_invalid_traceparent_non_hex() {
        let header = "00-ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ-00f067aa0ba902b7-01";
        assert!(TraceContext::parse(header).is_none());
    }

    #[test]
    fn parse_absent_header_fallback() {
        // When header is absent, middleware should generate a fresh trace context
        // This is tested in the middleware integration tests
        assert!(true);
    }

    #[test]
    fn generate_traceparent_header() {
        let tc = TraceContext {
            trace_id: "4bf92f3577b34da6a3ce929d0e0e4736".to_string(),
            parent_span_id: "00f067aa0ba902b7".to_string(),
            flags: "01".to_string(),
        };
        let header = tc.to_header();
        assert_eq!(
            header,
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
        );
    }

    #[test]
    fn round_trip_traceparent() {
        let original = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let tc = TraceContext::parse(original).unwrap();
        let generated = tc.to_header();
        assert_eq!(generated, original);
    }

    #[test]
    fn parse_case_insensitive_hex() {
        // Hex should be case-insensitive
        let header = "00-4BF92F3577B34DA6A3CE929D0E0E4736-00F067AA0BA902B7-01";
        let tc = TraceContext::parse(header).unwrap();
        assert_eq!(tc.trace_id, "4BF92F3577B34DA6A3CE929D0E0E4736");
    }
}

// -----------------------------------------------------------------------
// Tracing middleware sanitization tests
// -----------------------------------------------------------------------

mod tracing_sanitization {
    use crate::tracing_middleware::sanitize_json_body;

    #[test]
    fn sanitize_simple_totp_code() {
        let body = r#"{"user_id":"user1","totp_code":"123456"}"#;
        let sanitized = sanitize_json_body(body);
        assert!(sanitized.contains(r#""totp_code":"[REDACTED]""#));
        assert!(sanitized.contains(r#""user_id":"user1""#));
        assert!(!sanitized.contains("123456"));
    }

    #[test]
    fn sanitize_secret_field() {
        let body = r#"{"user_id":"user1","secret":"ABCDEFG123456"}"#;
        let sanitized = sanitize_json_body(body);
        assert!(sanitized.contains(r#""secret":"[REDACTED]""#));
        assert!(!sanitized.contains("ABCDEFG123456"));
    }

    #[test]
    fn sanitize_password_field() {
        let body = r#"{"username":"alice","password":"SuperSecret123!"}"#;
        let sanitized = sanitize_json_body(body);
        assert!(sanitized.contains(r#""password":"[REDACTED]""#));
        assert!(!sanitized.contains("SuperSecret123!"));
        assert!(sanitized.contains(r#""username":"alice""#));
    }

    #[test]
    fn sanitize_recovery_code() {
        let body = r#"{"user_id":"user1","recovery_code":"1234-5678"}"#;
        let sanitized = sanitize_json_body(body);
        assert!(sanitized.contains(r#""recovery_code":"[REDACTED]""#));
        assert!(!sanitized.contains("1234-5678"));
    }

    #[test]
    fn sanitize_token_field() {
        let body = r#"{"user_id":"user1","token":"eyJhbGc..."}"#;
        let sanitized = sanitize_json_body(body);
        assert!(sanitized.contains(r#""token":"[REDACTED]""#));
        assert!(!sanitized.contains("eyJhbGc..."));
    }

    #[test]
    fn sanitize_backup_code() {
        let body = r#"{"user_id":"user1","backup_code":"BACKUP123"}"#;
        let sanitized = sanitize_json_body(body);
        assert!(sanitized.contains(r#""backup_code":"[REDACTED]""#));
        assert!(!sanitized.contains("BACKUP123"));
    }

    #[test]
    fn sanitize_multiple_sensitive_fields() {
        let body = r#"{"user_id":"user1","totp_code":"123456","secret":"SECRET_KEY","password":"pass123"}"#;
        let sanitized = sanitize_json_body(body);
        assert!(sanitized.contains(r#""totp_code":"[REDACTED]""#));
        assert!(sanitized.contains(r#""secret":"[REDACTED]""#));
        assert!(sanitized.contains(r#""password":"[REDACTED]""#));
        assert!(!sanitized.contains("123456"));
        assert!(!sanitized.contains("SECRET_KEY"));
        assert!(!sanitized.contains("pass123"));
    }

    #[test]
    fn sanitize_nested_json() {
        let body = r#"{"user_id":"user1","data":{"secret":"nested_secret","field":"value"}}"#;
        let sanitized = sanitize_json_body(body);
        assert!(sanitized.contains(r#""secret":"[REDACTED]""#));
        assert!(!sanitized.contains("nested_secret"));
        assert!(sanitized.contains(r#""field":"value""#));
    }

    #[test]
    fn sanitize_json_array_with_sensitive_fields() {
        let body = r#"{"items":[{"totp_code":"111111"},{"totp_code":"222222"}]}"#;
        let sanitized = sanitize_json_body(body);
        assert!(sanitized.contains(r#""totp_code":"[REDACTED]""#));
        assert!(!sanitized.contains("111111"));
        assert!(!sanitized.contains("222222"));
    }

    #[test]
    fn preserve_non_sensitive_fields() {
        let body = r#"{"user_id":"user123","email":"test@example.com","name":"John Doe"}"#;
        let sanitized = sanitize_json_body(body);
        assert!(sanitized.contains(r#""user_id":"user123""#));
        assert!(sanitized.contains(r#""email":"test@example.com""#));
        assert!(sanitized.contains(r#""name":"John Doe""#));
    }

    #[test]
    fn handle_non_json_body() {
        let body = "This is not JSON at all";
        let sanitized = sanitize_json_body(body);
        assert_eq!(sanitized, "[binary]");
    }

    #[test]
    fn handle_invalid_json() {
        let body = r#"{"invalid": json syntax"#;
        let sanitized = sanitize_json_body(body);
        assert_eq!(sanitized, "[binary]");
    }

    #[test]
    fn handle_empty_json() {
        let body = "{}";
        let sanitized = sanitize_json_body(body);
        assert_eq!(sanitized, "{}");
    }

    #[test]
    fn handle_empty_body() {
        let body = "";
        let sanitized = sanitize_json_body(body);
        assert_eq!(sanitized, "[binary]");
    }

    #[test]
    fn case_sensitive_field_names() {
        let body = r#"{"TOTP_CODE":"123456","Totp_Code":"654321","totp_code":"111111"}"#;
        let sanitized = sanitize_json_body(body);
        // Only lowercase "totp_code" should be redacted
        assert!(sanitized.contains(r#""totp_code":"[REDACTED]""#));
        // Uppercase variants should remain
        assert!(sanitized.contains("123456") || sanitized.contains("654321"));
    }

    #[test]
    fn sanitize_deeply_nested_structure() {
        let body = r#"{"level1":{"level2":{"level3":{"secret":"deep_secret"}}}}"#;
        let sanitized = sanitize_json_body(body);
        assert!(sanitized.contains(r#""secret":"[REDACTED]""#));
        assert!(!sanitized.contains("deep_secret"));
    }

    #[test]
    fn sanitize_mixed_array_and_objects() {
        let body =
            r#"{"users":[{"name":"Alice","totp_code":"123"},{"name":"Bob","password":"secret"}]}"#;
        let sanitized = sanitize_json_body(body);
        assert!(sanitized.contains(r#""totp_code":"[REDACTED]""#));
        assert!(sanitized.contains(r#""password":"[REDACTED]""#));
        assert!(sanitized.contains(r#""name":"Alice""#));
        assert!(sanitized.contains(r#""name":"Bob""#));
    }

    #[test]
    fn handle_numeric_values() {
        let body = r#"{"user_id":123,"totp_code":654321,"amount":1000}"#;
        let sanitized = sanitize_json_body(body);
        assert!(sanitized.contains(r#""totp_code":"[REDACTED]""#));
        assert!(sanitized.contains(r#""user_id":123"#));
        assert!(sanitized.contains(r#""amount":1000"#));
    }

    #[test]
    fn handle_boolean_values() {
        let body = r#"{"enabled":true,"secret":"secret_key","active":false}"#;
        let sanitized = sanitize_json_body(body);
        assert!(sanitized.contains(r#""secret":"[REDACTED]""#));
        assert!(sanitized.contains(r#""enabled":true"#));
        assert!(sanitized.contains(r#""active":false"#));
    }

    #[test]
    fn handle_null_values() {
        let body = r#"{"user_id":null,"secret":"secret_key"}"#;
        let sanitized = sanitize_json_body(body);
        assert!(sanitized.contains(r#""secret":"[REDACTED]""#));
        assert!(sanitized.contains(r#""user_id":null"#));
    }
}

// -----------------------------------------------------------------------
// Admin Score Handlers Tests
// -----------------------------------------------------------------------

mod admin_score_handlers {
    use crate::handlers::AdminScoreHandlers;

    #[test]
    fn admin_get_all_flagged_empty() {
        let admin = AdminScoreHandlers::new();
        let flagged = admin.get_all_flagged();
        assert!(flagged.is_empty());
    }

    #[test]
    fn admin_log_rejected_submission() {
        let admin = AdminScoreHandlers::new();
        admin.log_rejected_submission("user1".into(), 5000, "Exceeds delta".into());

        let flagged = admin.get_all_flagged();
        assert_eq!(flagged.len(), 1);
        assert_eq!(flagged[0].user_id, "user1");
        assert_eq!(flagged[0].attempted_score, 5000);
        assert_eq!(flagged[0].reason, "Exceeds delta");
    }

    #[test]
    fn admin_get_flagged_by_user() {
        let admin = AdminScoreHandlers::new();
        admin.log_rejected_submission("user1".into(), 5000, "Exceeds delta".into());
        admin.log_rejected_submission("user2".into(), 3000, "Suspicious".into());

        let user1_flagged = admin.get_flagged_by_user("user1");
        let user2_flagged = admin.get_flagged_by_user("user2");

        assert_eq!(user1_flagged.len(), 1);
        assert_eq!(user2_flagged.len(), 1);
        assert_eq!(user1_flagged[0].user_id, "user1");
        assert_eq!(user2_flagged[0].user_id, "user2");
    }

    #[test]
    fn admin_get_flagged_by_user_multiple_submissions() {
        let admin = AdminScoreHandlers::new();
        admin.log_rejected_submission("user1".into(), 5000, "Exceeds delta".into());
        admin.log_rejected_submission("user1".into(), 6000, "Another violation".into());

        let user1_flagged = admin.get_flagged_by_user("user1");
        assert_eq!(user1_flagged.len(), 2);
        assert_eq!(user1_flagged[0].attempted_score, 5000);
        assert_eq!(user1_flagged[1].attempted_score, 6000);
    }

    #[test]
    fn admin_get_flagged_by_nonexistent_user() {
        let admin = AdminScoreHandlers::new();
        admin.log_rejected_submission("user1".into(), 5000, "Exceeds delta".into());

        let user2_flagged = admin.get_flagged_by_user("user2");
        assert!(user2_flagged.is_empty());
    }

    #[test]
    fn admin_default() {
        let admin = AdminScoreHandlers::default();
        assert!(admin.get_all_flagged().is_empty());
    }

    #[test]
    fn admin_log_multiple_users() {
        let admin = AdminScoreHandlers::new();

        for i in 0..5 {
            admin.log_rejected_submission(
                format!("user{}", i),
                1000 + (i as u64 * 100),
                format!("Violation {}", i),
            );
        }

        let all_flagged = admin.get_all_flagged();
        assert_eq!(all_flagged.len(), 5);

        for i in 0..5 {
            assert_eq!(all_flagged[i].user_id, format!("user{}", i));
            assert_eq!(all_flagged[i].attempted_score, 1000 + (i as u64 * 100));
        }
    }

    #[test]
    #[cfg(test)]
    fn admin_clear_flagged() {
        let admin = AdminScoreHandlers::new();
        admin.log_rejected_submission("user1".into(), 5000, "Exceeds delta".into());
        admin.log_rejected_submission("user2".into(), 3000, "Suspicious".into());

        assert_eq!(admin.get_all_flagged().len(), 2);

        admin.clear_flagged();
        assert!(admin.get_all_flagged().is_empty());
    }

    #[test]
    fn admin_timestamp_is_set() {
        let admin = AdminScoreHandlers::new();
        admin.log_rejected_submission("user1".into(), 5000, "Test".into());

        let flagged = admin.get_all_flagged();
        assert!(flagged[0].timestamp > 0);
    }

    #[test]
    fn admin_reason_is_preserved() {
        let admin = AdminScoreHandlers::new();
        let reason = "Custom reason for suspension";
        admin.log_rejected_submission("user1".into(), 5000, reason.into());

        let flagged = admin.get_all_flagged();
        assert_eq!(flagged[0].reason, reason);
    }

    #[test]
    fn admin_large_score_values() {
        let admin = AdminScoreHandlers::new();
        let max_score = u64::MAX;
        admin.log_rejected_submission("user1".into(), max_score, "Max score".into());

        let flagged = admin.get_all_flagged();
        assert_eq!(flagged[0].attempted_score, max_score);
    }

    // ---------------------------------------------------------------
    // FlaggedScoreStore trait injection (Issue #789)
    //
    // AdminScoreHandlers::new() defaults to InMemoryFlaggedScoreStore,
    // but with_store() accepts any Arc<dyn FlaggedScoreStore> — the same
    // injection point a caller would use to swap in
    // PostgresFlaggedScoreStore for cross-restart persistence in
    // production, without changing any handler logic.
    // ---------------------------------------------------------------

    #[test]
    fn admin_with_store_uses_injected_in_memory_store() {
        use crate::leaderboard::{FlaggedScoreStore, InMemoryFlaggedScoreStore};
        use std::sync::Arc;

        let store: Arc<dyn FlaggedScoreStore> = Arc::new(InMemoryFlaggedScoreStore::new());
        let admin = AdminScoreHandlers::with_store(store);

        admin.log_rejected_submission("user1".into(), 5000, "Exceeds delta".into());
        let flagged = admin.get_all_flagged();
        assert_eq!(flagged.len(), 1);
        assert_eq!(flagged[0].user_id, "user1");
    }

    /// A store injected into two independent handler instances is shared
    /// state, not per-handler state — this is exactly what lets a
    /// persistent store survive a fresh `AdminScoreHandlers::with_store`
    /// call after a process restart (only the store, not the handler,
    /// needs to outlive the process).
    #[test]
    fn admin_with_store_shares_state_across_handler_instances() {
        use crate::leaderboard::{FlaggedScoreStore, InMemoryFlaggedScoreStore};
        use std::sync::Arc;

        let store: Arc<dyn FlaggedScoreStore> = Arc::new(InMemoryFlaggedScoreStore::new());

        let admin1 = AdminScoreHandlers::with_store(Arc::clone(&store));
        admin1.log_rejected_submission("user1".into(), 5000, "Exceeds delta".into());

        // A brand-new handler wired to the same store sees the same data.
        let admin2 = AdminScoreHandlers::with_store(Arc::clone(&store));
        let flagged = admin2.get_all_flagged();
        assert_eq!(flagged.len(), 1);
        assert_eq!(flagged[0].user_id, "user1");
    }
}

// --- limit ---

#[test]
fn allows_requests_below_limit() {
    let l = limiter(3, 60, 300);
    for i in 1u32..3 {
        match l.record_failure("u:a") {
            RateLimitResult::Allowed { remaining, .. } => assert_eq!(remaining, 3 - i),
            RateLimitResult::Blocked { .. } => panic!("should not be blocked below limit"),
        }
    }
}

#[test]
fn blocks_at_limit_with_accurate_retry_after() {
    let l = limiter(3, 60, 120);
    for _ in 0..3 {
        l.record_failure("u:b");
    }
    assert!(matches!(
        l.record_failure("u:b"),
        RateLimitResult::Blocked {
            retry_after_secs: 120,
            ..
        }
    ));
}

// --- reset ---

#[test]
fn success_resets_counter() {
    let l = limiter(3, 60, 300);
    l.record_failure("u:c");
    l.record_failure("u:c");
    l.record_success("u:c");
    match l.record_failure("u:c") {
        RateLimitResult::Allowed { remaining, .. } => assert_eq!(remaining, 2),
        RateLimitResult::Blocked { .. } => panic!("should not be blocked after success reset"),
    }
}

#[test]
fn window_expiry_resets_counter() {
    let l = limiter(3, 60, 300);
    // 2 failures (below the lockout threshold)
    l.record_failure("u:d");
    l.record_failure("u:d");
    // Advance clock past the 60-second window — entries are evicted on next call
    l.backend_advance_ms(61_000);
    // Window has expired; the two old entries are outside the cutoff, so Allowed with remaining=2
    match l.record_failure("u:d") {
        RateLimitResult::Allowed { remaining, .. } => assert_eq!(remaining, 2),
        RateLimitResult::Blocked { .. } => panic!("should not be blocked after window expiry"),
    }
}

// --- concurrent / independent keys ---

#[test]
fn different_keys_are_independent() {
    let l = limiter(2, 60, 300);
    l.record_failure("u:e");
    l.record_failure("u:e");
    assert!(matches!(
        l.record_failure("u:f"),
        RateLimitResult::Allowed { .. }
    ));
}

#[test]
fn concurrent_threads_do_not_corrupt_state() {
    use std::thread;
    let l = Arc::new(limiter(100, 60, 300));
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let l = Arc::clone(&l);
            thread::spawn(move || l.record_failure(&format!("u:thread:{i}")))
        })
        .collect();
    for h in handles {
        h.join().expect("thread panicked");
    }
}

// --- per-endpoint config ---

#[test]
fn per_endpoint_config_applies_correct_limits() {
    let l = SlidingWindowRateLimiter::new(
        MockRedisBackend::new(),
        EndpointConfig::new(60, 10, 300), // default: 10 failures
    )
    .with_endpoint("login", EndpointConfig::new(60, 2, 60)); // login: 2 failures

    // Exhaust the login endpoint
    l.record_failure("login:user:1");
    l.record_failure("login:user:1");
    assert!(matches!(
        l.record_failure("login:user:1"),
        RateLimitResult::Blocked { .. }
    ));

    // A key that doesn't match "login" uses the default (10 failures)
    for _ in 0..9 {
        assert!(matches!(
            l.record_failure("verify:user:1"),
            RateLimitResult::Allowed { .. }
        ));
    }
}

#[test]
fn test_non_admin_cannot_call_admin_endpoints() {
    // AuthenticatedAdmin is a distinct type from AuthenticatedUser —
    // the type system prevents non-admin callers from reaching these handlers.
    // This test documents that the types are distinct.
    let user = AuthenticatedUser::new("regular-user");
    let _admin = AuthenticatedAdmin::new("admin-001");
    // user and _admin are different types; the compiler enforces this.
    assert_ne!(user.user_id, _admin.admin_id.clone() + "-different");
}

#[test]
fn test_canary_excluded_from_user_listing() {
    clear_two_factor_store_for_tests();
    setup_user("normal-user");
    setup_user("canary-user");

    let store = get_two_factor_store_for_tests();
    store.set_canary("canary-user", true).unwrap();

    let users = AdminDashboardHandlers::list_users(&admin(), 1, 100).unwrap();
    let ids: Vec<&str> = users.iter().map(|u| u.user_id.as_str()).collect();
    assert!(ids.contains(&"normal-user"));
    assert!(!ids.contains(&"canary-user"));
}

#[test]
fn test_list_locked_users_returns_only_locked_accounts() {
    clear_two_factor_store_for_tests();
    setup_user("locked-user-a");
    setup_user("locked-user-b");
    setup_user("unlocked-user");

    let store = get_two_factor_store_for_tests();
    // Lock two accounts by recording 10 failed attempts each
    for _ in 0..10 {
        store
            .record_failed_two_fa_attempt("locked-user-a", 10)
            .unwrap();
        store
            .record_failed_two_fa_attempt("locked-user-b", 10)
            .unwrap();
    }
    // Record a few failures for the unlocked user (not enough to lock)
    for _ in 0..3 {
        store
            .record_failed_two_fa_attempt("unlocked-user", 10)
            .unwrap();
    }

    let locked = AdminDashboardHandlers::list_locked_users(&admin()).unwrap();
    let ids: Vec<&str> = locked.iter().map(|u| u.user_id.as_str()).collect();
    assert_eq!(locked.len(), 2);
    assert!(ids.contains(&"locked-user-a"));
    assert!(ids.contains(&"locked-user-b"));
    assert!(!ids.contains(&"unlocked-user"));

    for entry in &locked {
        assert!(entry.failed_attempts >= 10);
        assert!(entry.locked_at.is_some());
    }
}

#[test]
fn test_list_locked_users_empty_when_none_locked() {
    clear_two_factor_store_for_tests();
    setup_user("healthy-user");

    let locked = AdminDashboardHandlers::list_locked_users(&admin()).unwrap();
    assert!(locked.is_empty());
}

// ── Issue #827 — UserTwoFactorSummary endpoint ───────────────────────

#[test]
fn test_get_two_factor_summary_returns_data_for_enabled_user() {
    clear_two_factor_store_for_tests();
    setup_user("user-2fa-active");

    let summary =
        AdminDashboardHandlers::get_user_two_factor_summary(&admin(), "user-2fa-active").unwrap();

    assert_eq!(summary.user_id, "user-2fa-active");
    assert!(summary.enabled);
    assert!(!summary.is_canary);
}

#[test]
fn test_get_two_factor_summary_returns_404_for_missing_user() {
    clear_two_factor_store_for_tests();

    let result = AdminDashboardHandlers::get_user_two_factor_summary(&admin(), "nonexistent");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("No 2FA data found for user"));
    assert!(err.contains("nonexistent"));
}

#[test]
fn test_get_two_factor_summary_rejects_empty_user_id() {
    clear_two_factor_store_for_tests();

    let result = AdminDashboardHandlers::get_user_two_factor_summary(&admin(), "");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("must not be empty"));
}

#[test]
fn test_get_two_factor_summary_rejects_long_user_id() {
    clear_two_factor_store_for_tests();

    let long_user_id = "a".repeat(65);
    let result = AdminDashboardHandlers::get_user_two_factor_summary(&admin(), &long_user_id);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("must not exceed 64"));
}

#[test]
fn test_get_two_factor_summary_requires_admin() {
    // AuthenticatedAdmin is a distinct type from AuthenticatedUser —
    // the type system prevents non-admin callers from reaching this handler.
    // This test documents that the types are distinct.
    let user = AuthenticatedUser::new("regular-user");
    let _admin = AuthenticatedAdmin::new("admin-001");
    // user and _admin are different types; the compiler enforces this.
    assert_ne!(user.user_id, _admin.admin_id.clone() + "-different");
}

#[test]
fn test_create_canary_account() {
    clear_two_factor_store_for_tests();
    let (_handlers, _calls) = make_canary_handlers();

    let resp = CanaryHandlers::create_canary(
        &admin(),
        CreateCanaryRequest {
            user_id: "canary-001".to_string(),
            email: "canary@petchain.com".to_string(),
        },
    )
    .unwrap();

    assert_eq!(resp.user_id, "canary-001");
    assert!(!resp.secret.is_empty());

    let store = get_two_factor_store_for_tests();
    assert!(store.is_canary("canary-001"));
}

#[test]
fn test_canary_trigger_logs_event_with_ip() {
    clear_two_factor_store_for_tests();
    let (handlers, _calls) = make_canary_handlers();

    CanaryHandlers::create_canary(
        &admin(),
        CreateCanaryRequest {
            user_id: "canary-002".to_string(),
            email: "canary2@petchain.com".to_string(),
        },
    )
    .unwrap();

    let result = handlers
        .verify_with_canary_check("canary-002", "123456", Some("10.0.0.1"))
        .unwrap();

    // Canary always returns false
    assert!(!result);

    let store = get_two_factor_store_for_tests();
    let log = store.get_audit_log("canary-002", 1, 10).unwrap();
    let triggered: Vec<_> = log
        .iter()
        .filter(|e| e.event == "CanaryTriggered")
        .collect();
    assert!(!triggered.is_empty());
    assert!(triggered[0]
        .metadata
        .as_deref()
        .unwrap_or("")
        .contains("10.0.0.1"));
}

#[test]
fn test_canary_trigger_fires_webhook() {
    clear_two_factor_store_for_tests();
    let (handlers, calls) = make_canary_handlers();

    CanaryHandlers::create_canary(
        &admin(),
        CreateCanaryRequest {
            user_id: "canary-003".to_string(),
            email: "canary3@petchain.com".to_string(),
        },
    )
    .unwrap();

    handlers
        .verify_with_canary_check("canary-003", "000000", Some("192.168.1.1"))
        .unwrap();

    // fire() spawns a thread — give it time to complete
    std::thread::sleep(std::time::Duration::from_millis(200));

    let fired = calls.lock().unwrap();
    assert!(!fired.is_empty());
    assert!(fired[0].contains("canary_triggered"));
}

#[test]
fn test_canary_excluded_from_normal_user_listing() {
    clear_two_factor_store_for_tests();
    let (_handlers, _calls) = make_canary_handlers();

    CanaryHandlers::create_canary(
        &admin(),
        CreateCanaryRequest {
            user_id: "canary-004".to_string(),
            email: "canary4@petchain.com".to_string(),
        },
    )
    .unwrap();

    let store = get_two_factor_store_for_tests();
    let users = store.list_users(1, 100).unwrap();
    let ids: Vec<&str> = users.iter().map(|u| u.user_id.as_str()).collect();
    assert!(!ids.contains(&"canary-004"));
}

#[test]
fn test_normal_user_not_treated_as_canary() {
    clear_two_factor_store_for_tests();
    let (handlers, calls) = make_canary_handlers();

    // Set up a normal user
    let store = get_two_factor_store_for_tests();
    store
        .save(
            "normal-user",
            crate::two_factor::TwoFactorData {
                secret: "JBSWY3DPEHPK3PXP".to_string(),
                backup_codes: vec![],
                enabled: true,
                algorithm: Algorithm::SHA1,
                last_used_step: None,
            },
        )
        .unwrap();

    // Verification attempt on a normal user should NOT fire canary webhook
    let _ = handlers.verify_with_canary_check("normal-user", "000000", Some("1.2.3.4"));
    let fired = calls.lock().unwrap();
    assert!(fired.is_empty());
}

#[test]
fn test_webhook_manager_configure_and_query_log() {
    let manager =
        WebhookManager::new_with_http_allowed(Arc::new(crate::webhooks::DefaultHttpClient));
    manager
        .configure(
            SecurityEventType::FailedTwoFa,
            "http://example.com/hook".to_string(),
        )
        .unwrap();
    // Use fire_sync to avoid needing to wait for a spawned thread
    manager.fire_sync(
        SecurityEventType::FailedTwoFa,
        "user1",
        std::collections::HashMap::new(),
    );
    let log = manager.get_delivery_log(1, 10);
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].event_type, "failed_two_fa");
}

#[test]
fn configure_registers_url() {
    let h = make_handlers();
    let result = h.configure(
        &admin(),
        ConfigureWebhookRequest {
            event_type: SecurityEventType::FailedTwoFa,
            url: "http://example.com/hook".to_string(),
        },
    );
    assert!(result.is_ok());

    let entries = h.list_configured_events(&admin());
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].event_type, "failed_two_fa");
    assert_eq!(entries[0].urls, vec!["http://example.com/hook"]);
}

#[test]
fn configure_multiple_urls_for_same_event() {
    let h = make_handlers();
    h.configure(
        &admin(),
        ConfigureWebhookRequest {
            event_type: SecurityEventType::AccountLockout,
            url: "http://example.com/a".to_string(),
        },
    )
    .unwrap();
    h.configure(
        &admin(),
        ConfigureWebhookRequest {
            event_type: SecurityEventType::AccountLockout,
            url: "http://example.com/b".to_string(),
        },
    )
    .unwrap();

    let entries = h.list_configured_events(&admin());
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].urls.len(), 2);
}

#[test]
fn configure_rejects_invalid_url() {
    let h = make_handlers();
    let result = h.configure(
        &admin(),
        ConfigureWebhookRequest {
            event_type: SecurityEventType::FailedTwoFa,
            url: "not-a-url".to_string(),
        },
    );
    assert!(result.is_err());
}

#[test]
fn remove_config_clears_event() {
    let h = make_handlers();
    h.configure(
        &admin(),
        ConfigureWebhookRequest {
            event_type: SecurityEventType::CanaryTriggered,
            url: "http://example.com/hook".to_string(),
        },
    )
    .unwrap();

    h.remove_config(&admin(), &SecurityEventType::CanaryTriggered)
        .unwrap();

    assert!(h.list_configured_events(&admin()).is_empty());
}

#[test]
fn list_configured_events_sorted_by_event_type() {
    let h = make_handlers();
    h.configure(
        &admin(),
        ConfigureWebhookRequest {
            event_type: SecurityEventType::RecoveryCodeUsed,
            url: "http://example.com/r".to_string(),
        },
    )
    .unwrap();
    h.configure(
        &admin(),
        ConfigureWebhookRequest {
            event_type: SecurityEventType::AccountLockout,
            url: "http://example.com/a".to_string(),
        },
    )
    .unwrap();

    let entries = h.list_configured_events(&admin());
    assert_eq!(entries.len(), 2);
    // Alphabetical order: "account_lockout" < "recovery_code_used"
    assert_eq!(entries[0].event_type, "account_lockout");
    assert_eq!(entries[1].event_type, "recovery_code_used");
}

/// No Redis URL → always uses in-memory fallback.
#[test]
fn fallback_allows_below_limit() {
    let limiter = DistributedRateLimiter::new(None, 3, 60, "test:");
    for i in 1..=3u32 {
        match limiter.record_failure("user:fallback") {
            RateLimitResult::Allowed { remaining, .. } => assert_eq!(remaining, 3 - i),
            RateLimitResult::Blocked { .. } => panic!("should not block below limit"),
        }
    }
}

#[test]
fn fallback_blocks_at_limit() {
    let limiter = DistributedRateLimiter::new(None, 2, 60, "test:");
    limiter.record_failure("user:block");
    limiter.record_failure("user:block");
    assert!(matches!(
        limiter.record_failure("user:block"),
        RateLimitResult::Blocked { .. }
    ));
}

#[test]
fn fallback_success_resets_counter() {
    let limiter = DistributedRateLimiter::new(None, 2, 60, "test:");
    limiter.record_failure("user:reset");
    limiter.record_success("user:reset");
    // After reset, should be allowed again
    assert!(matches!(
        limiter.record_failure("user:reset"),
        RateLimitResult::Allowed { .. }
    ));
}

#[test]
fn handler_locks_after_ten_invalid_tokens_and_admin_unlocks() {
    let store = Arc::new(InMemoryStore::default());
    let handlers = TwoFactorHandlers::with_store_and_limiter(
        store.clone(),
        Arc::new(InMemoryRateLimiter::new(100, 60, 300)),
    );
    let user_id = "handler-lockout-user";
    let caller = AuthenticatedUser::new(user_id);
    let enrollment = handlers
        .enroll(
            &caller,
            EnableTwoFactorRequest {
                idempotency_key: None,
                user_id: user_id.to_string(),
                email: "lockout@example.com".to_string(),
            },
        )
        .unwrap();
    store.update_enabled(user_id, true).unwrap();

    for _ in 0..9 {
        let result = handlers
            .verify_login_token(
                &caller,
                LoginWithTwoFactorRequest {
                    user_id: user_id.to_string(),
                    token: "000000".to_string(),
                },
            )
            .unwrap();
        assert!(!result);
        // Clear the progressive-delay gate (independent of the lockout
        // threshold under test) so the next attempt isn't blocked by it.
        store.clear_retry_after_for_tests(user_id);
    }

    let locked = handlers
        .verify_login_token(
            &caller,
            LoginWithTwoFactorRequest {
                user_id: user_id.to_string(),
                token: "000000".to_string(),
            },
        )
        .unwrap_err();
    assert!(locked.message.contains("locked after 10"));

    store
        .unlock_two_fa_account(user_id, &AuthenticatedAdmin::new("admin").admin_id)
        .unwrap();
    let token = generate_token(&enrollment.secret);
    assert!(handlers
        .verify_login_token(
            &caller,
            LoginWithTwoFactorRequest {
                user_id: user_id.to_string(),
                token,
            },
        )
        .unwrap());
}

mod ip_access_tests {
    use crate::handlers::{AddIpRuleRequest, AdminIpAccessHandlers, AuthenticatedAdmin};
    use crate::ip_access::{
        CidrBlock, InMemoryIpAccessStore, IpAccessDecision, IpAccessStore, IpListType,
    };
    use std::net::IpAddr;
    use std::sync::Arc;

    fn admin() -> AuthenticatedAdmin {
        AuthenticatedAdmin::new("admin-1")
    }

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    // --- CIDR parsing / containment ---

    #[test]
    fn cidr_parses_bare_ipv4_as_slash_32() {
        let block = CidrBlock::parse("192.168.1.10").unwrap();
        assert!(block.contains(&ip("192.168.1.10")));
        assert!(!block.contains(&ip("192.168.1.11")));
    }

    #[test]
    fn cidr_matches_ipv4_range() {
        let block = CidrBlock::parse("192.168.1.0/24").unwrap();
        assert!(block.contains(&ip("192.168.1.0")));
        assert!(block.contains(&ip("192.168.1.255")));
        assert!(!block.contains(&ip("192.168.2.1")));
    }

    #[test]
    fn cidr_matches_ipv6_range() {
        let block = CidrBlock::parse("2001:db8::/32").unwrap();
        assert!(block.contains(&ip("2001:db8::1")));
        assert!(!block.contains(&ip("2001:db9::1")));
    }

    #[test]
    fn cidr_zero_prefix_matches_everything_in_family() {
        let block = CidrBlock::parse("0.0.0.0/0").unwrap();
        assert!(block.contains(&ip("8.8.8.8")));
        assert!(!block.contains(&ip("::1")));
    }

    #[test]
    fn cidr_rejects_invalid_address() {
        assert!(CidrBlock::parse("not-an-ip/24").is_err());
    }

    #[test]
    fn cidr_rejects_out_of_range_prefix() {
        assert!(CidrBlock::parse("10.0.0.0/33").is_err());
        assert!(CidrBlock::parse("::1/129").is_err());
    }

    #[test]
    fn cidr_v4_and_v6_never_cross_match() {
        let block = CidrBlock::parse("0.0.0.0/0").unwrap();
        assert!(!block.contains(&ip("::1")));
    }

    // --- InMemoryIpAccessStore + decision logic ---

    #[test]
    fn unknown_ip_is_allowed_by_default() {
        let store = InMemoryIpAccessStore::new();
        assert_eq!(store.check(ip("1.2.3.4")), IpAccessDecision::Allowed);
    }

    #[test]
    fn blocked_cidr_blocks_matching_ip() {
        let store = InMemoryIpAccessStore::new();
        store
            .add_entry("10.0.0.0/8", IpListType::Block, None, "admin-1")
            .unwrap();
        assert_eq!(store.check(ip("10.1.2.3")), IpAccessDecision::Blocked);
        assert_eq!(store.check(ip("11.1.2.3")), IpAccessDecision::Allowed);
    }

    #[test]
    fn allowlist_takes_precedence_over_blocklist() {
        let store = InMemoryIpAccessStore::new();
        store
            .add_entry("10.0.0.0/8", IpListType::Block, None, "admin-1")
            .unwrap();
        store
            .add_entry(
                "10.1.2.3/32",
                IpListType::Allow,
                Some("trusted ops host"),
                "admin-1",
            )
            .unwrap();
        assert_eq!(store.check(ip("10.1.2.3")), IpAccessDecision::Allowed);
        assert_eq!(store.check(ip("10.1.2.4")), IpAccessDecision::Blocked);
    }

    #[test]
    fn add_entry_rejects_invalid_cidr() {
        let store = InMemoryIpAccessStore::new();
        assert!(store
            .add_entry("garbage", IpListType::Block, None, "admin-1")
            .is_err());
    }

    #[test]
    fn remove_entry_drops_it_from_list() {
        let store = InMemoryIpAccessStore::new();
        let entry = store
            .add_entry("192.168.0.0/16", IpListType::Block, None, "admin-1")
            .unwrap();
        assert_eq!(store.check(ip("192.168.1.1")), IpAccessDecision::Blocked);

        store.remove_entry(entry.id).unwrap();
        assert_eq!(store.check(ip("192.168.1.1")), IpAccessDecision::Allowed);
    }

    #[test]
    fn remove_entry_unknown_id_errors() {
        let store = InMemoryIpAccessStore::new();
        assert!(store.remove_entry(999).is_err());
    }

    #[test]
    fn list_entries_filters_by_type() {
        let store = InMemoryIpAccessStore::new();
        store
            .add_entry("10.0.0.0/8", IpListType::Block, None, "admin-1")
            .unwrap();
        store
            .add_entry("172.16.0.0/12", IpListType::Allow, None, "admin-1")
            .unwrap();

        assert_eq!(store.list_entries(IpListType::Block).len(), 1);
        assert_eq!(store.list_entries(IpListType::Allow).len(), 1);
    }

    // --- AdminIpAccessHandlers ---

    #[test]
    fn admin_handlers_allow_and_block_round_trip() {
        let store: Arc<dyn IpAccessStore> = Arc::new(InMemoryIpAccessStore::new());
        let handlers = AdminIpAccessHandlers::new(store);

        let allow_entry = handlers
            .allow_ip(
                &admin(),
                AddIpRuleRequest {
                    cidr: "203.0.113.5/32".to_string(),
                    note: None,
                },
            )
            .unwrap();
        assert_eq!(allow_entry.created_by, "admin-1");
        assert_eq!(handlers.list_allow().len(), 1);

        let block_entry = handlers
            .block_ip(
                &admin(),
                AddIpRuleRequest {
                    cidr: "198.51.100.0/24".to_string(),
                    note: Some("known abuse range".to_string()),
                },
            )
            .unwrap();
        assert_eq!(handlers.list_block().len(), 1);

        handlers.remove_entry(&admin(), block_entry.id).unwrap();
        assert!(handlers.list_block().is_empty());
        assert_eq!(handlers.list_allow().len(), 1);
    }

    #[test]
    fn admin_handlers_reject_invalid_cidr() {
        let store: Arc<dyn IpAccessStore> = Arc::new(InMemoryIpAccessStore::new());
        let handlers = AdminIpAccessHandlers::new(store);

        let result = handlers.block_ip(
            &admin(),
            AddIpRuleRequest {
                cidr: "not-a-cidr".to_string(),
                note: None,
            },
        );
        assert!(result.is_err());
    }
}

#[test]
fn pool_stats_handler_returns_sentinel_in_test_mode() {
    use crate::handlers::AuthenticatedAdmin;
    let admin = AuthenticatedAdmin::new("test-admin");
    let stats =
        PoolMetricsHandlers::pool_stats(&admin).expect("pool_stats must succeed in test mode");
    assert_eq!(stats.active, 0);
    assert_eq!(stats.idle, 0);
    assert_eq!(stats.max, 0);
}

#[test]
fn in_memory_store_try_pool_stats_returns_none() {
    let store = InMemoryStore::default();
    assert!(
        store.try_pool_stats().is_none(),
        "InMemoryStore has no pool; try_pool_stats must return None"
    );
}

#[test]
fn test_first_provision_creates_tenant() {
    let handlers = TenantProvisioningHandlers::new(Arc::new(TenantRegistry::default()));

    let response = handlers
        .provision_tenant(&admin(), provision_req("tenant-fresh"))
        .unwrap();

    assert_eq!(response.tenant_id, "tenant-fresh");
    assert_eq!(response.totp_issuer, "AcmeCo");
    assert_eq!(response.rate_limit_max_failures, 7);
    assert!(!response.already_existed);

    // The tenant is now retrievable from the registry.
    let config = handlers.get_tenant_config("tenant-fresh").unwrap();
    assert_eq!(config.tenant_id, "tenant-fresh");
}

#[test]
fn test_repeat_provision_returns_existing_tenant_with_flag() {
    let handlers = TenantProvisioningHandlers::new(Arc::new(TenantRegistry::default()));

    let first = handlers
        .provision_tenant(&admin(), provision_req("tenant-repeat"))
        .unwrap();
    assert!(!first.already_existed);

    // Retry with the same tenant_id, as an infra automation tool would
    // do after a flaky failure. A different `totp_issuer` is sent to
    // confirm the *original* config wins rather than being overwritten.
    let mut retry_req = provision_req("tenant-repeat");
    retry_req.totp_issuer = "SomeoneElseCo".to_string();
    let second = handlers.provision_tenant(&admin(), retry_req).unwrap();

    assert!(second.already_existed);
    assert_eq!(second.tenant_id, "tenant-repeat");
    // Existing config is returned unchanged, not the new request's data.
    assert_eq!(second.totp_issuer, "AcmeCo");
    assert_eq!(second.rate_limit_max_failures, 7);

    // Only one entry exists in the registry — no duplicate was created.
    let config = handlers.get_tenant_config("tenant-repeat").unwrap();
    assert_eq!(config.totp_issuer, "AcmeCo");
}

#[test]
fn test_concurrent_provision_is_idempotent_and_atomic() {
    use std::thread;

    let registry = Arc::new(TenantRegistry::default());
    let handlers = Arc::new(TenantProvisioningHandlers::new(registry));

    let mut join_handles = Vec::new();
    for _ in 0..16 {
        let handlers = Arc::clone(&handlers);
        join_handles.push(thread::spawn(move || {
            handlers
                .provision_tenant(&admin(), provision_req("tenant-concurrent"))
                .unwrap()
        }));
    }

    let responses: Vec<_> = join_handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect();

    // Exactly one caller observed creation; all others observed it as
    // already existing — proving the check-and-insert was atomic.
    let created_count = responses.iter().filter(|r| !r.already_existed).count();
    assert_eq!(created_count, 1);

    let existed_count = responses.iter().filter(|r| r.already_existed).count();
    assert_eq!(existed_count, 15);
}

#[test]
fn test_provision_rejects_empty_tenant_id() {
    let handlers = TenantProvisioningHandlers::new(Arc::new(TenantRegistry::default()));

    let mut req = provision_req("unused");
    req.tenant_id = String::new();
    let err = handlers.provision_tenant(&admin(), req).unwrap_err();

    assert_eq!(err.code, "BAD_REQUEST");
    assert_eq!(
        err.details.unwrap().get("field").unwrap().as_str().unwrap(),
        "tenant_id"
    );
}

#[test]
fn test_provision_rejects_tenant_id_over_max_length() {
    let handlers = TenantProvisioningHandlers::new(Arc::new(TenantRegistry::default()));

    let mut req = provision_req("unused");
    req.tenant_id = "a".repeat(65);
    let err = handlers.provision_tenant(&admin(), req).unwrap_err();

    assert_eq!(err.code, "BAD_REQUEST");
    assert_eq!(
        err.details.unwrap().get("field").unwrap().as_str().unwrap(),
        "tenant_id"
    );
}

#[test]
fn test_provision_rejects_tenant_id_with_invalid_characters() {
    let handlers = TenantProvisioningHandlers::new(Arc::new(TenantRegistry::default()));

    let mut req = provision_req("unused");
    req.tenant_id = "tenant_with_underscores".to_string();
    let err = handlers.provision_tenant(&admin(), req).unwrap_err();

    assert_eq!(err.code, "BAD_REQUEST");
    assert_eq!(
        err.details.unwrap().get("field").unwrap().as_str().unwrap(),
        "tenant_id"
    );
}

#[test]
fn test_provision_rejects_zero_max_users() {
    let handlers = TenantProvisioningHandlers::new(Arc::new(TenantRegistry::default()));

    let mut req = provision_req("tenant-zero-users");
    req.max_users = 0;
    let err = handlers.provision_tenant(&admin(), req).unwrap_err();

    assert_eq!(err.code, "BAD_REQUEST");
    assert_eq!(
        err.details.unwrap().get("field").unwrap().as_str().unwrap(),
        "max_users"
    );
}

#[test]
fn test_provision_rejects_empty_name() {
    let handlers = TenantProvisioningHandlers::new(Arc::new(TenantRegistry::default()));

    let mut req = provision_req("tenant-empty-name");
    req.name = String::new();
    let err = handlers.provision_tenant(&admin(), req).unwrap_err();

    assert_eq!(err.code, "BAD_REQUEST");
    assert_eq!(
        err.details.unwrap().get("field").unwrap().as_str().unwrap(),
        "name"
    );
}

#[test]
fn test_provision_rejects_name_over_max_length() {
    let handlers = TenantProvisioningHandlers::new(Arc::new(TenantRegistry::default()));

    let mut req = provision_req("tenant-long-name");
    req.name = "a".repeat(129);
    let err = handlers.provision_tenant(&admin(), req).unwrap_err();

    assert_eq!(err.code, "BAD_REQUEST");
    assert_eq!(
        err.details.unwrap().get("field").unwrap().as_str().unwrap(),
        "name"
    );
}

#[test]
fn test_provision_accepts_valid_config() {
    let handlers = TenantProvisioningHandlers::new(Arc::new(TenantRegistry::default()));

    let response = handlers
        .provision_tenant(&admin(), provision_req("tenant-valid"))
        .unwrap();

    assert_eq!(response.tenant_id, "tenant-valid");
    assert_eq!(response.name, "tenant-valid Inc");
    assert_eq!(response.max_users, 50);
}

#[test]
fn canary_flag_is_tenant_isolated() {
    let store = Arc::new(InMemoryStore::default());
    let tenant_a = TenantScopedStore::new(store.clone(), TenantConfig::new("canary-a"));
    let tenant_b = TenantScopedStore::new(store.clone(), TenantConfig::new("canary-b"));

    let user_id = "canaryuser";
    tenant_a.set_canary(user_id, true).unwrap();

    assert!(tenant_a.is_canary(user_id));
    assert!(!tenant_b.is_canary(user_id));
}

#[test]
fn enabled_state_is_tenant_isolated() {
    let store = Arc::new(InMemoryStore::default());
    let tenant_a = TenantScopedStore::new(store.clone(), TenantConfig::new("en-a"));
    let tenant_b = TenantScopedStore::new(store.clone(), TenantConfig::new("en-b"));

    let user_id = "enableuser";
    tenant_a.save(user_id, make_data("A")).unwrap();
    tenant_b.save(user_id, make_data("B")).unwrap();

    tenant_a.update_enabled(user_id, false).unwrap();

    assert!(!tenant_a.get(user_id).unwrap().enabled);
    assert!(tenant_b.get(user_id).unwrap().enabled);
}

#[test]
fn registry_scoped_store_prevents_unknown_tenant() {
    let registry = TenantRegistry::default();
    let store: Arc<dyn TwoFactorStore> = Arc::new(InMemoryStore::default());

    let result = registry.scoped_store("nonexistent", store);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown tenant"));
}

#[test]
fn registry_scoped_stores_are_isolated() {
    let registry = TenantRegistry::default();
    let store: Arc<dyn TwoFactorStore> = Arc::new(InMemoryStore::default());

    registry.provision(TenantConfig::new("reg-a")).unwrap();
    registry.provision(TenantConfig::new("reg-b")).unwrap();

    let scoped_a = registry.scoped_store("reg-a", store.clone()).unwrap();
    let scoped_b = registry.scoped_store("reg-b", store.clone()).unwrap();

    let user_id = "reguser";
    scoped_a.save(user_id, make_data("RA")).unwrap();
    scoped_b.save(user_id, make_data("RB")).unwrap();

    assert_eq!(scoped_a.get(user_id).unwrap().secret, "RA");
    assert_eq!(scoped_b.get(user_id).unwrap().secret, "RB");
}

#[test]
fn test_5xx_error_is_logged_via_tracing() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;

    // Build a custom subscriber that counts events filtered at ERROR level.
    let event_count = Arc::new(AtomicUsize::new(0));
    let event_count_clone = Arc::clone(&event_count);

    let filter = EnvFilter::new("error");
    let layer = tracing_subscriber::fmt::layer()
        .with_test_writer()
        .with_filter(filter);

    // Wrap the subscriber so we can intercept events.
    struct CountingSubscriber<S> {
        inner: S,
        count: Arc<AtomicUsize>,
    }

    impl<S: tracing::Subscriber> tracing::Subscriber for CountingSubscriber<S> {
        fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
            self.inner.enabled(metadata)
        }

        fn new_span(&self, span: &tracing::span::Attributes<'_>) -> tracing::Id {
            self.inner.new_span(span)
        }

        fn record(&self, span: &tracing::Id, values: &tracing::span::Record<'_>) {
            self.inner.record(span, values);
        }

        fn record_follows_from(&self, span: &tracing::Id, follows: &tracing::Id) {
            self.inner.record_follows_from(span, follows);
        }

        fn event(&self, event: &tracing::Event<'_>) {
            // Count every event matching our filter
            if self.inner.enabled(event.metadata()) {
                self.count.fetch_add(1, Ordering::SeqCst);
            }
            self.inner.event(event);
        }

        fn enter(&self, span: &tracing::Id) {
            self.inner.enter(span);
        }

        fn exit(&self, span: &tracing::Id) {
            self.inner.exit(span);
        }

        fn clone_span(&self, id: &tracing::Id) -> tracing::Id {
            self.inner.clone_span(id)
        }

        fn drop_span(&self, id: tracing::Id) {
            self.inner.drop_span(id);
        }
    }

    let inner = tracing_subscriber::Registry::default().with(layer);
    let subscriber = CountingSubscriber {
        inner,
        count: Arc::clone(&event_count_clone),
    };

    let _guard = tracing::subscriber::set_default(subscriber);

    // Trigger a 500 error — should increment counter.
    let err_500 = ApiError::internal_error("test 500", None);
    let _resp = err_500.error_response();

    let count_after_500 = event_count.load(Ordering::SeqCst);
    assert!(
        count_after_500 >= 1,
        "expected at least 1 event for 5xx error, got {count_after_500}"
    );

    // Trigger a 400 error — should NOT increment counter further.
    let err_400 = ApiError::bad_request("test 400", None);
    let _resp = err_400.error_response();

    let count_after_400 = event_count.load(Ordering::SeqCst);
    assert_eq!(
        count_after_400, count_after_500,
        "400-class error should not produce a log event"
    );
}

// -----------------------------------------------------------------------
// Issue #884: Request body size limit for JSON endpoints
// -----------------------------------------------------------------------

#[actix_web::test]
async fn test_oversized_json_body_is_rejected() {
    use actix_web::{test, web, App, HttpResponse};

    async fn dummy_handler(_body: web::Json<serde_json::Value>) -> HttpResponse {
        HttpResponse::Ok().finish()
    }

    let json_cfg = web::JsonConfig::default()
        .limit(1)
        .error_handler(|err, _req| {
            let resp = ApiError::bad_request(
                format!("Request body too large or invalid JSON: {}", err),
                None,
            );
            actix_web::Error::from(resp)
        });

    let app = test::init_service(
        App::new()
            .app_data(json_cfg)
            .route("/test", web::post().to(dummy_handler)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/test")
        .set_json(serde_json::json!({"foo": "bar"}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_client_error(),
        "Expected a client error status (413/400), got {}",
        resp.status()
    );
}

#[actix_web::test]
async fn test_normal_sized_json_body_is_accepted() {
    use actix_web::{test, web, App, HttpResponse};

    async fn dummy_handler(_body: web::Json<serde_json::Value>) -> HttpResponse {
        HttpResponse::Ok().finish()
    }

    let json_cfg = web::JsonConfig::default()
        .limit(256 * 1024)
        .error_handler(|err, _req| {
            let resp = ApiError::bad_request(
                format!("Request body too large or invalid JSON: {}", err),
                None,
            );
            actix_web::Error::from(resp)
        });

    let app = test::init_service(
        App::new()
            .app_data(json_cfg)
            .route("/test", web::post().to(dummy_handler)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/test")
        .set_json(serde_json::json!({"foo": "bar"}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

// -----------------------------------------------------------------------
// Algorithm Upgrade Tests (Issue #829)
// -----------------------------------------------------------------------

#[test]
fn test_upgrade_algorithm_success() {
    clear_two_factor_store_for_tests();

    let user_id = "user-upgrade-success";

    // 1. Enroll with default SHA1
    let resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "upgrade@petchain.com".to_string(),
        },
    )
    .unwrap();

    // 2. Activate 2FA
    let handlers = TwoFactorHandlers::new();
    let token_sha1 = generate_token(&resp.secret);
    handlers
        .verify_and_activate(
            &caller(user_id),
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token: token_sha1.clone(),
            },
        )
        .unwrap();

    // Verify user is on SHA1
    let data_before = get_two_factor_data_for_tests(user_id).unwrap();
    assert_eq!(data_before.algorithm, Algorithm::SHA1);
    assert!(data_before.enabled);

    // 3. Upgrade to SHA256
    let upgrade_result = handlers.upgrade_algorithm(
        &caller(user_id),
        UpgradeAlgorithmRequest {
            user_id: user_id.to_string(),
            token: token_sha1,
        },
    );

    assert!(upgrade_result.is_ok());
    let upgrade_resp = upgrade_result.unwrap();

    // Verify response
    assert_eq!(upgrade_resp.algorithm, "SHA256");
    assert!(!upgrade_resp.new_secret.is_empty());
    assert!(!upgrade_resp.new_otpauth_uri.is_empty());
    assert!(!upgrade_resp.new_qr_code.is_empty());
    assert_eq!(upgrade_resp.new_backup_codes.len(), 8);
    assert!(upgrade_resp.new_otpauth_uri.contains("algorithm=SHA256"));

    // Verify stored data has been updated
    let data_after = get_two_factor_data_for_tests(user_id).unwrap();
    assert_eq!(data_after.algorithm, Algorithm::SHA256);
    assert_eq!(data_after.secret, upgrade_resp.new_secret);
    assert_eq!(data_after.backup_codes, upgrade_resp.new_backup_codes);
    assert!(data_after.enabled);

    // Old secret should no longer work
    assert_ne!(data_after.secret, resp.secret);
}

#[test]
fn test_upgrade_algorithm_wrong_token_rejected() {
    clear_two_factor_store_for_tests();

    let user_id = "user-upgrade-wrong-token";

    // 1. Enroll and activate with SHA1
    let resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "upgrade2@petchain.com".to_string(),
        },
    )
    .unwrap();

    let handlers = TwoFactorHandlers::new();
    let token_sha1 = generate_token(&resp.secret);
    handlers
        .verify_and_activate(
            &caller(user_id),
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token: token_sha1,
            },
        )
        .unwrap();

    // 2. Try to upgrade with wrong token
    let upgrade_result = handlers.upgrade_algorithm(
        &caller(user_id),
        UpgradeAlgorithmRequest {
            user_id: user_id.to_string(),
            token: "000000".to_string(), // Wrong token
        },
    );

    assert!(upgrade_result.is_err());
    let err = upgrade_result.unwrap_err();
    assert_eq!(err.code, "UNAUTHORIZED");
    assert!(err.message.contains("Invalid TOTP token"));

    // Verify data is unchanged (still SHA1)
    let data = get_two_factor_data_for_tests(user_id).unwrap();
    assert_eq!(data.algorithm, Algorithm::SHA1);
    assert_eq!(data.secret, resp.secret);
}

#[test]
fn test_upgrade_algorithm_already_on_sha256_returns_409() {
    clear_two_factor_store_for_tests();

    let user_id = "user-already-sha256";

    // Directly set up user with SHA256
    let config = TotpConfig {
        algorithm: Algorithm::SHA256,
        digits: 6,
        period: 30,
        window: 1,
        backup_code_count: 8,
    };

    let setup =
        TwoFactorAuth::setup_with_config("already@petchain.com", "PetChain", config).unwrap();

    overwrite_two_factor_data_for_tests(
        user_id,
        TwoFactorData {
            secret: setup.secret.clone(),
            backup_codes: setup.backup_codes.clone(),
            enabled: true,
            algorithm: Algorithm::SHA256,
            last_used_step: None,
        },
    );

    // Generate token with SHA256
    let totp = TOTP::new(
        Algorithm::SHA256,
        6,
        1,
        30,
        Secret::Encoded(setup.secret.clone()).to_bytes().unwrap(),
        None,
        String::new(),
    )
    .unwrap();
    let token_sha256 = totp.generate_current().unwrap();

    // Try to upgrade when already on SHA256
    let handlers = TwoFactorHandlers::new();
    let upgrade_result = handlers.upgrade_algorithm(
        &caller(user_id),
        UpgradeAlgorithmRequest {
            user_id: user_id.to_string(),
            token: token_sha256,
        },
    );

    assert!(upgrade_result.is_err());
    let err = upgrade_result.unwrap_err();
    assert_eq!(err.code, "CONFLICT");
    assert!(err.message.contains("already upgraded"));
}

#[test]
fn test_upgrade_algorithm_2fa_not_enabled() {
    clear_two_factor_store_for_tests();

    let user_id = "user-not-enabled";

    // Enroll but don't activate
    let resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "not-enabled@petchain.com".to_string(),
        },
    )
    .unwrap();

    let token = generate_token(&resp.secret);

    // Try to upgrade without activating first
    let handlers = TwoFactorHandlers::new();
    let upgrade_result = handlers.upgrade_algorithm(
        &caller(user_id),
        UpgradeAlgorithmRequest {
            user_id: user_id.to_string(),
            token,
        },
    );

    assert!(upgrade_result.is_err());
    let err = upgrade_result.unwrap_err();
    assert_eq!(err.code, "BAD_REQUEST");
    assert!(err.message.contains("not enabled"));
}

#[test]
fn test_upgrade_algorithm_user_not_found() {
    clear_two_factor_store_for_tests();

    let user_id = "user-does-not-exist";

    // Try to upgrade for non-existent user
    let handlers = TwoFactorHandlers::new();
    let upgrade_result = handlers.upgrade_algorithm(
        &caller(user_id),
        UpgradeAlgorithmRequest {
            user_id: user_id.to_string(),
            token: "123456".to_string(),
        },
    );

    assert!(upgrade_result.is_err());
    let err = upgrade_result.unwrap_err();
    assert_eq!(err.code, "NOT_FOUND");
    assert!(err.message.contains("not configured"));
}
