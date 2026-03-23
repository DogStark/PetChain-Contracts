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
        // totp-rs >= 5.7 returns raw base64 (no data-URI prefix)
        assert!(!setup.qr_code_base64.is_empty());
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

    // -----------------------------------------------------------------------
    // Recovery policy tests
    // -----------------------------------------------------------------------

    /// After recovery, the old TOTP secret must no longer produce valid tokens.
    #[test]
    fn test_recovery_rotates_secret_old_token_invalid() {
        use totp_rs::{Algorithm, Secret, TOTP};

        // Simulate pre-recovery state
        let old_secret = TwoFactorAuth::generate_secret();
        let old_totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(old_secret.clone()).to_bytes().unwrap(),
            None,
            String::new(),
        ).unwrap();
        let old_token = old_totp.generate_current().unwrap();

        // Perform recovery
        let recovery = TwoFactorAuth::rotate_after_recovery();

        // Old token must NOT validate against the new secret
        let still_valid = TwoFactorAuth::verify_token(&recovery.new_secret, &old_token).unwrap();
        assert!(
            !still_valid,
            "Old TOTP token must be invalid after secret rotation"
        );
    }

    /// After recovery, the new secret must be different from the old one.
    #[test]
    fn test_recovery_produces_new_secret() {
        let old_secret = TwoFactorAuth::generate_secret();
        let recovery = TwoFactorAuth::rotate_after_recovery();

        assert_ne!(
            old_secret, recovery.new_secret,
            "Recovery must issue a new secret, not reuse the old one"
        );
        assert!(!recovery.new_secret.is_empty());
    }

    /// After recovery, all old backup codes must be invalid (they are replaced entirely).
    #[test]
    fn test_recovery_invalidates_old_backup_codes() {
        let old_codes = TwoFactorAuth::generate_backup_codes(8);
        let recovery = TwoFactorAuth::rotate_after_recovery();

        // None of the old codes should appear in the new set
        for old_code in &old_codes {
            assert!(
                !recovery.new_backup_codes.contains(old_code),
                "Old backup code '{}' must not appear in post-recovery codes",
                old_code
            );
        }
    }

    /// After recovery, a fresh set of 8 backup codes is issued.
    #[test]
    fn test_recovery_issues_fresh_backup_codes() {
        let recovery = TwoFactorAuth::rotate_after_recovery();

        assert_eq!(
            recovery.new_backup_codes.len(),
            8,
            "Recovery must issue exactly 8 fresh backup codes"
        );

        // All codes must follow the expected format
        for code in &recovery.new_backup_codes {
            assert!(code.contains('-'), "Backup code '{}' must contain a dash", code);
            assert_eq!(code.len(), 9, "Backup code '{}' must be 9 chars (XXXX-XXXX)", code);
        }
    }

    /// After recovery, 2FA must remain enabled — the account is not left unprotected.
    #[test]
    fn test_recovery_keeps_2fa_enabled() {
        let recovery = TwoFactorAuth::rotate_after_recovery();
        assert!(
            recovery.enabled,
            "2FA must remain enabled after backup-code recovery"
        );
    }

    /// The used backup code must not validate against the new backup code list.
    #[test]
    fn test_used_backup_code_invalid_after_recovery() {
        let old_codes = TwoFactorAuth::generate_backup_codes(8);
        let used_code = old_codes[0].clone();

        let recovery = TwoFactorAuth::rotate_after_recovery();

        // The used code must not be present in the new backup codes
        let still_valid = TwoFactorAuth::verify_backup_code(&recovery.new_backup_codes, &used_code);
        assert!(
            still_valid.is_none(),
            "Used backup code must be invalid after recovery"
        );
    }

    /// A token generated from the new post-recovery secret must be valid.
    #[test]
    fn test_new_secret_token_valid_after_recovery() {
        use totp_rs::{Algorithm, Secret, TOTP};

        let recovery = TwoFactorAuth::rotate_after_recovery();

        let new_totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(recovery.new_secret.clone()).to_bytes().unwrap(),
            None,
            String::new(),
        ).unwrap();

        let new_token = new_totp.generate_current().unwrap();
        let valid = TwoFactorAuth::verify_token(&recovery.new_secret, &new_token).unwrap();
        assert!(valid, "Token from new post-recovery secret must be valid");
    }
}
