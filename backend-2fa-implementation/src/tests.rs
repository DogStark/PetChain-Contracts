#[cfg(test)]
mod tests {
    use crate::two_factor::TwoFactorAuth;
    use proptest::prelude::*;

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
        ).unwrap();
        
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
    fn test_generate_backup_codes() {
        let codes = TwoFactorAuth::generate_backup_codes(8);
        assert_eq!(codes.len(), 8);
        
        for code in &codes {
            assert!(code.contains('-'));
            assert_eq!(code.len(), 9);
        }
    }

    #[test]
    fn test_verify_backup_code() {
        let codes = vec![
            "1234-5678".to_string(),
            "2345-6789".to_string(),
        ];
        
        let result = TwoFactorAuth::verify_backup_code(&codes, "2345-6789");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(1));
        
        let result = TwoFactorAuth::verify_backup_code(&codes, "9999-9999");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);

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
    fn test_handler_disable_two_factor() {
        let mut storage = MockStorage { data: HashMap::new() };
        let user_id = "user123".to_string();
        
        // 1. Enable 2FA
        let setup = TwoFactorHandlers::enable_two_factor(&mut storage, EnableTwoFactorRequest {
            user_id: user_id.clone(),
            email: "test@example.com".to_string(),
        }).unwrap();
        
        // 2. Verify and activate
        use totp_rs::{Algorithm, Secret, TOTP};
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(setup.secret.clone()).to_bytes().unwrap(),
            None,
            String::new(),
        ).unwrap();
        let token = totp.generate_current().unwrap();
        
        TwoFactorHandlers::verify_and_activate(&mut storage, VerifyTwoFactorRequest {
            user_id: user_id.clone(),
            token: token.clone(),
        }).unwrap();
        
        // Verify it is enabled in storage
        let data = storage.get_two_factor_data(&user_id).unwrap().unwrap();
        assert!(data.enabled);
        
        // 3. Disable 2FA
        let result = TwoFactorHandlers::disable_two_factor(&mut storage, DisableTwoFactorRequest {
            user_id: user_id.clone(),
            token,
        }).unwrap();
        
        assert!(result);
        
        // 4. Verify it is deleted/disabled in storage
        let data = storage.get_two_factor_data(&user_id).unwrap();
        assert!(data.is_none());
    }
}
