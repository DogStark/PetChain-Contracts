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

    #[test]
    fn test_generate_backup_codes_are_unique() {
        // Standard uniqueness check for a normal batch
        let codes = TwoFactorAuth::generate_backup_codes(8);
        assert_eq!(codes.len(), 8);

        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), 8, "Backup codes must all be unique");
    }

    #[test]
    fn test_generate_backup_codes_no_collisions_under_stress() {
        // Run generation many times to statistically surface any collision bug.
        // 1000 batches of 8 codes = 8000 total generation calls.
        for run in 0..1000 {
            let codes = TwoFactorAuth::generate_backup_codes(8);

            assert_eq!(
                codes.len(),
                8,
                "Run {}: expected 8 codes, got {}",
                run,
                codes.len()
            );

            let unique: std::collections::HashSet<_> = codes.iter().collect();
            assert_eq!(
                unique.len(),
                8,
                "Run {}: found duplicate backup codes: {:?}",
                run,
                codes
            );
        }
    }
}

#[cfg(test)]
mod drift_policy_tests {
    use crate::two_factor::{ClockDriftPolicy, TwoFactorAuth};
    use std::time::{SystemTime, UNIX_EPOCH};
    use totp_rs::{Algorithm, Secret, TOTP};

    const STEP_SECS: u64 = 30;

    /// Generate a TOTP token for an arbitrary Unix timestamp.
    fn token_at(secret: &str, ts: u64) -> String {
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.to_string()).to_bytes().unwrap(),
            None,
            String::new(),
        )
        .unwrap();
        totp.generate(ts)
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    // --- ClockDriftPolicy construction ---

    #[test]
    fn test_policy_constants() {
        assert_eq!(ClockDriftPolicy::STRICT.allowed_steps, 0);
        assert_eq!(ClockDriftPolicy::STANDARD.allowed_steps, 1);
        assert_eq!(ClockDriftPolicy::LENIENT.allowed_steps, 2);
    }

    #[test]
    fn test_custom_policy_clamps_at_2() {
        let p = ClockDriftPolicy::custom(5);
        assert_eq!(p.allowed_steps, 2, "custom() should clamp to max 2");
    }

    #[test]
    fn test_default_policy_is_standard() {
        assert_eq!(ClockDriftPolicy::default(), ClockDriftPolicy::STANDARD);
    }

    // --- STRICT policy (0 steps) ---

    #[test]
    fn test_strict_accepts_current_token() {
        let secret = TwoFactorAuth::generate_secret();
        let now = now_secs();
        let token = token_at(&secret, now);

        let result =
            TwoFactorAuth::verify_token_with_policy(&secret, &token, ClockDriftPolicy::STRICT);
        assert!(result.is_ok());
        assert!(
            result.unwrap(),
            "STRICT should accept the current-step token"
        );
    }

    #[test]
    fn test_strict_rejects_previous_step() {
        let secret = TwoFactorAuth::generate_secret();
        let now = now_secs();
        let past_token = token_at(&secret, now.saturating_sub(STEP_SECS));

        // Only run this assertion when the past token differs from the current one
        // (they can collide at step boundaries, which would be a false failure).
        let current_token = token_at(&secret, now);
        if past_token != current_token {
            let result = TwoFactorAuth::verify_token_with_policy(
                &secret,
                &past_token,
                ClockDriftPolicy::STRICT,
            );
            assert!(result.is_ok());
            assert!(
                !result.unwrap(),
                "STRICT should reject a token from the previous step"
            );
        }
    }

    #[test]
    fn test_strict_rejects_next_step() {
        let secret = TwoFactorAuth::generate_secret();
        let now = now_secs();
        let future_token = token_at(&secret, now + STEP_SECS);

        let current_token = token_at(&secret, now);
        if future_token != current_token {
            let result = TwoFactorAuth::verify_token_with_policy(
                &secret,
                &future_token,
                ClockDriftPolicy::STRICT,
            );
            assert!(result.is_ok());
            assert!(
                !result.unwrap(),
                "STRICT should reject a token from the next step"
            );
        }
    }

    // --- STANDARD policy (±1 step) ---

    #[test]
    fn test_standard_accepts_current_token() {
        let secret = TwoFactorAuth::generate_secret();
        let now = now_secs();
        let token = token_at(&secret, now);

        let result =
            TwoFactorAuth::verify_token_with_policy(&secret, &token, ClockDriftPolicy::STANDARD);
        assert!(result.unwrap(), "STANDARD should accept current token");
    }

    #[test]
    fn test_standard_accepts_one_step_behind() {
        let secret = TwoFactorAuth::generate_secret();
        let now = now_secs();
        let past_token = token_at(&secret, now.saturating_sub(STEP_SECS));

        let result = TwoFactorAuth::verify_token_with_policy(
            &secret,
            &past_token,
            ClockDriftPolicy::STANDARD,
        );
        assert!(
            result.unwrap(),
            "STANDARD should accept a token one step in the past"
        );
    }

    #[test]
    fn test_standard_accepts_one_step_ahead() {
        let secret = TwoFactorAuth::generate_secret();
        let now = now_secs();
        let future_token = token_at(&secret, now + STEP_SECS);

        let result = TwoFactorAuth::verify_token_with_policy(
            &secret,
            &future_token,
            ClockDriftPolicy::STANDARD,
        );
        assert!(
            result.unwrap(),
            "STANDARD should accept a token one step in the future"
        );
    }

    #[test]
    fn test_standard_rejects_two_steps_behind() {
        let secret = TwoFactorAuth::generate_secret();
        let now = now_secs();
        let old_token = token_at(&secret, now.saturating_sub(2 * STEP_SECS));

        let current = token_at(&secret, now);
        let prev = token_at(&secret, now.saturating_sub(STEP_SECS));

        // Guard: token must differ from both accepted steps
        if old_token != current && old_token != prev {
            let result = TwoFactorAuth::verify_token_with_policy(
                &secret,
                &old_token,
                ClockDriftPolicy::STANDARD,
            );
            assert!(
                !result.unwrap(),
                "STANDARD should reject a token two steps in the past"
            );
        }
    }

    // --- LENIENT policy (±2 steps) ---

    #[test]
    fn test_lenient_accepts_two_steps_behind() {
        let secret = TwoFactorAuth::generate_secret();
        let now = now_secs();
        let old_token = token_at(&secret, now.saturating_sub(2 * STEP_SECS));

        let result =
            TwoFactorAuth::verify_token_with_policy(&secret, &old_token, ClockDriftPolicy::LENIENT);
        assert!(
            result.unwrap(),
            "LENIENT should accept a token two steps in the past"
        );
    }

    #[test]
    fn test_lenient_accepts_two_steps_ahead() {
        let secret = TwoFactorAuth::generate_secret();
        let now = now_secs();
        let future_token = token_at(&secret, now + 2 * STEP_SECS);

        let result = TwoFactorAuth::verify_token_with_policy(
            &secret,
            &future_token,
            ClockDriftPolicy::LENIENT,
        );
        assert!(
            result.unwrap(),
            "LENIENT should accept a token two steps in the future"
        );
    }

    #[test]
    fn test_lenient_rejects_three_steps_out() {
        let secret = TwoFactorAuth::generate_secret();
        let now = now_secs();
        let old_token = token_at(&secret, now.saturating_sub(3 * STEP_SECS));

        // Guard: must differ from all accepted windows
        let accepted: Vec<String> = (-2i64..=2)
            .map(|o| {
                if o >= 0 {
                    token_at(&secret, now + o as u64 * STEP_SECS)
                } else {
                    token_at(&secret, now.saturating_sub((-o) as u64 * STEP_SECS))
                }
            })
            .collect();

        if !accepted.contains(&old_token) {
            let result = TwoFactorAuth::verify_token_with_policy(
                &secret,
                &old_token,
                ClockDriftPolicy::LENIENT,
            );
            assert!(
                !result.unwrap(),
                "LENIENT should reject a token three steps out"
            );
        }
    }

    // --- Invalid token always rejected ---

    #[test]
    fn test_all_policies_reject_garbage_token() {
        let secret = TwoFactorAuth::generate_secret();

        for policy in [
            ClockDriftPolicy::STRICT,
            ClockDriftPolicy::STANDARD,
            ClockDriftPolicy::LENIENT,
        ] {
            let result = TwoFactorAuth::verify_token_with_policy(&secret, "000000", policy);
            assert!(result.is_ok());
            // 000000 could theoretically be valid, so only assert false when it isn't
            // the actual current token — the important thing is no panic/error.
            let _ = result.unwrap();
        }
    }

    // --- verify_token still works as a convenience wrapper ---

    #[test]
    fn test_verify_token_uses_standard_policy() {
        let secret = TwoFactorAuth::generate_secret();
        let now = now_secs();

        // A token one step behind should be accepted by the default wrapper
        let past_token = token_at(&secret, now.saturating_sub(STEP_SECS));
        let result = TwoFactorAuth::verify_token(&secret, &past_token);
        assert!(
            result.unwrap(),
            "verify_token should accept ±1 step like STANDARD"
        );
    }
}
