#[cfg(test)]
mod tests {
    use crate::two_factor::TwoFactorAuth;

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
        assert!(setup.qr_code_base64.starts_with("data:image/png;base64,"));
        assert_eq!(setup.backup_codes.len(), 8);
    }

    #[test]
    fn test_verify_token() {
        let secret = TwoFactorAuth::generate_secret();

        // Generate current token
        use totp_rs::{Algorithm, Secret, TOTP};
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.clone()).to_bytes().unwrap(),
            None,
            String::new(),
        )
        .unwrap();

        let token = totp.generate_current().unwrap();

        // Verify it
        let result = TwoFactorAuth::verify_token(&secret, &token);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_invalid_token() {
        let secret = TwoFactorAuth::generate_secret();
        let result = TwoFactorAuth::verify_token(&secret, "000000");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_generate_backup_codes() {
        let codes = TwoFactorAuth::generate_backup_codes(8);
        assert_eq!(codes.len(), 8);

        for code in &codes {
            assert!(code.contains('-'));
            assert_eq!(code.len(), 9); // Format: 1234-5678
        }

        // Ensure uniqueness
        let unique_codes: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique_codes.len(), 8);
    }

    #[test]
    fn test_verify_backup_code() {
        let codes = vec![
            "1234-5678".to_string(),
            "2345-6789".to_string(),
            "3456-7890".to_string(),
        ];

        let result = TwoFactorAuth::verify_backup_code(&codes, "2345-6789");
        assert_eq!(result, Some(1));

        let result = TwoFactorAuth::verify_backup_code(&codes, "9999-9999");
        assert_eq!(result, None);
    }

    // ---------------------------------------------------------------------------
    // Rate limiter unit tests
    // ---------------------------------------------------------------------------

    #[cfg(test)]
    mod rate_limiter_tests {
        use crate::handlers::{
            DisableTwoFactorRequest, LoginWithTwoFactorRequest, TwoFactorHandlers,
            VerifyTwoFactorRequest,
        };
        use crate::rate_limiter::{InMemoryRateLimiter, RateLimitResult, RateLimiter};
        use std::sync::Arc;

        // --- InMemoryRateLimiter behaviour ---

        #[test]
        fn test_allows_attempts_below_limit() {
            let limiter = InMemoryRateLimiter::new(5, 60, 300);

            for i in 1..5 {
                let result = limiter.record_failure("user:test");
                match result {
                    RateLimitResult::Allowed { remaining } => {
                        assert_eq!(
                            remaining,
                            5 - i,
                            "remaining should decrease with each failure"
                        );
                    }
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

            let result = limiter.record_failure("user:lockout");
            match result {
                RateLimitResult::Blocked { retry_after_secs } => {
                    assert_eq!(retry_after_secs, 300);
                }
                RateLimitResult::Allowed { .. } => panic!("should be blocked after max failures"),
            }
        }

        #[test]
        fn test_success_clears_counter() {
            let limiter = InMemoryRateLimiter::new(3, 60, 300);

            limiter.record_failure("user:clear");
            limiter.record_failure("user:clear");
            limiter.record_success("user:clear");

            // Counter reset — should have full attempts again
            let result = limiter.record_failure("user:clear");
            match result {
                RateLimitResult::Allowed { remaining } => {
                    assert_eq!(remaining, 2, "counter should have reset after success");
                }
                RateLimitResult::Blocked { .. } => {
                    panic!("should not be blocked after success clears counter")
                }
            }
        }

        #[test]
        fn test_blocked_remains_blocked_within_lockout() {
            let limiter = InMemoryRateLimiter::new(2, 60, 300);

            limiter.record_failure("user:persist");
            limiter.record_failure("user:persist");

            // Multiple subsequent calls should all return Blocked
            for _ in 0..5 {
                let result = limiter.record_failure("user:persist");
                assert!(
                    matches!(result, RateLimitResult::Blocked { .. }),
                    "should remain blocked throughout lockout period"
                );
            }
        }

        #[test]
        fn test_different_keys_are_independent() {
            let limiter = InMemoryRateLimiter::new(2, 60, 300);

            limiter.record_failure("user:alice");
            limiter.record_failure("user:alice");

            // alice is now blocked — bob should be unaffected
            let result = limiter.record_failure("user:bob");
            assert!(
                matches!(result, RateLimitResult::Allowed { .. }),
                "bob should not be affected by alice's failures"
            );
        }

        // --- Pluggable limiter: test stub ---

        /// A stub limiter that always blocks, used to verify handler integration
        /// without depending on timing or real attempt counts.
        struct AlwaysBlockedLimiter;

        impl RateLimiter for AlwaysBlockedLimiter {
            fn record_failure(&self, _key: &str) -> RateLimitResult {
                RateLimitResult::Blocked {
                    retry_after_secs: 300,
                }
            }
            fn record_success(&self, _key: &str) {}
        }

        /// A stub limiter that always allows, used to verify handlers reach their
        /// token-verification logic normally when not rate-limited.
        struct AlwaysAllowedLimiter;

        impl RateLimiter for AlwaysAllowedLimiter {
            fn record_failure(&self, _key: &str) -> RateLimitResult {
                RateLimitResult::Allowed { remaining: 99 }
            }
            fn record_success(&self, _key: &str) {}
        }

        // --- Handler integration tests ---

        #[test]
        fn test_verify_and_activate_blocked_returns_error() {
            let handlers = TwoFactorHandlers::with_limiter(Arc::new(AlwaysBlockedLimiter));
            let result = handlers.verify_and_activate(VerifyTwoFactorRequest {
                user_id: "user1".to_string(),
                token: "123456".to_string(),
            });
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Too many failed attempts"));
        }

        #[test]
        fn test_verify_login_token_blocked_returns_error() {
            let handlers = TwoFactorHandlers::with_limiter(Arc::new(AlwaysBlockedLimiter));
            let result = handlers.verify_login_token(LoginWithTwoFactorRequest {
                user_id: "user1".to_string(),
                token: "123456".to_string(),
            });
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Too many failed attempts"));
        }

        #[test]
        fn test_disable_two_factor_blocked_returns_error() {
            let handlers = TwoFactorHandlers::with_limiter(Arc::new(AlwaysBlockedLimiter));
            let result = handlers.disable_two_factor(DisableTwoFactorRequest {
                user_id: "user1".to_string(),
                token: "123456".to_string(),
            });
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Too many failed attempts"));
        }

        #[test]
        fn test_rate_limit_is_per_endpoint_not_shared() {
            // Exhaust login limit for user1
            let limiter = Arc::new(InMemoryRateLimiter::new(2, 60, 300));
            let handlers = TwoFactorHandlers::with_limiter(limiter);

            handlers
                .verify_login_token(LoginWithTwoFactorRequest {
                    user_id: "user1".to_string(),
                    token: "bad".to_string(),
                })
                .ok();
            handlers
                .verify_login_token(LoginWithTwoFactorRequest {
                    user_id: "user1".to_string(),
                    token: "bad".to_string(),
                })
                .ok();

            // Login should now be blocked
            let login_result = handlers.verify_login_token(LoginWithTwoFactorRequest {
                user_id: "user1".to_string(),
                token: "bad".to_string(),
            });
            assert!(login_result.is_err(), "login should be blocked");

            // disable endpoint uses a different key — should still be allowed
            let disable_result = handlers.disable_two_factor(DisableTwoFactorRequest {
                user_id: "user1".to_string(),
                token: "bad".to_string(),
            });
            // It won't return a rate-limit error (it proceeds to token verification)
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
            use std::sync::Arc;
            use std::thread;

            let limiter = Arc::new(InMemoryRateLimiter::new(100, 60, 300));
            let mut handles = vec![];

            for i in 0..10 {
                let l = Arc::clone(&limiter);
                handles.push(thread::spawn(move || {
                    l.record_failure(&format!("user:{}", i));
                }));
            }

            for h in handles {
                h.join().expect("thread panicked");
            }
        }
    }
}
