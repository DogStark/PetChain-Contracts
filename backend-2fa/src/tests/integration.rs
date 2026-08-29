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

#[test]
fn test_concurrent_reuse_only_first_succeeds() {
    let mut codes = vec!["7777-8888".to_string()];

    let first = TwoFactorAuth::consume_backup_code(&mut codes, "7777-8888");
    let second = TwoFactorAuth::consume_backup_code(&mut codes, "7777-8888");

    assert!(first, "first recovery attempt must succeed");
    assert!(
        !second,
        "second recovery attempt must fail — code already consumed"
    );
}

// ── TwoFactorHandlers state-transition tests ───────────────────────────────────────

#[test]
fn test_handler_enable_persists_disabled_state() {
    clear_two_factor_store_for_tests();
    let user_id = "handler-user1";
    let resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "u1@petchain.com".to_string(),
        },
    );
    assert!(resp.is_ok());
    let resp = resp.unwrap();
    assert!(!resp.secret.is_empty());
    assert_eq!(resp.backup_codes.len(), 8);

    let stored = get_two_factor_data_for_tests(user_id).unwrap();
    assert!(!stored.enabled);
}

#[test]
fn test_handler_enable_unknown_user_returns_error() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();
    let err = handlers.verify_login_token(
        &caller("ghost-handler"),
        LoginWithTwoFactorRequest {
            user_id: "ghost-handler".to_string(),
            token: "000000".to_string(),
        },
    );
    assert!(err.is_err());
    assert!(err.unwrap_err().message.contains("not configured"));
}

// -------------------------------------------------------------------------
// Cross-tenant isolation tests
// -------------------------------------------------------------------------

#[test]
fn test_for_tenant_constructor_creates_tenant_scoped_store() {
    clear_two_factor_store_for_tests();
    let handlers_a = TwoFactorHandlers::for_tenant("tenant-a");
    let handlers_b = TwoFactorHandlers::for_tenant("tenant-b");

    // Both handlers should have different stores (different tenant configs)
    // Enroll the same user in both tenants
    let resp_a = handlers_a
        .enroll(
            &caller("alice"),
            EnableTwoFactorRequest {
                user_id: "alice".to_string(),
                email: "alice@tenant-a.com".to_string(),
                idempotency_key: None,
            },
        )
        .unwrap();

    let resp_b = handlers_b
        .enroll(
            &caller("alice"),
            EnableTwoFactorRequest {
                user_id: "alice".to_string(),
                email: "alice@tenant-b.com".to_string(),
                idempotency_key: None,
            },
        )
        .unwrap();

    // Secrets should be different (different enrollments)
    assert_ne!(resp_a.secret, resp_b.secret);
}

#[test]
fn test_cross_tenant_user_data_isolation() {
    clear_two_factor_store_for_tests();
    let handlers_a = TwoFactorHandlers::for_tenant("tenant-a");
    let handlers_b = TwoFactorHandlers::for_tenant("tenant-b");

    // Enroll user "alice" in tenant-a
    let resp_a = handlers_a
        .enroll(
            &caller("alice"),
            EnableTwoFactorRequest {
                user_id: "alice".to_string(),
                email: "alice@tenant-a.com".to_string(),
                idempotency_key: None,
            },
        )
        .unwrap();

    let token_a = generate_token(&resp_a.secret);

    // Activate 2FA for alice in tenant-a
    handlers_a
        .verify_and_activate(
            &caller("alice"),
            VerifyTwoFactorRequest {
                user_id: "alice".to_string(),
                token: token_a.clone(),
            },
        )
        .unwrap();

    // Try to login as alice from tenant-b using tenant-a's token
    // This should fail because tenant-b's alice doesn't exist
    let login_result = handlers_b.verify_login_token(
        &caller("alice"),
        LoginWithTwoFactorRequest {
            user_id: "alice".to_string(),
            token: token_a,
        },
    );

    // Should return false (user not found in tenant-b)
    assert!(login_result.is_ok());
    assert!(!login_result.unwrap());
}

#[test]
fn test_same_user_id_different_tenants_have_separate_data() {
    clear_two_factor_store_for_tests();
    let handlers_a = TwoFactorHandlers::for_tenant("tenant-a");
    let handlers_b = TwoFactorHandlers::for_tenant("tenant-b");

    // Enroll and activate user "bob" in tenant-a
    let resp_a = handlers_a
        .enroll(
            &caller("bob"),
            EnableTwoFactorRequest {
                user_id: "bob".to_string(),
                email: "bob@tenant-a.com".to_string(),
                idempotency_key: None,
            },
        )
        .unwrap();

    let token_a = generate_token(&resp_a.secret);
    handlers_a
        .verify_and_activate(
            &caller("bob"),
            VerifyTwoFactorRequest {
                user_id: "bob".to_string(),
                token: token_a,
            },
        )
        .unwrap();

    // Enroll and activate user "bob" in tenant-b
    let resp_b = handlers_b
        .enroll(
            &caller("bob"),
            EnableTwoFactorRequest {
                user_id: "bob".to_string(),
                email: "bob@tenant-b.com".to_string(),
                idempotency_key: None,
            },
        )
        .unwrap();

    let token_b = generate_token(&resp_b.secret);
    handlers_b
        .verify_and_activate(
            &caller("bob"),
            VerifyTwoFactorRequest {
                user_id: "bob".to_string(),
                token: token_b,
            },
        )
        .unwrap();

    // Both tenants should have enabled 2FA for "bob"
    let data_a = get_two_factor_data_for_tests("tenant-a::bob").unwrap();
    let data_b = get_two_factor_data_for_tests("tenant-b::bob").unwrap();

    assert!(data_a.enabled);
    assert!(data_b.enabled);

    // Secrets should be different
    assert_ne!(data_a.secret, data_b.secret);
}

#[test]
fn test_tenant_scoped_disable_does_not_affect_other_tenant() {
    clear_two_factor_store_for_tests();
    let handlers_a = TwoFactorHandlers::for_tenant("tenant-a");
    let handlers_b = TwoFactorHandlers::for_tenant("tenant-b");

    // Enroll and activate user "charlie" in both tenants
    let resp_a = handlers_a
        .enroll(
            &caller("charlie"),
            EnableTwoFactorRequest {
                user_id: "charlie".to_string(),
                email: "charlie@tenant-a.com".to_string(),
                idempotency_key: None,
            },
        )
        .unwrap();

    let token_a = generate_token(&resp_a.secret);
    handlers_a
        .verify_and_activate(
            &caller("charlie"),
            VerifyTwoFactorRequest {
                user_id: "charlie".to_string(),
                token: token_a.clone(),
            },
        )
        .unwrap();

    let resp_b = handlers_b
        .enroll(
            &caller("charlie"),
            EnableTwoFactorRequest {
                user_id: "charlie".to_string(),
                email: "charlie@tenant-b.com".to_string(),
                idempotency_key: None,
            },
        )
        .unwrap();

    let token_b = generate_token(&resp_b.secret);
    handlers_b
        .verify_and_activate(
            &caller("charlie"),
            VerifyTwoFactorRequest {
                user_id: "charlie".to_string(),
                token: token_b,
            },
        )
        .unwrap();

    // Disable 2FA for charlie in tenant-a
    let disable_result = handlers_a.disable_two_factor(
        &caller("charlie"),
        DisableTwoFactorRequest {
            user_id: "charlie".to_string(),
            token: token_a,
        },
    );
    assert!(disable_result.is_ok());
    assert!(disable_result.unwrap());

    // charlie in tenant-b should still have 2FA enabled
    let data_b = get_two_factor_data_for_tests("tenant-b::charlie").unwrap();
    assert!(data_b.enabled);

    // charlie in tenant-a should have 2FA disabled
    let data_a = get_two_factor_data_for_tests("tenant-a::charlie").unwrap();
    assert!(!data_a.enabled);
}
