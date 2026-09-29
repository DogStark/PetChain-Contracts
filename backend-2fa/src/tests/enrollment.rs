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

// -----------------------------------------------------------------------
// TwoFactorAuth unit tests
// -----------------------------------------------------------------------

#[test]
fn test_generate_secret() {
    let secret = TwoFactorAuth::generate_secret();
    assert!(!secret.is_empty());
    assert!(secret.len() >= 16);
}

#[test]
fn test_totp_config_default() {
    let config = TotpConfig::default();
    assert_eq!(config.algorithm, Algorithm::SHA1);
    assert_eq!(config.digits, 6);
    assert_eq!(config.period, 30);
    assert_eq!(config.window, 1);
}

#[test]
fn test_totp_config_legacy_sha1() {
    let config = TotpConfig::legacy_sha1();
    assert_eq!(config.algorithm, Algorithm::SHA1);
    assert_eq!(config.digits, 6);
    assert_eq!(config.period, 30);
    assert_eq!(config.window, 1);
}

#[test]
fn test_totp_config_high_security() {
    let config = TotpConfig::high_security();
    assert_eq!(config.algorithm, Algorithm::SHA512);
    assert_eq!(config.digits, 8);
    assert_eq!(config.period, 30);
    assert_eq!(config.window, 1);
}

#[test]
fn test_setup_two_factor_default() {
    let result = TwoFactorAuth::setup("test@petchain.com", "PetChain");
    assert!(result.is_ok());
    let setup = result.unwrap();
    assert!(!setup.secret.is_empty());
    assert!(!setup.qr_code_base64.is_empty());
    assert_eq!(setup.backup_codes.len(), 8);
    assert_eq!(setup.config.algorithm, Algorithm::SHA1);
    assert!(setup
        .otpauth_uri
        .starts_with("otpauth://totp/PetChain:test%40petchain.com?"));
    assert!(setup.otpauth_uri.contains("secret="));
    assert!(setup.otpauth_uri.contains("&issuer=PetChain"));
    assert!(setup.otpauth_uri.contains("&algorithm=SHA1"));
    assert!(setup.otpauth_uri.contains("&digits=6"));
    assert!(setup.otpauth_uri.contains("&period=30"));
}

#[test]
fn test_otpauth_uri_url_encodes_issuer_and_account() {
    let setup = TwoFactorAuth::setup("first.last+pet@example.com", "Pet Chain: Ops").unwrap();
    // Colon in issuer is replaced with a space by sanitize_issuer, resulting
    // in "Pet Chain  Ops" which URL-encodes to "Pet%20Chain%20%20Ops"
    assert!(setup
        .otpauth_uri
        .starts_with("otpauth://totp/Pet%20Chain%20%20Ops:first.last%2Bpet%40example.com?"));
    assert!(setup.otpauth_uri.contains("&issuer=Pet%20Chain%20%20Ops"));
    assert!(setup
        .otpauth_uri
        .contains("&algorithm=SHA1&digits=6&period=30"));
}

#[test]
fn test_issuer_with_colon_is_consistent_between_qr_and_uri() {
    // When issuer contains a colon, both the QR image and the otpauth_uri
    // must use the same sanitized issuer string (colon → space).
    let setup = TwoFactorAuth::setup("user@test.com", "MyApp:Prod").unwrap();

    // Sanitized "MyApp:Prod" → "MyApp Prod" → URL-encoded "MyApp%20Prod"
    assert!(
        setup.otpauth_uri.contains("&issuer=MyApp%20Prod"),
        "otpauth_uri should contain sanitized issuer, got: {}",
        setup.otpauth_uri
    );

    // URI label should use sanitized issuer as well
    assert!(
        setup
            .otpauth_uri
            .starts_with("otpauth://totp/MyApp%20Prod:user%40test.com?"),
        "otpauth_uri label does not match sanitized issuer"
    );

    // QR code must have been generated successfully
    assert!(
        !setup.qr_code_base64.is_empty(),
        "QR code must be generated"
    );
}

#[test]
fn test_setup_two_factor_with_sha1_config() {
    let config = TotpConfig::legacy_sha1();
    let result = TwoFactorAuth::setup_with_config("test@petchain.com", "PetChain", config.clone());
    assert!(result.is_ok());

    let setup = result.unwrap();
    assert!(!setup.secret.is_empty());
    assert!(setup.qr_code_base64.starts_with("data:image/png;base64,"));
    assert_eq!(setup.backup_codes.len(), 8);
    assert_eq!(setup.config.algorithm, Algorithm::SHA1);
}

#[test]
fn test_setup_two_factor_with_sha512_config() {
    let config = TotpConfig::high_security();
    let result = TwoFactorAuth::setup_with_config("test@petchain.com", "PetChain", config.clone());
    assert!(result.is_ok());

    let setup = result.unwrap();
    assert!(!setup.secret.is_empty());
    assert!(setup.qr_code_base64.starts_with("data:image/png;base64,"));
    assert_eq!(setup.backup_codes.len(), 8);
    assert_eq!(setup.config.algorithm, Algorithm::SHA512);
    assert_eq!(setup.config.digits, 8);
}

#[test]
fn test_enable_two_factor_protection() {
    clear_two_factor_store_for_tests();
    let user_id = "user123";
    let caller = AuthenticatedUser::new(user_id);
    let req = EnableTwoFactorRequest {
        idempotency_key: None,
        user_id: user_id.to_string(),
        email: "user@example.com".to_string(),
    };

    // 1. Initial enrollment - succeeds and returns secrets
    let result = TwoFactorHandlers::enable_two_factor(&caller, req.clone());
    assert!(result.is_ok());
    let secret = result.unwrap().secret;
    assert!(!secret.is_empty());

    // 2. Activate 2FA
    // (Since verify_token is a mock, we manually set enabled=true for this test)
    let mut data = crate::handlers::get_two_factor_data_for_tests(user_id).unwrap();
    data.enabled = true;
    overwrite_two_factor_data_for_tests(user_id, data);

    // 3. Subsequent enrollment attempt - must fail/refuse to re-disclose
    let result2 = TwoFactorHandlers::enable_two_factor(&caller, req);
    assert!(result2.is_err());
    assert!(result2.unwrap_err().message.contains("already enabled"));
}

// -----------------------------------------------------------------------
// enable_two_factor — persistence tests (core of this issue)
// -----------------------------------------------------------------------

/// Success path: enable_two_factor must persist TwoFactorData keyed by
/// user_id and the response must be consistent with what was stored.
#[test]
fn test_enable_two_factor_persists_data() {
    clear_two_factor_store_for_tests();

    let user_id = "user-persist";
    let resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "persist@petchain.com".to_string(),
        },
    )
    .expect("enable_two_factor should succeed");

    let stored = get_two_factor_data_for_tests(user_id)
        .expect("TwoFactorData must be persisted after enable_two_factor");

    // Response is consistent with what was stored
    assert_eq!(resp.secret, stored.secret);
    assert_eq!(resp.backup_codes, stored.backup_codes);
    // enabled starts as false — not yet verified
    assert!(!stored.enabled);
    // 8 backup codes generated
    assert_eq!(stored.backup_codes.len(), 8);
}

/// Calling enable_two_factor twice for the same user overwrites the old record.
#[test]
fn test_enable_two_factor_overwrites_existing_record() {
    clear_two_factor_store_for_tests();

    let user_id = "user-overwrite";
    let resp1 = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "overwrite@petchain.com".to_string(),
        },
    )
    .unwrap();

    let resp2 = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "overwrite@petchain.com".to_string(),
        },
    )
    .unwrap();

    let stored = get_two_factor_data_for_tests(user_id).unwrap();
    // Store holds the latest secret
    assert_eq!(stored.secret, resp2.secret);
    // The first secret is gone
    assert_ne!(stored.secret, resp1.secret);
}

#[test]
fn test_enroll_same_idempotency_key_returns_identical_secret() {
    clear_two_factor_store_for_tests();
    crate::handlers::clear_idempotency_store_for_tests();

    let user_id = "user-idempotent";
    let req = EnableTwoFactorRequest {
        user_id: user_id.to_string(),
        email: "idempotent@petchain.com".to_string(),
        idempotency_key: Some("retry-key-1".to_string()),
    };

    let resp1 = TwoFactorHandlers::enable_two_factor(&caller(user_id), req.clone()).unwrap();
    let resp2 = TwoFactorHandlers::enable_two_factor(&caller(user_id), req).unwrap();

    assert_eq!(resp1.secret, resp2.secret);
    assert_eq!(resp1.backup_codes, resp2.backup_codes);
    assert_eq!(resp1.otpauth_uri, resp2.otpauth_uri);
}

#[test]
fn test_enroll_different_idempotency_key_generates_new_secret() {
    clear_two_factor_store_for_tests();
    crate::handlers::clear_idempotency_store_for_tests();

    let user_id = "user-idempotent-2";

    let resp1 = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            user_id: user_id.to_string(),
            email: "idempotent2@petchain.com".to_string(),
            idempotency_key: Some("key-a".to_string()),
        },
    )
    .unwrap();

    // Same user, different key — but enroll() rejects a second enroll
    // once a record exists unless it is still unverified, so this proves
    // the new key path does NOT hit the cached response and instead
    // re-runs normal enroll logic overwriting the prior unverified record.
    let resp2 = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            user_id: user_id.to_string(),
            email: "idempotent2@petchain.com".to_string(),
            idempotency_key: Some("key-b".to_string()),
        },
    )
    .unwrap();

    assert_ne!(resp1.secret, resp2.secret);
}

#[test]
fn test_enroll_rate_limited_when_exceeding_limit() {
    use crate::rate_limiter::InMemoryRateLimiter;
    clear_two_factor_store_for_tests();

    let limiter = std::sync::Arc::new(InMemoryRateLimiter::new(
        2,   // max 2 failures
        60,  // window 60s
        300, // lockout 300s
    ));

    let handlers = TwoFactorHandlers::with_limiter(limiter);
    let user_id = "rate-limited-user";

    // First attempt should succeed
    let result1 = handlers.enroll(
        &caller(user_id),
        EnableTwoFactorRequest {
            user_id: user_id.to_string(),
            email: "user1@petchain.com".to_string(),
            idempotency_key: None,
        },
    );
    assert!(result1.is_ok(), "First enrollment should succeed");

    // Second attempt should succeed
    let result2 = handlers.enroll(
        &caller(user_id),
        EnableTwoFactorRequest {
            user_id: user_id.to_string(),
            email: "user2@petchain.com".to_string(),
            idempotency_key: None,
        },
    );
    assert!(result2.is_ok(), "Second enrollment should succeed");

    // Third attempt should be rate-limited
    let result3 = handlers.enroll(
        &caller(user_id),
        EnableTwoFactorRequest {
            user_id: user_id.to_string(),
            email: "user3@petchain.com".to_string(),
            idempotency_key: None,
        },
    );
    assert!(result3.is_err(), "Third enrollment should be rate-limited");
    let err = result3.unwrap_err();
    assert_eq!(err.code, "RATE_LIMITED");
    assert!(
        err.message.contains("Too many enrollment attempts"),
        "Error message should mention too many enrollment attempts"
    );
}

#[test]
fn test_enroll_allowed_under_limit() {
    use crate::rate_limiter::InMemoryRateLimiter;
    clear_two_factor_store_for_tests();

    let limiter = std::sync::Arc::new(InMemoryRateLimiter::new(
        5,   // max 5 failures
        60,  // window 60s
        300, // lockout 300s
    ));

    let handlers = TwoFactorHandlers::with_limiter(limiter);
    let user_id = "within-limit-user";

    // Should succeed for each attempt within the limit
    for i in 1..=3 {
        let result = handlers.enroll(
            &caller(user_id),
            EnableTwoFactorRequest {
                user_id: user_id.to_string(),
                email: format!("user{}@petchain.com", i),
                idempotency_key: None,
            },
        );
        assert!(result.is_ok(), "Enrollment attempt {} should succeed", i);
    }
}

/// Failure path: wrong caller is rejected before any persistence occurs.
#[test]
fn test_enable_two_factor_forbidden_does_not_persist() {
    clear_two_factor_store_for_tests();

    let result = TwoFactorHandlers::enable_two_factor(
        &caller("attacker"),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: "victim".to_string(),
            email: "victim@petchain.com".to_string(),
        },
    );

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "FORBIDDEN");
    // Nothing was written to the store
    assert!(get_two_factor_data_for_tests("victim").is_none());
}

// -------------------------------------------------------------------------
// Input validation tests
// -------------------------------------------------------------------------

#[test]
fn test_enroll_empty_user_id_returns_bad_request() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    let result = handlers.enroll(
        &caller(""),
        EnableTwoFactorRequest {
            user_id: "".to_string(),
            email: "test@example.com".to_string(),
            idempotency_key: None,
        },
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, "BAD_REQUEST");
    assert!(err.message.contains("user_id must not be empty"));
}

#[test]
fn test_enroll_empty_email_returns_bad_request() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    let result = handlers.enroll(
        &caller("test-user"),
        EnableTwoFactorRequest {
            user_id: "test-user".to_string(),
            email: "".to_string(),
            idempotency_key: None,
        },
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, "BAD_REQUEST");
    assert!(err.message.contains("email must not be empty"));
}

#[test]
fn test_enroll_overlong_user_id_returns_bad_request() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    let overlong_user_id = "a".repeat(256);
    let result = handlers.enroll(
        &caller(&overlong_user_id),
        EnableTwoFactorRequest {
            user_id: overlong_user_id.clone(),
            email: "test@example.com".to_string(),
            idempotency_key: None,
        },
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, "BAD_REQUEST");
    assert!(err
        .message
        .contains("user_id must not exceed 255 characters"));
}

#[test]
fn test_enroll_overlong_email_returns_bad_request() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    let overlong_email = format!("{}@example.com", "a".repeat(300));
    let result = handlers.enroll(
        &caller("test-user"),
        EnableTwoFactorRequest {
            user_id: "test-user".to_string(),
            email: overlong_email,
            idempotency_key: None,
        },
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, "BAD_REQUEST");
    assert!(err.message.contains("email must not exceed 255 characters"));
}
