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
fn test_setup_with_default_backup_code_count() {
    // Default backup code count should be 8
    let config = TotpConfig::default();
    assert_eq!(config.backup_code_count, 8);
    let result = TwoFactorAuth::setup_with_config("test@petchain.com", "PetChain", config.clone());
    assert!(result.is_ok());
    let setup = result.unwrap();
    assert_eq!(setup.backup_codes.len(), 8);
}

#[test]
fn test_setup_with_custom_backup_code_count() {
    // Custom backup code count should be respected
    let mut config = TotpConfig::default();
    config.backup_code_count = 12;
    let result = TwoFactorAuth::setup_with_config("test@petchain.com", "PetChain", config.clone());
    assert!(result.is_ok());
    let setup = result.unwrap();
    assert_eq!(setup.backup_codes.len(), 12);
}

#[test]
fn test_generate_backup_codes() {
    let codes = TwoFactorAuth::generate_backup_codes(8);
    assert_eq!(codes.len(), 8);
    for code in &codes {
        assert!(code.contains('-'));
        assert_eq!(code.len(), 9);
    }
    let unique: std::collections::HashSet<_> = codes.iter().collect();
    assert_eq!(unique.len(), 8);
}

#[test]
fn test_verify_backup_code() {
    let codes = vec!["1234-5678".to_string(), "2345-6789".to_string()];
    assert_eq!(
        TwoFactorAuth::verify_backup_code(&codes, "2345-6789"),
        Some(1)
    );
    assert_eq!(TwoFactorAuth::verify_backup_code(&codes, "9999-9999"), None);
}

#[test]
fn test_handlers_use_configurable_mock_store_for_enroll_verify_disable_recover() {
    let store = std::sync::Arc::new(MockTwoFactorStore::new());
    let handlers = TwoFactorHandlers::with_store_and_issuer(store.clone(), "Pet Chain: Ops");
    let user_id = "mock-user";

    let enrollment = handlers
        .enroll(
            &caller(user_id),
            EnableTwoFactorRequest {
                idempotency_key: None,
                user_id: user_id.to_string(),
                email: "mock+user@example.com".to_string(),
            },
        )
        .unwrap();

    assert!(enrollment
        .otpauth_uri
        .starts_with("otpauth://totp/Pet%20Chain%20%20Ops:mock%2Buser%40example.com?"));
    assert!(enrollment
        .otpauth_uri
        .contains("&issuer=Pet%20Chain%20%20Ops"));

    let activated = handlers
        .verify_and_activate(
            &caller(user_id),
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token: generate_token(&enrollment.secret),
            },
        )
        .unwrap();
    assert!(activated);
    assert!(store.get_data(user_id).unwrap().enabled);

    let disabled = handlers
        .disable_two_factor(
            &caller(user_id),
            DisableTwoFactorRequest {
                user_id: user_id.to_string(),
                token: generate_token(&enrollment.secret),
            },
        )
        .unwrap();
    assert!(disabled);

    let mut data = store.get_data(user_id).unwrap();
    data.enabled = true;
    let backup_code = data.backup_codes[0].clone();
    store.seed(user_id, data);

    let recovered = handlers
        .recover(
            &caller(user_id),
            RecoverWithBackupRequest {
                user_id: user_id.to_string(),
                backup_code,
            },
            Some("127.0.0.1"),
        )
        .unwrap();
    assert!(recovered.enabled);
    assert!(recovered.new_otpauth_uri.starts_with("otpauth://totp/"));
}

#[test]
fn test_mock_store_error_and_timeout_injection() {
    let failing_save = std::sync::Arc::new(MockTwoFactorStore::with_config(MockStoreConfig {
        save: Some(MockStoreFailure::Error("save failed".to_string())),
        ..Default::default()
    }));
    let handlers = TwoFactorHandlers::with_store(failing_save);
    let err = handlers
        .enroll(
            &caller("mock-fail"),
            EnableTwoFactorRequest {
                idempotency_key: None,
                user_id: "mock-fail".to_string(),
                email: "mock-fail@example.com".to_string(),
            },
        )
        .unwrap_err();
    assert_eq!(err.message, "save failed");

    let timeout_get = std::sync::Arc::new(MockTwoFactorStore::with_config(MockStoreConfig {
        get: Some(MockStoreFailure::Timeout),
        ..Default::default()
    }));
    let handlers = TwoFactorHandlers::with_store(timeout_get);
    let err = handlers
        .verify_and_activate(
            &caller("mock-timeout"),
            VerifyTwoFactorRequest {
                user_id: "mock-timeout".to_string(),
                token: "123456".to_string(),
            },
        )
        .unwrap_err();
    assert!(err.message.contains("not configured"));
}

// --- backup code single-use tests ---

#[test]
fn test_consume_backup_code_removes_code() {
    let mut codes = vec![
        "1111-2222".to_string(),
        "3333-4444".to_string(),
        "5555-6666".to_string(),
    ];

    let consumed = TwoFactorAuth::consume_backup_code(&mut codes, "3333-4444");
    assert!(consumed);
    assert_eq!(codes.len(), 2);
    assert!(!codes.contains(&"3333-4444".to_string()));
}

#[test]
fn test_handler_recovery_invalid_code_returns_error() {
    clear_two_factor_store_for_tests();
    let user_id = "handler-user8";
    let resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "u8@petchain.com".to_string(),
        },
    )
    .unwrap();
    overwrite_two_factor_data_for_tests(
        user_id,
        crate::two_factor::TwoFactorData {
            secret: resp.secret,
            backup_codes: resp.backup_codes,
            enabled: true,
            algorithm: Algorithm::SHA1,
            last_used_step: None,
        },
    );

    let result = TwoFactorHandlers::recover_with_backup(
        &caller(user_id),
        RecoverWithBackupRequest {
            user_id: user_id.to_string(),
            backup_code: "0000-0000".to_string(),
        },
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("InvalidRecoveryCode"));
}

#[test]
fn test_handler_recovery_when_not_enabled_returns_error() {
    clear_two_factor_store_for_tests();
    let user_id = "handler-user9";
    TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "u9@petchain.com".to_string(),
        },
    )
    .unwrap();

    let err = TwoFactorHandlers::recover_with_backup(
        &caller(user_id),
        RecoverWithBackupRequest {
            user_id: user_id.to_string(),
            backup_code: "1234-5678".to_string(),
        },
    );
    assert!(err.is_err());
    assert!(err.unwrap_err().message.contains("not enabled"));
}

// -----------------------------------------------------------------------
// Flow 2: enable → recover with backup code → login with new secret
// -----------------------------------------------------------------------

/// A user loses their authenticator app. They recover using a backup code,
/// which issues a new secret. They can then log in with the new secret.
#[test]
fn test_full_enable_recover_login_flow() {
    let user_id = "integration-recover-flow-user";
    let handlers = TwoFactorHandlers::new();

    // Enable 2FA
    let enable_resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "recover@petchain.com".to_string(),
        },
    )
    .unwrap();

    // Activate via verify_and_activate (no overwrite needed)
    let activated = handlers
        .verify_and_activate(
            &caller(user_id),
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token: generate_token(&enable_resp.secret),
            },
        )
        .unwrap();
    assert!(activated);

    // Pick the first backup code
    let backup_code = enable_resp.backup_codes[0].clone();

    // Recover — should issue a brand-new secret and backup codes
    let recovery_resp = TwoFactorHandlers::recover_with_backup(
        &caller(user_id),
        RecoverWithBackupRequest {
            user_id: user_id.to_string(),
            backup_code: backup_code.clone(),
        },
    )
    .expect("recovery should succeed with valid backup code");

    assert!(
        recovery_resp.enabled,
        "2FA must remain enabled after recovery"
    );
    assert_ne!(
        recovery_resp.new_secret, enable_resp.secret,
        "recovery must issue a new secret"
    );
    assert_eq!(recovery_resp.new_backup_codes.len(), 8);

    // The consumed backup code must no longer work
    let second_recovery = TwoFactorHandlers::recover_with_backup(
        &caller(user_id),
        RecoverWithBackupRequest {
            user_id: user_id.to_string(),
            backup_code,
        },
    );
    assert!(
        second_recovery.is_err(),
        "consumed backup code must not be reusable"
    );

    // Login with the new secret must succeed
    let logged_in = handlers
        .verify_login_token(
            &caller(user_id),
            LoginWithTwoFactorRequest {
                user_id: user_id.to_string(),
                token: generate_token(&recovery_resp.new_secret),
            },
        )
        .expect("login with new secret should not error");

    assert!(
        logged_in,
        "login must succeed with the new secret after recovery"
    );

    // Login with the OLD secret must fail
    let old_login = handlers
        .verify_login_token(
            &caller(user_id),
            LoginWithTwoFactorRequest {
                user_id: user_id.to_string(),
                token: generate_token(&enable_resp.secret),
            },
        )
        .expect("login call with old secret should not error");

    assert!(
        !old_login,
        "old secret must no longer be valid after recovery"
    );
}

// -----------------------------------------------------------------------
// Flow 2b: recovery code rotation atomicity
// -----------------------------------------------------------------------

/// After a successful recovery-code login, ALL old backup codes (both
/// the consumed one and the unused ones) must be invalidated atomically,
/// and a fresh set of recovery codes must be returned.
#[test]
fn test_recovery_code_rotation_atomicity() {
    let user_id = "integration-rotation-atomicity-user";
    let handlers = TwoFactorHandlers::new();

    // Enable 2FA
    let enable_resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "rotation@petchain.com".to_string(),
        },
    )
    .unwrap();

    // Activate
    handlers
        .verify_and_activate(
            &caller(user_id),
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token: generate_token(&enable_resp.secret),
            },
        )
        .unwrap();

    let old_backup_codes = enable_resp.backup_codes.clone();
    assert_eq!(old_backup_codes.len(), 8);

    // Recover using the first backup code
    let recovery_resp = TwoFactorHandlers::recover_with_backup(
        &caller(user_id),
        RecoverWithBackupRequest {
            user_id: user_id.to_string(),
            backup_code: old_backup_codes[0].clone(),
        },
    )
    .expect("recovery should succeed");

    // New recovery codes are returned in the response
    assert_eq!(recovery_resp.new_recovery_codes.len(), 8);
    assert_eq!(
        recovery_resp.new_backup_codes, recovery_resp.new_recovery_codes,
        "new_backup_codes and new_recovery_codes must match"
    );
    assert_ne!(
        recovery_resp.new_recovery_codes, old_backup_codes,
        "new recovery codes must differ from old ones"
    );

    // The consumed old code must be invalid
    let reuse_consumed = TwoFactorHandlers::recover_with_backup(
        &caller(user_id),
        RecoverWithBackupRequest {
            user_id: user_id.to_string(),
            backup_code: old_backup_codes[0].clone(),
        },
    );
    assert!(
        reuse_consumed.is_err(),
        "consumed backup code must not be reusable after rotation"
    );

    // UNUSED old codes must also be invalid (this is the atomicity check)
    for i in 1..old_backup_codes.len() {
        let reuse_unused = TwoFactorHandlers::recover_with_backup(
            &caller(user_id),
            RecoverWithBackupRequest {
                user_id: user_id.to_string(),
                backup_code: old_backup_codes[i].clone(),
            },
        );
        assert!(
            reuse_unused.is_err(),
            "unused old backup code [{}] must also be invalidated after rotation",
            i
        );
    }

    // Login with the new secret must succeed
    let logged_in = handlers
        .verify_login_token(
            &caller(user_id),
            LoginWithTwoFactorRequest {
                user_id: user_id.to_string(),
                token: generate_token(&recovery_resp.new_secret),
            },
        )
        .expect("login with new secret after rotation should not error");
    assert!(
        logged_in,
        "login must succeed with the new secret after rotation"
    );

    // A new recovery code from the rotated set must be usable
    let second_recovery = TwoFactorHandlers::recover_with_backup(
        &caller(user_id),
        RecoverWithBackupRequest {
            user_id: user_id.to_string(),
            backup_code: recovery_resp.new_recovery_codes[0].clone(),
        },
    );
    assert!(
        second_recovery.is_ok(),
        "a fresh recovery code from the rotated set must be valid"
    );
}

// ── Recovery Code Single-Use Enforcement Tests ──

#[test]
fn test_recovery_code_first_use_succeeds() {
    clear_two_factor_store_for_tests();
    let user_id = "recovery-user-1";
    let caller_user = caller(user_id);

    // Enable 2FA
    let setup = TwoFactorHandlers::enable_two_factor(
        &caller_user,
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "user@petchain.com".to_string(),
        },
    )
    .unwrap();

    let token = generate_token(&setup.secret);
    let handler = TwoFactorHandlers::new();
    handler
        .verify_and_activate(
            &caller_user,
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token: token.clone(),
            },
        )
        .unwrap();

    // Attempt recovery
    let backup_code = setup.backup_codes[0].clone();
    let result = TwoFactorHandlers::recover_with_backup_with_ip(
        &caller_user,
        RecoverWithBackupRequest {
            user_id: user_id.to_string(),
            backup_code: backup_code.clone(),
        },
        Some("192.168.1.1"),
    );

    assert!(result.is_ok(), "First recovery use should succeed");
}

#[test]
fn test_recovery_code_second_use_rejected() {
    clear_two_factor_store_for_tests();
    let user_id = "recovery-user-2";
    let caller_user = caller(user_id);

    // Enable 2FA
    let setup = TwoFactorHandlers::enable_two_factor(
        &caller_user,
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "user@petchain.com".to_string(),
        },
    )
    .unwrap();

    let token = generate_token(&setup.secret);
    let handler = TwoFactorHandlers::new();
    handler
        .verify_and_activate(
            &caller_user,
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token,
            },
        )
        .unwrap();

    let backup_code = setup.backup_codes[0].clone();

    // First recovery succeeds
    let first = TwoFactorHandlers::recover_with_backup_with_ip(
        &caller_user,
        RecoverWithBackupRequest {
            user_id: user_id.to_string(),
            backup_code: backup_code.clone(),
        },
        Some("192.168.1.1"),
    );
    assert!(first.is_ok());

    // Second recovery should fail
    let second = TwoFactorHandlers::recover_with_backup_with_ip(
        &caller_user,
        RecoverWithBackupRequest {
            user_id: user_id.to_string(),
            backup_code,
        },
        Some("192.168.1.2"),
    );

    assert!(second.is_err());
    assert!(second.unwrap_err().message.contains("InvalidRecoveryCode"));
}

#[test]
fn test_recovery_log_entry_written() {
    clear_two_factor_store_for_tests();
    let user_id = "recovery-user-3";
    let caller_user = caller(user_id);

    // Enable 2FA
    let setup = TwoFactorHandlers::enable_two_factor(
        &caller_user,
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "user@petchain.com".to_string(),
        },
    )
    .unwrap();

    let token = generate_token(&setup.secret);
    let handler = TwoFactorHandlers::new();
    handler
        .verify_and_activate(
            &caller_user,
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token,
            },
        )
        .unwrap();

    let backup_code = setup.backup_codes[0].clone();

    // Use recovery code
    let _ = TwoFactorHandlers::recover_with_backup_with_ip(
        &caller_user,
        RecoverWithBackupRequest {
            user_id: user_id.to_string(),
            backup_code,
        },
        Some("10.0.0.1"),
    );

    // Check recovery log
    let log = AdminRecoveryHandlers::get_recovery_log(1, 10).unwrap();
    assert!(
        log.len() > 0,
        "Recovery log should have entries after code usage"
    );

    let entry = &log[0];
    assert_eq!(entry.user_id, user_id);
    assert_eq!(entry.code_index, 0);
    assert_eq!(entry.ip_address, Some("10.0.0.1".to_string()));
}

#[test]
fn test_recovery_log_pagination() {
    clear_two_factor_store_for_tests();

    // Create multiple recovery log entries
    for i in 0..15 {
        let user_id = format!("user-{}", i);
        let c = caller(&user_id);

        let setup = TwoFactorHandlers::enable_two_factor(
            &c,
            EnableTwoFactorRequest {
                idempotency_key: None,
                user_id: user_id.clone(),
                email: format!("{}@petchain.com", user_id),
            },
        )
        .unwrap();

        let token = generate_token(&setup.secret);
        let handler = TwoFactorHandlers::new();
        handler
            .verify_and_activate(
                &c,
                VerifyTwoFactorRequest {
                    user_id: user_id.clone(),
                    token,
                },
            )
            .ok();

        let backup_code = setup.backup_codes[0].clone();
        let _ = TwoFactorHandlers::recover_with_backup_with_ip(
            &c,
            RecoverWithBackupRequest {
                user_id,
                backup_code,
            },
            None,
        );
    }

    // Test pagination
    let page1 = AdminRecoveryHandlers::get_recovery_log(1, 10).unwrap();
    let page2 = AdminRecoveryHandlers::get_recovery_log(2, 10).unwrap();

    assert_eq!(page1.len(), 10);
    assert!(page2.len() > 0);
    assert!(page2.len() <= 10);

    // Verify reverse chronological order
    if page1.len() > 1 {
        assert!(page1[0].used_at >= page1[1].used_at);
    }
}

#[test]
fn test_recovery_log_fields_correct() {
    clear_two_factor_store_for_tests();
    let user_id = "field-test-user";
    let caller_user = caller(user_id);

    // Enable and setup
    let setup = TwoFactorHandlers::enable_two_factor(
        &caller_user,
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "user@petchain.com".to_string(),
        },
    )
    .unwrap();

    let token = generate_token(&setup.secret);
    let handler = TwoFactorHandlers::new();
    handler
        .verify_and_activate(
            &caller_user,
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token,
            },
        )
        .unwrap();

    // Use second backup code (index 1)
    let backup_code = setup.backup_codes[1].clone();
    let ip = "203.0.113.42";

    let _ = TwoFactorHandlers::recover_with_backup_with_ip(
        &caller_user,
        RecoverWithBackupRequest {
            user_id: user_id.to_string(),
            backup_code,
        },
        Some(ip),
    );

    // Verify log entry
    let log = AdminRecoveryHandlers::get_recovery_log(1, 10).unwrap();
    let entry = log.iter().find(|e| e.user_id == user_id).unwrap();

    assert_eq!(entry.user_id, user_id);
    assert_eq!(entry.code_index, 1);
    assert_eq!(entry.ip_address.as_deref(), Some(ip));
    assert!(!entry.used_at.is_empty());
}

#[test]
fn test_recovery_all_backup_codes_exhausted() {
    clear_two_factor_store_for_tests();
    let user_id = "recovery-exhausted-user";
    let caller_user = caller(user_id);

    // Enable and activate 2FA
    let setup = TwoFactorHandlers::enable_two_factor(
        &caller_user,
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "user@petchain.com".to_string(),
        },
    )
    .unwrap();
    assert_eq!(setup.backup_codes.len(), 8);

    let token = generate_token(&setup.secret);
    let handler = TwoFactorHandlers::new();
    handler
        .verify_and_activate(
            &caller_user,
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token,
            },
        )
        .unwrap();

    // Use all 8 backup codes one at a time; each recovery issues a fresh set.
    let mut current_codes = setup.backup_codes.clone();
    for i in 0..8 {
        let resp = TwoFactorHandlers::recover_with_backup(
            &caller_user,
            RecoverWithBackupRequest {
                user_id: user_id.to_string(),
                backup_code: current_codes[0].clone(),
            },
        )
        .unwrap_or_else(|e| panic!("Recovery {} failed: {:?}", i, e));
        current_codes = resp.new_backup_codes;
        assert_eq!(current_codes.len(), 8);
    }

    // The original codes are entirely stale — none should work any more.
    for old_code in &setup.backup_codes {
        let result = TwoFactorHandlers::recover_with_backup(
            &caller_user,
            RecoverWithBackupRequest {
                user_id: user_id.to_string(),
                backup_code: old_code.clone(),
            },
        );
        assert!(
            result.is_err(),
            "Old code should be invalid after exhaustion"
        );
        assert!(result.unwrap_err().message.contains("InvalidRecoveryCode"));
    }

    // The newest code set works exactly once.
    let fresh_code = current_codes[0].clone();
    let first_use = TwoFactorHandlers::recover_with_backup(
        &caller_user,
        RecoverWithBackupRequest {
            user_id: user_id.to_string(),
            backup_code: fresh_code.clone(),
        },
    );
    assert!(first_use.is_ok(), "First use of fresh code should succeed");

    let second_use = TwoFactorHandlers::recover_with_backup(
        &caller_user,
        RecoverWithBackupRequest {
            user_id: user_id.to_string(),
            backup_code: fresh_code,
        },
    );
    assert!(
        second_use.is_err(),
        "Second use of same code should be rejected"
    );
    assert!(second_use
        .unwrap_err()
        .message
        .contains("InvalidRecoveryCode"));
}

#[test]
fn test_upgrade_algorithm_new_backup_codes_generated() {
    clear_two_factor_store_for_tests();

    let user_id = "user-new-backup-codes";

    // Enroll and activate
    let resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "newcodes@petchain.com".to_string(),
        },
    )
    .unwrap();

    let old_backup_codes = resp.backup_codes.clone();

    let handlers = TwoFactorHandlers::new();
    let token = generate_token(&resp.secret);
    handlers
        .verify_and_activate(
            &caller(user_id),
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token: token.clone(),
            },
        )
        .unwrap();

    // Upgrade
    let upgrade_resp = handlers
        .upgrade_algorithm(
            &caller(user_id),
            UpgradeAlgorithmRequest {
                user_id: user_id.to_string(),
                token,
            },
        )
        .unwrap();

    // New backup codes should be different
    assert_ne!(upgrade_resp.new_backup_codes, old_backup_codes);
    assert_eq!(upgrade_resp.new_backup_codes.len(), 8);

    // Verify they're stored
    let data = get_two_factor_data_for_tests(user_id).unwrap();
    assert_eq!(data.backup_codes, upgrade_resp.new_backup_codes);
}

#[test]
fn test_upgrade_algorithm_old_secret_invalidated() {
    clear_two_factor_store_for_tests();

    let user_id = "user-old-secret-invalid";

    // Enroll and activate
    let resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "invalid@petchain.com".to_string(),
        },
    )
    .unwrap();

    let old_secret = resp.secret.clone();
    let handlers = TwoFactorHandlers::new();
    let token = generate_token(&old_secret);

    handlers
        .verify_and_activate(
            &caller(user_id),
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token: token.clone(),
            },
        )
        .unwrap();

    // Upgrade
    let upgrade_resp = handlers
        .upgrade_algorithm(
            &caller(user_id),
            UpgradeAlgorithmRequest {
                user_id: user_id.to_string(),
                token,
            },
        )
        .unwrap();

    // Old secret should be replaced
    let data = get_two_factor_data_for_tests(user_id).unwrap();
    assert_ne!(data.secret, old_secret);
    assert_eq!(data.secret, upgrade_resp.new_secret);

    // Old tokens should no longer work
    let old_token = generate_token(&old_secret);
    let login_result = handlers.verify_login_token(
        &caller(user_id),
        LoginWithTwoFactorRequest {
            user_id: user_id.to_string(),
            token: old_token,
        },
    );

    // Should fail because the secret changed
    assert!(login_result.is_ok());
    assert!(!login_result.unwrap());
}

#[test]
fn test_two_factor_handlers_new_with_defaults() {
    let handlers = TwoFactorHandlers::new_with_defaults();
    let _limiter = handlers.limiter();
}

#[test]
fn test_recover_empty_user_id_returns_bad_request() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    let result = handlers.recover(
        &caller(""),
        RecoverWithBackupRequest {
            user_id: "".to_string(),
            backup_code: "12345678".to_string(),
        },
        None,
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, "BAD_REQUEST");
    assert!(err.message.contains("user_id must not be empty"));
}

#[test]
fn test_upgrade_algorithm_empty_user_id_returns_bad_request() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    let result = handlers.upgrade_algorithm(
        &caller(""),
        UpgradeAlgorithmRequest {
            user_id: "".to_string(),
            token: "123456".to_string(),
        },
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, "BAD_REQUEST");
    assert!(err.message.contains("user_id must not be empty"));
}

#[test]
fn test_upgrade_algorithm_invalid_token_returns_bad_request() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    let result = handlers.upgrade_algorithm(
        &caller("test-user"),
        UpgradeAlgorithmRequest {
            user_id: "test-user".to_string(),
            token: "12a456".to_string(),
        },
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, "BAD_REQUEST");
    assert!(err
        .message
        .contains("token must contain only decimal digits"));
}

#[test]
fn test_tenant_scoped_recovery_isolation() {
    clear_two_factor_store_for_tests();
    let handlers_a = TwoFactorHandlers::for_tenant("tenant-a");
    let handlers_b = TwoFactorHandlers::for_tenant("tenant-b");

    // Enroll and activate user "dave" in tenant-a
    let resp_a = handlers_a
        .enroll(
            &caller("dave"),
            EnableTwoFactorRequest {
                user_id: "dave".to_string(),
                email: "dave@tenant-a.com".to_string(),
                idempotency_key: None,
            },
        )
        .unwrap();

    let token_a = generate_token(&resp_a.secret);
    handlers_a
        .verify_and_activate(
            &caller("dave"),
            VerifyTwoFactorRequest {
                user_id: "dave".to_string(),
                token: token_a,
            },
        )
        .unwrap();

    let backup_code_a = resp_a.backup_codes[0].clone();

    // Try to recover using tenant-b handler with tenant-a's backup code
    let recover_result = handlers_b.recover(
        &caller("dave"),
        RecoverWithBackupRequest {
            user_id: "dave".to_string(),
            backup_code: backup_code_a,
        },
        None,
    );

    // Should fail because dave doesn't exist in tenant-b
    assert!(recover_result.is_err());
}
