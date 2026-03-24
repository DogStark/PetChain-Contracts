#[cfg(test)]
mod tests {
    use crate::two_factor::{TwoFactorAuth, TotpConfig};
    use totp_rs::{Algorithm, Secret, TOTP};

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
}
