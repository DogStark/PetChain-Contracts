use hmac::{Hmac, Mac};
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use totp_rs::{Algorithm, Secret, TOTP};

/// HMAC algorithm used for TOTP generation and verification.
///
/// Existing enrollments that pre-date this field should be treated as SHA1
/// until the user re-enrolls with a fresh secret.
pub type HmacAlgorithm = Algorithm;

/// Configuration for TOTP parameters to ensure cryptographic agility
#[derive(Debug, Clone)]
pub struct TotpConfig {
    pub algorithm: Algorithm,
    pub digits: usize,
    pub period: u64,
    pub window: u8,
    /// Number of backup codes to generate during setup (default: 8).
    pub backup_code_count: usize,
}

impl Default for TotpConfig {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::SHA1,
            digits: 6,
            period: 30,
            window: 1,
            backup_code_count: 8,
        }
    }
}

impl TotpConfig {
    pub fn new(algorithm: Algorithm, digits: usize, period: u64, window: u8) -> Result<Self, String> {
        // Validate digits: RFC 6238 recommends 6-8 digits
        if !(6..=8).contains(&digits) {
            return Err(format!("digits must be between 6 and 8, got {}", digits));
        }
        // Validate period: must be > 0
        if period == 0 {
            return Err("period must be greater than 0".to_string());
        }
        // Validate window: reasonable bound (0-10 is sane)
        if window > 10 {
            return Err(format!("window must be <= 10, got {}", window));
        }
        Ok(Self {
            algorithm,
            digits,
            period,
            window,
            backup_code_count: 8,
        })
    }

    pub fn legacy_sha1() -> Self {
        Self {
            algorithm: Algorithm::SHA1,
            digits: 6,
            period: 30,
            window: 1,
            backup_code_count: 8,
        }
    }

    pub fn high_security() -> Self {
        Self {
            algorithm: Algorithm::SHA512,
            digits: 8,
            period: 30,
            window: 1,
            backup_code_count: 8,
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

/// A revoked session, keyed by JTI (JWT ID claim from the bearer token).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RevokedSession {
    pub session_id: String,
    pub user_id: String,
    pub revoked_at: u64,
}

#[derive(Clone, Debug)]
pub struct TwoFactorData {
    pub secret: String,
    pub backup_codes: Vec<String>,
    pub enabled: bool,
    pub algorithm: HmacAlgorithm,
    /// The last successfully-used TOTP time-step for replay protection.
    /// Once a token for a given step is accepted, repeated attempts with
    /// the same step are rejected even if the token is numerically valid.
    pub last_used_step: Option<u64>,
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

    /// Replace colons with spaces so the issuer is consistent across QR and otpauth URI.
    /// Colons in the issuer label conflict with the `otpauth://` URI format where the
    /// delimiter between issuer and account is also a colon.
    pub fn sanitize_issuer(issuer: &str) -> String {
        issuer.replace(':', " ")
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

    fn sample_crypto_rng<R: Rng + CryptoRng>(rng: &mut R) -> String {
        const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        let range = Uniform::from(0..BASE32_ALPHABET.len());
        (0..32)
            .map(|_| BASE32_ALPHABET[range.sample(rng)] as char)
            .collect()
    }

    pub fn generate_secret() -> String {
        Self::sample_crypto_rng(&mut thread_rng())
    }

    /// Setup 2FA with default configuration (SHA1).
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
        let sanitized_issuer = Self::sanitize_issuer(issuer);
        let totp = TOTP::new(
            config.algorithm,
            config.digits,
            config.window,
            config.period,
            Secret::Encoded(secret.clone())
                .to_bytes()
                .map_err(|e| e.to_string())?,
            Some(sanitized_issuer.clone()),
            user_email.to_string(),
        )
        .map_err(|e| e.to_string())?;

        let qr_code_base64 = format!(
            "data:image/png;base64,{}",
            totp.get_qr_base64().map_err(|e| e.to_string())?
        );
        let backup_codes = Self::generate_backup_codes(config.backup_code_count);
        let otpauth_uri = Self::generate_otpauth_uri(&sanitized_issuer, user_email, &secret, &config);

        Ok(TwoFactorSetup {
            secret,
            otpauth_uri,
            qr_code_base64,
            backup_codes,
            config,
        })
    }

    /// Verify token with default configuration (SHA1).
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct TwoFactorLockoutState {
    pub failed_attempts: u32,
    pub locked: bool,
    pub locked_at: Option<u64>,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LockedUserSummary {
    pub user_id: String,
    pub failed_attempts: u32,
    pub locked_at: Option<u64>,
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

    /// Persistent lockout state, used after Redis restarts.
    fn get_lockout_state(&self, user_id: &str) -> Result<TwoFactorLockoutState, String>;

    /// Increment failed 2FA attempts and persist lockout once the threshold is reached.
    fn record_failed_two_fa_attempt(
        &self,
        user_id: &str,
        lockout_threshold: u32,
    ) -> Result<TwoFactorLockoutState, String>;

    /// Reset failed attempts after a successful TOTP verification or recovery.
    fn reset_two_fa_failures(&self, user_id: &str) -> Result<(), String>;

    /// Record the last successfully-used TOTP time-step for replay protection.
    /// Returns an error if the store cannot be updated.
    fn set_last_used_step(&self, user_id: &str, step: u64) -> Result<(), String>;

    /// Admin/recovery unlock for fully locked accounts.
    fn unlock_two_fa_account(&self, user_id: &str, actor: &str) -> Result<(), String>;

    /// Return all currently locked-out user accounts.
    fn list_locked_users(&self) -> Result<Vec<LockedUserSummary>, String>;

    /// Return pool utilisation stats when the backing store supports it.
    /// Returns `None` for stores that have no connection pool (e.g. in-memory).
    fn try_pool_stats(&self) -> Option<crate::db::PoolStats> {
        None
    }

    /// Clear all recovery code usage log entries for a user.
    /// Called after successful backup-code recovery so that rotated codes are
    /// not blocked by log entries from the previous code set.
    /// Default implementation is a no-op (safe for stores that enforce
    /// replay protection differently, e.g. Postgres relies on the secret
    /// being rotated).
    fn reset_recovery_log(&self, _user_id: &str) -> Result<(), String> {
        Ok(())
    }

    /// Revoke a single session by its JTI. Idempotent — revoking an
    /// already-revoked session is not an error.
    fn revoke_session(&self, user_id: &str, session_id: &str) -> Result<(), String>;

    /// Revoke all currently-tracked sessions for a user (e.g. on 2FA disable
    /// or suspected compromise). Implementations only need to invalidate
    /// sessions they know about; for the in-memory store this means every
    /// session_id ever passed to `revoke_session` for this user, plus a
    /// "revoke everything before this timestamp" marker so that even
    /// not-yet-seen JTIs issued before now are rejected.
    fn revoke_all_sessions(&self, user_id: &str) -> Result<(), String>;

    /// Check whether a given session (JTI) has been revoked for a user.
    /// Returns true if `revoke_session` was called with this exact
    /// session_id, or if `revoke_all_sessions` was called for this user
    /// and `issued_at` predates that revocation.
    fn is_session_revoked(&self, user_id: &str, session_id: &str, issued_at: u64) -> bool;
}

/// In-memory implementation of TwoFactorStore for testing
#[derive(Default, Clone)]
pub struct InMemoryStore {
    data: Arc<Mutex<HashMap<String, TwoFactorData>>>,
    recovery_log: Arc<Mutex<Vec<RecoveryCodeUsageLog>>>,
    audit_log: Arc<Mutex<Vec<AuditLogEntry>>>,
    canary_flags: Arc<Mutex<HashMap<String, bool>>>,
    lockouts: Arc<Mutex<HashMap<String, TwoFactorLockoutState>>>,
    revoked_sessions: Arc<Mutex<HashSet<String>>>,
    revoke_all_before: Arc<Mutex<HashMap<String, u64>>>,
    /// Per-user Unix timestamp: log entries recorded before this time are
    /// ignored for replay-protection purposes (but still returned in audit queries).
    recovery_log_reset_at: Arc<Mutex<HashMap<String, u64>>>,
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

// ---------------------------------------------------------------------------
// Mock store for tests
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct MockStoreConfig {
    pub get: Option<MockStoreFailure>,
    pub save: Option<MockStoreFailure>,
}

#[derive(Clone, Debug)]
pub enum MockStoreFailure {
    Error(String),
    Timeout,
}

impl Default for MockStoreFailure {
    fn default() -> Self {
        MockStoreFailure::Error("mock failure".to_string())
    }
}

pub struct MockTwoFactorStore {
    data: Arc<Mutex<HashMap<String, TwoFactorData>>>,
    config: MockStoreConfig,
}

impl MockTwoFactorStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(cfg: MockStoreConfig) -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            config: cfg,
        }
    }

    pub fn seed(&self, user_id: &str, data: TwoFactorData) {
        self.data.lock().unwrap().insert(user_id.to_string(), data);
    }

    pub fn get_data(&self, user_id: &str) -> Result<TwoFactorData, String> {
        self.data
            .lock()
            .unwrap()
            .get(user_id)
            .cloned()
            .ok_or_else(|| "not found".to_string())
    }
}

impl Default for MockTwoFactorStore {
    fn default() -> Self {
        Self::with_config(MockStoreConfig::default())
    }
}

impl TwoFactorStore for MockTwoFactorStore {
    fn save(&self, user_id: &str, data: TwoFactorData) -> Result<(), String> {
        if let Some(MockStoreFailure::Error(msg)) = &self.config.save {
            return Err(msg.clone());
        }
        if let Some(MockStoreFailure::Timeout) = &self.config.save {
            return Err("timeout".to_string());
        }
        self.data.lock().unwrap().insert(user_id.to_string(), data);
        Ok(())
    }

    fn get(&self, user_id: &str) -> Result<TwoFactorData, String> {
        if let Some(MockStoreFailure::Error(msg)) = &self.config.get {
            return Err(msg.clone());
        }
        if let Some(MockStoreFailure::Timeout) = &self.config.get {
            return Err("timeout".to_string());
        }
        self.data
            .lock()
            .unwrap()
            .get(user_id)
            .cloned()
            .ok_or_else(|| "not found".to_string())
    }

    fn delete(&self, user_id: &str) -> Result<(), String> {
        self.data.lock().unwrap().remove(user_id);
        Ok(())
    }

    fn update_enabled(&self, user_id: &str, enabled: bool) -> Result<(), String> {
        if let Some(d) = self.data.lock().unwrap().get_mut(user_id) {
            d.enabled = enabled;
            return Ok(());
        }
        Err("not found".to_string())
    }

    fn update_backup_codes(&self, user_id: &str, codes: Vec<String>) -> Result<(), String> {
        if let Some(d) = self.data.lock().unwrap().get_mut(user_id) {
            d.backup_codes = codes;
            return Ok(());
        }
        Err("not found".to_string())
    }

    fn log_recovery_code_usage(
        &self,
        _user_id: &str,
        _code_index: i32,
        _ip_address: Option<&str>,
    ) -> Result<(), String> {
        Ok(())
    }

    fn get_recovery_usage_log(
        &self,
        _page: u32,
        _page_size: u32,
    ) -> Result<Vec<RecoveryCodeUsageLog>, String> {
        Ok(vec![])
    }

    fn list_users(&self, _page: u32, _page_size: u32) -> Result<Vec<UserTwoFactorSummary>, String> {
        Ok(vec![])
    }

    fn admin_disable_two_fa(&self, _user_id: &str, _admin_id: &str) -> Result<(), String> {
        Ok(())
    }

    fn get_audit_log(
        &self,
        _user_id: &str,
        _page: u32,
        _page_size: u32,
    ) -> Result<Vec<AuditLogEntry>, String> {
        Ok(vec![])
    }

    fn append_audit_log(
        &self,
        _user_id: &str,
        _event: &str,
        _actor: &str,
        _metadata: Option<&str>,
    ) -> Result<(), String> {
        Ok(())
    }

    fn set_canary(&self, _user_id: &str, _is_canary: bool) -> Result<(), String> {
        Ok(())
    }

    fn is_canary(&self, _user_id: &str) -> bool {
        false
    }

    fn get_lockout_state(&self, _user_id: &str) -> Result<TwoFactorLockoutState, String> {
        Ok(TwoFactorLockoutState::default())
    }

    fn record_failed_two_fa_attempt(
        &self,
        _user_id: &str,
        _lockout_threshold: u32,
    ) -> Result<TwoFactorLockoutState, String> {
        Ok(TwoFactorLockoutState::default())
    }

    fn reset_two_fa_failures(&self, _user_id: &str) -> Result<(), String> {
        Ok(())
    }

    fn set_last_used_step(&self, _user_id: &str, _step: u64) -> Result<(), String> {
        Ok(())
    }

    fn unlock_two_fa_account(&self, _user_id: &str, _actor: &str) -> Result<(), String> {
        Ok(())
    }

    fn list_locked_users(&self) -> Result<Vec<LockedUserSummary>, String> {
        Ok(vec![])
    }

    fn revoke_session(&self, _user_id: &str, _session_id: &str) -> Result<(), String> {
        Ok(())
    }

    fn revoke_all_sessions(&self, _user_id: &str) -> Result<(), String> {
        Ok(())
    }

    fn is_session_revoked(&self, _user_id: &str, _session_id: &str, _issued_at: u64) -> bool {
        false
    }
}

impl TwoFactorStore for InMemoryStore {

    fn revoke_session(&self, user_id: &str, session_id: &str) -> Result<(), String> {
        let key = format!("{}::{}", user_id, session_id);
        self.revoked_sessions.lock().unwrap().insert(key);
        Ok(())
    }

    fn revoke_all_sessions(&self, user_id: &str) -> Result<(), String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.revoke_all_before
            .lock()
            .unwrap()
            .insert(user_id.to_string(), now);
        Ok(())
    }

    fn is_session_revoked(&self, user_id: &str, session_id: &str, issued_at: u64) -> bool {
        let key = format!("{}::{}", user_id, session_id);
        if self.revoked_sessions.lock().unwrap().contains(&key) {
            return true;
        }
        if let Some(&cutoff) = self.revoke_all_before.lock().unwrap().get(user_id) {
            if issued_at <= cutoff {
                return true;
            }
        }
        false
    }
    
    fn save(&self, user_id: &str, data: TwoFactorData) -> Result<(), String> {
        self.data.lock().unwrap().insert(user_id.to_string(), data);
        Ok(())
    }

    fn reset_recovery_log(&self, user_id: &str) -> Result<(), String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.recovery_log_reset_at
            .lock()
            .unwrap()
            .insert(user_id.to_string(), now);
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
        let reset_cutoff = self
            .recovery_log_reset_at
            .lock()
            .unwrap()
            .get(user_id)
            .copied();

        let mut log = self.recovery_log.lock().unwrap();

        // Check if already used — ignore entries that predate a log reset
        // (i.e., entries from a previous backup-code generation).
        if log.iter().any(|e| {
            if e.user_id != user_id || e.code_index != code_index {
                return false;
            }
            if let Some(cutoff) = reset_cutoff {
                // Parse stored timestamp; entries at-or-before the cutoff are stale.
                let entry_ts: u64 = e.used_at.parse().unwrap_or(0);
                entry_ts > cutoff
            } else {
                true
            }
        }) {
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

    fn get_lockout_state(&self, user_id: &str) -> Result<TwoFactorLockoutState, String> {
        Ok(self
            .lockouts
            .lock()
            .unwrap()
            .get(user_id)
            .cloned()
            .unwrap_or_default())
    }

    fn record_failed_two_fa_attempt(
        &self,
        user_id: &str,
        lockout_threshold: u32,
    ) -> Result<TwoFactorLockoutState, String> {
        let mut lockouts = self.lockouts.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let state = lockouts.entry(user_id.to_string()).or_default();
        state.failed_attempts = state.failed_attempts.saturating_add(1);
        state.updated_at = now;
        if state.failed_attempts >= lockout_threshold {
            state.locked = true;
            state.locked_at = Some(now);
        }
        Ok(state.clone())
    }

    fn reset_two_fa_failures(&self, user_id: &str) -> Result<(), String> {
        self.lockouts.lock().unwrap().remove(user_id);
        Ok(())
    }

    fn set_last_used_step(&self, user_id: &str, step: u64) -> Result<(), String> {
        let mut data = self.data.lock().unwrap();
        if let Some(entry) = data.get_mut(user_id) {
            entry.last_used_step = Some(step);
        }
        Ok(())
    }

    fn unlock_two_fa_account(&self, user_id: &str, actor: &str) -> Result<(), String> {
        self.reset_two_fa_failures(user_id)?;
        self.append_audit_log(user_id, "two_fa_account_unlocked", actor, None)?;
        Ok(())
    }

    fn list_locked_users(&self) -> Result<Vec<LockedUserSummary>, String> {
        let lockouts = self.lockouts.lock().unwrap();
        let mut result: Vec<LockedUserSummary> = lockouts
            .iter()
            .filter(|(_, state)| state.locked)
            .map(|(uid, state)| LockedUserSummary {
                user_id: uid.clone(),
                failed_attempts: state.failed_attempts,
                locked_at: state.locked_at,
            })
            .collect();
        result.sort_by(|a, b| a.user_id.cmp(&b.user_id));
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Multi-tenant support
// ---------------------------------------------------------------------------

/// Per-tenant configuration: TOTP issuer name and rate-limit max failures.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantConfig {
    pub tenant_id: String,
    pub totp_issuer: String,
    pub rate_limit_max_failures: u32,
    pub lockout_threshold: u32,
}

impl TenantConfig {
    pub fn new(tenant_id: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            totp_issuer: "PetChain".to_string(),
            rate_limit_max_failures: 5,
            lockout_threshold: 10,
        }
    }
}

/// A namespaced key that scopes any store operation to a specific tenant.
/// All store methods that accept a `user_id` are prefixed with `{tenant_id}::`
/// so data is fully isolated between tenants.
#[derive(Clone)]
pub struct TenantScopedStore {
    inner: Arc<dyn TwoFactorStore>,
    pub config: TenantConfig,
}

impl std::fmt::Debug for TenantScopedStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TenantScopedStore")
            .field("config", &self.config)
            .finish()
    }
}

impl TenantScopedStore {
    pub fn new(inner: Arc<dyn TwoFactorStore>, config: TenantConfig) -> Self {
        Self { inner, config }
    }

    /// Produce a namespaced user key: `"{tenant_id}::{user_id}"`.
    fn key(&self, user_id: &str) -> String {
        format!("{}::{}", self.config.tenant_id, user_id)
    }

    pub fn save(&self, user_id: &str, data: TwoFactorData) -> Result<(), String> {
        self.inner.save(&self.key(user_id), data)
    }

    pub fn get(&self, user_id: &str) -> Result<TwoFactorData, String> {
        self.inner.get(&self.key(user_id))
    }

    pub fn delete(&self, user_id: &str) -> Result<(), String> {
        self.inner.delete(&self.key(user_id))
    }

    pub fn update_enabled(&self, user_id: &str, enabled: bool) -> Result<(), String> {
        self.inner.update_enabled(&self.key(user_id), enabled)
    }

    pub fn update_backup_codes(&self, user_id: &str, codes: Vec<String>) -> Result<(), String> {
        self.inner.update_backup_codes(&self.key(user_id), codes)
    }

    pub fn log_recovery_code_usage(
        &self,
        user_id: &str,
        code_index: i32,
        ip_address: Option<&str>,
    ) -> Result<(), String> {
        self.inner
            .log_recovery_code_usage(&self.key(user_id), code_index, ip_address)
    }

    pub fn append_audit_log(
        &self,
        user_id: &str,
        event: &str,
        actor: &str,
        metadata: Option<&str>,
    ) -> Result<(), String> {
        self.inner
            .append_audit_log(&self.key(user_id), event, actor, metadata)
    }

    pub fn get_audit_log(
        &self,
        user_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<AuditLogEntry>, String> {
        self.inner
            .get_audit_log(&self.key(user_id), page, page_size)
    }

    pub fn set_canary(&self, user_id: &str, is_canary: bool) -> Result<(), String> {
        self.inner.set_canary(&self.key(user_id), is_canary)
    }

    pub fn is_canary(&self, user_id: &str) -> bool {
        self.inner.is_canary(&self.key(user_id))
    }

    pub fn get_lockout_state(&self, user_id: &str) -> Result<TwoFactorLockoutState, String> {
        self.inner.get_lockout_state(&self.key(user_id))
    }

    pub fn record_failed_two_fa_attempt(
        &self,
        user_id: &str,
    ) -> Result<TwoFactorLockoutState, String> {
        self.inner
            .record_failed_two_fa_attempt(&self.key(user_id), self.config.lockout_threshold)
    }

    pub fn reset_two_fa_failures(&self, user_id: &str) -> Result<(), String> {
        self.inner.reset_two_fa_failures(&self.key(user_id))
    }

    pub fn set_last_used_step(&self, user_id: &str, step: u64) -> Result<(), String> {
        self.inner.set_last_used_step(&self.key(user_id), step)
    }

    pub fn unlock_two_fa_account(&self, user_id: &str, actor: &str) -> Result<(), String> {
        self.inner.unlock_two_fa_account(&self.key(user_id), actor)
    }

    /// TOTP issuer name for this tenant (used when generating QR codes).
    pub fn issuer(&self) -> &str {
        &self.config.totp_issuer
    }
}

/// Registry of tenants. Super-admin provisions tenants; all lookups are
/// scoped so cross-tenant access is structurally impossible.
#[derive(Default, Clone)]
pub struct TenantRegistry {
    tenants: Arc<Mutex<HashMap<String, TenantConfig>>>,
}

impl TenantRegistry {
    /// Provision a tenant idempotently.
    ///
    /// If `tenant_id` is unknown, it is created and `(config, false)` is
    /// returned. If it already exists, the existing config is returned
    /// unchanged along with `(existing_config, true)` — the caller can use
    /// the `bool` to signal `already_existed` without treating this as an
    /// error. The check-and-insert happens under a single lock acquisition
    /// (via `Entry`), so concurrent calls for the same `tenant_id` cannot
    /// race past each other and create duplicates.
    pub fn provision(&self, config: TenantConfig) -> Result<(TenantConfig, bool), String> {
        let mut map = self.tenants.lock().unwrap();
        match map.entry(config.tenant_id.clone()) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                Ok((entry.get().clone(), true))
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(config.clone());
                Ok((config, false))
            }
        }
    }

    /// Retrieve a scoped store for the given tenant. Returns `Err` if the
    /// tenant does not exist, preventing cross-tenant access.
    pub fn scoped_store(
        &self,
        tenant_id: &str,
        inner: Arc<dyn TwoFactorStore>,
    ) -> Result<TenantScopedStore, String> {
        let map = self.tenants.lock().unwrap();
        let config = map
            .get(tenant_id)
            .cloned()
            .ok_or_else(|| format!("Unknown tenant: {}", tenant_id))?;
        Ok(TenantScopedStore::new(inner, config))
    }

    pub fn get_config(&self, tenant_id: &str) -> Option<TenantConfig> {
        self.tenants.lock().unwrap().get(tenant_id).cloned()
    }
}

// ---------------------------------------------------------------------------
// Lightweight JWT verification (Issue #783 — leaderboard WebSocket auth)
// ---------------------------------------------------------------------------

type HmacSha256 = Hmac<Sha256>;

/// Claims extracted from a verified leaderboard-WS JWT.
#[derive(Debug, Clone, Deserialize)]
pub struct JwtClaims {
    /// Subject — the authenticated user's ID.
    pub sub: String,
    /// Expiry, in unix seconds. The token is rejected once `now >= exp`.
    pub exp: u64,
}

/// Decode a base64url (no padding) string into bytes, per RFC 4648 §5.
fn base64url_decode(input: &str) -> Result<Vec<u8>, String> {
    fn value(byte: u8) -> Option<u8> {
        match byte {
            b'A'..=b'Z' => Some(byte - b'A'),
            b'a'..=b'z' => Some(byte - b'a' + 26),
            b'0'..=b'9' => Some(byte - b'0' + 52),
            b'-' => Some(62),
            b'_' => Some(63),
            _ => None,
        }
    }

    let mut out = Vec::with_capacity(input.len() * 3 / 4 + 3);
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;

    for &byte in input.as_bytes() {
        let v = value(byte).ok_or_else(|| "invalid base64url character".to_string())?;
        buffer = (buffer << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xFF) as u8);
        }
    }

    Ok(out)
}

/// Verify an HS256-signed JWT and return its claims.
///
/// Returns `Err` if the token is malformed, uses an unsupported algorithm,
/// has an invalid signature, or is expired (`exp <= now_unix_secs`).
/// `now_unix_secs` is passed in explicitly so callers can test expiry
/// without depending on wall-clock time.
pub fn verify_jwt(token: &str, secret: &[u8], now_unix_secs: u64) -> Result<JwtClaims, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("malformed token".to_string());
    }
    let header_b64 = parts[0];
    let payload_b64 = parts[1];
    let sig_b64 = parts[2];

    let header_bytes = base64url_decode(header_b64)?;
    let header: serde_json::Value =
        serde_json::from_slice(&header_bytes).map_err(|_| "invalid header".to_string())?;
    if header.get("alg").and_then(|v| v.as_str()) != Some("HS256") {
        return Err("unsupported algorithm".to_string());
    }

    let signature = base64url_decode(sig_b64)?;
    let signing_input = format!("{header_b64}.{payload_b64}");
    let mut mac =
        HmacSha256::new_from_slice(secret).map_err(|_| "invalid secret".to_string())?;
    mac.update(signing_input.as_bytes());
    mac.verify_slice(&signature)
        .map_err(|_| "invalid signature".to_string())?;

    let payload_bytes = base64url_decode(payload_b64)?;
    let claims: JwtClaims =
        serde_json::from_slice(&payload_bytes).map_err(|_| "invalid claims".to_string())?;

    if claims.exp <= now_unix_secs {
        return Err("token expired".to_string());
    }

    Ok(claims)
}
