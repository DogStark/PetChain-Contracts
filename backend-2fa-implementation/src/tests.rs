#[cfg(test)]
mod tests {
    use crate::handlers::{
        clear_two_factor_store_for_tests, get_two_factor_data_for_tests,
        overwrite_two_factor_data_for_tests, AuthenticatedUser, DisableTwoFactorRequest,
        EnableTwoFactorRequest, LoginWithTwoFactorRequest, RecoverWithBackupRequest,
        TwoFactorHandlers, VerifyTwoFactorRequest,
    };
    use crate::two_factor::{TwoFactorAuth, TwoFactorData};

    fn caller(id: &str) -> AuthenticatedUser {
        AuthenticatedUser::new(id)
    }

    fn generate_token(secret: &str) -> String {
        use totp_rs::{Algorithm, Secret, TOTP};
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
    fn test_setup_two_factor() {
        let result = TwoFactorAuth::setup("test@petchain.com", "PetChain");
        assert!(result.is_ok());
        let setup = result.unwrap();
        assert!(!setup.secret.is_empty());
        assert!(!setup.qr_code_base64.is_empty());
        assert_eq!(setup.backup_codes.len(), 8);
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
    fn test_verify_invalid_token_format() {
        let secret = TwoFactorAuth::generate_secret();
        // Too short / non-numeric → InvalidToken
        assert_eq!(
            TwoFactorAuth::verify_token(&secret, "abc"),
            Err(TwoFactorError::InvalidToken)
        );
    }

    proptest! {
        #[test]
        fn prop_test_verify_token_never_panics(s in "\\PC*") {
            let secret = "JBSWY3DPEHPK3PXP";
            let _ = TwoFactorAuth::verify_token(secret, &s);
        }

        #[test]
        fn prop_test_verify_backup_code_never_panics(s in "\\PC*") {
            let codes = vec!["1234-5678".to_string()];
            let _ = TwoFactorAuth::verify_backup_code(&codes, &s);
        }

        #[test]
        fn prop_test_validate_token_format(s in "\\PC*") {
            let res = TwoFactorAuth::validate_token_format(&s);
            if let Ok(valid) = res {
                assert_eq!(valid.len(), 6);
                assert!(valid.chars().all(|c| c.is_ascii_digit()));
            }
        }

        #[test]
        fn prop_test_validate_backup_code_format(s in "\\PC*") {
            let res = TwoFactorAuth::validate_backup_code_format(&s);
            if let Ok(valid) = res {
                assert_eq!(valid.len(), 9);
                assert!(valid.contains('-'));
            }
        }
    }

    #[test]
    fn test_validate_token_format() {
        // Valid
        assert!(TwoFactorAuth::validate_token_format("123456").is_ok());
        assert!(TwoFactorAuth::validate_token_format(" 123456 ").is_ok());

        // Invalid length
        assert!(TwoFactorAuth::validate_token_format("12345").is_err());
        assert!(TwoFactorAuth::validate_token_format("1234567").is_err());

        // Non-numeric
        assert!(TwoFactorAuth::validate_token_format("123a56").is_err());
    }

    #[test]
    fn test_validate_backup_code_format() {
        // Valid
        assert!(TwoFactorAuth::validate_backup_code_format("1234-5678").is_ok());
        assert!(TwoFactorAuth::validate_backup_code_format(" 1234-5678 ").is_ok());

        // Invalid
        assert!(TwoFactorAuth::validate_backup_code_format("12345678").is_err());
        assert!(TwoFactorAuth::validate_backup_code_format("1234-567a").is_err());
    }

    #[test]
    fn test_validate_token_format() {
        // Valid
        assert!(TwoFactorAuth::validate_token_format("123456").is_ok());
        assert!(TwoFactorAuth::validate_token_format(" 123456 ").is_ok());

        // Invalid length
        assert!(TwoFactorAuth::validate_token_format("12345").is_err());
        assert!(TwoFactorAuth::validate_token_format("1234567").is_err());

        // Non-numeric
        assert!(TwoFactorAuth::validate_token_format("123a56").is_err());
        assert!(TwoFactorAuth::validate_token_format("abcdef").is_err());
    }

    #[test]
    fn test_validate_backup_code_format() {
        // Valid
        assert!(TwoFactorAuth::validate_backup_code_format("1234-5678").is_ok());
        assert!(TwoFactorAuth::validate_backup_code_format(" 1234-5678 ").is_ok());

        // Invalid length/format
        assert!(TwoFactorAuth::validate_backup_code_format("12345678").is_err());
        assert!(TwoFactorAuth::validate_backup_code_format("123-45678").is_err());
        assert!(TwoFactorAuth::validate_backup_code_format("1234-567").is_err());

        // Non-numeric
        assert!(TwoFactorAuth::validate_backup_code_format("123a-5678").is_err());
        assert!(TwoFactorAuth::validate_backup_code_format("1234-567b").is_err());
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
        let codes = vec![
            "1234-5678".to_string(),
            "2345-6789".to_string(),
        ];
        assert_eq!(TwoFactorAuth::verify_backup_code(&codes, "2345-6789"), Some(1));
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
            TwoFactorData { secret, backup_codes: vec![], enabled: false },
        );

        let handlers = TwoFactorHandlers::new();
        let result = handlers
            .verify_login_token(
                &caller(user_id),
                LoginWithTwoFactorRequest { user_id: user_id.to_string(), token },
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

main
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

        assert!(!result, "placeholder token must not validate against the stored secret");
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
                RateLimitResult::Blocked { retry_after_secs: 300 }
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
                RateLimitResult::Blocked { retry_after_secs } => assert_eq!(retry_after_secs, 300),
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
                VerifyTwoFactorRequest { user_id: "user1".to_string(), token: "123456".to_string() },
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
                LoginWithTwoFactorRequest { user_id: "user1".to_string(), token: "123456".to_string() },
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
                DisableTwoFactorRequest { user_id: "user1".to_string(), token: "123456".to_string() },
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
                    LoginWithTwoFactorRequest { user_id: "user1".to_string(), token: "bad".to_string() },
                )
                .ok();
            handlers
                .verify_login_token(
                    &caller("user1"),
                    LoginWithTwoFactorRequest { user_id: "user1".to_string(), token: "bad".to_string() },
                )
                .ok();

            let login_result = handlers.verify_login_token(
                &caller("user1"),
                LoginWithTwoFactorRequest { user_id: "user1".to_string(), token: "bad".to_string() },
            );
            assert!(login_result.is_err(), "login should be blocked");

            // disable endpoint uses a different key — should not be rate-limited
            overwrite_two_factor_data_for_tests(
                "user1",
                TwoFactorData { secret: "AAAA".to_string(), backup_codes: vec![], enabled: true },
            );
            let disable_result = handlers.disable_two_factor(
                &caller("user1"),
                DisableTwoFactorRequest { user_id: "user1".to_string(), token: "bad".to_string() },
            );
            assert!(
                !disable_result.as_ref().err().map(|e| e.contains("Too many")).unwrap_or(false),
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
            assert!(result.is_ok(), "Owner should be able to enable their own 2FA");
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
                VerifyTwoFactorRequest { user_id: "user-99".to_string(), token: "123456".to_string() },
            );
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Forbidden"));
        }

        #[test]
        fn test_verify_login_token_wrong_user_is_forbidden() {
            let handlers = TwoFactorHandlers::new();
            let result = handlers.verify_login_token(
                &caller("user-1"),
                LoginWithTwoFactorRequest { user_id: "user-99".to_string(), token: "123456".to_string() },
            );
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Forbidden"));
        }

        #[test]
        fn test_disable_two_factor_wrong_user_is_forbidden() {
            let handlers = TwoFactorHandlers::new();
            let result = handlers.disable_two_factor(
                &caller("user-1"),
                DisableTwoFactorRequest { user_id: "user-99".to_string(), token: "123456".to_string() },
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
                err.contains("Invalid backup code") || err.contains("not configured"),
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
=======
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

        let result = TwoFactorAuth::verify_backup_code(&codes, "invalid");
        assert!(result.is_ok()); // Should return Ok(None) now because we handle format error internally
        assert_eq!(result.unwrap(), None);
    }

    use std::collections::HashMap;
    use crate::two_factor::{TwoFactorStorage, TwoFactorData};
    use crate::handlers::{TwoFactorHandlers, DisableTwoFactorRequest, EnableTwoFactorRequest, VerifyTwoFactorRequest};

    struct MockStorage {
        data: HashMap<String, TwoFactorData>,
    }

    impl TwoFactorStorage for MockStorage {
        fn get_two_factor_data(&self, user_id: &str) -> Result<Option<TwoFactorData>, String> {
            Ok(self.data.get(user_id).cloned())
        }
        fn save_two_factor_data(&mut self, user_id: &str, data: TwoFactorData) -> Result<(), String> {
            self.data.insert(user_id.to_string(), data);
            Ok(())
        }
        fn delete_two_factor_data(&mut self, user_id: &str) -> Result<(), String> {
            self.data.remove(user_id);
            Ok(())
        }
    }

    #[test]
    fn test_concurrent_reuse_only_first_succeeds() {
        let mut codes = vec!["7777-8888".to_string()];

        // Simulate two "threads" both reading the same code list snapshot
        // and attempting to consume the same code.
        let first = TwoFactorAuth::consume_backup_code(&mut codes, "7777-8888");
        let second = TwoFactorAuth::consume_backup_code(&mut codes, "7777-8888");

        assert!(first,  "first recovery attempt must succeed");
        assert!(!second, "second recovery attempt must fail — code already consumed");
 main
    }
}
