#[cfg(test)]
mod tests {
    use crate::handlers::{
        clear_two_factor_store_for_tests, overwrite_two_factor_data_for_tests,
        EnableTwoFactorRequest, LoginWithTwoFactorRequest, TwoFactorHandlers,
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
    fn test_verify_token() {
        let secret = TwoFactorAuth::generate_secret();

        let token = generate_token(&secret);

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
    fn test_verify_login_token_uses_stored_secret_for_user() {
        clear_two_factor_store_for_tests();

        let user_id = "user-secret-check";
        let stored_secret = TwoFactorAuth::generate_secret();
        let placeholder_secret = "JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PXP";
        let placeholder_token = generate_token(placeholder_secret);

        overwrite_two_factor_data_for_tests(
            user_id,
            TwoFactorData {
                secret: stored_secret,
                backup_codes: vec![],
                enabled: true,
            },
        );

        let result = TwoFactorHandlers::verify_login_token(LoginWithTwoFactorRequest {
            user_id: user_id.to_string(),
            token: placeholder_token,
        })
        .unwrap();

        assert!(!result);
    }

    #[test]
    fn test_verify_login_token_succeeds_with_correct_token_when_enabled() {
        clear_two_factor_store_for_tests();

        let user_id = "user-enabled-ok";
        let setup = TwoFactorHandlers::enable_two_factor(EnableTwoFactorRequest {
            user_id: user_id.to_string(),
            email: "enabled@petchain.com".to_string(),
        })
        .unwrap();

        overwrite_two_factor_data_for_tests(
            user_id,
            TwoFactorData {
                secret: setup.secret.clone(),
                backup_codes: setup.backup_codes,
                enabled: true,
            },
        );

        let result = TwoFactorHandlers::verify_login_token(LoginWithTwoFactorRequest {
            user_id: user_id.to_string(),
            token: generate_token(&setup.secret),
        })
        .unwrap();

        assert!(result);
    }

    #[test]
    fn test_verify_login_token_fails_with_wrong_token_when_enabled() {
        clear_two_factor_store_for_tests();

        let user_id = "user-enabled-bad-token";
        let setup = TwoFactorHandlers::enable_two_factor(EnableTwoFactorRequest {
            user_id: user_id.to_string(),
            email: "wrong-token@petchain.com".to_string(),
        })
        .unwrap();
        let wrong_secret = "JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PXP";
        assert_ne!(setup.secret, wrong_secret);

        overwrite_two_factor_data_for_tests(
            user_id,
            TwoFactorData {
                secret: setup.secret,
                backup_codes: setup.backup_codes,
                enabled: true,
            },
        );

        let result = TwoFactorHandlers::verify_login_token(LoginWithTwoFactorRequest {
            user_id: user_id.to_string(),
            token: generate_token(wrong_secret),
        })
        .unwrap();

        assert!(!result);
    }

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

        let result = TwoFactorHandlers::verify_login_token(LoginWithTwoFactorRequest {
            user_id: user_id.to_string(),
            token,
        })
        .unwrap();

        assert!(!result);
    }
}
