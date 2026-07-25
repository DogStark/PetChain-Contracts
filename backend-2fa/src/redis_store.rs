use redis::Commands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::two_factor::{
    AuditLogEntry, LockedUserSummary, RecoveryCodeUsageLog, TwoFactorData, TwoFactorLockoutState,
    TwoFactorStore, UserTwoFactorSummary,
};

/// Key prefix for all 2FA Redis entries.
const KEY_PREFIX: &str = "petchain:2fa";

fn user_key(user_id: &str) -> String {
    format!("{}:{}", KEY_PREFIX, user_id)
}

fn lockout_key(user_id: &str) -> String {
    format!("{}:lockout:{}", KEY_PREFIX, user_id)
}

fn canary_key(user_id: &str) -> String {
    format!("{}:canary:{}", KEY_PREFIX, user_id)
}

fn audit_key(user_id: &str) -> String {
    format!("{}:audit:{}", KEY_PREFIX, user_id)
}

fn recovery_log_key() -> String {
    format!("{}:recovery_log", KEY_PREFIX)
}

fn session_revoke_key(user_id: &str, session_id: &str) -> String {
    format!("{}:revoked:{}:{}", KEY_PREFIX, user_id, session_id)
}

fn revoke_all_key(user_id: &str) -> String {
    format!("{}:revoke_all:{}", KEY_PREFIX, user_id)
}

/// Serializable form of `TwoFactorData` for Redis hash storage.
#[derive(Serialize, Deserialize)]
struct TwoFactorDataRedis {
    secret: String,
    backup_codes: Vec<String>,
    enabled: bool,
    algorithm: String,
    last_used_step: Option<u64>,
}

impl From<TwoFactorData> for TwoFactorDataRedis {
    fn from(d: TwoFactorData) -> Self {
        Self {
            secret: d.secret,
            backup_codes: d.backup_codes,
            enabled: d.enabled,
            algorithm: format!("{:?}", d.algorithm),
            last_used_step: d.last_used_step,
        }
    }
}

impl TwoFactorDataRedis {
    fn into_data(self) -> TwoFactorData {
        let algorithm = match self.algorithm.as_str() {
            "SHA256" => totp_rs::Algorithm::SHA256,
            "SHA512" => totp_rs::Algorithm::SHA512,
            _ => totp_rs::Algorithm::SHA1,
        };
        TwoFactorData {
            secret: self.secret,
            backup_codes: self.backup_codes,
            enabled: self.enabled,
            algorithm,
            last_used_step: self.last_used_step,
        }
    }
}

/// Redis-backed implementation of [`TwoFactorStore`].
///
/// Stores 2FA data as JSON values under `petchain:2fa:{user_id}`.
/// Data persists until explicitly deleted (no TTL).
pub struct RedisTwoFactorStore {
    client: redis::Client,
}

impl RedisTwoFactorStore {
    pub fn new(redis_url: &str) -> Result<Self, String> {
        let client = redis::Client::open(redis_url).map_err(|e| e.to_string())?;
        Ok(Self { client })
    }

    fn get_connection(&self) -> Result<redis::Connection, String> {
        self.client.get_connection().map_err(|e| e.to_string())
    }
}

impl TwoFactorStore for RedisTwoFactorStore {
    fn save(&self, user_id: &str, data: TwoFactorData) -> Result<(), String> {
        let mut con = self.get_connection()?;
        let redis_data = TwoFactorDataRedis::from(data);
        let json = serde_json::to_string(&redis_data).map_err(|e| e.to_string())?;
        con.set::<_, _, ()>(&user_key(user_id), json)
            .map_err(|e| e.to_string())
    }

    fn get(&self, user_id: &str) -> Result<TwoFactorData, String> {
        let mut con = self.get_connection()?;
        let json: String = con
            .get(&user_key(user_id))
            .map_err(|e| e.to_string())?;
        let redis_data: TwoFactorDataRedis =
            serde_json::from_str(&json).map_err(|e| e.to_string())?;
        Ok(redis_data.into_data())
    }

    fn delete(&self, user_id: &str) -> Result<(), String> {
        let mut con = self.get_connection()?;
        con.del::<_, ()>(&user_key(user_id))
            .map_err(|e| e.to_string())
    }

    fn update_enabled(&self, user_id: &str, enabled: bool) -> Result<(), String> {
        let mut data = self.get(user_id)?;
        data.enabled = enabled;
        self.save(user_id, data)
    }

    fn update_backup_codes(&self, user_id: &str, codes: Vec<String>) -> Result<(), String> {
        let mut data = self.get(user_id)?;
        data.backup_codes = codes;
        self.save(user_id, data)
    }

    fn log_recovery_code_usage(
        &self,
        user_id: &str,
        code_index: i32,
        ip_address: Option<&str>,
    ) -> Result<(), String> {
        let mut con = self.get_connection()?;
        let entry = RecoveryCodeUsageLog {
            id: 0,
            user_id: user_id.to_string(),
            code_index,
            used_at: chrono_now(),
            ip_address: ip_address.map(|s| s.to_string()),
        };
        let json = serde_json::to_string(&entry).map_err(|e| e.to_string())?;
        con.rpush::<_, _, ()>(&recovery_log_key(), json)
            .map_err(|e| e.to_string())
    }

    fn get_recovery_usage_log(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<RecoveryCodeUsageLog>, String> {
        let mut con = self.get_connection()?;
        let start = ((page.saturating_sub(1)) * page_size) as isize;
        let end = (start + page_size as isize) - 1;
        let items: Vec<String> = con
            .lrange(&recovery_log_key(), start, end)
            .map_err(|e| e.to_string())?;
        items
            .into_iter()
            .enumerate()
            .map(|(i, json)| {
                let mut entry: RecoveryCodeUsageLog =
                    serde_json::from_str(&json).map_err(|e| e.to_string())?;
                entry.id = start as usize + i;
                Ok(entry)
            })
            .collect()
    }

    fn list_users(
        &self,
        _page: u32,
        _page_size: u32,
    ) -> Result<Vec<UserTwoFactorSummary>, String> {
        // Redis doesn't have a natural way to enumerate all keys efficiently.
        // Return empty for now — admin user listing should use Postgres.
        Ok(vec![])
    }

    fn admin_disable_two_fa(&self, user_id: &str, admin_id: &str) -> Result<(), String> {
        self.update_enabled(user_id, false)?;
        self.append_audit_log(user_id, "admin_disable_2fa", admin_id, None)
    }

    fn get_audit_log(
        &self,
        user_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<AuditLogEntry>, String> {
        let mut con = self.get_connection()?;
        let start = ((page.saturating_sub(1)) * page_size) as isize;
        let end = (start + page_size as isize) - 1;
        let items: Vec<String> = con
            .lrange(&audit_key(user_id), start, end)
            .map_err(|e| e.to_string())?;
        items
            .into_iter()
            .enumerate()
            .map(|(i, json)| {
                let mut entry: AuditLogEntry =
                    serde_json::from_str(&json).map_err(|e| e.to_string())?;
                entry.id = start as usize + i;
                Ok(entry)
            })
            .collect()
    }

    fn append_audit_log(
        &self,
        user_id: &str,
        event: &str,
        actor: &str,
        metadata: Option<&str>,
    ) -> Result<(), String> {
        let mut con = self.get_connection()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let entry = AuditLogEntry {
            id: 0,
            user_id: user_id.to_string(),
            event: event.to_string(),
            timestamp: now,
            actor: actor.to_string(),
            metadata: metadata.map(|s| s.to_string()),
        };
        let json = serde_json::to_string(&entry).map_err(|e| e.to_string())?;
        con.rpush::<_, _, ()>(&audit_key(user_id), json)
            .map_err(|e| e.to_string())
    }

    fn set_canary(&self, user_id: &str, is_canary: bool) -> Result<(), String> {
        let mut con = self.get_connection()?;
        con.set::<_, _, ()>(&canary_key(user_id), if is_canary { "1" } else { "0" })
            .map_err(|e| e.to_string())
    }

    fn is_canary(&self, user_id: &str) -> bool {
        self.get_connection()
            .and_then(|mut con| {
                con.get::<_, Option<String>>(&canary_key(user_id))
                    .map_err(|e| e.to_string())
            })
            .map(|v| v.as_deref() == Some("1"))
            .unwrap_or(false)
    }

    fn get_lockout_state(&self, user_id: &str) -> Result<TwoFactorLockoutState, String> {
        let mut con = self.get_connection()?;
        let json: Option<String> = con
            .get(&lockout_key(user_id))
            .map_err(|e| e.to_string())?;
        match json {
            Some(j) => serde_json::from_str(&j).map_err(|e| e.to_string()),
            None => Ok(TwoFactorLockoutState::default()),
        }
    }

    fn record_failed_two_fa_attempt(
        &self,
        user_id: &str,
        lockout_threshold: u32,
    ) -> Result<TwoFactorLockoutState, String> {
        let mut state = self.get_lockout_state(user_id)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        state.failed_attempts += 1;
        state.updated_at = now;
        if state.failed_attempts >= lockout_threshold {
            state.locked = true;
            state.locked_at = Some(now);
        }
        let mut con = self.get_connection()?;
        let json = serde_json::to_string(&state).map_err(|e| e.to_string())?;
        con.set::<_, _, ()>(&lockout_key(user_id), json)
            .map_err(|e| e.to_string())?;
        Ok(state)
    }

    fn reset_two_fa_failures(&self, user_id: &str) -> Result<(), String> {
        let mut con = self.get_connection()?;
        let state = TwoFactorLockoutState::default();
        let json = serde_json::to_string(&state).map_err(|e| e.to_string())?;
        con.set::<_, _, ()>(&lockout_key(user_id), json)
            .map_err(|e| e.to_string())
    }

    fn set_last_used_step(&self, user_id: &str, step: u64) -> Result<(), String> {
        let mut data = self.get(user_id)?;
        data.last_used_step = Some(step);
        self.save(user_id, data)
    }

    fn unlock_two_fa_account(&self, user_id: &str, actor: &str) -> Result<(), String> {
        self.reset_two_fa_failures(user_id)?;
        self.append_audit_log(user_id, "admin_unlock", actor, None)
    }

    fn list_locked_users(&self) -> Result<Vec<LockedUserSummary>, String> {
        // Redis doesn't support efficient key enumeration for this.
        // Return empty — admin listing should use Postgres.
        Ok(vec![])
    }

    fn revoke_session(&self, user_id: &str, session_id: &str) -> Result<(), String> {
        let mut con = self.get_connection()?;
        con.set::<_, _, ()>(&session_revoke_key(user_id, session_id), "1")
            .map_err(|e| e.to_string())
    }

    fn revoke_all_sessions(&self, user_id: &str) -> Result<(), String> {
        let mut con = self.get_connection()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        con.set::<_, _, ()>(&revoke_all_key(user_id), now.to_string())
            .map_err(|e| e.to_string())
    }

    fn is_session_revoked(&self, user_id: &str, session_id: &str, issued_at: u64) -> bool {
        let Ok(mut con) = self.get_connection() else {
            return false;
        };

        // Check individual session revocation
        let exists: bool = con
            .exists(&session_revoke_key(user_id, session_id))
            .unwrap_or(false);
        if exists {
            return true;
        }

        // Check blanket revocation
        let cutoff: Option<String> = con.get(&revoke_all_key(user_id)).unwrap_or(None);
        if let Some(cutoff_str) = cutoff {
            if let Ok(cutoff_ts) = cutoff_str.parse::<u64>() {
                if issued_at <= cutoff_ts {
                    return true;
                }
            }
        }

        false
    }
}

fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", now)
}

// ---------------------------------------------------------------------------
// Mock Redis backend for testing without a real Redis instance
// ---------------------------------------------------------------------------

/// In-memory mock that implements the same interface as `RedisTwoFactorStore`
/// but uses `HashMap`s instead of a real Redis connection.
pub struct MockRedisTwoFactorStore {
    data: Arc<Mutex<HashMap<String, TwoFactorData>>>,
    lockouts: Arc<Mutex<HashMap<String, TwoFactorLockoutState>>>,
    canary_flags: Arc<Mutex<HashMap<String, bool>>>,
    audit_log: Arc<Mutex<Vec<AuditLogEntry>>>,
    recovery_log: Arc<Mutex<Vec<RecoveryCodeUsageLog>>>,
    revoked_sessions: Arc<Mutex<std::collections::HashSet<String>>>,
    revoke_all_before: Arc<Mutex<HashMap<String, u64>>>,
}

impl Default for MockRedisTwoFactorStore {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            lockouts: Arc::new(Mutex::new(HashMap::new())),
            canary_flags: Arc::new(Mutex::new(HashMap::new())),
            audit_log: Arc::new(Mutex::new(Vec::new())),
            recovery_log: Arc::new(Mutex::new(Vec::new())),
            revoked_sessions: Arc::new(Mutex::new(std::collections::HashSet::new())),
            revoke_all_before: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl MockRedisTwoFactorStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TwoFactorStore for MockRedisTwoFactorStore {
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
        self.data.lock().unwrap().remove(user_id);
        Ok(())
    }

    fn update_enabled(&self, user_id: &str, enabled: bool) -> Result<(), String> {
        self.data
            .lock()
            .unwrap()
            .get_mut(user_id)
            .ok_or_else(|| "not found".to_string())
            .map(|d| d.enabled = enabled)
    }

    fn update_backup_codes(&self, user_id: &str, codes: Vec<String>) -> Result<(), String> {
        self.data
            .lock()
            .unwrap()
            .get_mut(user_id)
            .ok_or_else(|| "not found".to_string())
            .map(|d| d.backup_codes = codes)
    }

    fn log_recovery_code_usage(
        &self,
        user_id: &str,
        code_index: i32,
        ip_address: Option<&str>,
    ) -> Result<(), String> {
        let mut log = self.recovery_log.lock().unwrap();
        let entry = RecoveryCodeUsageLog {
            id: log.len(),
            user_id: user_id.to_string(),
            code_index,
            used_at: chrono_now(),
            ip_address: ip_address.map(|s| s.to_string()),
        };
        log.push(entry);
        Ok(())
    }

    fn get_recovery_usage_log(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<RecoveryCodeUsageLog>, String> {
        let log = self.recovery_log.lock().unwrap();
        let start = ((page.saturating_sub(1)) * page_size) as usize;
        let end = (start + page_size as usize).min(log.len());
        Ok(log[start..end].to_vec())
    }

    fn list_users(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<UserTwoFactorSummary>, String> {
        let data = self.data.lock().unwrap();
        let canary = self.canary_flags.lock().unwrap();
        let start = ((page.saturating_sub(1)) * page_size) as usize;
        let users: Vec<_> = data
            .iter()
            .filter(|(uid, _)| !canary.get(*uid).copied().unwrap_or(false))
            .map(|(uid, d)| UserTwoFactorSummary {
                user_id: uid.clone(),
                enabled: d.enabled,
                is_canary: false,
            })
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok(users)
    }

    fn admin_disable_two_fa(&self, user_id: &str, admin_id: &str) -> Result<(), String> {
        self.update_enabled(user_id, false)?;
        self.append_audit_log(user_id, "admin_disable_2fa", admin_id, None)
    }

    fn get_audit_log(
        &self,
        user_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<AuditLogEntry>, String> {
        let log = self.audit_log.lock().unwrap();
        let start = ((page.saturating_sub(1)) * page_size) as usize;
        let filtered: Vec<_> = log
            .iter()
            .filter(|e| e.user_id == user_id)
            .skip(start)
            .take(page_size as usize)
            .cloned()
            .collect();
        Ok(filtered)
    }

    fn append_audit_log(
        &self,
        user_id: &str,
        event: &str,
        actor: &str,
        metadata: Option<&str>,
    ) -> Result<(), String> {
        let mut log = self.audit_log.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let id = log.len();
        log.push(AuditLogEntry {
            id,
            user_id: user_id.to_string(),
            event: event.to_string(),
            timestamp: now,
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
        let state = lockouts
            .entry(user_id.to_string())
            .or_insert_with(TwoFactorLockoutState::default);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        state.failed_attempts += 1;
        state.updated_at = now;
        if state.failed_attempts >= lockout_threshold {
            state.locked = true;
            state.locked_at = Some(now);
        }
        Ok(state.clone())
    }

    fn reset_two_fa_failures(&self, user_id: &str) -> Result<(), String> {
        self.lockouts
            .lock()
            .unwrap()
            .insert(user_id.to_string(), TwoFactorLockoutState::default());
        Ok(())
    }

    fn set_last_used_step(&self, user_id: &str, step: u64) -> Result<(), String> {
        let mut data = self.get(user_id)?;
        data.last_used_step = Some(step);
        self.save(user_id, data)
    }

    fn unlock_two_fa_account(&self, user_id: &str, actor: &str) -> Result<(), String> {
        self.reset_two_fa_failures(user_id)?;
        self.append_audit_log(user_id, "admin_unlock", actor, None)
    }

    fn list_locked_users(&self) -> Result<Vec<LockedUserSummary>, String> {
        let lockouts = self.lockouts.lock().unwrap();
        let locked: Vec<_> = lockouts
            .iter()
            .filter(|(_, s)| s.locked)
            .map(|(uid, s)| LockedUserSummary {
                user_id: uid.clone(),
                failed_attempts: s.failed_attempts,
                locked_at: s.locked_at,
            })
            .collect();
        Ok(locked)
    }

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
}

// ---------------------------------------------------------------------------
// Tests using MockRedisTwoFactorStore
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use totp_rs::Algorithm;

    fn sample_data() -> TwoFactorData {
        TwoFactorData {
            secret: "JBSWY3DPEHPK3PXP".to_string(),
            backup_codes: vec!["code1".to_string(), "code2".to_string()],
            enabled: true,
            algorithm: Algorithm::SHA1,
            last_used_step: None,
        }
    }

    #[test]
    fn test_save_and_get() {
        let store = MockRedisTwoFactorStore::new();
        let data = sample_data();
        store.save("user1", data.clone()).unwrap();

        let retrieved = store.get("user1").unwrap();
        assert_eq!(retrieved.secret, data.secret);
        assert_eq!(retrieved.enabled, true);
        assert_eq!(retrieved.backup_codes.len(), 2);
    }

    #[test]
    fn test_get_nonexistent_user_fails() {
        let store = MockRedisTwoFactorStore::new();
        let result = store.get("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete() {
        let store = MockRedisTwoFactorStore::new();
        store.save("user1", sample_data()).unwrap();
        store.delete("user1").unwrap();
        assert!(store.get("user1").is_err());
    }

    #[test]
    fn test_enable_disable() {
        let store = MockRedisTwoFactorStore::new();
        store.save("user1", sample_data()).unwrap();

        store.update_enabled("user1", false).unwrap();
        assert!(!store.get("user1").unwrap().enabled);

        store.update_enabled("user1", true).unwrap();
        assert!(store.get("user1").unwrap().enabled);
    }

    #[test]
    fn test_update_backup_codes() {
        let store = MockRedisTwoFactorStore::new();
        store.save("user1", sample_data()).unwrap();

        let new_codes = vec!["new1".to_string(), "new2".to_string(), "new3".to_string()];
        store.update_backup_codes("user1", new_codes.clone()).unwrap();

        let retrieved = store.get("user1").unwrap();
        assert_eq!(retrieved.backup_codes, new_codes);
    }

    #[test]
    fn test_lockout_state() {
        let store = MockRedisTwoFactorStore::new();

        // Initially no lockout
        let state = store.get_lockout_state("user1").unwrap();
        assert_eq!(state.failed_attempts, 0);
        assert!(!state.locked);

        // Record failures
        let state = store.record_failed_two_fa_attempt("user1", 3).unwrap();
        assert_eq!(state.failed_attempts, 1);
        assert!(!state.locked);

        let state = store.record_failed_two_fa_attempt("user1", 3).unwrap();
        assert_eq!(state.failed_attempts, 2);
        assert!(!state.locked);

        // Third attempt triggers lockout
        let state = store.record_failed_two_fa_attempt("user1", 3).unwrap();
        assert_eq!(state.failed_attempts, 3);
        assert!(state.locked);
        assert!(state.locked_at.is_some());
    }

    #[test]
    fn test_reset_failures() {
        let store = MockRedisTwoFactorStore::new();
        store.record_failed_two_fa_attempt("user1", 5).unwrap();
        store.record_failed_two_fa_attempt("user1", 5).unwrap();

        store.reset_two_fa_failures("user1").unwrap();
        let state = store.get_lockout_state("user1").unwrap();
        assert_eq!(state.failed_attempts, 0);
        assert!(!state.locked);
    }

    #[test]
    fn test_canary_flag() {
        let store = MockRedisTwoFactorStore::new();
        assert!(!store.is_canary("user1"));

        store.set_canary("user1", true).unwrap();
        assert!(store.is_canary("user1"));

        store.set_canary("user1", false).unwrap();
        assert!(!store.is_canary("user1"));
    }

    #[test]
    fn test_audit_log() {
        let store = MockRedisTwoFactorStore::new();
        store
            .append_audit_log("user1", "enable_2fa", "user1", None)
            .unwrap();
        store
            .append_audit_log("user1", "verify_totp", "user1", Some("success"))
            .unwrap();

        let log = store.get_audit_log("user1", 1, 10).unwrap();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].event, "enable_2fa");
        assert_eq!(log[1].event, "verify_totp");
        assert_eq!(log[1].metadata, Some("success".to_string()));
    }

    #[test]
    fn test_session_revocation() {
        let store = MockRedisTwoFactorStore::new();
        assert!(!store.is_session_revoked("user1", "session1", 100));

        store.revoke_session("user1", "session1").unwrap();
        assert!(store.is_session_revoked("user1", "session1", 100));
        assert!(!store.is_session_revoked("user1", "session2", 100));
    }

    #[test]
    fn test_revoke_all_sessions() {
        let store = MockRedisTwoFactorStore::new();

        // Sessions issued before revoke_all should be revoked
        store.revoke_all_sessions("user1").unwrap();

        // Session issued at time 0 should be revoked (before the cutoff)
        assert!(store.is_session_revoked("user1", "old_session", 0));
    }

    #[test]
    fn test_set_last_used_step() {
        let store = MockRedisTwoFactorStore::new();
        store.save("user1", sample_data()).unwrap();

        store.set_last_used_step("user1", 12345).unwrap();
        let data = store.get("user1").unwrap();
        assert_eq!(data.last_used_step, Some(12345));
    }

    #[test]
    fn test_unlock_account() {
        let store = MockRedisTwoFactorStore::new();

        // Lock the account
        store.record_failed_two_fa_attempt("user1", 1).unwrap();
        let state = store.get_lockout_state("user1").unwrap();
        assert!(state.locked);

        // Unlock
        store.unlock_two_fa_account("user1", "admin").unwrap();
        let state = store.get_lockout_state("user1").unwrap();
        assert!(!state.locked);
        assert_eq!(state.failed_attempts, 0);

        // Audit log should have the unlock event
        let log = store.get_audit_log("user1", 1, 10).unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].event, "admin_unlock");
    }

    #[test]
    fn test_list_locked_users() {
        let store = MockRedisTwoFactorStore::new();
        store.record_failed_two_fa_attempt("user1", 1).unwrap();
        store.record_failed_two_fa_attempt("user2", 5).unwrap(); // not locked yet

        let locked = store.list_locked_users().unwrap();
        assert_eq!(locked.len(), 1);
        assert_eq!(locked[0].user_id, "user1");
    }

    #[test]
    fn test_recovery_code_log() {
        let store = MockRedisTwoFactorStore::new();
        store
            .log_recovery_code_usage("user1", 0, Some("127.0.0.1"))
            .unwrap();
        store
            .log_recovery_code_usage("user1", 1, None)
            .unwrap();

        let log = store.get_recovery_usage_log(1, 10).unwrap();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].code_index, 0);
        assert_eq!(log[0].ip_address, Some("127.0.0.1".to_string()));
        assert_eq!(log[1].code_index, 1);
    }

    #[test]
    fn test_admin_disable() {
        let store = MockRedisTwoFactorStore::new();
        store.save("user1", sample_data()).unwrap();

        store.admin_disable_two_fa("user1", "admin1").unwrap();

        assert!(!store.get("user1").unwrap().enabled);
        let log = store.get_audit_log("user1", 1, 10).unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].event, "admin_disable_2fa");
    }
}
