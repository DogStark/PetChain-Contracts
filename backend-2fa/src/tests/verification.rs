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
fn test_totp_config_validation_valid_configs() {
    // Valid configs should succeed
    let config = TotpConfig::new(Algorithm::SHA1, 6, 30, 1).unwrap();
    assert_eq!(config.digits, 6);
    assert_eq!(config.period, 30);
    assert_eq!(config.window, 1);

    let config = TotpConfig::new(Algorithm::SHA256, 7, 60, 2).unwrap();
    assert_eq!(config.digits, 7);
    assert_eq!(config.period, 60);
    assert_eq!(config.window, 2);

    let config = TotpConfig::new(Algorithm::SHA512, 8, 90, 10).unwrap();
    assert_eq!(config.digits, 8);
    assert_eq!(config.period, 90);
    assert_eq!(config.window, 10);
}

#[test]
fn test_totp_config_validation_invalid_digits() {
    // Digits too small
    let result = TotpConfig::new(Algorithm::SHA1, 5, 30, 1);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("digits must be between 6 and 8"));

    // Digits too large
    let result = TotpConfig::new(Algorithm::SHA1, 9, 30, 1);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("digits must be between 6 and 8"));

    // Digits zero
    let result = TotpConfig::new(Algorithm::SHA1, 0, 30, 1);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("digits must be between 6 and 8"));
}

#[test]
fn test_totp_config_validation_invalid_period() {
    // Period zero
    let result = TotpConfig::new(Algorithm::SHA1, 6, 0, 1);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("period must be greater than 0"));
}

#[test]
fn test_totp_config_validation_invalid_window() {
    // Window too large
    let result = TotpConfig::new(Algorithm::SHA1, 6, 30, 11);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("window must be <= 10"));
}

#[test]
fn test_verify_token_default_sha256() {
    let secret = TwoFactorAuth::generate_secret();
    let config = TotpConfig::default();

    let totp = TOTP::new(
        config.algorithm,
        config.digits,
        config.window,
        config.period,
        Secret::Encoded(secret.clone()).to_bytes().unwrap(),
        None,
        String::new(),
    )
    .unwrap();

    let token = totp.generate_current().unwrap();

    let result = TwoFactorAuth::verify_token(&secret, &token);
    assert!(result.is_ok());
    assert!(result.unwrap());

    let result = TwoFactorAuth::verify_token_with_config(&secret, &token, config);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_verify_token_valid() {
    let secret = TwoFactorAuth::generate_secret();
    let token = generate_token(&secret);
    let result = TwoFactorAuth::verify_token(&secret, &token);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_verify_token_sha1_config() {
    let secret = TwoFactorAuth::generate_secret();
    let config = TotpConfig::legacy_sha1();

    // Generate current token with SHA1
    let totp = TOTP::new(
        config.algorithm,
        config.digits,
        config.window,
        config.period,
        Secret::Encoded(secret.clone()).to_bytes().unwrap(),
        None,
        String::new(),
    )
    .unwrap();

    let token = totp.generate_current().unwrap();

    // Verify it with SHA1 config
    let result = TwoFactorAuth::verify_token_with_config(&secret, &token, config);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_verify_token_sha512_config() {
    let secret = TwoFactorAuth::generate_secret();
    let config = TotpConfig::high_security();

    // Generate current token with SHA512 and 8 digits
    let totp = TOTP::new(
        config.algorithm,
        config.digits,
        config.window,
        config.period,
        Secret::Encoded(secret.clone()).to_bytes().unwrap(),
        None,
        String::new(),
    )
    .unwrap();

    let token = totp.generate_current().unwrap();
    assert_eq!(token.len(), 8); // Should be 8 digits

    // Verify it with SHA512 config
    let result = TwoFactorAuth::verify_token_with_config(&secret, &token, config);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_algorithm_mismatch() {
    let secret = TwoFactorAuth::generate_secret();
    let sha1_config = TotpConfig::legacy_sha1();
    let sha256_config = TotpConfig {
        algorithm: Algorithm::SHA256,
        digits: 6,
        period: 30,
        window: 1,
        backup_code_count: 8,
    };

    // Generate token with SHA1
    let totp_sha1 = TOTP::new(
        sha1_config.algorithm,
        sha1_config.digits,
        sha1_config.window,
        sha1_config.period,
        Secret::Encoded(secret.clone()).to_bytes().unwrap(),
        None,
        String::new(),
    )
    .unwrap();

    let token = totp_sha1.generate_current().unwrap();

    // Should work with SHA1 config
    let result = TwoFactorAuth::verify_token_with_config(&secret, &token, sha1_config);
    assert!(result.is_ok());
    assert!(result.unwrap());

    // Should NOT work with SHA256 config (different algorithm)
    let result = TwoFactorAuth::verify_token_with_config(&secret, &token, sha256_config);
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

/// Failure path: user with no 2FA record cannot activate.
#[test]
fn test_verify_and_activate_fails_when_no_record() {
    clear_two_factor_store_for_tests();

    let handlers = TwoFactorHandlers::new();
    let result = handlers.verify_and_activate(
        &caller("ghost"),
        VerifyTwoFactorRequest {
            user_id: "ghost".to_string(),
            token: "123456".to_string(),
        },
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("not configured"));
}

// -----------------------------------------------------------------------
// verify_and_activate
// -----------------------------------------------------------------------

#[test]
fn test_verify_and_activate_persists_enabled_state() {
    clear_two_factor_store_for_tests();

    let user_id = "user-activate";
    let resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "activate@petchain.com".to_string(),
        },
    )
    .unwrap();

    assert!(!get_two_factor_data_for_tests(user_id).unwrap().enabled);

    let handlers = TwoFactorHandlers::new();
    let ok = handlers
        .verify_and_activate(
            &caller(user_id),
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token: generate_token(&resp.secret),
            },
        )
        .unwrap();

    assert!(ok);
    let stored = get_two_factor_data_for_tests(user_id).unwrap();
    assert!(stored.enabled);
    assert_eq!(stored.secret, resp.secret);
}

#[test]
fn test_activation_does_not_persist_on_failed_verification() {
    clear_two_factor_store_for_tests();

    let user_id = "user-no-partial-activation";
    let resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "no-partial@petchain.com".to_string(),
        },
    )
    .unwrap();

    let invalid_secret = "JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PX";
    assert_ne!(resp.secret, invalid_secret);

    let handlers = TwoFactorHandlers::new();
    let result = handlers
        .verify_and_activate(
            &caller(user_id),
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token: generate_token(invalid_secret),
            },
        )
        .unwrap();

    assert!(!result);
    assert!(!get_two_factor_data_for_tests(user_id).unwrap().enabled);
}

// -----------------------------------------------------------------------
// verify_login_token
// -----------------------------------------------------------------------

#[test]
fn test_verify_login_token_returns_false_when_disabled() {
    clear_two_factor_store_for_tests();

    let user_id = "user-disabled";
    let secret = TwoFactorAuth::generate_secret();
    let token = generate_token(&secret);

    overwrite_two_factor_data_for_tests(
        user_id,
        TwoFactorData {
            secret,
            backup_codes: vec![],
            enabled: false,
            algorithm: Algorithm::SHA1,
            last_used_step: None,
        },
    );

    let handlers = TwoFactorHandlers::new();
    let result = handlers
        .verify_login_token(
            &caller(user_id),
            LoginWithTwoFactorRequest {
                user_id: user_id.to_string(),
                token,
            },
        )
        .unwrap();

    assert!(!result);
    assert!(!get_two_factor_data_for_tests(user_id).unwrap().enabled);
}

#[test]
fn test_verify_login_token_succeeds_with_correct_token_when_enabled() {
    clear_two_factor_store_for_tests();

    let user_id = "user-enabled-ok";
    let resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "enabled@petchain.com".to_string(),
        },
    )
    .unwrap();

    overwrite_two_factor_data_for_tests(
        user_id,
        TwoFactorData {
            secret: resp.secret.clone(),
            backup_codes: resp.backup_codes,
            enabled: true,
            algorithm: Algorithm::SHA1,
            last_used_step: None,
        },
    );

    let handlers = TwoFactorHandlers::new();
    let result = handlers
        .verify_login_token(
            &caller(user_id),
            LoginWithTwoFactorRequest {
                user_id: user_id.to_string(),
                token: generate_token(&resp.secret),
            },
        )
        .unwrap();

    assert!(result);
}

/// Verifies that the stored secret (not a placeholder) is used for token validation.
#[test]
fn test_verify_uses_stored_secret_not_placeholder() {
    clear_two_factor_store_for_tests();

    let user_id = "user-secret-check";
    let stored_secret = TwoFactorAuth::generate_secret();
    let placeholder_secret = "JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PX";
    let placeholder_token = generate_token(placeholder_secret);

    overwrite_two_factor_data_for_tests(
        user_id,
        TwoFactorData {
            secret: stored_secret.clone(),
            backup_codes: vec![],
            enabled: true,
            algorithm: Algorithm::SHA1,
            last_used_step: None,
        },
    );

    // A token generated from the placeholder secret must NOT validate
    // against the stored (different) secret.
    let handlers = TwoFactorHandlers::new();
    let result = handlers
        .verify_login_token(
            &caller(user_id),
            LoginWithTwoFactorRequest {
                user_id: user_id.to_string(),
                token: placeholder_token,
            },
        )
        .unwrap();

    assert!(
        !result,
        "placeholder token must not validate against the stored secret"
    );
}

// -----------------------------------------------------------------------
// Rate limiter unit tests
// -----------------------------------------------------------------------

// -----------------------------------------------------------------------
// TOTP Replay Prevention Tests (Issue #840)
// -----------------------------------------------------------------------

#[cfg(test)]
mod replay_tests {
    use crate::two_factor::{TotpConfig, TwoFactorAuth};
    use totp_rs::Algorithm;

    fn generate_token(secret: &str) -> String {
        use totp_rs::{Secret, TOTP};
        TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.to_string()).to_bytes().unwrap(),
            None,
            String::new(),
        )
        .unwrap()
        .generate_current()
        .unwrap()
    }

    fn current_step(period: u64) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() / period)
            .unwrap_or(0)
    }

    #[test]
    fn test_totp_replay_single_acceptance() {
        let secret = TwoFactorAuth::generate_secret();
        let config = TotpConfig::default();
        let token = generate_token(&secret);
        let result = TwoFactorAuth::verify_token_with_config(&secret, &token, config).unwrap();
        assert!(result, "First use of token should be valid");
    }

    #[test]
    fn test_totp_replay_same_token_still_valid_in_window() {
        // verify_token_with_config has no replay protection built in —
        // that lives at the store/handler layer (set_last_used_step).
        // Here we just confirm the token verifies twice (no internal state).
        let secret = TwoFactorAuth::generate_secret();
        let config = TotpConfig::default();
        let token = generate_token(&secret);
        let r1 = TwoFactorAuth::verify_token_with_config(&secret, &token, config.clone()).unwrap();
        let r2 = TwoFactorAuth::verify_token_with_config(&secret, &token, config).unwrap();
        assert!(r1);
        assert!(r2);
    }

    #[test]
    fn test_totp_current_step_increases_over_time() {
        let step1 = current_step(30);
        // Just verify the helper returns a reasonable value (> 0)
        assert!(step1 > 0, "time step should be positive");
    }
}

// -----------------------------------------------------------------------
// Authorization tests
// -----------------------------------------------------------------------

mod test_authorization {
    use crate::handlers::{
        AuthenticatedUser, DisableTwoFactorRequest, EnableTwoFactorRequest,
        LoginWithTwoFactorRequest, RecoverWithBackupRequest, TwoFactorHandlers,
        VerifyTwoFactorRequest,
    };

    fn caller(id: &str) -> AuthenticatedUser {
        AuthenticatedUser::new(id)
    }

    #[test]
    fn test_enable_two_factor_correct_user_succeeds() {
        let result = TwoFactorHandlers::enable_two_factor(
            &caller("user-1"),
            EnableTwoFactorRequest {
                idempotency_key: None,
                user_id: "user-1".to_string(),
                email: "user1@petchain.com".to_string(),
            },
        );
        assert!(
            result.is_ok(),
            "Owner should be able to enable their own 2FA"
        );
    }

    #[test]
    fn test_enable_two_factor_wrong_user_is_forbidden() {
        let result = TwoFactorHandlers::enable_two_factor(
            &caller("user-1"),
            EnableTwoFactorRequest {
                idempotency_key: None,
                user_id: "user-2".to_string(),
                email: "user2@petchain.com".to_string(),
            },
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "FORBIDDEN");
    }

    #[test]
    fn test_verify_and_activate_wrong_user_is_forbidden() {
        let handlers = TwoFactorHandlers::new();
        let result = handlers.verify_and_activate(
            &caller("user-1"),
            VerifyTwoFactorRequest {
                user_id: "user-99".to_string(),
                token: "123456".to_string(),
            },
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "FORBIDDEN");
    }

    #[test]
    fn test_verify_login_token_wrong_user_is_forbidden() {
        let handlers = TwoFactorHandlers::new();
        let result = handlers.verify_login_token(
            &caller("user-1"),
            LoginWithTwoFactorRequest {
                user_id: "user-99".to_string(),
                token: "123456".to_string(),
            },
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "FORBIDDEN");
    }

    #[test]
    fn test_disable_two_factor_wrong_user_is_forbidden() {
        let handlers = TwoFactorHandlers::new();
        let result = handlers.disable_two_factor(
            &caller("user-1"),
            DisableTwoFactorRequest {
                user_id: "user-99".to_string(),
                token: "123456".to_string(),
            },
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "FORBIDDEN");
    }

    #[test]
    fn test_recover_with_backup_correct_user_proceeds_to_code_check() {
        let result = TwoFactorHandlers::recover_with_backup(
            &caller("user-1"),
            RecoverWithBackupRequest {
                user_id: "user-1".to_string(),
                backup_code: "wrong-code".to_string(),
            },
        );
        assert!(result.is_err());
        // Should fail on missing record or invalid code, NOT on authorization
        let err = result.unwrap_err();
        assert!(
            err.message.contains("Invalid backup code")
                || err.message.contains("not configured")
                || err.message.contains("not enabled"),
            "Correct user should reach the backup code validation step, got: {} ({})",
            err.message,
            err.code
        );
    }

    #[test]
    fn test_recover_with_backup_wrong_user_is_forbidden() {
        let result = TwoFactorHandlers::recover_with_backup(
            &caller("user-1"),
            RecoverWithBackupRequest {
                user_id: "user-99".to_string(),
                backup_code: "1234-5678".to_string(),
            },
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "FORBIDDEN");
    }

    #[test]
    fn test_authorize_same_user_ok() {
        assert!(caller("alice").authorize("alice").is_ok());
    }

    #[test]
    fn test_authorize_different_user_err() {
        let result = caller("alice").authorize("bob");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "FORBIDDEN");
    }

    #[test]
    fn test_authorize_empty_vs_nonempty_is_forbidden() {
        assert!(caller("").authorize("someone").is_err());
    }
}

#[test]
fn test_handler_verify_activates_2fa() {
    clear_two_factor_store_for_tests();
    let user_id = "handler-user2";
    let resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "u2@petchain.com".to_string(),
        },
    )
    .unwrap();
    let token = generate_token(&resp.secret);

    let handlers = TwoFactorHandlers::new();
    let result = handlers.verify_and_activate(
        &caller(user_id),
        VerifyTwoFactorRequest {
            user_id: user_id.to_string(),
            token,
        },
    );
    assert!(result.is_ok());
    assert!(result.unwrap());
    assert!(get_two_factor_data_for_tests(user_id).unwrap().enabled);
}

#[test]
fn test_handler_verify_invalid_token_does_not_activate() {
    clear_two_factor_store_for_tests();
    let user_id = "handler-user3";
    TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "u3@petchain.com".to_string(),
        },
    )
    .unwrap();

    let handlers = TwoFactorHandlers::new();
    let result = handlers.verify_and_activate(
        &caller(user_id),
        VerifyTwoFactorRequest {
            user_id: user_id.to_string(),
            token: "000000".to_string(),
        },
    );
    assert!(result.is_ok());
    assert!(!result.unwrap());
    assert!(!get_two_factor_data_for_tests(user_id).unwrap().enabled);
}

#[test]
fn test_handler_disable_when_not_enabled_returns_false() {
    clear_two_factor_store_for_tests();
    let user_id = "handler-user6";
    TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "u6@petchain.com".to_string(),
        },
    )
    .unwrap();

    let handlers = TwoFactorHandlers::new();
    let result = handlers.disable_two_factor(
        &caller(user_id),
        DisableTwoFactorRequest {
            user_id: user_id.to_string(),
            token: "000000".to_string(),
        },
    );
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

// -----------------------------------------------------------------------
// Flow 1: enable → verify → login → disable
// -----------------------------------------------------------------------

/// Full happy-path: a user enables 2FA, activates it with a valid TOTP
/// token, logs in successfully, then disables it with another valid token.
#[test]
fn test_full_enable_verify_login_disable_flow() {
    let user_id = "integration-enable-verify-login-disable-user";
    let handlers = TwoFactorHandlers::new();

    // Step 1: enable — returns secret + backup codes, 2FA not yet active
    let enable_resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "user1@petchain.com".to_string(),
        },
    )
    .expect("enable should succeed");

    assert!(!enable_resp.secret.is_empty());
    assert_eq!(enable_resp.backup_codes.len(), 8);
    assert!(!get_two_factor_data_for_tests(user_id).unwrap().enabled);

    // Step 2: verify & activate with a live TOTP token
    let activated = handlers
        .verify_and_activate(
            &caller(user_id),
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token: generate_token(&enable_resp.secret),
            },
        )
        .expect("verify_and_activate should succeed");

    assert!(activated, "activation must return true on valid token");
    assert!(get_two_factor_data_for_tests(user_id).unwrap().enabled);

    // Step 3: login with a fresh TOTP token
    let logged_in = handlers
        .verify_login_token(
            &caller(user_id),
            LoginWithTwoFactorRequest {
                user_id: user_id.to_string(),
                token: generate_token(&enable_resp.secret),
            },
        )
        .expect("login should succeed");

    assert!(logged_in, "login must succeed with valid token");

    // Step 4: disable with another valid token
    let disabled = handlers
        .disable_two_factor(
            &caller(user_id),
            DisableTwoFactorRequest {
                user_id: user_id.to_string(),
                token: generate_token(&enable_resp.secret),
            },
        )
        .expect("disable should succeed");

    assert!(disabled, "disable must return true on valid token");
    assert!(!get_two_factor_data_for_tests(user_id).unwrap().enabled);

    // Step 5: login after disable returns false (2FA inactive)
    let post_disable_login = handlers
        .verify_login_token(
            &caller(user_id),
            LoginWithTwoFactorRequest {
                user_id: user_id.to_string(),
                token: generate_token(&enable_resp.secret),
            },
        )
        .expect("login call should not error after disable");

    assert!(
        !post_disable_login,
        "login must return false when 2FA is disabled"
    );
}

/// Rate limit on verify_and_activate is independent from login.
#[test]
fn test_rate_limit_exhaustion_blocks_activation() {
    let user_id = "integration-rate-limit-activation-user";

    let limiter: Arc<dyn RateLimiter> = Arc::new(InMemoryRateLimiter::new(3, 60, 300));
    let handlers = TwoFactorHandlers::with_limiter(Arc::clone(&limiter));

    let enable_resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "user4@petchain.com".to_string(),
        },
    )
    .unwrap();

    // Exhaust verify limit
    for _ in 0..3 {
        let _ = handlers.verify_and_activate(
            &caller(user_id),
            VerifyTwoFactorRequest {
                user_id: user_id.to_string(),
                token: "000000".to_string(),
            },
        );
    }

    // Correct token is still blocked
    let blocked = handlers.verify_and_activate(
        &caller(user_id),
        VerifyTwoFactorRequest {
            user_id: user_id.to_string(),
            token: generate_token(&enable_resp.secret),
        },
    );

    assert!(blocked.is_err());
    assert!(blocked
        .unwrap_err()
        .message
        .contains("Too many failed attempts"));
}

#[test]
fn setup_then_verify_succeeds_for_all_algorithms_and_windows() {
    for &algorithm in &ALGORITHMS {
        for &window in &WINDOWS {
            let config = TotpConfig::new(algorithm, 6, 30, window).expect("valid config");
            let setup =
                TwoFactorAuth::setup_with_config("prop@petchain.com", "PetChain", config.clone())
                    .unwrap_or_else(|e| {
                        panic!("setup failed for {:?} window={}: {}", algorithm, window, e)
                    });

            let totp = TOTP::new(
                algorithm,
                6,
                window,
                30,
                Secret::Encoded(setup.secret.clone()).to_bytes().unwrap(),
                None,
                String::new(),
            )
            .unwrap();
            let token = totp.generate_current().unwrap();

            let verified = TwoFactorAuth::verify_token_with_config(&setup.secret, &token, config)
                .unwrap_or_else(|e| {
                    panic!("verify failed for {:?} window={}: {}", algorithm, window, e)
                });
            assert!(
                verified,
                "token generated at current time must verify for {:?} window={}",
                algorithm, window
            );
        }
    }
}

#[test]
fn algorithm_db_round_trip_preserves_identity() {
    use crate::db::PostgresTwoFactorStore;
    for &alg in &ALGORITHMS {
        let db_val = PostgresTwoFactorStore::algorithm_to_db_pub(alg);
        let round_tripped = PostgresTwoFactorStore::algorithm_from_db_pub(Some(&db_val));
        assert_eq!(
            alg, round_tripped,
            "algorithm round-trip failed for {:?} (db value: {})",
            alg, db_val
        );
    }
}

#[test]
fn eight_digit_tokens_verify_for_all_algorithms() {
    for &algorithm in &ALGORITHMS {
        let config = TotpConfig::new(algorithm, 8, 30, 1).unwrap();
        let setup =
            TwoFactorAuth::setup_with_config("8dig@petchain.com", "PetChain", config.clone())
                .unwrap();
        let totp = TOTP::new(
            algorithm,
            8,
            1,
            30,
            Secret::Encoded(setup.secret.clone()).to_bytes().unwrap(),
            None,
            String::new(),
        )
        .unwrap();
        let token = totp.generate_current().unwrap();
        assert_eq!(token.len(), 8);
        let ok = TwoFactorAuth::verify_token_with_config(&setup.secret, &token, config).unwrap();
        assert!(ok, "8-digit token must verify for {:?}", algorithm);
    }
}

#[test]
fn cross_algorithm_token_never_verifies() {
    for &gen_alg in &ALGORITHMS {
        for &ver_alg in &ALGORITHMS {
            if gen_alg == ver_alg {
                continue;
            }
            let secret = TwoFactorAuth::generate_secret();
            let gen_cfg = TotpConfig::new(gen_alg, 6, 30, 1).unwrap();
            let ver_cfg = TotpConfig::new(ver_alg, 6, 30, 1).unwrap();

            let totp = TOTP::new(
                gen_alg,
                6,
                1,
                30,
                Secret::Encoded(secret.clone()).to_bytes().unwrap(),
                None,
                String::new(),
            )
            .unwrap();
            let token = totp.generate_current().unwrap();

            let result = TwoFactorAuth::verify_token_with_config(&secret, &token, ver_cfg).unwrap();
            assert!(
                !result,
                "token from {:?} must NOT verify under {:?}",
                gen_alg, ver_alg
            );
        }
    }
}

#[test]
fn same_user_id_different_tenants_have_independent_secrets() {
    let store = Arc::new(InMemoryStore::default());
    let tenant_a = TenantScopedStore::new(store.clone(), TenantConfig::new("tenant-a"));
    let tenant_b = TenantScopedStore::new(store.clone(), TenantConfig::new("tenant-b"));

    let user_id = "shared-uid";
    tenant_a.save(user_id, make_data("SECRET_A")).unwrap();
    tenant_b.save(user_id, make_data("SECRET_B")).unwrap();

    assert_eq!(tenant_a.get(user_id).unwrap().secret, "SECRET_A");
    assert_eq!(tenant_b.get(user_id).unwrap().secret, "SECRET_B");
}

#[test]
fn deleting_in_one_tenant_does_not_affect_other() {
    let store = Arc::new(InMemoryStore::default());
    let tenant_a = TenantScopedStore::new(store.clone(), TenantConfig::new("t1"));
    let tenant_b = TenantScopedStore::new(store.clone(), TenantConfig::new("t2"));

    let user_id = "uid";
    tenant_a.save(user_id, make_data("A")).unwrap();
    tenant_b.save(user_id, make_data("B")).unwrap();

    tenant_a.delete(user_id).unwrap();
    assert!(tenant_a.get(user_id).is_err());
    assert_eq!(tenant_b.get(user_id).unwrap().secret, "B");
}

#[test]
fn test_upgrade_algorithm_unauthorized_caller() {
    clear_two_factor_store_for_tests();

    let user_id = "user-upgrade-unauthorized";

    // Enroll and activate
    let resp = TwoFactorHandlers::enable_two_factor(
        &caller(user_id),
        EnableTwoFactorRequest {
            idempotency_key: None,
            user_id: user_id.to_string(),
            email: "unauth@petchain.com".to_string(),
        },
    )
    .unwrap();

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

    // Try to upgrade as a different user
    let upgrade_result = handlers.upgrade_algorithm(
        &caller("attacker"),
        UpgradeAlgorithmRequest {
            user_id: user_id.to_string(),
            token,
        },
    );

    assert!(upgrade_result.is_err());
    let err = upgrade_result.unwrap_err();
    assert_eq!(err.code, "FORBIDDEN");
    assert!(err.message.contains("your own 2FA"));
}

#[test]
fn test_verify_and_activate_empty_user_id_returns_bad_request() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    let result = handlers.verify_and_activate(
        &caller(""),
        VerifyTwoFactorRequest {
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
fn test_verify_and_activate_token_too_short_returns_bad_request() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    let result = handlers.verify_and_activate(
        &caller("test-user"),
        VerifyTwoFactorRequest {
            user_id: "test-user".to_string(),
            token: "12345".to_string(), // 5 digits
        },
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, "BAD_REQUEST");
    assert!(err
        .message
        .contains("token must be exactly 6-8 decimal digits"));
}

#[test]
fn test_verify_and_activate_token_too_long_returns_bad_request() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    let result = handlers.verify_and_activate(
        &caller("test-user"),
        VerifyTwoFactorRequest {
            user_id: "test-user".to_string(),
            token: "123456789".to_string(), // 9 digits
        },
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, "BAD_REQUEST");
    assert!(err
        .message
        .contains("token must be exactly 6-8 decimal digits"));
}

#[test]
fn test_verify_and_activate_token_non_digits_returns_bad_request() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    let result = handlers.verify_and_activate(
        &caller("test-user"),
        VerifyTwoFactorRequest {
            user_id: "test-user".to_string(),
            token: "abcdef".to_string(),
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
fn test_verify_login_token_empty_user_id_returns_bad_request() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    let result = handlers.verify_login_token(
        &caller(""),
        LoginWithTwoFactorRequest {
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
fn test_verify_login_token_invalid_token_returns_bad_request() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    let result = handlers.verify_login_token(
        &caller("test-user"),
        LoginWithTwoFactorRequest {
            user_id: "test-user".to_string(),
            token: "abc123".to_string(),
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
fn test_disable_two_factor_empty_user_id_returns_bad_request() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    let result = handlers.disable_two_factor(
        &caller(""),
        DisableTwoFactorRequest {
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
fn test_valid_6_digit_token_passes_validation() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    // This should pass validation (though it may fail for other reasons)
    let result = handlers.verify_and_activate(
        &caller("test-user"),
        VerifyTwoFactorRequest {
            user_id: "test-user".to_string(),
            token: "123456".to_string(),
        },
    );

    // Should not be a validation error
    if let Err(err) = result {
        assert_ne!(err.code, "BAD_REQUEST");
    }
}

#[test]
fn test_valid_8_digit_token_passes_validation() {
    clear_two_factor_store_for_tests();
    let handlers = TwoFactorHandlers::new();

    // This should pass validation (though it may fail for other reasons)
    let result = handlers.verify_and_activate(
        &caller("test-user"),
        VerifyTwoFactorRequest {
            user_id: "test-user".to_string(),
            token: "12345678".to_string(),
        },
    );

    // Should not be a validation error
    if let Err(err) = result {
        assert_ne!(err.code, "BAD_REQUEST");
    }
}
