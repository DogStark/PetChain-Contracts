#[cfg(not(test))]
use crate::db::PostgresTwoFactorStore;
use crate::rate_limiter::{InMemoryRateLimiter, RateLimitResult, RateLimiter, RedisRateLimiter};
use crate::two_factor::{InMemoryStore, TwoFactorAuth, TwoFactorData, TwoFactorStore};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashSet;
use jsonwebtoken::{encode, decode, Header, Algorithm, EncodingKey, DecodingKey, Validation};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

pub fn generate_jwt(user_id: &str, secret: &[u8], duration_secs: u64) -> Result<String, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now as usize,
        exp: (now + duration_secs) as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| e.to_string())
}

pub fn decode_jwt(token: &str, secret: &[u8]) -> Result<Claims, String> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &validation,
    )
    .map_err(|e| e.to_string())?;
    Ok(token_data.claims)
}

pub enum RevocationStore {
    Redis { client: redis::Client },
    InMemory { revoked: Mutex<HashSet<String>> },
}

impl RevocationStore {
    pub fn new() -> Self {
        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            if let Ok(client) = redis::Client::open(redis_url) {
                if client.get_connection().is_ok() {
                    return Self::Redis { client };
                }
            }
        }
        Self::InMemory {
            revoked: Mutex::new(HashSet::new()),
        }
    }

    pub fn new_redis(client: redis::Client) -> Self {
        Self::Redis { client }
    }

    pub fn revoke(&self, token: &str, ttl_secs: u64) -> Result<(), String> {
        match self {
            Self::Redis { client } => {
                let mut conn = client.get_connection().map_err(|e| e.to_string())?;
                let key = format!("revoked:{}", token);
                let _: Result<(), _> = redis::cmd("SETEX")
                    .arg(&key)
                    .arg(ttl_secs)
                    .arg(1)
                    .query(&mut conn);
                Ok(())
            }
            Self::InMemory { revoked } => {
                let mut set = revoked.lock().map_err(|_| "Lock poisoned".to_string())?;
                set.insert(token.to_string());
                Ok(())
            }
        }
    }

    pub fn is_revoked(&self, token: &str) -> Result<bool, String> {
        match self {
            Self::Redis { client } => {
                let mut conn = client.get_connection().map_err(|e| e.to_string())?;
                let key = format!("revoked:{}", token);
                let exists: bool = redis::cmd("EXISTS")
                    .arg(&key)
                    .query(&mut conn)
                    .unwrap_or(false);
                Ok(exists)
            }
            Self::InMemory { revoked } => {
                let set = revoked.lock().map_err(|_| "Lock poisoned".to_string())?;
                Ok(set.contains(token))
            }
        }
    }
}

#[cfg(test)]
fn test_two_factor_store() -> &'static Arc<InMemoryStore> {
    static STORE: OnceLock<Arc<InMemoryStore>> = OnceLock::new();
    STORE.get_or_init(|| Arc::new(InMemoryStore::default()))
}

#[cfg(test)]
fn two_factor_store() -> Arc<Mutex<dyn TwoFactorStore>> {
    test_two_factor_store().clone()

    Arc::new(Mutex::new(InMemoryStore::default()))
}


#[cfg(not(test))]
fn two_factor_store() -> Arc<dyn TwoFactorStore> {
    static STORE: OnceLock<Arc<dyn TwoFactorStore>> = OnceLock::new();
    STORE
        .get_or_init(|| match std::env::var("DATABASE_URL") {
            Ok(database_url) => match PostgresTwoFactorStore::connect(&database_url) {
                Ok(store) => Arc::new(store),
                Err(_) => Arc::new(InMemoryStore::default()),
            },
            Err(_) => Arc::new(InMemoryStore::default()),
        })
        .clone()
}

fn store_insert(user_id: &str, data: TwoFactorData) -> Result<(), String> {
    two_factor_store().save(user_id, data)
}

fn store_get(user_id: &str) -> Result<TwoFactorData, String> {
    two_factor_store()
        .get(user_id)
        .map_err(|_| format!("2FA not configured for user {}", user_id))
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthenticatedUser {
    pub user_id: String,
}

impl AuthenticatedUser {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
        }
    }

    pub fn authorize(&self, requested_user_id: &str) -> Result<(), String> {
        if self.user_id != requested_user_id {
            return Err("Forbidden: you can only manage your own 2FA".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct EnableTwoFactorRequest {
    pub user_id: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct EnableTwoFactorResponse {
    pub secret: String,
    pub qr_code: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VerifyTwoFactorRequest {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoginWithTwoFactorRequest {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DisableTwoFactorRequest {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RecoverWithBackupRequest {
    pub user_id: String,
    pub backup_code: String,
}

#[derive(Debug, Serialize)]
pub struct RecoverWithBackupResponse {
    pub new_secret: String,
    pub new_backup_codes: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RegenerateRecoveryCodesRequest {
    pub user_id: String,
    pub token: String,
    pub ip: String,
}

#[derive(Debug, Serialize)]
pub struct RegenerateRecoveryCodesResponse {
    pub new_backup_codes: Vec<String>,
}

pub struct TwoFactorHandlers {
    limiter: Arc<dyn RateLimiter>,
    jwt_secret: Vec<u8>,
    token_duration_secs: u64,
    revocation_store: Arc<RevocationStore>,
}

impl TwoFactorHandlers {
    pub fn new() -> Self {
        Self {
            limiter: Arc::new(InMemoryRateLimiter::default()),
            jwt_secret: b"super_secret_petchain_key_12345".to_vec(),
            token_duration_secs: 1800, // 30 mins
            revocation_store: Arc::new(RevocationStore::new()),
        }
    }

    pub fn with_limiter(limiter: Arc<dyn RateLimiter>) -> Self {
        let revocation_store = if let Some(redis_limiter) = limiter.as_any().downcast_ref::<RedisRateLimiter>() {
            Arc::new(RevocationStore::new_redis(redis_limiter.get_client()))
        } else {
            Arc::new(RevocationStore::new())
        };

        Self {
            limiter,
            jwt_secret: b"super_secret_petchain_key_12345".to_vec(),
            token_duration_secs: 1800,
            revocation_store,
        }
    }

    pub fn with_jwt_config(mut self, secret: Vec<u8>, duration_secs: u64) -> Self {
        self.jwt_secret = secret;
        self.token_duration_secs = duration_secs;
        self
    }

    pub fn enable_two_factor(
        caller: &AuthenticatedUser,
        req: EnableTwoFactorRequest,
    ) -> Result<EnableTwoFactorResponse, String> {
        caller.authorize(&req.user_id)?;

        {
            let store = two_factor_store()
                .lock()
                .map_err(|_| "2FA storage lock poisoned".to_string())?;
            if let Some(existing) = store.get(&req.user_id) {
                if existing.enabled {
                    return Err(
                        "2FA is already enabled. To re-enroll, you must first disable it."
                            .to_string(),
                    );
                }
            }
        }

        let setup = TwoFactorAuth::setup(&req.email, "PetChain")?;

        store_insert(
            &req.user_id,
            TwoFactorData {
                secret: setup.secret.clone(),
                backup_codes: setup.backup_codes.clone(),
                enabled: false,
            },
        )?;

        Ok(EnableTwoFactorResponse {
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
            backup_codes: setup.backup_codes,
        })
    }

    pub fn verify_and_activate(
        &self,
        caller: &AuthenticatedUser,
        req: VerifyTwoFactorRequest,
    ) -> Result<bool, String> {
        caller.authorize(&req.user_id)?;

        let key = format!("verify:{}", req.user_id);
        if let RateLimitResult::Blocked { retry_after_secs } = self.limiter.record_failure(&key) {
            return Err(format!(
                "Too many failed attempts. Retry after {} seconds.",
                retry_after_secs
            ));
        }

        let data = store_get(&req.user_id)?;
        let result = TwoFactorAuth::verify_token(&data.secret, &req.token)?;
        if result {
            two_factor_store().update_enabled(&req.user_id, true)?;
        }

        if result {
            self.limiter.record_success(&key);
        }

        Ok(result)
    }

    pub fn verify_and_activate_jwt(
        &self,
        caller: &AuthenticatedUser,
        req: VerifyTwoFactorRequest,
    ) -> Result<String, String> {
        let is_valid = self.verify_and_activate(caller, req.clone())?;
        if is_valid {
            let token = generate_jwt(&req.user_id, &self.jwt_secret, self.token_duration_secs)?;
            Ok(token)
        } else {
            Err("Invalid 2FA token".to_string())
        }
    }

    pub fn verify_login_token(
        &self,
        caller: &AuthenticatedUser,
        req: LoginWithTwoFactorRequest,
    ) -> Result<bool, String> {
        caller.authorize(&req.user_id)?;

        let key = format!("login:{}", req.user_id);
        if let RateLimitResult::Blocked { retry_after_secs } = self.limiter.record_failure(&key) {
            return Err(format!(
                "Too many failed attempts. Retry after {} seconds.",
                retry_after_secs
            ));
        }

        let data = store_get(&req.user_id)?;
        if !data.enabled {
            return Ok(false);
        }

        let is_valid = TwoFactorAuth::verify_token(&data.secret, &req.token)?;

        if is_valid {
            self.limiter.record_success(&key);
        }

        Ok(is_valid)
    }

    pub fn verify_login_token_jwt(
        &self,
        caller: &AuthenticatedUser,
        req: LoginWithTwoFactorRequest,
    ) -> Result<String, String> {
        let is_valid = self.verify_login_token(caller, req.clone())?;
        if is_valid {
            let token = generate_jwt(&req.user_id, &self.jwt_secret, self.token_duration_secs)?;
            Ok(token)
        } else {
            Err("Invalid 2FA token".to_string())
        }
    }

    pub fn authenticate_request(&self, token: &str) -> Result<(AuthenticatedUser, String), String> {
        let claims = decode_jwt(token, &self.jwt_secret)?;

        if self.revocation_store.is_revoked(token)? {
            return Err("Token has been revoked".to_string());
        }

        // Extend expiry (sliding window) by issuing a new token
        let new_token = generate_jwt(&claims.sub, &self.jwt_secret, self.token_duration_secs)?;

        Ok((AuthenticatedUser::new(claims.sub), new_token))
    }

    pub fn logout(&self, token: &str) -> Result<(), String> {
        let claims = decode_jwt(token, &self.jwt_secret)?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as usize;

        let remaining_secs = if claims.exp > now {
            claims.exp - now
        } else {
            0
        };

        if remaining_secs > 0 {
            self.revocation_store.revoke(token, remaining_secs as u64)?;
        }

        Ok(())
    }

    pub fn disable_two_factor(
        &self,
        caller: &AuthenticatedUser,
        req: DisableTwoFactorRequest,
    ) -> Result<bool, String> {
        caller.authorize(&req.user_id)?;

        let key = format!("disable:{}", req.user_id);
        if let RateLimitResult::Blocked { retry_after_secs } = self.limiter.record_failure(&key) {
            return Err(format!(
                "Too many failed attempts. Retry after {} seconds.",
                retry_after_secs
            ));
        }

        let data = store_get(&req.user_id)?;
        if !data.enabled {
            return Ok(false);
        }

        let result = TwoFactorAuth::verify_token(&data.secret, &req.token)?;
        if result {
            two_factor_store().update_enabled(&req.user_id, false)?;
        }

        if result {
            self.limiter.record_success(&key);
        }

        Ok(result)
    }

    pub fn recover_with_backup(
        caller: &AuthenticatedUser,
        req: RecoverWithBackupRequest,
    ) -> Result<RecoverWithBackupResponse, String> {
        caller.authorize(&req.user_id)?;

        let data = store_get(&req.user_id)?;

        if !data.enabled {
            return Err("2FA not enabled for user".to_string());
        }

        let mut backup_codes = data.backup_codes.clone();
        if !TwoFactorAuth::consume_backup_code(&mut backup_codes, &req.backup_code) {
            return Err("Invalid backup code".to_string());
        }

        let setup = TwoFactorAuth::setup("recovery", "PetChain")?;

        store_insert(
            &req.user_id,
            TwoFactorData {
                secret: setup.secret.clone(),
                backup_codes: setup.backup_codes.clone(),
                enabled: true,
            },
        )?;

        Ok(RecoverWithBackupResponse {
            new_secret: setup.secret,
            new_backup_codes: setup.backup_codes,
            enabled: true,
        })
    }

    pub fn regenerate_recovery_codes(
        &self,
        caller: &AuthenticatedUser,
        req: RegenerateRecoveryCodesRequest,
    ) -> Result<RegenerateRecoveryCodesResponse, String> {
        caller.authorize(&req.user_id)?;

        let key = format!("regenerate:{}", req.user_id);
        if let RateLimitResult::Blocked { retry_after_secs } = self.limiter.record_failure(&key) {
            return Err(format!(
                "Too many failed attempts. Retry after {} seconds.",
                retry_after_secs
            ));
        }

        let data = store_get(&req.user_id)?;
        if !data.enabled {
            return Err("2FA not enabled for user".to_string());
        }

        let is_valid = TwoFactorAuth::verify_token(&data.secret, &req.token)?;
        if !is_valid {
            return Err("Invalid 2FA token".to_string());
        }

        self.limiter.record_success(&key);

        let new_backup_codes = TwoFactorAuth::generate_backup_codes(8);
        two_factor_store().update_backup_codes(&req.user_id, new_backup_codes.clone())?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();
        println!(
            "REGENERATION EVENT: user_id={}, ip={}, timestamp={}",
            req.user_id, req.ip, now
        );

        Ok(RegenerateRecoveryCodesResponse {
            new_backup_codes,
        })
    }
}

impl Default for TwoFactorHandlers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub(crate) fn get_two_factor_data_for_tests(user_id: &str) -> Option<TwoFactorData> {
    two_factor_store().get(user_id).ok()
}

#[cfg(test)]
pub(crate) fn overwrite_two_factor_data_for_tests(user_id: &str, data: TwoFactorData) {
    let _ = two_factor_store().save(user_id, data);
}

#[cfg(test)]
pub(crate) fn clear_two_factor_store_for_tests() {
    test_two_factor_store().clear();
}
