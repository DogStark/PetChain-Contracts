use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use totp_rs::{Algorithm, Secret, TOTP};

/// Configuration for TOTP parameters to ensure cryptographic agility
#[derive(Debug, Clone)]
pub struct TotpConfig {
    pub algorithm: Algorithm,
    pub digits: usize,
    pub period: u64,
    pub window: u8,
}

impl Default for TotpConfig {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::SHA1,
            digits: 6,
            period: 30,
            window: 1,
        }
    }
}

impl TotpConfig {
    pub fn legacy_sha1() -> Self {
        Self {
            algorithm: Algorithm::SHA1,
            digits: 6,
            period: 30,
            window: 1,
        }
    }

    pub fn high_security() -> Self {
        Self {
            algorithm: Algorithm::SHA512,
            digits: 8,
            period: 30,
            window: 1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TwoFactorSetup {
    pub secret: String,
    pub otpauth_uri: String,
    pub qr_code_base64: String,
    pub backup_codes: Vec<String>,
    pub config: TotpConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TwoFactorData {
    pub secret: String,
    pub backup_codes: Vec<String>,
    pub enabled: bool,
}

/// Returned after a successful backup-code recovery.
#[derive(Debug, Serialize, Deserialize)]
pub struct RecoveryResult {
    pub new_secret: String,
    pub new_backup_codes: Vec<String>,
    pub enabled: bool,
}

pub struct TwoFactorAuth;

impl TwoFactorAuth {
    fn algorithm_name(algorithm: Algorithm) -> &'static str {
        match algorithm {
            Algorithm::SHA1 => "SHA1",
            Algorithm::SHA256 => "SHA256",
            Algorithm::SHA512 => "SHA512",
        }
    }

    fn url_encode(value: &str) -> String {
        const HEX: &[u8; 16] = b"0123456789ABCDEF";
        let mut encoded = String::new();
        for byte in value.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                    encoded.push(byte as char)
                }
                _ => {
                    encoded.push('%');
                    encoded.push(HEX[(byte >> 4) as usize] as char);
                    encoded.push(HEX[(byte & 0x0f) as usize] as char);
                }
            }
        }
        encoded
    }

    pub fn generate_otpauth_uri(
        issuer: &str,
        account: &str,
        secret: &str,
        config: &TotpConfig,
    ) -> String {
        let issuer = Self::url_encode(issuer);
        let account = Self::url_encode(account);
        format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm={}&digits={}&period={}",
            issuer,
            account,
            secret,
            issuer,
            Self::algorithm_name(config.algorithm),
            config.digits,
            config.period
        )
    }

    pub fn generate_secret() -> String {
        const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        let mut rng = thread_rng();
        let range = Uniform::from(0..BASE32_ALPHABET.len());
        (0..32)
            .map(|_| BASE32_ALPHABET[range.sample(&mut rng)] as char)
            .collect()
    }

    /// Setup 2FA with default configuration (SHA256)
    pub fn setup(user_email: &str, issuer: &str) -> Result<TwoFactorSetup, String> {
        Self::setup_with_config(user_email, issuer, TotpConfig::default())
    }

    /// Setup 2FA with custom configuration
    pub fn setup_with_config(
        user_email: &str,
        issuer: &str,
        config: TotpConfig,
    ) -> Result<TwoFactorSetup, String> {
        let secret = Self::generate_secret();
        let qr_issuer = issuer.replace(':', " ");
        let totp = TOTP::new(
            config.algorithm,
            config.digits,
            config.window,
            config.period,
            Secret::Encoded(secret.clone())
                .to_bytes()
                .map_err(|e| e.to_string())?,
            Some(qr_issuer),
            user_email.to_string(),
        )
        .map_err(|e| e.to_string())?;

        let qr_code_base64 = format!(
            "data:image/png;base64,{}",
            totp.get_qr_base64().map_err(|e| e.to_string())?
        );
        let backup_codes = Self::generate_backup_codes(8);
        let otpauth_uri = Self::generate_otpauth_uri(issuer, user_email, &secret, &config);

        Ok(TwoFactorSetup {
            secret,
            otpauth_uri,
            qr_code_base64,
            backup_codes,
            config,
        })
    }

    /// Verify token with default configuration (SHA256)
    pub fn verify_token(secret: &str, token: &str) -> Result<bool, String> {
        Self::verify_token_with_config(secret, token, TotpConfig::default())
    }

    /// Verify token with custom configuration
    pub fn verify_token_with_config(
        secret: &str,
        token: &str,
        config: TotpConfig,
    ) -> Result<bool, String> {
        let totp = TOTP::new(
            config.algorithm,
            config.digits,
            config.window,
            config.period,
            Secret::Encoded(secret.to_string())
                .to_bytes()
                .map_err(|e| e.to_string())?,
            None,
            String::new(),
        )
        .map_err(|e| e.to_string())?;

        totp.check_current(token).map_err(|e| e.to_string())
    }

    pub fn generate_backup_codes(count: usize) -> Vec<String> {
        let mut rng = thread_rng();
        let mut codes = HashSet::new();
        while codes.len() < count {
            codes.insert(format!(
                "{:04}-{:04}",
                rng.gen_range(0..10000),
                rng.gen_range(0..10000)
            ));
        }
        codes.into_iter().collect()
    }

    pub fn verify_backup_code(stored_codes: &[String], provided_code: &str) -> Option<usize> {
        stored_codes.iter().position(|code| code == provided_code)
    }

    /// Consume a backup code: removes it from the list if found and returns true.
    pub fn consume_backup_code(stored_codes: &mut Vec<String>, provided_code: &str) -> bool {
        if let Some(index) = Self::verify_backup_code(stored_codes, provided_code) {
            stored_codes.remove(index);
            true
        } else {
            false
        }
    }
}

/// Audit log entry for recovery code usage
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecoveryCodeUsageLog {
    pub id: usize,
    pub user_id: String,
    pub code_index: i32,
    pub used_at: String,
    pub ip_address: Option<String>,
}

/// Summary of a user's 2FA status for admin listings.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserTwoFactorSummary {
    pub user_id: String,
    pub enabled: bool,
    pub is_canary: bool,
}

/// Audit log entry for admin-visible 2FA events.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: usize,
    pub user_id: String,
    pub event: String,
    pub timestamp: u64,
    pub actor: String,
    pub metadata: Option<String>,
}

/// Persistence abstraction for 2FA state (kept for compatibility)
pub trait TwoFactorStore: Send + Sync {
    fn save(&self, user_id: &str, data: TwoFactorData) -> Result<(), String>;
    fn get(&self, user_id: &str) -> Result<TwoFactorData, String>;
    fn delete(&self, user_id: &str) -> Result<(), String>;
    fn update_enabled(&self, user_id: &str, enabled: bool) -> Result<(), String>;
    fn update_backup_codes(&self, user_id: &str, codes: Vec<String>) -> Result<(), String>;

    /// Check if a recovery code has been used and log the usage atomically
    /// Returns error if the code has already been used
    fn log_recovery_code_usage(
        &self,
        user_id: &str,
        code_index: i32,
        ip_address: Option<&str>,
    ) -> Result<(), String>;

    /// Get paginated recovery code usage log (page starts at 1)
    fn get_recovery_usage_log(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<RecoveryCodeUsageLog>, String>;

    // --- Admin dashboard (Issue #688) ---

    /// Paginated list of all users with their 2FA status.
    /// Canary accounts are excluded from this listing.
    fn list_users(&self, page: u32, page_size: u32) -> Result<Vec<UserTwoFactorSummary>, String>;

    /// Force-disable 2FA for a user and append an audit log entry.
    fn admin_disable_two_fa(&self, user_id: &str, admin_id: &str) -> Result<(), String>;

    /// Get the full audit log for a user (paginated, page starts at 1).
    fn get_audit_log(
        &self,
        user_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<AuditLogEntry>, String>;

    /// Append an entry to the audit log.
    fn append_audit_log(
        &self,
        user_id: &str,
        event: &str,
        actor: &str,
        metadata: Option<&str>,
    ) -> Result<(), String>;

    // --- Canary tokens (Issue #713) ---

    /// Mark a user account as a canary token account.
    fn set_canary(&self, user_id: &str, is_canary: bool) -> Result<(), String>;

    /// Check whether a user account is a canary.
    fn is_canary(&self, user_id: &str) -> bool;
}

/// In-memory implementation of TwoFactorStore for testing
#[derive(Default, Clone)]
pub struct InMemoryStore {
    data: Arc<Mutex<HashMap<String, TwoFactorData>>>,
    recovery_log: Arc<Mutex<Vec<RecoveryCodeUsageLog>>>,
    audit_log: Arc<Mutex<Vec<AuditLogEntry>>>,
    canary_flags: Arc<Mutex<HashMap<String, bool>>>,
}

impl InMemoryStore {
    pub fn clear(&self) {
        self.data.lock().unwrap().clear();
    }

    pub fn save(&self, user_id: &str, data: TwoFactorData) -> Result<(), String> {
        <Self as TwoFactorStore>::save(self, user_id, data)
    }

    pub fn get(&self, user_id: &str) -> Result<TwoFactorData, String> {
        <Self as TwoFactorStore>::get(self, user_id)
    }

    pub fn append_audit_log(
        &self,
        user_id: &str,
        event: &str,
        actor: &str,
        metadata: Option<&str>,
    ) -> Result<(), String> {
        <Self as TwoFactorStore>::append_audit_log(self, user_id, event, actor, metadata)
    }

    pub fn get_audit_log(
        &self,
        user_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<AuditLogEntry>, String> {
        <Self as TwoFactorStore>::get_audit_log(self, user_id, page, page_size)
    }

    pub fn list_users(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<UserTwoFactorSummary>, String> {
        <Self as TwoFactorStore>::list_users(self, page, page_size)
    }

    pub fn set_canary(&self, user_id: &str, is_canary: bool) -> Result<(), String> {
        <Self as TwoFactorStore>::set_canary(self, user_id, is_canary)
    }

    pub fn is_canary(&self, user_id: &str) -> bool {
        <Self as TwoFactorStore>::is_canary(self, user_id)
    }
}

impl TwoFactorStore for InMemoryStore {
    fn save(&self, user_id: &str, data: TwoFactorData) -> Result<(), String> {
        self.data.lock().unwrap().insert(user_id.to_string(), data);
        Ok(())
    }

    fn get(&self, user_id: &str) -> Result<TwoFactorData, String> {
        self.data
            .lock()
            .unwrap()
            .get(user_id)
            .cloned()
            .ok_or_else(|| format!("No 2FA data found for user: {}", user_id))
    }

    fn delete(&self, user_id: &str) -> Result<(), String> {
        self.data
            .lock()
            .unwrap()
            .remove(user_id)
            .ok_or_else(|| format!("No 2FA data found for user: {}", user_id))?;
        Ok(())
    }

    fn update_enabled(&self, user_id: &str, enabled: bool) -> Result<(), String> {
        let mut store = self.data.lock().unwrap();
        store
            .get_mut(user_id)
            .ok_or_else(|| format!("No 2FA data found for user: {}", user_id))
            .map(|d| d.enabled = enabled)
    }

    fn update_backup_codes(&self, user_id: &str, codes: Vec<String>) -> Result<(), String> {
        let mut store = self.data.lock().unwrap();
        store
            .get_mut(user_id)
            .ok_or_else(|| format!("No 2FA data found for user: {}", user_id))
            .map(|d| d.backup_codes = codes)
    }

    fn log_recovery_code_usage(
        &self,
        user_id: &str,
        code_index: i32,
        ip_address: Option<&str>,
    ) -> Result<(), String> {
        let mut log = self.recovery_log.lock().unwrap();

        // Check if already used
        if log
            .iter()
            .any(|e| e.user_id == user_id && e.code_index == code_index)
        {
            return Err("InvalidRecoveryCode".to_string());
        }

        // Get the next id before pushing
        let next_id = log.len();

        // Add entry
        log.push(RecoveryCodeUsageLog {
            id: next_id,
            user_id: user_id.to_string(),
            code_index,
            used_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_else(|_| "0".to_string()),
            ip_address: ip_address.map(|s| s.to_string()),
        });

        Ok(())
    }

    fn get_recovery_usage_log(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<RecoveryCodeUsageLog>, String> {
        let log = self.recovery_log.lock().unwrap();
        let offset = (page.saturating_sub(1) as usize) * (page_size as usize);
        let limit = page_size as usize;

        let mut entries: Vec<_> = log.iter().cloned().collect();
        entries.sort_by(|a, b| b.used_at.cmp(&a.used_at)); // Reverse chronological

        Ok(entries.into_iter().skip(offset).take(limit).collect())
    }

    fn list_users(&self, page: u32, page_size: u32) -> Result<Vec<UserTwoFactorSummary>, String> {
        let data = self.data.lock().unwrap();
        let canary_flags = self.canary_flags.lock().unwrap();
        let offset = (page.saturating_sub(1) as usize) * (page_size as usize);

        let mut summaries: Vec<UserTwoFactorSummary> = data
            .iter()
            .filter(|(uid, _)| !canary_flags.get(*uid).copied().unwrap_or(false))
            .map(|(uid, d)| UserTwoFactorSummary {
                user_id: uid.clone(),
                enabled: d.enabled,
                is_canary: false,
            })
            .collect();

        summaries.sort_by(|a, b| a.user_id.cmp(&b.user_id));

        Ok(summaries
            .into_iter()
            .skip(offset)
            .take(page_size as usize)
            .collect())
    }

    fn admin_disable_two_fa(&self, user_id: &str, admin_id: &str) -> Result<(), String> {
        self.update_enabled(user_id, false)?;
        self.append_audit_log(user_id, "admin_disabled_2fa", admin_id, None)?;
        Ok(())
    }

    fn get_audit_log(
        &self,
        user_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<AuditLogEntry>, String> {
        let log = self.audit_log.lock().unwrap();
        let offset = (page.saturating_sub(1) as usize) * (page_size as usize);

        let entries: Vec<AuditLogEntry> = log
            .iter()
            .filter(|e| e.user_id == user_id)
            .cloned()
            .collect();

        Ok(entries
            .into_iter()
            .skip(offset)
            .take(page_size as usize)
            .collect())
    }

    fn append_audit_log(
        &self,
        user_id: &str,
        event: &str,
        actor: &str,
        metadata: Option<&str>,
    ) -> Result<(), String> {
        let mut log = self.audit_log.lock().unwrap();
        let id = log.len();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        log.push(AuditLogEntry {
            id,
            user_id: user_id.to_string(),
            event: event.to_string(),
            timestamp,
            actor: actor.to_string(),
            metadata: metadata.map(|s| s.to_string()),
        });
        Ok(())
    }

    fn set_canary(&self, user_id: &str, is_canary: bool) -> Result<(), String> {
        self.canary_flags
            .lock()
            .unwrap()
            .insert(user_id.to_string(), is_canary);
        Ok(())
    }

    fn is_canary(&self, user_id: &str) -> bool {
        self.canary_flags
            .lock()
            .unwrap()
            .get(user_id)
            .copied()
            .unwrap_or(false)
    }
}

#[cfg(test)]
#[derive(Clone, Debug)]
pub enum MockStoreFailure {
    Error(String),
    Timeout,
}

#[cfg(test)]
impl MockStoreFailure {
    fn message(&self) -> String {
        match self {
            Self::Error(message) => message.clone(),
            Self::Timeout => "mock store timeout".to_string(),
        }
    }
}

#[cfg(test)]
#[derive(Clone, Debug, Default)]
pub struct MockStoreConfig {
    pub save: Option<MockStoreFailure>,
    pub get: Option<MockStoreFailure>,
    pub delete: Option<MockStoreFailure>,
    pub update_enabled: Option<MockStoreFailure>,
    pub update_backup_codes: Option<MockStoreFailure>,
    pub log_recovery_code_usage: Option<MockStoreFailure>,
}

#[cfg(test)]
#[derive(Default, Clone)]
pub struct MockTwoFactorStore {
    inner: InMemoryStore,
    config: Arc<Mutex<MockStoreConfig>>,
}

#[cfg(test)]
impl MockTwoFactorStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: MockStoreConfig) -> Self {
        Self {
            inner: InMemoryStore::default(),
            config: Arc::new(Mutex::new(config)),
        }
    }

    pub fn seed(&self, user_id: &str, data: TwoFactorData) {
        let _ = self.inner.save(user_id, data);
    }

    pub fn get_data(&self, user_id: &str) -> Option<TwoFactorData> {
        self.inner.get(user_id).ok()
    }

    fn fail(&self, failure: &Option<MockStoreFailure>) -> Result<(), String> {
        match failure {
            Some(failure) => Err(failure.message()),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
impl TwoFactorStore for MockTwoFactorStore {
    fn save(&self, user_id: &str, data: TwoFactorData) -> Result<(), String> {
        self.fail(&self.config.lock().unwrap().save)?;
        self.inner.save(user_id, data)
    }

    fn get(&self, user_id: &str) -> Result<TwoFactorData, String> {
        self.fail(&self.config.lock().unwrap().get)?;
        self.inner.get(user_id)
    }

    fn delete(&self, user_id: &str) -> Result<(), String> {
        self.fail(&self.config.lock().unwrap().delete)?;
        self.inner.delete(user_id)
    }

    fn update_enabled(&self, user_id: &str, enabled: bool) -> Result<(), String> {
        self.fail(&self.config.lock().unwrap().update_enabled)?;
        self.inner.update_enabled(user_id, enabled)
    }

    fn update_backup_codes(&self, user_id: &str, codes: Vec<String>) -> Result<(), String> {
        self.fail(&self.config.lock().unwrap().update_backup_codes)?;
        self.inner.update_backup_codes(user_id, codes)
    }

    fn log_recovery_code_usage(
        &self,
        user_id: &str,
        code_index: i32,
        ip_address: Option<&str>,
    ) -> Result<(), String> {
        self.fail(&self.config.lock().unwrap().log_recovery_code_usage)?;
        self.inner
            .log_recovery_code_usage(user_id, code_index, ip_address)
    }

    fn get_recovery_usage_log(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<RecoveryCodeUsageLog>, String> {
        self.inner.get_recovery_usage_log(page, page_size)
    }

    fn list_users(&self, page: u32, page_size: u32) -> Result<Vec<UserTwoFactorSummary>, String> {
        self.inner.list_users(page, page_size)
    }

    fn admin_disable_two_fa(&self, user_id: &str, admin_id: &str) -> Result<(), String> {
        self.inner.admin_disable_two_fa(user_id, admin_id)
    }

    fn get_audit_log(
        &self,
        user_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<AuditLogEntry>, String> {
        self.inner.get_audit_log(user_id, page, page_size)
    }

    fn append_audit_log(
        &self,
        user_id: &str,
        event: &str,
        actor: &str,
        metadata: Option<&str>,
    ) -> Result<(), String> {
        self.inner.append_audit_log(user_id, event, actor, metadata)
    }

    fn set_canary(&self, user_id: &str, is_canary: bool) -> Result<(), String> {
        self.inner.set_canary(user_id, is_canary)
    }

    fn is_canary(&self, user_id: &str) -> bool {
        self.inner.is_canary(user_id)
    }
}
