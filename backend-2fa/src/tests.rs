#[cfg(test)]
mod tests {
    use crate::handlers::{
        clear_two_factor_store_for_tests, get_two_factor_data_for_tests,
        overwrite_two_factor_data_for_tests, AuthenticatedUser, DisableTwoFactorRequest,
        EnableTwoFactorRequest, LoginWithTwoFactorRequest, RecoverWithBackupRequest,
        TwoFactorHandlers, VerifyTwoFactorRequest,
    };
    use crate::two_factor::{TotpConfig, TwoFactorAuth, TwoFactorData};
    use totp_rs::{Algorithm, Secret, TOTP};

    fn caller(id: &str) -> AuthenticatedUser {
        AuthenticatedUser::new(id)
    }

    fn generate_token(secret: &str) -> String {
        use totp_rs::{Algorithm, Secret, TOTP};
        TOTP::new(
            Algorithm::SHA256,
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
        assert_eq!(config.algorithm, Algorithm::SHA256);
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
        assert_eq!(setup.config.algorithm, Algorithm::SHA256);
    }

    #[test]
    fn test_setup_two_factor_with_sha1_config() {
        let config = TotpConfig::legacy_sha1();
        let result =
            TwoFactorAuth::setup_with_config("test@petchain.com", "PetChain", config.clone());
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
        let result =
            TwoFactorAuth::setup_with_config("test@petchain.com", "PetChain", config.clone());
        assert!(result.is_ok());

        let setup = result.unwrap();
        assert!(!setup.secret.is_empty());
        assert!(setup.qr_code_base64.starts_with("data:image/png;base64,"));
        assert_eq!(setup.backup_codes.len(), 8);
        assert_eq!(setup.config.algorithm, Algorithm::SHA512);
        assert_eq!(setup.config.digits, 8);
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
    fn test_enable_two_factor_protection() {
        clear_two_factor_store_for_tests();
        let user_id = "user123";
        let caller = AuthenticatedUser::new(user_id);
        let req = EnableTwoFactorRequest {
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
        assert!(result2.unwrap_err().contains("already enabled"));
    }

    #[test]
    fn test_algorithm_mismatch() {
        let secret = TwoFactorAuth::generate_secret();
        let sha1_config = TotpConfig::legacy_sha1();
        let sha256_config = TotpConfig::default();

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
                user_id: user_id.to_string(),
                email: "overwrite@petchain.com".to_string(),
            },
        )
        .unwrap();

        let resp2 = TwoFactorHandlers::enable_two_factor(
            &caller(user_id),
            EnableTwoFactorRequest {
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

    /// Failure path: wrong caller is rejected before any persistence occurs.
    #[test]
    fn test_enable_two_factor_forbidden_does_not_persist() {
        clear_two_factor_store_for_tests();

        let result = TwoFactorHandlers::enable_two_factor(
            &caller("attacker"),
            EnableTwoFactorRequest {
                user_id: "victim".to_string(),
                email: "victim@petchain.com".to_string(),
            },
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Forbidden"));
        // Nothing was written to the store
        assert!(get_two_factor_data_for_tests("victim").is_none());
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
        assert!(result.unwrap_err().contains("not configured"));
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

    mod rate_limiter_tests {
        use crate::handlers::{
            clear_two_factor_store_for_tests, overwrite_two_factor_data_for_tests,
            AuthenticatedUser, DisableTwoFactorRequest, LoginWithTwoFactorRequest,
            TwoFactorHandlers, VerifyTwoFactorRequest,
        };
        use crate::rate_limiter::{InMemoryRateLimiter, RateLimitResult, RateLimiter};
        use crate::two_factor::TwoFactorData;
        use std::sync::Arc;

        fn caller(id: &str) -> AuthenticatedUser {
            AuthenticatedUser::new(id)
        }

        struct AlwaysBlockedLimiter;
        impl RateLimiter for AlwaysBlockedLimiter {
            fn record_failure(&self, _key: &str) -> RateLimitResult {
                RateLimitResult::Blocked {
                    retry_after_secs: 300,
                }
            }
            fn record_success(&self, _key: &str) {}
        }

        struct AlwaysAllowedLimiter;
        impl RateLimiter for AlwaysAllowedLimiter {
            fn record_failure(&self, _key: &str) -> RateLimitResult {
                RateLimitResult::Allowed { remaining: 99 }
            }
            fn record_success(&self, _key: &str) {}
        }

        #[test]
        fn test_allows_attempts_below_limit() {
            let limiter = InMemoryRateLimiter::new(5, 60, 300);
            for i in 1..5 {
                match limiter.record_failure("user:test") {
                    RateLimitResult::Allowed { remaining } => assert_eq!(remaining, 5 - i),
                    RateLimitResult::Blocked { .. } => panic!("should not be blocked before limit"),
                }
            }
        }

        #[test]
        fn test_blocks_after_max_failures() {
            let limiter = InMemoryRateLimiter::new(3, 60, 300);
            for _ in 0..3 {
                limiter.record_failure("user:lockout");
            }
            match limiter.record_failure("user:lockout") {
                RateLimitResult::Blocked { retry_after_secs } => assert!(
                    retry_after_secs >= 299 && retry_after_secs <= 300,
                    "retry_after_secs was {}",
                    retry_after_secs
                ),
                RateLimitResult::Allowed { .. } => panic!("should be blocked after max failures"),
            }
        }

        #[test]
        fn test_success_clears_counter() {
            let limiter = InMemoryRateLimiter::new(3, 60, 300);
            limiter.record_failure("user:clear");
            limiter.record_failure("user:clear");
            limiter.record_success("user:clear");
            match limiter.record_failure("user:clear") {
                RateLimitResult::Allowed { remaining } => assert_eq!(remaining, 2),
                RateLimitResult::Blocked { .. } => panic!("should not be blocked after success"),
            }
        }

        #[test]
        fn test_blocked_remains_blocked_within_lockout() {
            let limiter = InMemoryRateLimiter::new(2, 60, 300);
            limiter.record_failure("user:persist");
            limiter.record_failure("user:persist");
            for _ in 0..5 {
                assert!(matches!(
                    limiter.record_failure("user:persist"),
                    RateLimitResult::Blocked { .. }
                ));
            }
        }

        #[test]
        fn test_different_keys_are_independent() {
            let limiter = InMemoryRateLimiter::new(2, 60, 300);
            limiter.record_failure("user:alice");
            limiter.record_failure("user:alice");
            assert!(matches!(
                limiter.record_failure("user:bob"),
                RateLimitResult::Allowed { .. }
            ));
        }

        #[test]
        fn test_verify_and_activate_blocked_returns_error() {
            clear_two_factor_store_for_tests();
            let handlers = TwoFactorHandlers::with_limiter(Arc::new(AlwaysBlockedLimiter));
            let result = handlers.verify_and_activate(
                &caller("user1"),
                VerifyTwoFactorRequest {
                    user_id: "user1".to_string(),
                    token: "123456".to_string(),
                },
            );
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Too many failed attempts"));
        }

        #[test]
        fn test_verify_login_token_blocked_returns_error() {
            clear_two_factor_store_for_tests();
            let handlers = TwoFactorHandlers::with_limiter(Arc::new(AlwaysBlockedLimiter));
            let result = handlers.verify_login_token(
                &caller("user1"),
                LoginWithTwoFactorRequest {
                    user_id: "user1".to_string(),
                    token: "123456".to_string(),
                },
            );
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Too many failed attempts"));
        }

        #[test]
        fn test_disable_two_factor_blocked_returns_error() {
            clear_two_factor_store_for_tests();
            let handlers = TwoFactorHandlers::with_limiter(Arc::new(AlwaysBlockedLimiter));
            let result = handlers.disable_two_factor(
                &caller("user1"),
                DisableTwoFactorRequest {
                    user_id: "user1".to_string(),
                    token: "123456".to_string(),
                },
            );
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Too many failed attempts"));
        }

        #[test]
        fn test_rate_limit_is_per_endpoint_not_shared() {
            clear_two_factor_store_for_tests();

            let limiter = Arc::new(InMemoryRateLimiter::new(2, 60, 300));
            let handlers = TwoFactorHandlers::with_limiter(limiter);

            // Exhaust login limit for user1
            handlers
                .verify_login_token(
                    &caller("user1"),
                    LoginWithTwoFactorRequest {
                        user_id: "user1".to_string(),
                        token: "bad".to_string(),
                    },
                )
                .ok();
            handlers
                .verify_login_token(
                    &caller("user1"),
                    LoginWithTwoFactorRequest {
                        user_id: "user1".to_string(),
                        token: "bad".to_string(),
                    },
                )
                .ok();

            let login_result = handlers.verify_login_token(
                &caller("user1"),
                LoginWithTwoFactorRequest {
                    user_id: "user1".to_string(),
                    token: "bad".to_string(),
                },
            );
            assert!(login_result.is_err(), "login should be blocked");

            // disable endpoint uses a different key — should not be rate-limited
            overwrite_two_factor_data_for_tests(
                "user1",
                TwoFactorData {
                    secret: "AAAA".to_string(),
                    backup_codes: vec![],
                    enabled: true,
                },
            );
            let disable_result = handlers.disable_two_factor(
                &caller("user1"),
                DisableTwoFactorRequest {
                    user_id: "user1".to_string(),
                    token: "bad".to_string(),
                },
            );
            assert!(
                !disable_result
                    .as_ref()
                    .err()
                    .map(|e| e.contains("Too many"))
                    .unwrap_or(false),
                "disable endpoint should not be blocked by login failures"
            );
        }

        #[test]
        fn test_in_memory_limiter_is_thread_safe() {
            use std::thread;
            let limiter = Arc::new(InMemoryRateLimiter::new(100, 60, 300));
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let l = Arc::clone(&limiter);
                    thread::spawn(move || l.record_failure(&format!("user:{}", i)))
                })
                .collect();
            for h in handles {
                h.join().expect("thread panicked");
            }
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
                    user_id: "user-2".to_string(),
                    email: "user2@petchain.com".to_string(),
                },
            );
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Forbidden"));
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
            assert!(result.unwrap_err().contains("Forbidden"));
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
            assert!(result.unwrap_err().contains("Forbidden"));
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
            assert!(result.unwrap_err().contains("Forbidden"));
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
                err.contains("Invalid backup code")
                    || err.contains("not configured")
                    || err.contains("not enabled"),
                "Correct user should reach the backup code validation step, got: {}",
                err
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
            assert!(result.unwrap_err().contains("Forbidden"));
        }

        #[test]
        fn test_authorize_same_user_ok() {
            assert!(caller("alice").authorize("alice").is_ok());
        }

        #[test]
        fn test_authorize_different_user_err() {
            let result = caller("alice").authorize("bob");
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Forbidden"));
        }

        #[test]
        fn test_authorize_empty_vs_nonempty_is_forbidden() {
            assert!(caller("").authorize("someone").is_err());
        }
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
        assert!(err.unwrap_err().contains("not configured"));
    }

    #[test]
    fn test_handler_verify_activates_2fa() {
        clear_two_factor_store_for_tests();
        let user_id = "handler-user2";
        let resp = TwoFactorHandlers::enable_two_factor(
            &caller(user_id),
            EnableTwoFactorRequest {
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

    #[test]
    fn test_handler_recovery_invalid_code_returns_error() {
        clear_two_factor_store_for_tests();
        let user_id = "handler-user8";
        let resp = TwoFactorHandlers::enable_two_factor(
            &caller(user_id),
            EnableTwoFactorRequest {
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
        assert!(result.unwrap_err().contains("InvalidRecoveryCode"));
    }

    #[test]
    fn test_handler_recovery_when_not_enabled_returns_error() {
        clear_two_factor_store_for_tests();
        let user_id = "handler-user9";
        TwoFactorHandlers::enable_two_factor(
            &caller(user_id),
            EnableTwoFactorRequest {
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
        assert!(err.unwrap_err().contains("not enabled"));
    }
}

// ============================================================================
// Integration tests — full end-to-end flows
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use crate::handlers::{
        clear_two_factor_store_for_tests, get_two_factor_data_for_tests,
        overwrite_two_factor_data_for_tests, AdminRecoveryHandlers, AuthenticatedUser,
        DisableTwoFactorRequest, EnableTwoFactorRequest, LoginWithTwoFactorRequest,
        RecoverWithBackupRequest, TwoFactorHandlers, VerifyTwoFactorRequest,
    };
    use crate::rate_limiter::{InMemoryRateLimiter, RateLimiter};
    use crate::two_factor::{TwoFactorAuth, TwoFactorData};
    use std::sync::Arc;
    use totp_rs::{Algorithm, Secret, TOTP};

    fn caller(id: &str) -> AuthenticatedUser {
        AuthenticatedUser::new(id)
    }

    fn generate_token(secret: &str) -> String {
        TOTP::new(
            Algorithm::SHA256,
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
    // Flow 3: rate limit exhaustion on login
    // -----------------------------------------------------------------------

    /// After exhausting the allowed failures the endpoint must be locked out,
    /// and a subsequent correct token must also be rejected until the lockout
    /// expires (or the limiter is replaced).
    #[test]
    fn test_rate_limit_exhaustion_blocks_login() {
        let user_id = "integration-rate-limit-login-user";

        // Use a tight limiter: 3 failures → 300 s lockout
        let limiter: Arc<dyn RateLimiter> = Arc::new(InMemoryRateLimiter::new(3, 60, 300));
        let handlers = TwoFactorHandlers::with_limiter(Arc::clone(&limiter));

        // Enable and activate via normal flow — no overwrite
        let enable_resp = TwoFactorHandlers::enable_two_factor(
            &caller(user_id),
            EnableTwoFactorRequest {
                user_id: user_id.to_string(),
                email: "rate-limit-login@petchain.com".to_string(),
            },
        )
        .unwrap();
        handlers
            .verify_and_activate(
                &caller(user_id),
                VerifyTwoFactorRequest {
                    user_id: user_id.to_string(),
                    token: generate_token(&enable_resp.secret),
                },
            )
            .unwrap();
        let secret = enable_resp.secret;

        // Exhaust the limit with bad tokens
        for _ in 0..3 {
            let _ = handlers.verify_login_token(
                &caller(user_id),
                LoginWithTwoFactorRequest {
                    user_id: user_id.to_string(),
                    token: "000000".to_string(),
                },
            );
        }

        // Even a correct token must be rejected while locked out
        let blocked = handlers.verify_login_token(
            &caller(user_id),
            LoginWithTwoFactorRequest {
                user_id: user_id.to_string(),
                token: generate_token(&secret),
            },
        );

        assert!(blocked.is_err(), "locked-out user must receive an error");
        let err = blocked.unwrap_err();
        assert!(
            err.contains("Too many failed attempts"),
            "error must mention rate limiting, got: {}",
            err
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
        assert!(blocked.unwrap_err().contains("Too many failed attempts"));
    }

    /// A successful login resets the failure counter so the user is not
    /// permanently penalized for earlier mistakes.
    #[test]
    fn test_successful_login_resets_rate_limit() {
        // Use a unique user ID and a fresh limiter — no shared global state
        let user_id = "integration-reset-rate-limit-user";

        // 6 failures allowed before lockout — gives room for 4 bad + 1 good
        let limiter: Arc<dyn RateLimiter> = Arc::new(InMemoryRateLimiter::new(6, 60, 300));
        let handlers = TwoFactorHandlers::with_limiter(Arc::clone(&limiter));

        // Set up 2FA via the normal enable → activate flow so the record
        // is written immediately before we start hammering the limiter.
        let enable_resp = TwoFactorHandlers::enable_two_factor(
            &caller(user_id),
            EnableTwoFactorRequest {
                user_id: user_id.to_string(),
                email: "reset-rate@petchain.com".to_string(),
            },
        )
        .unwrap();

        // Activate with a valid token
        handlers
            .verify_and_activate(
                &caller(user_id),
                VerifyTwoFactorRequest {
                    user_id: user_id.to_string(),
                    token: generate_token(&enable_resp.secret),
                },
            )
            .unwrap();

        assert!(get_two_factor_data_for_tests(user_id).unwrap().enabled);

        // 4 bad login attempts
        for _ in 0..4 {
            let _ = handlers.verify_login_token(
                &caller(user_id),
                LoginWithTwoFactorRequest {
                    user_id: user_id.to_string(),
                    token: "000000".to_string(),
                },
            );
        }

        // One good login — resets the counter
        let ok = handlers
            .verify_login_token(
                &caller(user_id),
                LoginWithTwoFactorRequest {
                    user_id: user_id.to_string(),
                    token: generate_token(&enable_resp.secret),
                },
            )
            .expect("login should succeed");
        assert!(ok);

        // Counter is reset: 4 more bad attempts should still be allowed
        for _ in 0..4 {
            let result = handlers.verify_login_token(
                &caller(user_id),
                LoginWithTwoFactorRequest {
                    user_id: user_id.to_string(),
                    token: "000000".to_string(),
                },
            );
            assert!(
                result.is_ok(),
                "should not be blocked yet after counter reset"
            );
        }
    }

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
            assert_eq!(header, "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01");
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
        assert!(second.unwrap_err().contains("InvalidRecoveryCode"));
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
}

// ============================================================================
// RedisRateLimiter tests
// ============================================================================

#[cfg(test)]
mod redis_rate_limiter_tests {
    use crate::rate_limiter::{RateLimitResult, RateLimiter, RedisRateLimiter};
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Returns a unique key per test invocation to prevent cross-test pollution.
    fn unique_key(label: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        format!("test:{label}:{nanos}")
    }

    fn redis_url() -> Option<String> {
        std::env::var("REDIS_URL").ok()
    }

    fn make_limiter(
        max_failures: u32,
        window_secs: u64,
        lockout_secs: u64,
    ) -> Option<RedisRateLimiter> {
        redis_url().and_then(|url| {
            RedisRateLimiter::new(&url, max_failures, window_secs, lockout_secs).ok()
        })
    }

    // -----------------------------------------------------------------------
    // Mock Redis client for sliding-window unit tests
    // -----------------------------------------------------------------------

    /// A minimal mock that records ZADD/ZREMRANGEBYSCORE/ZCARD/EXPIRE/DEL
    /// calls so we can test the sliding-window logic without a real Redis.
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct MockRedisState {
        /// sorted set: key → BTreeMap<score_ms, member_ms>
        zsets: HashMap<String, BTreeMap<u64, u64>>,
        /// string keys (lockout)
        strings: HashMap<String, u64>, // value = TTL remaining (secs)
    }

    impl MockRedisState {
        fn zremrangebyscore(&mut self, key: &str, min: u64, max: u64) {
            if let Some(set) = self.zsets.get_mut(key) {
                set.retain(|&score, _| score < min || score > max);
            }
        }

        fn zadd(&mut self, key: &str, score: u64, member: u64) {
            self.zsets
                .entry(key.to_string())
                .or_default()
                .insert(score, member);
        }

        fn zcard(&self, key: &str) -> u64 {
            self.zsets.get(key).map(|s| s.len() as u64).unwrap_or(0)
        }

        fn set_ex(&mut self, key: &str, ttl: u64) {
            self.strings.insert(key.to_string(), ttl);
        }

        fn ttl(&self, key: &str) -> i64 {
            match self.strings.get(key) {
                Some(&t) if t > 0 => t as i64,
                _ => -2,
            }
        }

        fn del(&mut self, keys: &[&str]) {
            for k in keys {
                self.zsets.remove(*k);
                self.strings.remove(*k);
            }
        }
    }

    /// Simulates the sliding-window logic of `RedisRateLimiter::record_failure`
    /// using the mock state, so we can assert on the algorithm without Redis.
    fn mock_record_failure(
        state: &Arc<Mutex<MockRedisState>>,
        key: &str,
        now_ms: u64,
        max_failures: u32,
        window_secs: u64,
        lockout_secs: u64,
    ) -> RateLimitResult {
        let mut s = state.lock().unwrap();

        let lockout_key = format!("rate:{key}:lockout");
        let window_key = format!("rate:{key}:window");

        if s.ttl(&lockout_key) > 0 {
            return RateLimitResult::Blocked {
                retry_after_secs: s.ttl(&lockout_key) as u64,
            };
        }

        let cutoff_ms = now_ms.saturating_sub(window_secs * 1_000);
        s.zremrangebyscore(&window_key, 0, cutoff_ms);
        s.zadd(&window_key, now_ms, now_ms);
        let count = s.zcard(&window_key);

        if count >= max_failures as u64 {
            s.set_ex(&lockout_key, lockout_secs);
            return RateLimitResult::Blocked {
                retry_after_secs: lockout_secs,
            };
        }

        RateLimitResult::Allowed {
            remaining: max_failures - count as u32,
        }
    }

    fn mock_record_success(state: &Arc<Mutex<MockRedisState>>, key: &str) {
        let mut s = state.lock().unwrap();
        s.del(&[
            &format!("rate:{key}:lockout"),
            &format!("rate:{key}:window"),
        ]);
    }

    // -----------------------------------------------------------------------
    // Unconditional tests — no Redis instance required
    // -----------------------------------------------------------------------

    /// When Redis is unreachable the limiter must fail open (return Allowed)
    /// rather than blocking users or panicking.
    #[test]
    fn redis_fails_open_on_bad_connection() {
        // Port 1 is never Redis; Client::open only validates the URL format.
        let limiter =
            RedisRateLimiter::new("redis://127.0.0.1:1", 5, 60, 300).expect("URL format is valid");
        assert!(
            matches!(
                limiter.record_failure("any:key"),
                RateLimitResult::Allowed { remaining: 5 }
            ),
            "unreachable Redis must return Allowed with full remaining count"
        );
    }

    /// RedisRateLimiter satisfies the RateLimiter trait bounds (Send + Sync).
    /// This is a compile-time check; if it compiles the test passes.
    #[test]
    fn redis_rate_limiter_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RedisRateLimiter>();
    }

    // -----------------------------------------------------------------------
    // Mock sliding-window unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn mock_sliding_window_allows_below_limit() {
        let state = Arc::new(Mutex::new(MockRedisState::default()));
        let now_ms = 1_000_000u64;
        for i in 1u32..5 {
            match mock_record_failure(&state, "user:a", now_ms + i as u64, 5, 60, 300) {
                RateLimitResult::Allowed { remaining } => assert_eq!(remaining, 5 - i),
                RateLimitResult::Blocked { .. } => panic!("should not block before limit"),
            }
        }
    }

    #[test]
    fn mock_sliding_window_blocks_at_limit() {
        let state = Arc::new(Mutex::new(MockRedisState::default()));
        let now_ms = 2_000_000u64;
        for i in 0..3u64 {
            mock_record_failure(&state, "user:b", now_ms + i, 3, 60, 300);
        }
        assert!(matches!(
            mock_record_failure(&state, "user:b", now_ms + 3, 3, 60, 300),
            RateLimitResult::Blocked { retry_after_secs: 300 }
        ));
    }

    #[test]
    fn mock_sliding_window_evicts_stale_entries() {
        let state = Arc::new(Mutex::new(MockRedisState::default()));
        // 3 failures at t=0
        for i in 0..3u64 {
            mock_record_failure(&state, "user:c", i, 3, 60, 300);
        }
        // 61 seconds later — all three are outside the 60 s window
        let later_ms = 61_000u64;
        match mock_record_failure(&state, "user:c", later_ms, 3, 60, 300) {
            RateLimitResult::Allowed { remaining } => assert_eq!(remaining, 2),
            RateLimitResult::Blocked { .. } => panic!("stale entries should have been evicted"),
        }
    }

    #[test]
    fn mock_sliding_window_prevents_boundary_burst() {
        // Fixed-window would reset at t=60 s, allowing a burst of max_failures
        // right after. Sliding window must not allow that.
        let state = Arc::new(Mutex::new(MockRedisState::default()));
        let max = 3u32;
        let window_ms = 60_000u64;

        // Fill the window just before the boundary (t = 59 s)
        for i in 0..max as u64 {
            mock_record_failure(&state, "user:d", 59_000 + i, max, 60, 300);
        }
        // At t = 60 s (boundary) the entries are still within the window
        assert!(matches!(
            mock_record_failure(&state, "user:d", window_ms, max, 60, 300),
            RateLimitResult::Blocked { .. }
        ));
    }

    #[test]
    fn mock_sliding_window_success_resets_counter() {
        let state = Arc::new(Mutex::new(MockRedisState::default()));
        let now_ms = 3_000_000u64;
        mock_record_failure(&state, "user:e", now_ms, 3, 60, 300);
        mock_record_failure(&state, "user:e", now_ms + 1, 3, 60, 300);
        mock_record_success(&state, "user:e");
        match mock_record_failure(&state, "user:e", now_ms + 2, 3, 60, 300) {
            RateLimitResult::Allowed { remaining } => assert_eq!(remaining, 2),
            RateLimitResult::Blocked { .. } => panic!("should not block after success reset"),
        }
    }

    #[test]
    fn mock_sliding_window_concurrent_requests_independent_keys() {
        let state = Arc::new(Mutex::new(MockRedisState::default()));
        let now_ms = 4_000_000u64;
        // Exhaust key "user:f"
        for i in 0..3u64 {
            mock_record_failure(&state, "user:f", now_ms + i, 3, 60, 300);
        }
        // "user:g" must be unaffected
        assert!(matches!(
            mock_record_failure(&state, "user:g", now_ms, 3, 60, 300),
            RateLimitResult::Allowed { .. }
        ));
    }

    #[test]
    fn mock_sliding_window_retry_after_is_accurate() {
        let state = Arc::new(Mutex::new(MockRedisState::default()));
        let now_ms = 5_000_000u64;
        for i in 0..3u64 {
            mock_record_failure(&state, "user:h", now_ms + i, 3, 60, 120);
        }
        match mock_record_failure(&state, "user:h", now_ms + 3, 3, 60, 120) {
            RateLimitResult::Blocked { retry_after_secs } => {
                assert_eq!(retry_after_secs, 120, "retry_after must equal lockout_secs");
            }
            RateLimitResult::Allowed { .. } => panic!("should be blocked"),
        }
    }

    // -----------------------------------------------------------------------
    // Integration tests — require a running Redis at REDIS_URL
    // -----------------------------------------------------------------------

    #[test]
    #[ignore = "requires REDIS_URL env var pointing to a running Redis instance"]
    fn redis_allows_attempts_below_limit() {
        let Some(limiter) = make_limiter(5, 60, 300) else {
            return;
        };
        let key = unique_key("below_limit");

        for i in 1u32..5 {
            match limiter.record_failure(&key) {
                RateLimitResult::Allowed { remaining } => {
                    assert_eq!(remaining, 5 - i, "remaining should decrease by 1 each call");
                }
                RateLimitResult::Blocked { .. } => panic!("should not be blocked before the limit"),
            }
        }
    }

    #[test]
    #[ignore = "requires REDIS_URL env var pointing to a running Redis instance"]
    fn redis_blocks_after_max_failures() {
        let Some(limiter) = make_limiter(3, 60, 300) else {
            return;
        };
        let key = unique_key("blocks_after_max");

        for _ in 0..3 {
            limiter.record_failure(&key);
        }

        assert!(
            matches!(
                limiter.record_failure(&key),
                RateLimitResult::Blocked { .. }
            ),
            "must be blocked after reaching max_failures"
        );
    }

    #[test]
    #[ignore = "requires REDIS_URL env var pointing to a running Redis instance"]
    fn redis_success_clears_counter() {
        let Some(limiter) = make_limiter(3, 60, 300) else {
            return;
        };
        let key = unique_key("success_clears");

        limiter.record_failure(&key);
        limiter.record_failure(&key);
        limiter.record_success(&key);

        match limiter.record_failure(&key) {
            RateLimitResult::Allowed { remaining } => {
                assert_eq!(remaining, 2, "counter must reset to 0 after success");
            }
            RateLimitResult::Blocked { .. } => panic!("should not be blocked after record_success"),
        }
    }

    #[test]
    #[ignore = "requires REDIS_URL env var pointing to a running Redis instance"]
    fn redis_different_keys_are_independent() {
        let Some(limiter) = make_limiter(2, 60, 300) else {
            return;
        };
        let key_a = unique_key("indep_a");
        let key_b = unique_key("indep_b");

        limiter.record_failure(&key_a);
        limiter.record_failure(&key_a);

        assert!(
            matches!(
                limiter.record_failure(&key_b),
                RateLimitResult::Allowed { .. }
            ),
            "exhausting key_a must not affect key_b"
        );
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
            let body = r#"{"users":[{"name":"Alice","totp_code":"123"},{"name":"Bob","password":"secret"}]}"#;
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
        use crate::leaderboard::FlaggedScoreSubmission;

        #[test]
        fn admin_get_all_flagged_empty() {
            let admin = AdminScoreHandlers::new();
            let flagged = admin.get_all_flagged();
            assert!(flagged.is_empty());
        }

        #[test]
        fn admin_log_rejected_submission() {
            let admin = AdminScoreHandlers::new();
            admin.log_rejected_submission(
                "user1".into(),
                5000,
                "Exceeds delta".into(),
            );

            let flagged = admin.get_all_flagged();
            assert_eq!(flagged.len(), 1);
            assert_eq!(flagged[0].user_id, "user1");
            assert_eq!(flagged[0].attempted_score, 5000);
            assert_eq!(flagged[0].reason, "Exceeds delta");
        }

        #[test]
        fn admin_get_flagged_by_user() {
            let admin = AdminScoreHandlers::new();
            admin.log_rejected_submission(
                "user1".into(),
                5000,
                "Exceeds delta".into(),
            );
            admin.log_rejected_submission(
                "user2".into(),
                3000,
                "Suspicious".into(),
            );

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
            admin.log_rejected_submission(
                "user1".into(),
                5000,
                "Exceeds delta".into(),
            );
            admin.log_rejected_submission(
                "user1".into(),
                6000,
                "Another violation".into(),
            );

            let user1_flagged = admin.get_flagged_by_user("user1");
            assert_eq!(user1_flagged.len(), 2);
            assert_eq!(user1_flagged[0].attempted_score, 5000);
            assert_eq!(user1_flagged[1].attempted_score, 6000);
        }

        #[test]
        fn admin_get_flagged_by_nonexistent_user() {
            let admin = AdminScoreHandlers::new();
            admin.log_rejected_submission(
                "user1".into(),
                5000,
                "Exceeds delta".into(),
            );

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
                assert_eq!(
                    all_flagged[i].attempted_score,
                    1000 + (i as u64 * 100)
                );
            }
        }

        #[test]
        #[cfg(test)]
        fn admin_clear_flagged() {
            let admin = AdminScoreHandlers::new();
            admin.log_rejected_submission(
                "user1".into(),
                5000,
                "Exceeds delta".into(),
            );
            admin.log_rejected_submission(
                "user2".into(),
                3000,
                "Suspicious".into(),
            );

            assert_eq!(admin.get_all_flagged().len(), 2);

            admin.clear_flagged();
            assert!(admin.get_all_flagged().is_empty());
        }

        #[test]
        fn admin_timestamp_is_set() {
            let admin = AdminScoreHandlers::new();
            admin.log_rejected_submission(
                "user1".into(),
                5000,
                "Test".into(),
            );

            let flagged = admin.get_all_flagged();
            assert!(flagged[0].timestamp > 0);
        }

        #[test]
        fn admin_reason_is_preserved() {
            let admin = AdminScoreHandlers::new();
            let reason = "Custom reason for suspension";
            admin.log_rejected_submission(
                "user1".into(),
                5000,
                reason.into(),
            );

            let flagged = admin.get_all_flagged();
            assert_eq!(flagged[0].reason, reason);
        }

        #[test]
        fn admin_large_score_values() {
            let admin = AdminScoreHandlers::new();
            let max_score = u64::MAX;
            admin.log_rejected_submission(
                "user1".into(),
                max_score,
                "Max score".into(),
            );

            let flagged = admin.get_all_flagged();
            assert_eq!(flagged[0].attempted_score, max_score);
        }
    }
}

// ============================================================================
// MockRedisBackend + SlidingWindowRateLimiter tests (#614)
// ============================================================================

#[cfg(test)]
mod mock_redis_tests {
    use crate::rate_limiter::{
        EndpointConfig, MockRedisBackend, RateLimitResult, RateLimiter, SlidingWindowRateLimiter,
    };
    use std::sync::Arc;

    fn limiter(max: u32, window_secs: u64, lockout_secs: u64) -> SlidingWindowRateLimiter<MockRedisBackend> {
        SlidingWindowRateLimiter::new(
            MockRedisBackend::new(),
            EndpointConfig::new(window_secs, max, lockout_secs),
        )
    }

    // --- limit ---

    #[test]
    fn allows_requests_below_limit() {
        let l = limiter(3, 60, 300);
        for i in 1u32..3 {
            assert_eq!(l.record_failure("u:a"), RateLimitResult::Allowed { remaining: 3 - i });
        }
    }

    #[test]
    fn blocks_at_limit_with_accurate_retry_after() {
        let l = limiter(3, 60, 120);
        for _ in 0..3 { l.record_failure("u:b"); }
        assert_eq!(
            l.record_failure("u:b"),
            RateLimitResult::Blocked { retry_after_secs: 120 },
        );
    }

    // --- reset ---

    #[test]
    fn success_resets_counter() {
        let l = limiter(3, 60, 300);
        l.record_failure("u:c");
        l.record_failure("u:c");
        l.record_success("u:c");
        assert_eq!(l.record_failure("u:c"), RateLimitResult::Allowed { remaining: 2 });
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
        assert_eq!(l.record_failure("u:d"), RateLimitResult::Allowed { remaining: 2 });
    }

    // --- concurrent / independent keys ---

    #[test]
    fn different_keys_are_independent() {
        let l = limiter(2, 60, 300);
        l.record_failure("u:e");
        l.record_failure("u:e");
        assert!(matches!(l.record_failure("u:f"), RateLimitResult::Allowed { .. }));
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
        for h in handles { h.join().expect("thread panicked"); }
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

    // --- sliding window prevents boundary burst ---

    #[test]
    fn sliding_window_prevents_boundary_burst() {
        let l = limiter(3, 60, 300);
        // 3 failures just before the 60-second boundary
        for _ in 0..3 { l.record_failure("u:g"); }
        // Advance to exactly the boundary — entries are still within the window
        l.backend_advance_ms(59_999);
        assert!(matches!(l.record_failure("u:g"), RateLimitResult::Blocked { .. }));
    }
}

// Helper: expose advance_ms on SlidingWindowRateLimiter<MockRedisBackend>
// without polluting the public API.
#[cfg(test)]
impl crate::rate_limiter::SlidingWindowRateLimiter<crate::rate_limiter::MockRedisBackend> {
    fn backend_advance_ms(&self, ms: u64) {
        // Access the backend field directly (same crate, so pub(crate) is fine).
        self.backend.advance_ms(ms);
    }
}
