#[cfg(test)]
mod tests {
    use crate::two_factor::{TwoFactorAuth, TotpConfig, InMemoryStore};
    use crate::handlers::{
        TwoFactorHandlers, EnableTwoFactorRequest, VerifyTwoFactorRequest,
        LoginWithTwoFactorRequest, DisableTwoFactorRequest, RecoverWithBackupRequest,
    };
    use totp_rs::{Algorithm, Secret, TOTP};

    // ── helpers ──────────────────────────────────────────────────────────────

    fn make_handlers() -> TwoFactorHandlers<InMemoryStore> {
        TwoFactorHandlers::new(InMemoryStore::default())
    }

    /// Enable 2FA for a user and return the secret + a valid current token
    fn enable_and_get_token(handlers: &TwoFactorHandlers<InMemoryStore>, user_id: &str) -> (String, String) {
        let resp = handlers.enable_two_factor(EnableTwoFactorRequest {
            user_id: user_id.to_string(),
            email: format!("{}@petchain.com", user_id),
        }).unwrap();

        let config = TotpConfig::default();
        let totp = TOTP::new(
            config.algorithm, config.digits, config.window, config.period,
            Secret::Encoded(resp.secret.clone()).to_bytes().unwrap(),
            None, String::new(),
        ).unwrap();

        let token = totp.generate_current().unwrap();
        (resp.secret, token)
    }

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
        assert!(setup.qr_code_base64.starts_with("data:image/png;base64,"));
        assert_eq!(setup.backup_codes.len(), 8);
        assert_eq!(setup.config.algorithm, Algorithm::SHA256);
    }

    #[test]
    fn test_setup_two_factor_with_sha1_config() {
        let config = TotpConfig::legacy_sha1();
        let result = TwoFactorAuth::setup_with_config("test@petchain.com", "PetChain", config.clone());
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
        let result = TwoFactorAuth::setup_with_config("test@petchain.com", "PetChain", config.clone());
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
        
        // Generate current token with SHA256
        let totp = TOTP::new(
            config.algorithm,
            config.digits,
            config.window,
            config.period,
            Secret::Encoded(secret.clone()).to_bytes().unwrap(),
            None,
            String::new(),
        ).unwrap();
        
        let token = totp.generate_current().unwrap();
        
        // Verify it with default method (should use SHA256)
        let result = TwoFactorAuth::verify_token(&secret, &token);
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Verify it with explicit config
        let result = TwoFactorAuth::verify_token_with_config(&secret, &token, config);
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
        ).unwrap();
        
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
        ).unwrap();
        
        let token = totp.generate_current().unwrap();
        assert_eq!(token.len(), 8); // Should be 8 digits
        
        // Verify it with SHA512 config
        let result = TwoFactorAuth::verify_token_with_config(&secret, &token, config);
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
        ).unwrap();
        
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

    // ── TwoFactorHandlers state-transition tests ───────────────────────────────────────

    // ─ enable ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_handler_enable_persists_disabled_state() {
        let handlers = make_handlers();
        let resp = handlers.enable_two_factor(EnableTwoFactorRequest {
            user_id: "user1".to_string(),
            email: "user1@petchain.com".to_string(),
        });
        assert!(resp.is_ok());
        let resp = resp.unwrap();
        assert!(!resp.secret.is_empty());
        assert_eq!(resp.backup_codes.len(), 8);

        // 2FA should be stored but NOT yet enabled
        let err = handlers.verify_login_token(LoginWithTwoFactorRequest {
            user_id: "user1".to_string(),
            token: "000000".to_string(),
        });
        assert!(err.is_err());
        assert_eq!(err.unwrap_err(), "2FA is not enabled for this user");
    }

    #[test]
    fn test_handler_enable_unknown_user_returns_error() {
        let handlers = make_handlers();
        let err = handlers.verify_login_token(LoginWithTwoFactorRequest {
            user_id: "ghost".to_string(),
            token: "000000".to_string(),
        });
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("No 2FA data found for user"));
    }

    // ─ verify & activate ─────────────────────────────────────────────────────

    #[test]
    fn test_handler_verify_activates_2fa() {
        let handlers = make_handlers();
        let (_, token) = enable_and_get_token(&handlers, "user2");

        let result = handlers.verify_and_activate(VerifyTwoFactorRequest {
            user_id: "user2".to_string(),
            token,
        });
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Now login should succeed with a fresh token
        let config = TotpConfig::default();
        let stored = handlers.store_ref().get("user2").unwrap();
        let totp = TOTP::new(
            config.algorithm, config.digits, config.window, config.period,
            Secret::Encoded(stored.secret).to_bytes().unwrap(),
            None, String::new(),
        ).unwrap();
        let login_token = totp.generate_current().unwrap();

        let login = handlers.verify_login_token(LoginWithTwoFactorRequest {
            user_id: "user2".to_string(),
            token: login_token,
        });
        assert!(login.is_ok());
        assert!(login.unwrap());
    }

    #[test]
    fn test_handler_verify_invalid_token_does_not_activate() {
        let handlers = make_handlers();
        enable_and_get_token(&handlers, "user3");

        let result = handlers.verify_and_activate(VerifyTwoFactorRequest {
            user_id: "user3".to_string(),
            token: "000000".to_string(),
        });
        assert!(result.is_ok());
        assert!(!result.unwrap()); // not activated

        // Login should still be blocked
        let err = handlers.verify_login_token(LoginWithTwoFactorRequest {
            user_id: "user3".to_string(),
            token: "000000".to_string(),
        });
        assert!(err.is_err());
        assert_eq!(err.unwrap_err(), "2FA is not enabled for this user");
    }

    #[test]
    fn test_handler_verify_already_enabled_returns_error() {
        let handlers = make_handlers();
        let (_, token) = enable_and_get_token(&handlers, "user4");
        handlers.verify_and_activate(VerifyTwoFactorRequest {
            user_id: "user4".to_string(),
            token: token.clone(),
        }).unwrap();

        // Trying to activate again should fail
        let err = handlers.verify_and_activate(VerifyTwoFactorRequest {
            user_id: "user4".to_string(),
            token,
        });
        assert!(err.is_err());
        assert_eq!(err.unwrap_err(), "2FA is already enabled");
    }

    // ─ disable ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_handler_disable_removes_2fa() {
        let handlers = make_handlers();
        let (_, token) = enable_and_get_token(&handlers, "user5");
        handlers.verify_and_activate(VerifyTwoFactorRequest {
            user_id: "user5".to_string(),
            token,
        }).unwrap();

        // Generate a fresh token to disable
        let stored = handlers.store_ref().get("user5").unwrap();
        let config = TotpConfig::default();
        let totp = TOTP::new(
            config.algorithm, config.digits, config.window, config.period,
            Secret::Encoded(stored.secret).to_bytes().unwrap(),
            None, String::new(),
        ).unwrap();
        let disable_token = totp.generate_current().unwrap();

        let result = handlers.disable_two_factor(DisableTwoFactorRequest {
            user_id: "user5".to_string(),
            token: disable_token,
        });
        assert!(result.is_ok());
        assert!(result.unwrap());

        // After disable, login should fail with "not enabled"
        let err = handlers.verify_login_token(LoginWithTwoFactorRequest {
            user_id: "user5".to_string(),
            token: "000000".to_string(),
        });
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("No 2FA data found for user"));
    }

    #[test]
    fn test_handler_disable_when_not_enabled_returns_error() {
        let handlers = make_handlers();
        enable_and_get_token(&handlers, "user6"); // stored but not activated

        let err = handlers.disable_two_factor(DisableTwoFactorRequest {
            user_id: "user6".to_string(),
            token: "000000".to_string(),
        });
        assert!(err.is_err());
        assert_eq!(err.unwrap_err(), "2FA is not enabled for this user");
    }

    // ─ recovery ───────────────────────────────────────────────────────────────

    #[test]
    fn test_handler_recovery_consumes_backup_code() {
        let handlers = make_handlers();
        let (_, token) = enable_and_get_token(&handlers, "user7");
        handlers.verify_and_activate(VerifyTwoFactorRequest {
            user_id: "user7".to_string(),
            token,
        }).unwrap();

        let backup_code = handlers.store_ref().get("user7").unwrap().backup_codes[0].clone();

        // First use: should succeed
        let result = handlers.recover_with_backup(RecoverWithBackupRequest {
            user_id: "user7".to_string(),
            backup_code: backup_code.clone(),
        });
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Code is consumed - second use must fail
        let result = handlers.recover_with_backup(RecoverWithBackupRequest {
            user_id: "user7".to_string(),
            backup_code,
        });
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Remaining codes count should be 7
        let remaining = handlers.store_ref().get("user7").unwrap().backup_codes.len();
        assert_eq!(remaining, 7);
    }

    #[test]
    fn test_handler_recovery_invalid_code_returns_false() {
        let handlers = make_handlers();
        let (_, token) = enable_and_get_token(&handlers, "user8");
        handlers.verify_and_activate(VerifyTwoFactorRequest {
            user_id: "user8".to_string(),
            token,
        }).unwrap();

        let result = handlers.recover_with_backup(RecoverWithBackupRequest {
            user_id: "user8".to_string(),
            backup_code: "0000-0000".to_string(),
        });
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_handler_recovery_when_not_enabled_returns_error() {
        let handlers = make_handlers();
        enable_and_get_token(&handlers, "user9"); // stored but not activated

        let err = handlers.recover_with_backup(RecoverWithBackupRequest {
            user_id: "user9".to_string(),
            backup_code: "1234-5678".to_string(),
        });
        assert!(err.is_err());
        assert_eq!(err.unwrap_err(), "2FA is not enabled for this user");
    }
}
