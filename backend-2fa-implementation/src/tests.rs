#[cfg(test)]
mod tests {
    use crate::two_factor::{TwoFactorAuth, TwoFactorError};
    use crate::handlers::{TwoFactorHandlers, EnableTwoFactorRequest, RecoverWithBackupRequest};

    // ── existing core tests (updated for new verify_token signature) ──────────

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
        assert_eq!(TwoFactorAuth::verify_token(&secret, &token), Ok(()));
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
    }
}
