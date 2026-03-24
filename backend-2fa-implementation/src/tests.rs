#[cfg(test)]
mod tests {
    use crate::handlers::{
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
    fn test_verify_and_activate_persists_enabled_state() {
        clear_two_factor_store_for_tests();

        let user_id = "user-activate";
        let setup = TwoFactorHandlers::enable_two_factor(EnableTwoFactorRequest {
            user_id: user_id.to_string(),
            email: "activate@petchain.com".to_string(),
        })
        .unwrap();

        let before = get_two_factor_data_for_tests(user_id).unwrap();
        assert!(!before.enabled);

        let result = TwoFactorHandlers::verify_and_activate(VerifyTwoFactorRequest {
            user_id: user_id.to_string(),
            token: generate_token(&setup.secret),
        })
        .unwrap();

        assert!(result);

        let after = get_two_factor_data_for_tests(user_id).unwrap();
        assert!(after.enabled);
        assert_eq!(after.secret, setup.secret);
    }

    #[test]
    fn test_verify_login_token_fails_when_two_factor_is_disabled() {
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

    #[test]
    fn test_verify_uses_stored_secret_instead_of_placeholder_secret() {
        clear_two_factor_store_for_tests();

        let user_id = "user-secret-check";
        let stored_secret = TwoFactorAuth::generate_secret();
        let placeholder_secret = "JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PXP";
        let placeholder_token = generate_token(placeholder_secret);

        overwrite_two_factor_data_for_tests(
            user_id,
            TwoFactorData {
                secret: stored_secret.clone(),
                backup_codes: vec![],
                enabled: false,
            },
        );

        let result = TwoFactorHandlers::verify_and_activate(VerifyTwoFactorRequest {
            user_id: user_id.to_string(),
            token: placeholder_token,
        })
        .unwrap();

        assert!(!result);

        let stored = get_two_factor_data_for_tests(user_id).unwrap();
        assert_eq!(stored.secret, stored_secret);
        assert!(!stored.enabled);
    }

    #[test]
    fn test_activation_does_not_persist_on_failed_verification() {
        clear_two_factor_store_for_tests();

        let user_id = "user-no-partial-activation";
        let setup = TwoFactorHandlers::enable_two_factor(EnableTwoFactorRequest {
            user_id: user_id.to_string(),
            email: "no-partial@petchain.com".to_string(),
        })
        .unwrap();

        let invalid_secret = "JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PXP";
        let invalid_token = generate_token(invalid_secret);
        assert_ne!(setup.secret, invalid_secret);

        let result = TwoFactorHandlers::verify_and_activate(VerifyTwoFactorRequest {
            user_id: user_id.to_string(),
            token: invalid_token,
        })
        .unwrap();

        assert!(!result);
        assert!(!get_two_factor_data_for_tests(user_id).unwrap().enabled);
    }
}
