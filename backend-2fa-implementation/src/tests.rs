#[cfg(test)]
mod tests {
    use crate::handlers::{
        clear_two_factor_store_for_tests, overwrite_two_factor_data_for_tests,
        EnableTwoFactorRequest, LoginWithTwoFactorRequest, TwoFactorHandlers,
        clear_two_factor_store_for_tests, get_two_factor_data_for_tests,
        overwrite_two_factor_data_for_tests, EnableTwoFactorRequest, LoginWithTwoFactorRequest,
        TwoFactorHandlers, VerifyTwoFactorRequest,
    };
    use crate::two_factor::{TwoFactorAuth, TwoFactorData};

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
        let token = generate_token(&secret);

        // Verify it
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

 main
        assert_eq!(TwoFactorAuth::verify_backup_code(&codes, "2345-6789"), Some(1));
        assert_eq!(TwoFactorAuth::verify_backup_code(&codes, "9999-9999"), None);
    }

    // ── new typed-error tests ─────────────────────────────────────────────────

    #[test]
    fn test_token_mismatch_returns_token_mismatch_variant() {
        let secret = TwoFactorAuth::generate_secret();
        // "000000" is a valid format but almost certainly wrong
        let err = TwoFactorAuth::verify_token(&secret, "000000").unwrap_err();
        assert_eq!(err, TwoFactorError::TokenMismatch);
    }

    #[test]
    fn test_invalid_token_format_too_long() {
        let secret = TwoFactorAuth::generate_secret();
        let err = TwoFactorAuth::verify_token(&secret, "1234567").unwrap_err();
        assert_eq!(err, TwoFactorError::InvalidToken);
    }

    #[test]
    fn test_invalid_token_format_non_numeric() {
        let secret = TwoFactorAuth::generate_secret();
        let err = TwoFactorAuth::verify_token(&secret, "12345a").unwrap_err();
        assert_eq!(err, TwoFactorError::InvalidToken);
    }

    #[test]
    fn test_invalid_token_format_empty() {
        let secret = TwoFactorAuth::generate_secret();
        let err = TwoFactorAuth::verify_token(&secret, "").unwrap_err();
        assert_eq!(err, TwoFactorError::InvalidToken);
    }

    #[test]
    fn test_recover_with_backup_invalid_code_returns_invalid_backup_code() {
        let req = RecoverWithBackupRequest {
            user_id: "user1".to_string(),
            backup_code: "9999-9999".to_string(),
        };
        let err = TwoFactorHandlers::recover_with_backup(req).unwrap_err();
        assert_eq!(err, TwoFactorError::InvalidBackupCode);
    }

    #[test]
    fn test_recover_with_backup_valid_code_succeeds() {
        let req = RecoverWithBackupRequest {
            user_id: "user1".to_string(),
            backup_code: "1234-5678".to_string(),
        };
        assert!(TwoFactorHandlers::recover_with_backup(req).is_ok());
    }

    #[test]
    fn test_http_status_token_mismatch_is_401() {
        assert_eq!(TwoFactorError::TokenMismatch.http_status(), 401);
    }

    #[test]
    fn test_http_status_not_found_is_401() {
        // NotFound must NOT leak "secret exists" vs "token wrong" — same 401
        assert_eq!(TwoFactorError::NotFound.http_status(), 401);
    }

    #[test]
    fn test_http_status_invalid_backup_code_is_401() {
        assert_eq!(TwoFactorError::InvalidBackupCode.http_status(), 401);
    }

    #[test]
    fn test_http_status_invalid_token_is_422() {
        assert_eq!(TwoFactorError::InvalidToken.http_status(), 422);
    }

    #[test]
    fn test_http_status_setup_failure_is_500() {
        assert_eq!(TwoFactorError::SetupFailure("err".into()).http_status(), 500);
    }

    #[test]
    fn test_http_status_storage_error_is_500() {
        assert_eq!(TwoFactorError::StorageError("db down".into()).http_status(), 500);
    }

    #[test]
    fn test_display_does_not_leak_secret_existence() {
        // Both NotFound and TokenMismatch must produce the same user-facing message
        assert_eq!(
            TwoFactorError::NotFound.to_string(),
            TwoFactorError::TokenMismatch.to_string()
        );
    }

    #[test]
    fn test_enable_two_factor_returns_setup_data() {
        let req = EnableTwoFactorRequest {
            user_id: "user1".to_string(),
            email: "user@petchain.com".to_string(),
        };
        let resp = TwoFactorHandlers::enable_two_factor(req).unwrap();
        assert!(!resp.secret.is_empty());
        assert_eq!(resp.backup_codes.len(), 8);

        let result = TwoFactorAuth::verify_backup_code(&codes, "2345-6789");
        assert_eq!(result, Some(1));

        let result = TwoFactorAuth::verify_backup_code(&codes, "9999-9999");
        assert_eq!(result, None);
main
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
    fn test_backup_code_cannot_be_reused_after_consumption() {
        let mut codes = vec!["1234-5678".to_string()];

        // First use succeeds
        assert!(TwoFactorAuth::consume_backup_code(&mut codes, "1234-5678"));
        // Second use on the now-empty list must fail
        assert!(!TwoFactorAuth::consume_backup_code(&mut codes, "1234-5678"));
        assert!(codes.is_empty());
    }

    #[test]
    fn test_consume_invalid_backup_code_returns_false() {
        let mut codes = vec!["1234-5678".to_string()];

        let result = TwoFactorAuth::consume_backup_code(&mut codes, "9999-9999");
        assert!(!result);
        // List must be unchanged
        assert_eq!(codes.len(), 1);
    }

    #[test]
    fn test_each_backup_code_single_use_across_all_codes() {
        let originals = vec![
            "1111-1111".to_string(),
            "2222-2222".to_string(),
            "3333-3333".to_string(),
        ];
        let mut codes = originals.clone();

        // Consume every code exactly once
        for code in &originals {
            assert!(TwoFactorAuth::consume_backup_code(&mut codes, code));
        }
        assert!(codes.is_empty());

        // Attempting any code again must fail
        for code in &originals {
            assert!(!TwoFactorAuth::consume_backup_code(&mut codes, code));
        }
    }

    /// Simulates two concurrent recovery attempts using the same backup code.
    /// In a real system these would race against the DB; here we model atomicity
    /// by applying both operations sequentially on the same mutable list —
    /// only the first must succeed.
    #[test]
    fn test_concurrent_reuse_only_first_succeeds() {
        let mut codes = vec!["7777-8888".to_string()];

        // Simulate two "threads" both reading the same code list snapshot
        // and attempting to consume the same code.
        let first = TwoFactorAuth::consume_backup_code(&mut codes, "7777-8888");
        let second = TwoFactorAuth::consume_backup_code(&mut codes, "7777-8888");

        assert!(first,  "first recovery attempt must succeed");
        assert!(!second, "second recovery attempt must fail — code already consumed");
    }
}
