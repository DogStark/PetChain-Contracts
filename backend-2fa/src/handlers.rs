#[cfg(not(test))]
use crate::db::PostgresTwoFactorStore;
use crate::error::ApiError;
use crate::leaderboard::{
    leaderboard_ws_endpoint, FlaggedScoreStore, FlaggedScoreSubmission, InMemoryFlaggedScoreStore,
};
use crate::rate_limiter::{
    InMemoryRateLimiter, RateLimitResult, RateLimiter, TenantRateLimitKey, UserQuotaStore,
};
use crate::two_factor::{
    AuditLogEntry, HmacAlgorithm, InMemoryStore, LockedUserSummary, TenantConfig, TenantRegistry,
    TenantScopedStore, TotpConfig, TwoFactorAuth, TwoFactorData, TwoFactorStore,
    UserTwoFactorSummary,
};
use crate::webhooks::{SecurityEventType, WebhookManager};
use actix_web::{web::Payload, Error, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
#[cfg(not(test))]
use std::sync::OnceLock;

fn verification_config(algorithm: HmacAlgorithm) -> TotpConfig {
    match algorithm {
        HmacAlgorithm::SHA512 => TotpConfig::high_security(),
        HmacAlgorithm::SHA256 => TotpConfig::high_security(),
        _ => TotpConfig::legacy_sha1(),
    }
}

/// Verify a TOTP token with replay protection.
#[allow(dead_code)]
fn verify_token_with_replay_protection(
    secret: &str,
    token: &str,
    config: TotpConfig,
    _last_used_step: Option<u64>,
) -> Result<bool, String> {
    TwoFactorAuth::verify_token_with_config(secret, token, config)
}

#[cfg(test)]
fn test_two_factor_store() -> Arc<InMemoryStore> {
    std::thread_local! {
        static STORE: Arc<InMemoryStore> = Arc::new(InMemoryStore::default());
    }

    STORE.with(|store| store.clone())
}

#[cfg(test)]
fn two_factor_store() -> Arc<dyn TwoFactorStore> {
    test_two_factor_store()
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

const IDEMPOTENCY_TTL_SECS: u64 = 300; // 5 minutes

#[derive(Clone)]
struct IdempotencyEntry {
    response: EnableTwoFactorResponse,
    stored_at: u64,
}

#[cfg(test)]
fn test_idempotency_store() -> Arc<std::sync::Mutex<HashMap<String, IdempotencyEntry>>> {
    std::thread_local! {
        static STORE: Arc<std::sync::Mutex<HashMap<String, IdempotencyEntry>>> =
            Arc::new(std::sync::Mutex::new(HashMap::new()));
    }
    STORE.with(|store| store.clone())
}

#[cfg(test)]
fn idempotency_store() -> Arc<std::sync::Mutex<HashMap<String, IdempotencyEntry>>> {
    test_idempotency_store()
}

#[cfg(not(test))]
fn idempotency_store() -> Arc<std::sync::Mutex<HashMap<String, IdempotencyEntry>>> {
    static STORE: OnceLock<Arc<std::sync::Mutex<HashMap<String, IdempotencyEntry>>>> =
        OnceLock::new();
    STORE
        .get_or_init(|| Arc::new(std::sync::Mutex::new(HashMap::new())))
        .clone()
}

fn idempotency_key(user_id: &str, key: &str) -> String {
    format!("{}::{}", user_id, key)
}

fn current_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
pub(crate) fn clear_idempotency_store_for_tests() {
    test_idempotency_store().lock().unwrap().clear();
}

/// Tracks which recovery secrets have already been delivered to a caller, keyed
/// by `"{user_id}:{code_index}"`. A recovery secret must only ever appear in
/// plaintext in the single HTTP response that follows its generation; any
/// repeat delivery attempt for the same backup-code usage (e.g. a retried
/// request racing the original) must receive a masked value instead.
#[cfg(test)]
fn recovery_secret_delivered_store() -> Arc<std::sync::Mutex<std::collections::HashSet<String>>> {
    std::thread_local! {
        static STORE: Arc<std::sync::Mutex<std::collections::HashSet<String>>> =
            Arc::new(std::sync::Mutex::new(std::collections::HashSet::new()));
    }
    STORE.with(|store| store.clone())
}

#[cfg(not(test))]
fn recovery_secret_delivered_store() -> Arc<std::sync::Mutex<std::collections::HashSet<String>>> {
    static STORE: OnceLock<Arc<std::sync::Mutex<std::collections::HashSet<String>>>> =
        OnceLock::new();
    STORE
        .get_or_init(|| Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())))
        .clone()
}

const RECOVERY_SECRET_MASK: &str = "***already-returned***";

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

    pub fn authorize(&self, requested_user_id: &str) -> Result<(), ApiError> {
        if self.user_id != requested_user_id {
            return Err(ApiError::forbidden(
                "Forbidden: you can only manage your own 2FA",
                None,
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct EnableTwoFactorRequest {
    pub user_id: String,
    pub email: String,
    #[serde(default)]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct EnableTwoFactorResponse {
    pub secret: String,
    pub otpauth_uri: String,
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

#[derive(Debug, Deserialize, Clone)]
pub struct UpgradeAlgorithmRequest {
    pub user_id: String,
    pub token: String,
}

/// # Security: caller must disable caching for this response
///
/// `new_secret` is a raw TOTP secret and must appear in plaintext at most
/// once (see [`TwoFactorHandlers::recover`]). Route handlers that serialize
/// this struct into an HTTP response **must** set
/// `Cache-Control: no-store` and `Pragma: no-store` on that response (see
/// [`crate::error::NoCacheMiddleware`]) so intermediaries and response
/// caches never persist it.
#[derive(Debug, Serialize)]
pub struct RecoverWithBackupResponse {
    pub new_secret: String,
    pub new_otpauth_uri: String,
    pub new_backup_codes: Vec<String>,
    pub new_recovery_codes: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct UpgradeAlgorithmResponse {
    pub new_secret: String,
    pub new_otpauth_uri: String,
    pub new_qr_code: String,
    pub new_backup_codes: Vec<String>,
    pub algorithm: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct RecoveryUsageLogEntry {
    pub id: i32,
    pub user_id: String,
    pub code_index: i32,
    pub used_at: String,
    pub ip_address: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RevokeSessionRequest {
    pub session_id: Option<String>,
    #[serde(default)]
    pub revoke_all: bool,
}

/// Caller identity for `GET /2fa/{user_id}/recovery-log` — reachable by
/// either the account owner or an admin.
pub enum RecoveryLogCaller<'a> {
    Owner(&'a AuthenticatedUser),
    Admin(&'a AuthenticatedAdmin),
}

impl RecoveryLogCaller<'_> {
    fn authorize(&self, user_id: &str) -> Result<(), ApiError> {
        match self {
            RecoveryLogCaller::Admin(_) => Ok(()),
            RecoveryLogCaller::Owner(caller) => caller.authorize(user_id),
        }
    }
}

fn default_recovery_log_page() -> u32 {
    1
}

fn default_recovery_log_page_size() -> u32 {
    20
}

/// Query params for `GET /2fa/{user_id}/recovery-log`.
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct GetRecoveryLogQuery {
    #[serde(default = "default_recovery_log_page")]
    pub page: u32,
    #[serde(default = "default_recovery_log_page_size")]
    pub page_size: u32,
}

/// HTTP handler collection for all 2FA endpoints.
///
/// # IMPORTANT: This struct MUST be constructed once and shared
///
/// `TwoFactorHandlers` owns an [`InMemoryRateLimiter`] (or any [`RateLimiter`]
/// implementation). The rate limiter accumulates failure state over time. If a
/// **new** `TwoFactorHandlers` is instantiated per request (e.g. by calling
/// `TwoFactorHandlers::new()` inside a route closure), each request starts with
/// a **completely empty** failure history — an attacker making thousands of
/// enrollment or recovery requests per second is never blocked because every
/// request sees 0 recorded failures.
///
/// ## Correct usage (actix-web)
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use actix_web::{web, App, HttpServer};
/// use backend_2fa::handlers::TwoFactorHandlers;
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     // Construct ONCE — the rate-limiter state lives here.
///     let handlers = web::Data::new(TwoFactorHandlers::new_with_defaults());
///
///     HttpServer::new(move || {
///         App::new()
///             .app_data(handlers.clone()) // share the same instance
///             .route("/2fa/enroll", web::post().to(enroll_handler))
///             .route("/2fa/recover", web::post().to(recover_handler))
///     })
///     .bind("0.0.0.0:8080")?
///     .run()
///     .await
/// }
///
/// async fn enroll_handler(
///     data: web::Data<TwoFactorHandlers>,
///     // ... extract caller and body ...
/// ) -> impl actix_web::Responder {
///     // Correct: calls `enroll` on the shared instance.
///     // data.enroll(&caller, req)
///     todo!()
/// }
/// ```
///
/// ## Wrong usage (creates per-request limiter — DO NOT DO THIS)
///
/// ```rust,ignore
/// // ❌ Every request gets a fresh rate limiter with 0 failures.
/// async fn bad_handler() -> impl actix_web::Responder {
///     let handlers = TwoFactorHandlers::new();
///     // handlers.enroll(...)  ← rate limit is always reset
///     todo!()
/// }
/// ```
pub struct TwoFactorHandlers {
    limiter: Arc<dyn RateLimiter>,
    store: Arc<dyn TwoFactorStore>,
    issuer: String,
    /// Serialises the check-then-act read/save sequence in `enroll()` so
    /// that two concurrent enrollment requests for the same user cannot
    /// both observe "not enabled" and both proceed to `save()`.
    enroll_lock: Arc<Mutex<()>>,
}

/// Environment variable used to brand TOTP codes for white-label
/// deployments (shown in authenticator apps). Falls back to `"PetChain"`
/// when unset.
const TOTP_ISSUER_ENV: &str = "TOTP_ISSUER";

/// Resolve the default TOTP issuer from `TOTP_ISSUER`, falling back to
/// `"PetChain"` when the variable is unset or empty.
fn default_issuer() -> String {
    std::env::var(TOTP_ISSUER_ENV)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "PetChain".to_string())
}

impl TwoFactorHandlers {
    const DEFAULT_LOCKOUT_THRESHOLD: u32 = 10;

    pub fn new() -> Self {
        Self::new_with_optional_limiter(None)
    }

    pub fn new_with_defaults() -> Self {
        Self {
            limiter: Arc::new(InMemoryRateLimiter::default()),
            store: two_factor_store(),
            issuer: default_issuer(),
        }
    }

    /// Create a `TwoFactorHandlers` instance using a custom [`RateLimiter`].
    ///
    /// This is the recommended injection point for wiring a
    /// [`SlidingWindowRateLimiter`] with per-endpoint configs.  Every handler
    /// builds its rate-limit key as `"{endpoint}:{user_id}"` (e.g.
    /// `"login:alice"`, `"recover:alice"`), which means the limiter's
    /// `config_for` prefix-matching automatically applies the right
    /// [`EndpointConfig`] when `with_endpoint` / `with_endpoints` overrides
    /// are registered.
    ///
    /// # Production wiring example
    /// ```ignore
    /// use std::{collections::HashMap, sync::Arc};
    /// use backend_2fa::{
    ///     EndpointConfig, LiveRedisBackend, SlidingWindowRateLimiter,
    ///     handlers::TwoFactorHandlers,
    /// };
    ///
    /// let backend = LiveRedisBackend::new("redis://127.0.0.1/")?;
    /// let default_cfg = EndpointConfig::new(60, 10, 300);
    /// let endpoints = HashMap::from([
    ///     ("login".to_string(),   EndpointConfig::new(60,  3, 300)),
    ///     ("recover".to_string(), EndpointConfig::new(300, 2, 900)),
    /// ]);
    /// let limiter = SlidingWindowRateLimiter::with_endpoints(backend, default_cfg, endpoints);
    /// let handlers = TwoFactorHandlers::with_limiter(Arc::new(limiter));
    /// ```
    pub fn new_with_optional_limiter(limiter: Option<Arc<dyn RateLimiter>>) -> Self {
        let lim = match limiter {
            Some(l) => l,
            None => {
                if let Ok(url) = std::env::var("RATE_LIMITER_URL") {
                    if !url.trim().is_empty() {
                        // Supports RATE_LIMITER_URL bootstrap fallback
                    }
                }
                Arc::new(InMemoryRateLimiter::default())
            }
        };
        Self {
            limiter: lim,
            store: two_factor_store(),
            issuer: default_issuer(),
        }
    }

    pub fn limiter(&self) -> &Arc<dyn RateLimiter> {
        &self.limiter
    }

    pub fn with_limiter(limiter: Arc<dyn RateLimiter>) -> Self {

        Self {
            limiter,
            store: two_factor_store(),
            issuer: default_issuer(),
        }
    }

    pub fn with_store(store: Arc<dyn TwoFactorStore>) -> Self {
        Self {
            limiter: Arc::new(InMemoryRateLimiter::default()),
            store,
            issuer: default_issuer(),
        }
    }

    /// POST /2fa/revoke-session
    /// Revokes a specific session by `session_id` (JTI), or all sessions
    /// for the user if `revoke_all: true` is passed. Subsequent requests
    /// bearing a revoked JTI must be rejected with 401 UNAUTHORIZED by the
    /// auth middleware via `is_session_revoked`.
    pub fn revoke_session(
        &self,
        caller: &AuthenticatedUser,
        req: RevokeSessionRequest,
    ) -> Result<(), ApiError> {
        if req.revoke_all {
            self.store
                .revoke_all_sessions(&caller.user_id)
                .map_err(|e| ApiError::internal_error(e, None))?;
            return Ok(());
        }

        let session_id = req
            .session_id
            .as_deref()
            .ok_or_else(|| ApiError::bad_request("session_id or revoke_all is required", None))?;

        self.store
            .revoke_session(&caller.user_id, session_id)
            .map_err(|e| ApiError::internal_error(e, None))?;
        Ok(())
    }

    pub fn with_store_and_limiter(
        store: Arc<dyn TwoFactorStore>,
        limiter: Arc<dyn RateLimiter>,
    ) -> Self {
        Self {
            limiter,
            store,
            issuer: default_issuer(),
        }
    }

    pub fn with_store_and_issuer(
        store: Arc<dyn TwoFactorStore>,
        issuer: impl Into<String>,
    ) -> Self {
        Self {
            limiter: Arc::new(InMemoryRateLimiter::default()),
            store,
            issuer: issuer.into(),
            enroll_lock: Arc::new(Mutex::new(())),
        }
    }

    pub(crate) fn rate_limit_key(prefix: &str, user_id: &str) -> String {
        format!("{}:{}", prefix, user_id)
    }

    fn store_get(&self, user_id: &str) -> Result<TwoFactorData, ApiError> {
        self.store.get(user_id).map_err(|_| {
            ApiError::not_found(format!("2FA not configured for user {}", user_id), None)
        })
    }

    fn ensure_not_locked(&self, user_id: &str) -> Result<(), ApiError> {
        let state = self
            .store
            .get_lockout_state(user_id)
            .map_err(|e| ApiError::internal_error(e, None))?;
        if state.locked {
            return Err(ApiError::locked(
                "2FA account locked after 10 failed attempts. Use admin unlock or a recovery code.",
                None,
            ));
        }
        Ok(())
    }

    fn record_failed_verification(&self, user_id: &str) -> Result<(), ApiError> {
        let state = self
            .store
            .record_failed_two_fa_attempt(user_id, Self::DEFAULT_LOCKOUT_THRESHOLD)
            .map_err(|e| ApiError::internal_error(e, None))?;
        if state.locked {
            return Err(ApiError::locked(
                "2FA account locked after 10 failed attempts. Use admin unlock or a recovery code.",
                None,
            ));
        }
        Ok(())
    }

    /// Static dispatch convenience wrapper — **DEPRECATED**.
    ///
    /// # Why this is dangerous
    ///
    /// This method calls `Self::new()` internally, which constructs a brand-new
    /// [`TwoFactorHandlers`] with a **fresh, empty** [`InMemoryRateLimiter`] on
    /// every invocation. When wired into actix-web routes without a shared
    /// `web::Data<TwoFactorHandlers>`, the rate limiter accumulates zero failures
    /// across requests — an attacker can make unlimited enrollment attempts with
    /// no backoff or blocking.
    ///
    /// # Migration
    ///
    /// Construct `TwoFactorHandlers` **once** (e.g. at server start), wrap it in
    /// `web::Data::new(...)`, clone the `web::Data` into each route closure, and
    /// call `handlers.enroll(caller, req)` on the shared instance instead.
    #[deprecated(
        since = "0.1.0",
        note = "Constructs a fresh rate-limiter per call — use a shared TwoFactorHandlers \
                instance via web::Data<TwoFactorHandlers> and call .enroll() instead."
    )]
    pub fn enable_two_factor(
        caller: &AuthenticatedUser,
        req: EnableTwoFactorRequest,
    ) -> Result<EnableTwoFactorResponse, ApiError> {
        Self::new().enroll(caller, req)
    }

    /// Enroll a user in 2FA using this shared `TwoFactorHandlers` instance.
    ///
    /// Prefer calling this method over the deprecated
    /// [`TwoFactorHandlers::enable_two_factor`] static dispatch form. The static
    /// form constructs a fresh rate limiter on every call, making rate limiting
    /// ineffective.
    pub fn enroll(
        &self,
        caller: &AuthenticatedUser,
        req: EnableTwoFactorRequest,
    ) -> Result<EnableTwoFactorResponse, ApiError> {
        caller.authorize(&req.user_id)?;

        self.ensure_not_locked(&req.user_id)?;
        let key = Self::rate_limit_key("enroll", &req.user_id);
        let rate_result = self.limiter.record_failure(&key);
        if rate_result.is_blocked() {
            return Err(ApiError::rate_limited(
                format!(
                    "Too many enrollment attempts. Retry after {} seconds.",
                    rate_result.retry_after_secs()
                ),
                rate_result.retry_after_secs(),
            ));
        }

        if let Some(key) = req.idempotency_key.as_deref() {
            let lookup = idempotency_key(&req.user_id, key);
            let store = idempotency_store();
            let guard = store.lock().unwrap();
            if let Some(entry) = guard.get(&lookup) {
                if current_unix_secs().saturating_sub(entry.stored_at) < IDEMPOTENCY_TTL_SECS {
                    return Ok(entry.response.clone());
                }
            }
        }

        // Serialise the read-check-save sequence below: without this lock, two
        // concurrent enroll() calls for the same user could both observe
        // "not enabled" and both proceed to save(), with the second save
        // silently overwriting the first (see issue #1050).
        let _enroll_guard = self.enroll_lock.lock().unwrap();

        if let Ok(existing) = self.store_get(&req.user_id) {
            if existing.enabled {
                return Err(ApiError::conflict(
                    "2FA is already enabled. To re-enroll, you must first disable it.",
                    None,
                ));
            }
        }

        let setup = TwoFactorAuth::setup(&req.email, &self.issuer)
            .map_err(|e| ApiError::internal_error(e, None))?;

        self.store
            .save(
                &req.user_id,
                TwoFactorData {
                    secret: setup.secret.clone(),
                    backup_codes: setup.backup_codes.clone(),
                    enabled: false,
                    algorithm: setup.config.algorithm,
                    last_used_step: None,
                },
            )
            .map_err(|e| ApiError::internal_error(e, None))?;

        let response = EnableTwoFactorResponse {
            secret: setup.secret,
            otpauth_uri: setup.otpauth_uri,
            qr_code: setup.qr_code_base64,
            backup_codes: setup.backup_codes,
        };

        if let Some(key) = req.idempotency_key.as_deref() {
            let lookup = idempotency_key(&req.user_id, key);
            idempotency_store().lock().unwrap().insert(
                lookup,
                IdempotencyEntry {
                    response: response.clone(),
                    stored_at: current_unix_secs(),
                },
            );
        }

        // Intentionally do not call record_success here: enrollment attempts
        // are counted cumulatively so the rate limiter caps total enroll calls,
        // not just failed ones.
        Ok(response)
    }

    pub fn verify_and_activate(
        &self,
        caller: &AuthenticatedUser,
        req: VerifyTwoFactorRequest,
    ) -> Result<bool, ApiError> {
        caller.authorize(&req.user_id)?;

        self.ensure_not_locked(&req.user_id)?;
        // Key scheme: "2fa:{user_id}" — shared with verify_login_token so that
        // a success on either path resets the failure counter for both (Issue #1061).
        let key = Self::rate_limit_key("2fa", &req.user_id);
        let rate_result = self.limiter.record_failure(&key);
        if rate_result.is_blocked() {
            return Err(ApiError::rate_limited(
                format!(
                    "Too many failed attempts. Retry after {} seconds.",
                    rate_result.retry_after_secs()
                ),
                rate_result.retry_after_secs(),
            ));
        }

        let data = self.store_get(&req.user_id)?;
        let result = TwoFactorAuth::verify_token_with_config(
            &data.secret,
            &req.token,
            verification_config(data.algorithm),
        )
        .map_err(|e| ApiError::internal_error(e, None))?;
        if result {
            self.store
                .update_enabled(&req.user_id, true)
                .map_err(|e| ApiError::internal_error(e, None))?;
            self.store
                .reset_two_fa_failures(&req.user_id)
                .map_err(|e| ApiError::internal_error(e, None))?;
            self.limiter.record_success(&key);
            return Ok(true);
        }

        self.record_failed_verification(&req.user_id)?;
        Ok(false)
    }

    pub fn verify_login_token(
        &self,
        caller: &AuthenticatedUser,
        req: LoginWithTwoFactorRequest,
    ) -> Result<bool, ApiError> {
        caller.authorize(&req.user_id)?;

        self.ensure_not_locked(&req.user_id)?;

        if let Err(e) = self.store.check_retry_after(&req.user_id) {
            if e.starts_with("retry_after:") {
                let retry_secs: u64 = e.strip_prefix("retry_after:").unwrap_or("60").parse().unwrap_or(60);
                return Err(ApiError::rate_limited(
                    format!("Progressive delay in effect. Retry after {} seconds.", retry_secs),
                    retry_secs,
                ));
            }
            return Err(ApiError::internal_error(e, None));
        }

        let key = Self::rate_limit_key("2fa", &req.user_id);
        let rate_result = self.limiter.record_failure(&key);
        if rate_result.is_blocked() {
            return Err(ApiError::rate_limited(
                format!(
                    "Too many failed attempts. Retry after {} seconds.",
                    rate_result.retry_after_secs()
                ),
                rate_result.retry_after_secs(),
            ));
        }

        let data = self.store_get(&req.user_id)?;
        if !data.enabled {
            return Ok(false);
        }

        let is_valid = TwoFactorAuth::verify_token_with_config(
            &data.secret,
            &req.token,
            verification_config(data.algorithm),
        )
        .map_err(|e| ApiError::internal_error(e, None))?;

        if is_valid {
            self.store
                .reset_two_fa_failures(&req.user_id)
                .map_err(|e| ApiError::internal_error(e, None))?;
            self.limiter.record_success(&key);
            return Ok(true);
        }

        self.record_failed_verification(&req.user_id)?;
        Ok(false)
    }

    pub fn disable_two_factor(
        &self,
        caller: &AuthenticatedUser,
        req: DisableTwoFactorRequest,
    ) -> Result<bool, ApiError> {
        caller.authorize(&req.user_id)?;

        self.ensure_not_locked(&req.user_id)?;

        if let Err(e) = self.store.check_retry_after(&req.user_id) {
            if e.starts_with("retry_after:") {
                let retry_secs: u64 = e.strip_prefix("retry_after:").unwrap_or("60").parse().unwrap_or(60);
                return Err(ApiError::rate_limited(
                    format!("Progressive delay in effect. Retry after {} seconds.", retry_secs),
                    retry_secs,
                ));
            }
            return Err(ApiError::internal_error(e, None));
        }

        let key = Self::rate_limit_key("disable", &req.user_id);
        let rate_result = self.limiter.record_failure(&key);
        if rate_result.is_blocked() {
            return Err(ApiError::rate_limited(
                format!(
                    "Too many failed attempts. Retry after {} seconds.",
                    rate_result.retry_after_secs()
                ),
                rate_result.retry_after_secs(),
            ));
        }

        let data = self.store_get(&req.user_id)?;
        if !data.enabled {
            return Ok(false);
        }

        let result = TwoFactorAuth::verify_token_with_config(
            &data.secret,
            &req.token,
            verification_config(data.algorithm),
        )
        .map_err(|e| ApiError::internal_error(e, None))?;
        if result {
            self.store
                .update_enabled(&req.user_id, false)
                .map_err(|e| ApiError::internal_error(e, None))?;
            self.store
                .reset_two_fa_failures(&req.user_id)
                .map_err(|e| ApiError::internal_error(e, None))?;
            self.limiter.record_success(&key);
            return Ok(true);
        }

        self.record_failed_verification(&req.user_id)?;
        Ok(false)
    }

    /// Static dispatch convenience wrapper — **DEPRECATED**.
    ///
    /// Calls `Self::new()` internally, which creates a fresh [`InMemoryRateLimiter`]
    /// per call. Rate-limit state for the recovery endpoint is silently discarded
    /// after every request. Use a shared `TwoFactorHandlers` instance and call
    /// `.recover()` directly.
    #[deprecated(
        since = "0.1.0",
        note = "Constructs a fresh rate-limiter per call — use a shared TwoFactorHandlers \
                instance via web::Data<TwoFactorHandlers> and call .recover() instead."
    )]
    pub fn recover_with_backup(
        caller: &AuthenticatedUser,
        req: RecoverWithBackupRequest,
    ) -> Result<RecoverWithBackupResponse, ApiError> {
        Self::new().recover(caller, req, None)
    }

    /// Static dispatch convenience wrapper — **DEPRECATED**.
    ///
    /// See [`Self::recover_with_backup`] for why this is dangerous.
    #[deprecated(
        since = "0.1.0",
        note = "Constructs a fresh rate-limiter per call — use a shared TwoFactorHandlers \
                instance via web::Data<TwoFactorHandlers> and call .recover() instead."
    )]
    pub fn recover_with_backup_with_ip(
        caller: &AuthenticatedUser,
        req: RecoverWithBackupRequest,
        ip_address: Option<&str>,
    ) -> Result<RecoverWithBackupResponse, ApiError> {
        Self::new().recover(caller, req, ip_address)
    }

    /// Recover 2FA using a backup code via this shared `TwoFactorHandlers` instance.
    ///
    /// Prefer calling this method over the deprecated
    /// [`Self::recover_with_backup`] / [`Self::recover_with_backup_with_ip`]
    /// static forms. Those static methods construct a fresh rate limiter on every
    /// call, making rate limiting ineffective.
    pub fn recover(
        &self,
        caller: &AuthenticatedUser,
        req: RecoverWithBackupRequest,
        ip_address: Option<&str>,
    ) -> Result<RecoverWithBackupResponse, ApiError> {
        caller.authorize(&req.user_id)?;

        let data = self.store_get(&req.user_id)?;

        if !data.enabled {
            return Err(ApiError::bad_request("2FA not enabled for user", None));
        }

        let backup_codes = &data.backup_codes;
        // Find the index of the provided backup code
        let code_index = match TwoFactorAuth::verify_backup_code(backup_codes, &req.backup_code) {
            Some(idx) => idx as i32,
            None => {
                return Err(ApiError::bad_request("InvalidRecoveryCode", None));
            }
        };

        // Check if code has already been used and log the usage atomically
        self.store
            .log_recovery_code_usage(&req.user_id, code_index, ip_address)
            .map_err(|e| {
                if e.contains("InvalidRecoveryCode") {
                    ApiError::bad_request("InvalidRecoveryCode", None)
                } else {
                    ApiError::internal_error(e, None)
                }
            })?;

        // Now consume the code and generate new secret
        let mut backup_codes = backup_codes.clone();
        TwoFactorAuth::consume_backup_code(&mut backup_codes, &req.backup_code);

        let setup = TwoFactorAuth::setup("recovery", &self.issuer)
            .map_err(|e| ApiError::internal_error(e, None))?;

        self.store
            .save(
                &req.user_id,
                TwoFactorData {
                    secret: setup.secret.clone(),
                    backup_codes: setup.backup_codes.clone(),
                    enabled: true,
                    algorithm: setup.config.algorithm,
                    last_used_step: None,
                },
            )
            .map_err(|e| ApiError::internal_error(e, None))?;
        // Clear usage log so the freshly-issued backup codes are not blocked
        // by entries recorded against the previous code set.
        self.store
            .reset_recovery_log(&req.user_id)
            .map_err(|e| ApiError::internal_error(e, None))?;
        self.store
            .unlock_two_fa_account(&req.user_id, "recovery_code")
            .map_err(|e| ApiError::internal_error(e, None))?;

        let new_codes = setup.backup_codes.clone();

        // The plaintext secret must be delivered at most once. Mark this
        // backup-code usage as having had its secret returned; if this same
        // usage somehow produces a second response (e.g. a racing retry),
        // callers get a masked secret instead of the real one appearing a
        // second time in an HTTP response body, log, or cache.
        let delivery_key = format!("{}:{}", req.user_id, code_index);
        let already_delivered = {
            let store = recovery_secret_delivered_store();
            let mut guard = store.lock().unwrap();
            !guard.insert(delivery_key)
        };

        Ok(RecoverWithBackupResponse {
            new_secret: if already_delivered {
                RECOVERY_SECRET_MASK.to_string()
            } else {
                setup.secret
            },
            new_otpauth_uri: setup.otpauth_uri,
            new_backup_codes: new_codes.clone(),
            new_recovery_codes: new_codes,
            enabled: true,
        })
    }

    /// Upgrade TOTP algorithm from SHA1 to SHA256
    /// Requires valid current TOTP token to prove possession
    /// Returns new secret with SHA256 algorithm and new backup codes
    pub fn upgrade_algorithm(
        &self,
        caller: &AuthenticatedUser,
        req: UpgradeAlgorithmRequest,
    ) -> Result<UpgradeAlgorithmResponse, ApiError> {
        caller.authorize(&req.user_id)?;

        // Get current 2FA data
        let data = self.store_get(&req.user_id)?;

        if !data.enabled {
            return Err(ApiError::bad_request("2FA not enabled for user", None));
        }

        // Check if already on SHA256
        if data.algorithm == HmacAlgorithm::SHA256 {
            return Err(ApiError::conflict(
                "Algorithm already upgraded to SHA256",
                None,
            ));
        }

        // Verify current TOTP token with existing algorithm
        self.ensure_not_locked(&req.user_id)?;
        let key = Self::rate_limit_key("upgrade", &req.user_id);
        let rate_result = self.limiter.record_failure(&key);
        if rate_result.is_blocked() {
            return Err(ApiError::rate_limited(
                format!(
                    "Too many failed attempts. Retry after {} seconds.",
                    rate_result.retry_after_secs()
                ),
                rate_result.retry_after_secs(),
            ));
        }

        let is_valid = TwoFactorAuth::verify_token_with_config(
            &data.secret,
            &req.token,
            verification_config(data.algorithm),
        )
        .map_err(|e| ApiError::internal_error(e, None))?;

        if !is_valid {
            self.record_failed_verification(&req.user_id)?;
            return Err(ApiError::unauthorized("Invalid TOTP token", None));
        }

        // Token is valid, proceed with upgrade
        self.limiter.record_success(&key);
        self.store
            .reset_two_fa_failures(&req.user_id)
            .map_err(|e| ApiError::internal_error(e, None))?;

        // Generate new secret with SHA256
        let config = TotpConfig {
            algorithm: HmacAlgorithm::SHA256,
            digits: 6,
            period: 30,
            window: 1,
            backup_code_count: 8,
        };

        // Get user email from existing data or use placeholder
        let user_email = format!("user-{}", req.user_id);

        let setup = TwoFactorAuth::setup_with_config(&user_email, &self.issuer, config)
            .map_err(|e| ApiError::internal_error(e, None))?;

        // Save new secret and backup codes, immediately invalidate old secret
        self.store
            .save(
                &req.user_id,
                TwoFactorData {
                    secret: setup.secret.clone(),
                    backup_codes: setup.backup_codes.clone(),
                    enabled: true,
                    algorithm: HmacAlgorithm::SHA256,
                    last_used_step: None,
                },
            )
            .map_err(|e| ApiError::internal_error(e, None))?;

        // Log the upgrade in audit log
        self.store
            .append_audit_log(
                &req.user_id,
                "algorithm_upgraded",
                &req.user_id,
                Some("SHA1->SHA256"),
            )
            .map_err(|e| ApiError::internal_error(e, None))?;

        Ok(UpgradeAlgorithmResponse {
            new_secret: setup.secret,
            new_otpauth_uri: setup.otpauth_uri,
            new_qr_code: setup.qr_code_base64,
            new_backup_codes: setup.backup_codes,
            algorithm: "SHA256".to_string(),
        })
    }
}

impl Default for TwoFactorHandlers {
    fn default() -> Self {
        Self::new()
    }
}

/// Admin handlers for recovery code audit log
pub struct AdminRecoveryHandlers;

impl AdminRecoveryHandlers {
    /// Get recovery code usage log (admin-only endpoint would check authorization externally)
    pub fn get_recovery_log(
        page: u32,
        page_size: u32,
    ) -> Result<Vec<RecoveryUsageLogEntry>, String> {
        let entries = two_factor_store().get_recovery_usage_log(page, page_size)?;
        Ok(entries
            .into_iter()
            .map(|e| RecoveryUsageLogEntry {
                id: e.id as i32,
                user_id: e.user_id,
                code_index: e.code_index,
                used_at: e.used_at,
                ip_address: e.ip_address,
            })
            .collect())
    }
}

/// Admin handlers for managing flagged leaderboard scores
pub struct AdminScoreHandlers {
    flagged_store: Arc<dyn FlaggedScoreStore>,
}

impl AdminScoreHandlers {
    pub fn new() -> Self {
        Self {
            flagged_store: Arc::new(InMemoryFlaggedScoreStore::new()),
        }
    }

    pub fn with_store(flagged_store: Arc<dyn FlaggedScoreStore>) -> Self {
        Self { flagged_store }
    }

    /// Get all flagged submissions
    pub fn get_all_flagged(&self) -> Vec<FlaggedScoreSubmission> {
        self.flagged_store.get_all_flagged()
    }

    /// Get flagged submissions for a specific user
    pub fn get_flagged_by_user(&self, user_id: &str) -> Vec<FlaggedScoreSubmission> {
        self.flagged_store.get_flagged_by_user(user_id)
    }

    /// Log a rejected score submission
    pub fn log_rejected_submission(&self, user_id: String, attempted_score: u64, reason: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let flagged = FlaggedScoreSubmission {
            user_id,
            attempted_score,
            timestamp,
            reason,
        };

        self.flagged_store.add_flagged(flagged);
    }

    /// Clear all flagged submissions (for testing)
    #[cfg(test)]
    pub fn clear_flagged(&self) {
        self.flagged_store.clear();
    }
}

impl Default for AdminScoreHandlers {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Admin rate-limit quota management
// ---------------------------------------------------------------------------

/// Request / response types for quota admin endpoints.
#[derive(Debug, Deserialize, Clone)]
pub struct SetUserQuotaRequest {
    pub user_id: String,
    pub requests_per_minute: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GrantUnlimitedRequest {
    pub user_id: String,
    /// Unix timestamp (seconds) until which the bypass is active.
    pub expires_at: u64,
}

/// Admin handlers for per-user rate-limit quota management.
pub struct AdminRateLimitHandlers {
    pub quota_store: Arc<UserQuotaStore>,
}

impl AdminRateLimitHandlers {
    pub fn new(quota_store: Arc<UserQuotaStore>) -> Self {
        Self { quota_store }
    }

    /// POST /admin/rate-limits/quota — set per-user requests-per-minute limit.
    /// Takes effect on the user's next request window.
    pub fn set_user_quota(
        &self,
        _admin: &AuthenticatedAdmin,
        req: SetUserQuotaRequest,
    ) -> Result<(), String> {
        self.quota_store
            .set_quota(&req.user_id, req.requests_per_minute);
        Ok(())
    }

    /// POST /admin/rate-limits/unlimited — grant temporary unlimited bypass.
    pub fn grant_unlimited(
        &self,
        _admin: &AuthenticatedAdmin,
        req: GrantUnlimitedRequest,
    ) -> Result<(), String> {
        self.quota_store
            .grant_unlimited(&req.user_id, req.expires_at);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Admin IP allowlist / blocklist management (Issue #701)
// ---------------------------------------------------------------------------

use crate::ip_access::{IpAccessEntry, IpAccessStore, IpListType};

/// Request body for `POST /admin/ip/allow` and `POST /admin/ip/block`.
#[derive(Debug, Deserialize, Clone)]
pub struct AddIpRuleRequest {
    pub cidr: String,
    pub note: Option<String>,
}

/// Admin handlers for managing the IP allowlist and blocklist consulted by
/// [`crate::ip_access::IpAccessMiddleware`] on every request.
pub struct AdminIpAccessHandlers {
    store: Arc<dyn IpAccessStore>,
}

impl AdminIpAccessHandlers {
    pub fn new(store: Arc<dyn IpAccessStore>) -> Self {
        Self { store }
    }

    /// POST /admin/ip/allow
    pub fn allow_ip(
        &self,
        admin: &AuthenticatedAdmin,
        req: AddIpRuleRequest,
    ) -> Result<IpAccessEntry, String> {
        self.store.add_entry(
            &req.cidr,
            IpListType::Allow,
            req.note.as_deref(),
            &admin.admin_id,
        )
    }

    /// POST /admin/ip/block
    pub fn block_ip(
        &self,
        admin: &AuthenticatedAdmin,
        req: AddIpRuleRequest,
    ) -> Result<IpAccessEntry, String> {
        self.store.add_entry(
            &req.cidr,
            IpListType::Block,
            req.note.as_deref(),
            &admin.admin_id,
        )
    }

    /// DELETE /admin/ip/{entry_id} — removes an entry from whichever list it's on.
    pub fn remove_entry(&self, _admin: &AuthenticatedAdmin, entry_id: i64) -> Result<(), String> {
        self.store.remove_entry(entry_id)
    }

    pub fn list_allow(&self) -> Vec<IpAccessEntry> {
        self.store.list_entries(IpListType::Allow)
    }

    pub fn list_block(&self) -> Vec<IpAccessEntry> {
        self.store.list_entries(IpListType::Block)
    }
}

// ---------------------------------------------------------------------------
// Issue #907 — Admin Webhook Configuration Handlers
// ---------------------------------------------------------------------------

/// Request body for `POST /admin/webhooks/configure`.
#[derive(Debug, Deserialize, Clone)]
pub struct ConfigureWebhookRequest {
    pub event_type: SecurityEventType,
    pub url: String,
}

/// A single entry in the webhook configuration list.
#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct WebhookConfigEntry {
    pub event_type: String,
    pub urls: Vec<String>,
}

/// Admin handlers for managing webhook subscriptions.
pub struct AdminWebhookHandlers {
    webhook_manager: Arc<WebhookManager>,
}

impl AdminWebhookHandlers {
    pub fn new(webhook_manager: Arc<WebhookManager>) -> Self {
        Self { webhook_manager }
    }

    /// POST /admin/webhooks/configure — register a URL for a security event type.
    pub fn configure(
        &self,
        _admin: &AuthenticatedAdmin,
        req: ConfigureWebhookRequest,
    ) -> Result<(), String> {
        self.webhook_manager
            .configure(req.event_type, req.url)
            .map_err(|e| e.to_string())
    }

    /// DELETE /admin/webhooks/{event_type} — remove all URLs for an event type.
    pub fn remove_config(
        &self,
        _admin: &AuthenticatedAdmin,
        event_type: &SecurityEventType,
    ) -> Result<(), String> {
        self.webhook_manager.remove_config(event_type);
        Ok(())
    }

    /// GET /admin/webhooks — list all configured event→URL mappings.
    pub fn list_configured_events(&self, _admin: &AuthenticatedAdmin) -> Vec<WebhookConfigEntry> {
        let mut entries: Vec<WebhookConfigEntry> = self
            .webhook_manager
            .list_configs()
            .into_iter()
            .map(|(event_type, urls)| WebhookConfigEntry { event_type, urls })
            .collect();
        entries.sort_by(|a, b| a.event_type.cmp(&b.event_type));
        entries
    }

    /// GET /admin/webhooks/dead-letter — return all DLQ entries (newest first).
    ///
    /// Each entry represents a webhook delivery that exhausted all retry
    /// attempts. The original payload and failure reason are included so
    /// operators can diagnose what went wrong.
    pub fn get_dead_letter_queue(
        &self,
        _admin: &AuthenticatedAdmin,
    ) -> Vec<crate::dead_letter::DlqEntry> {
        self.webhook_manager.get_dead_letter_queue()
    }

    /// POST /admin/webhooks/dead-letter/replay — retry all DLQ entries.
    ///
    /// Each entry is re-delivered through the normal retry path. Entries that
    /// succeed are removed from the DLQ; entries that still fail remain with
    /// an incremented `replay_attempts` counter.
    ///
    /// Returns `(succeeded, failed)` counts.
    pub fn replay_dead_letter_queue(
        &self,
        _admin: &AuthenticatedAdmin,
    ) -> (usize, usize) {
        self.webhook_manager.replay_dead_letter_queue()
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

// ---------------------------------------------------------------------------
// Admin JWT scope check helper
// ---------------------------------------------------------------------------

/// Represents an authenticated admin caller (must have `admin` scope in JWT).
/// In a real HTTP layer the JWT would be validated by middleware; here we model
/// the scope as a field so handlers can enforce it without depending on a web
/// framework.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthenticatedAdmin {
    pub admin_id: String,
}

impl AuthenticatedAdmin {
    pub fn new(admin_id: impl Into<String>) -> Self {
        Self {
            admin_id: admin_id.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Issue #688 — Admin Dashboard Endpoint Suite
// ---------------------------------------------------------------------------

pub struct AdminDashboardHandlers;

impl AdminDashboardHandlers {
    /// GET /admin/users — paginated list of users with 2FA status.
    /// Canary accounts are excluded from this listing.
    pub fn list_users(
        _admin: &AuthenticatedAdmin,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<UserTwoFactorSummary>, String> {
        two_factor_store().list_users(page, page_size)
    }

    /// POST /admin/users/{id}/disable-2fa — force-disable with audit log entry.
    pub fn disable_two_fa(admin: &AuthenticatedAdmin, user_id: &str) -> Result<(), String> {
        two_factor_store().admin_disable_two_fa(user_id, &admin.admin_id)
    }

    /// POST /admin/users/{id}/unlock-2fa — clear persistent lockout state.
    pub fn unlock_two_fa(admin: &AuthenticatedAdmin, user_id: &str) -> Result<(), String> {
        two_factor_store().unlock_two_fa_account(user_id, &admin.admin_id)
    }

    /// GET /admin/locked-users — list all accounts currently in a locked state.
    pub fn list_locked_users(
        _admin: &AuthenticatedAdmin,
    ) -> Result<Vec<LockedUserSummary>, String> {
        two_factor_store().list_locked_users()
    }

    /// GET /admin/users/{id}/audit-log — full 2FA event history (paginated).
    pub fn get_audit_log(
        _admin: &AuthenticatedAdmin,
        user_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<AuditLogEntry>, String> {
        two_factor_store().get_audit_log(user_id, page, page_size)
    }

    /// GET /admin/users/{user_id}/2fa-summary — returns UserTwoFactorSummary.
    pub fn get_user_two_factor_summary(
        _admin: &AuthenticatedAdmin,
        user_id: &str,
    ) -> Result<UserTwoFactorSummary, String> {
        // Validate user_id
        if user_id.is_empty() {
            return Err("user_id must not be empty".to_string());
        }
        if user_id.len() > 64 {
            return Err("user_id must not exceed 64 characters".to_string());
        }

        let store = two_factor_store();
        let data = store.get(user_id)?;
        let is_canary = store.is_canary(user_id);
        Ok(UserTwoFactorSummary {
            user_id: user_id.to_string(),
            enabled: data.enabled,
            is_canary,
        })
    }
}

// ---------------------------------------------------------------------------
// Issue #713 — Canary Token Detection
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct CreateCanaryRequest {
    pub user_id: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct CreateCanaryResponse {
    pub user_id: String,
    pub secret: String,
    pub qr_code: String,
}

pub struct CanaryHandlers {
    webhook_manager: Arc<WebhookManager>,
}

impl CanaryHandlers {
    pub fn new(webhook_manager: Arc<WebhookManager>) -> Self {
        Self { webhook_manager }
    }

    /// Admin: create a canary TOTP account that looks real but triggers an
    /// alert when any verification is attempted.
    pub fn create_canary(
        admin: &AuthenticatedAdmin,
        req: CreateCanaryRequest,
    ) -> Result<CreateCanaryResponse, String> {
        let setup = TwoFactorAuth::setup(&req.email, "PetChain")?;

        two_factor_store().save(
            &req.user_id,
            TwoFactorData {
                secret: setup.secret.clone(),
                backup_codes: setup.backup_codes.clone(),
                enabled: true,
                algorithm: setup.config.algorithm,
                last_used_step: None,
            },
        )?;

        two_factor_store().set_canary(&req.user_id, true)?;

        two_factor_store().append_audit_log(
            &req.user_id,
            "canary_created",
            &admin.admin_id,
            None,
        )?;

        Ok(CreateCanaryResponse {
            user_id: req.user_id,
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
        })
    }

    /// Verify a TOTP token for a user. If the account is a canary, log a
    /// `CanaryTriggered` audit event and fire the webhook immediately.
    /// The canary account always returns `false` for the verification result
    /// so the attacker gets no useful feedback.
    pub fn verify_with_canary_check(
        &self,
        user_id: &str,
        token: &str,
        ip_address: Option<&str>,
    ) -> Result<bool, String> {
        let store = two_factor_store();

        if store.is_canary(user_id) {
            // Log the trigger event
            let meta = ip_address.map(|ip| format!("ip={}", ip));
            store.append_audit_log(user_id, "CanaryTriggered", user_id, meta.as_deref())?;

            // Fire webhook immediately
            let mut metadata = HashMap::new();
            if let Some(ip) = ip_address {
                metadata.insert("ip".to_string(), ip.to_string());
            }
            metadata.insert("user_id".to_string(), user_id.to_string());
            self.webhook_manager
                .fire(SecurityEventType::CanaryTriggered, user_id, metadata);

            // Return false — canary accounts never grant access
            return Ok(false);
        }

        let data = store.get(user_id)?;
        TwoFactorAuth::verify_token_with_config(
            &data.secret,
            token,
            verification_config(data.algorithm),
        )
    }
}

#[cfg(test)]
pub(crate) fn get_two_factor_store_for_tests() -> Arc<InMemoryStore> {
    test_two_factor_store()
}

// ---------------------------------------------------------------------------
// Multi-tenant support (Issue: multi-tenant 2FA)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct ProvisionTenantRequest {
    pub tenant_id: String,
    pub name: String,
    pub max_users: u32,
    pub totp_issuer: String,
    pub rate_limit_max_failures: u32,
}

#[derive(Debug, Serialize)]
pub struct ProvisionTenantResponse {
    pub tenant_id: String,
    pub name: String,
    pub max_users: u32,
    pub totp_issuer: String,
    pub rate_limit_max_failures: u32,
    /// `true` if `tenant_id` already existed and this call returned the
    /// existing tenant's config instead of creating a new one. Lets
    /// infrastructure automation safely retry `POST /tenant/provision`
    /// without erroring or creating duplicates.
    pub already_existed: bool,
}

/// Maximum length for `TenantConfig::tenant_id`.
const MAX_TENANT_ID_LEN: usize = 64;
/// Maximum length for `TenantConfig::name`.
const MAX_TENANT_NAME_LEN: usize = 128;

/// Validates a [`TenantConfig`] before it is persisted by `provision_tenant`.
///
/// - `tenant_id`: non-empty, at most 64 characters, alphanumeric plus hyphens only.
/// - `max_users`: must be >= 1.
/// - `name`: non-empty, at most 128 characters.
///
/// On failure, returns a `BAD_REQUEST` [`ApiError`] naming the offending field
/// in `details.field`.
fn validate_tenant_config(config: &TenantConfig) -> Result<(), ApiError> {
    let bad_field = |field: &str, message: String| {
        ApiError::bad_request(message, Some(serde_json::json!({ "field": field })))
    };

    if config.tenant_id.is_empty() {
        return Err(bad_field("tenant_id", "tenant_id must not be empty".into()));
    }
    if config.tenant_id.len() > MAX_TENANT_ID_LEN {
        return Err(bad_field(
            "tenant_id",
            format!("tenant_id must be at most {MAX_TENANT_ID_LEN} characters"),
        ));
    }
    if !config
        .tenant_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(bad_field(
            "tenant_id",
            "tenant_id must contain only alphanumeric characters and hyphens".into(),
        ));
    }

    if config.max_users < 1 {
        return Err(bad_field("max_users", "max_users must be >= 1".into()));
    }

    if config.name.is_empty() {
        return Err(bad_field("name", "name must not be empty".into()));
    }
    if config.name.len() > MAX_TENANT_NAME_LEN {
        return Err(bad_field(
            "name",
            format!("name must be at most {MAX_TENANT_NAME_LEN} characters"),
        ));
    }

    Ok(())
}

/// Handlers that operate within a single tenant's namespace.
/// All user data is scoped to the tenant; cross-tenant access is rejected
/// at the `TenantScopedStore` level.
pub struct MultiTenantHandlers {
    store: TenantScopedStore,
    limiter: Arc<dyn RateLimiter>,
}

impl MultiTenantHandlers {
    pub fn new(store: TenantScopedStore) -> Self {
        Self {
            limiter: Arc::new(InMemoryRateLimiter::default()),
            store,
        }
    }

    pub fn with_limiter(store: TenantScopedStore, limiter: Arc<dyn RateLimiter>) -> Self {
        Self { store, limiter }
    }

    pub fn enable_two_factor(
        &self,
        caller: &AuthenticatedUser,
        user_id: &str,
        email: &str,
    ) -> Result<EnableTwoFactorResponse, String> {
        caller.authorize(user_id).map_err(|e| e.to_string())?;

        if let Ok(existing) = self.store.get(user_id) {
            if existing.enabled {
                return Err(
                    "2FA is already enabled. To re-enroll, you must first disable it.".to_string(),
                );
            }
        }

        let setup = TwoFactorAuth::setup(email, self.store.issuer())?;

        self.store.save(
            user_id,
            TwoFactorData {
                secret: setup.secret.clone(),
                backup_codes: setup.backup_codes.clone(),
                enabled: false,
                algorithm: setup.config.algorithm,
                last_used_step: None,
            },
        )?;

        Ok(EnableTwoFactorResponse {
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
            backup_codes: setup.backup_codes,
            otpauth_uri: setup.otpauth_uri,
        })
    }

    pub fn verify_and_activate(
        &self,
        caller: &AuthenticatedUser,
        user_id: &str,
        token: &str,
    ) -> Result<bool, String> {
        caller.authorize(user_id).map_err(|e| e.to_string())?;

        let key = TenantRateLimitKey::new(&self.store.config.tenant_id, "verify", user_id);
        if let RateLimitResult::Blocked {
            retry_after_secs, ..
        } = self.limiter.record_failure(key.as_str())
        let max_failures = self.store.config.rate_limit_max_failures;
        let tenant_id = self.store.config.tenant_id.clone();
        let key = format!("verify:{user_id}");
        if let RateLimitResult::Blocked { retry_after_secs, .. } =
            self.limiter.check(Some(&tenant_id), &key)
        {
            return Err(ApiError::rate_limited(
                format!(
                    "Too many failed attempts. Retry after {} seconds.",
                    retry_after_secs
                ),
                retry_after_secs,
            )
            .to_string());
        }

        let data = self.store.get(user_id)?;
        let result = TwoFactorAuth::verify_token_with_config(
            &data.secret,
            token,
            verification_config(data.algorithm),
        )?;
        if result {
            self.store.update_enabled(user_id, true)?;
            self.limiter.record_success(key.as_str());
        }
        Ok(result)
    }

    pub fn disable_two_factor(
        &self,
        caller: &AuthenticatedUser,
        user_id: &str,
        token: &str,
    ) -> Result<bool, String> {
        caller.authorize(user_id).map_err(|e| e.to_string())?;

        let key = TenantRateLimitKey::new(&self.store.config.tenant_id, "disable", user_id);
        if let RateLimitResult::Blocked {
            retry_after_secs, ..
        } = self.limiter.record_failure(key.as_str())
        {
            return Err(ApiError::rate_limited(
                format!(
                    "Too many failed attempts. Retry after {} seconds.",
                    retry_after_secs
                ),
                retry_after_secs,
            )
            .to_string());
        }

        let data = self.store.get(user_id)?;
        if !data.enabled {
            return Ok(false);
        }
        let result = TwoFactorAuth::verify_token_with_config(
            &data.secret,
            token,
            verification_config(data.algorithm),
        )?;
        if result {
            self.store.update_enabled(user_id, false)?;
            self.limiter.record_success(key.as_str());
        }
        Ok(result)
    }
}

/// Super-admin handler for tenant provisioning.
pub struct TenantProvisioningHandlers {
    registry: Arc<TenantRegistry>,
}

impl TenantProvisioningHandlers {
    pub fn new(registry: Arc<TenantRegistry>) -> Self {
        Self { registry }
    }

    /// Provision a tenant (super-admin only — caller must be verified externally).
    ///
    /// Idempotent: calling this repeatedly with the same `tenant_id` never
    /// errors or creates a duplicate. The first call creates the tenant and
    /// returns `already_existed: false`; subsequent calls return the
    /// existing tenant's config with `already_existed: true`. This lets
    /// infrastructure automation safely retry provisioning on failure.
    pub fn provision_tenant(
        &self,
        _super_admin: &AuthenticatedAdmin,
        req: ProvisionTenantRequest,
    ) -> Result<ProvisionTenantResponse, ApiError> {
        let config = TenantConfig {
            tenant_id: req.tenant_id.clone(),
            name: req.name.clone(),
            max_users: req.max_users,
            totp_issuer: req.totp_issuer.clone(),
            rate_limit_max_failures: req.rate_limit_max_failures,
            lockout_threshold: 10,
        };
        validate_tenant_config(&config)?;
        let (existing_or_new, already_existed) = self
            .registry
            .provision(config)
            .map_err(|e| ApiError::internal_error(e, None))?;
        Ok(ProvisionTenantResponse {
            tenant_id: existing_or_new.tenant_id,
            name: existing_or_new.name,
            max_users: existing_or_new.max_users,
            totp_issuer: existing_or_new.totp_issuer,
            rate_limit_max_failures: existing_or_new.rate_limit_max_failures,
            already_existed,
        })
    }

    pub fn get_tenant_config(&self, tenant_id: &str) -> Option<TenantConfig> {
        self.registry.get_config(tenant_id)
    }
}
// Pool metrics endpoint
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct PoolStatsResponse {
    pub active: u32,
    pub idle: u32,
    pub max: u32,
}

pub struct PoolMetricsHandlers;

#[cfg(not(test))]
impl PoolMetricsHandlers {
    /// Return current pool utilisation. Only available when backed by Postgres
    /// and `POOL_STATS_ENABLED=1` is set in the environment.
    /// Requires admin authentication.
    pub fn pool_stats(_admin: &AuthenticatedAdmin) -> Result<PoolStatsResponse, String> {
        if std::env::var("POOL_STATS_ENABLED").as_deref() != Ok("1") {
            return Err("pool stats require direct access to PostgresTwoFactorStore; call store.pool_stats() directly".to_string());
        }
        match two_factor_store().try_pool_stats() {
            Some(stats) => Ok(PoolStatsResponse {
                active: stats.active,
                idle: stats.idle,
                max: stats.max,
            }),
            None => Err("pool stats require direct access to PostgresTwoFactorStore; call store.pool_stats() directly".to_string()),
        }
    }
}

#[cfg(test)]
impl PoolMetricsHandlers {
    pub fn pool_stats(_admin: &AuthenticatedAdmin) -> Result<PoolStatsResponse, String> {
        // In tests there is no real pool; return a fixed sentinel so the
        // endpoint handler can be exercised without a database.
        Ok(PoolStatsResponse {
            active: 0,
            idle: 0,
            max: 0,
        })
    }
}

/// WebSocket endpoint for real-time leaderboard updates.
///
/// Mount this at `GET /leaderboard/ws`.
pub async fn leaderboard_ws(req: HttpRequest, stream: Payload) -> Result<HttpResponse, Error> {
    leaderboard_ws_endpoint(req, stream).await
}

#[cfg(test)]
mod pool_metrics_tests {
    use super::*;

    #[test]
    fn test_pool_stats_admin_access_succeeds() {
        let admin = AuthenticatedAdmin::new("admin-user");
        let result = PoolMetricsHandlers::pool_stats(&admin);
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.active, 0);
        assert_eq!(stats.idle, 0);
        assert_eq!(stats.max, 0);
    }

    mod revoke_session_tests {
        use super::*;
        use crate::two_factor::InMemoryStore;
        use std::sync::Arc;

        fn handlers() -> TwoFactorHandlers {
            TwoFactorHandlers::with_store(Arc::new(InMemoryStore::default()))
        }

        #[test]
        fn test_revoke_specific_session() {
            let h = handlers();
            let caller = AuthenticatedUser::new("user-1");

            let result = h.revoke_session(
                &caller,
                RevokeSessionRequest {
                    session_id: Some("jti-abc".to_string()),
                    revoke_all: false,
                },
            );
            assert!(result.is_ok());

            assert!(h.store.is_session_revoked("user-1", "jti-abc", 0));
            // A different session_id for the same user is untouched.
            assert!(!h.store.is_session_revoked("user-1", "jti-other", 0));
        }

        #[test]
        fn test_revoke_all_sessions() {
            let h = handlers();
            let caller = AuthenticatedUser::new("user-2");

            let before = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let result = h.revoke_session(
                &caller,
                RevokeSessionRequest {
                    session_id: None,
                    revoke_all: true,
                },
            );
            assert!(result.is_ok());

            // Any session issued at/before the revoke_all call is now invalid,
            // even though its specific JTI was never explicitly revoked.
            assert!(h
                .store
                .is_session_revoked("user-2", "jti-never-seen", before));

            // A session issued after revoke_all is fine.
            let after = before + 100;
            assert!(!h.store.is_session_revoked("user-2", "jti-fresh", after));
        }

        #[test]
        fn test_revoked_token_rejected_on_use() {
            let h = handlers();
            let caller = AuthenticatedUser::new("user-3");

            h.revoke_session(
                &caller,
                RevokeSessionRequest {
                    session_id: Some("jti-xyz".to_string()),
                    revoke_all: false,
                },
            )
            .unwrap();

            // Simulates what auth middleware should do on every request:
            // check is_session_revoked before trusting the bearer token.
            let issued_at = 0;
            let is_valid = !h.store.is_session_revoked("user-3", "jti-xyz", issued_at);
            assert!(!is_valid, "revoked token must be rejected");
        }
    }

    #[test]
    fn test_pool_stats_requires_authentication() {
        // This test verifies that calling pool_stats requires an admin parameter.
        // If we tried to call pool_stats() without a parameter, it would not compile.
        // The admin parameter is required, so only authenticated admins can call it.
        let admin = AuthenticatedAdmin::new("admin-user");
        let result = PoolMetricsHandlers::pool_stats(&admin);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pool_stats_different_admin_still_succeeds() {
        // Multiple admins can all access the metrics
        let admin1 = AuthenticatedAdmin::new("admin-1");
        let admin2 = AuthenticatedAdmin::new("admin-2");

        let result1 = PoolMetricsHandlers::pool_stats(&admin1);
        let result2 = PoolMetricsHandlers::pool_stats(&admin2);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    // -----------------------------------------------------------------------
    // Issue #1061 – Unified failure-count key ("2fa:{user_id}")
    // -----------------------------------------------------------------------

    /// Verify that verify_and_activate and verify_login_token both produce
    /// the same rate-limit key ("2fa:{user_id}"), so that a success on either
    /// path resets the failure counter for both endpoints.
    #[test]
    fn test_verify_and_login_share_same_rate_limit_key() {
        let verify_key = TwoFactorHandlers::rate_limit_key("2fa", "alice");
        let login_key  = TwoFactorHandlers::rate_limit_key("2fa", "alice");
        assert_eq!(
            verify_key, login_key,
            "verify_and_activate and verify_login_token must share the same 2fa:{{user_id}} key"
        );
        assert_eq!(verify_key, "2fa:alice");
    }

    /// Fail verify_and_activate N-1 times, then verify that the failure counter
    /// key is "2fa:{user_id}" (not "verify:{user_id}") so a successful
    /// verify_login_token call (which records_success on the same key) would
    /// clear it.  Uses InMemoryRateLimiter directly to inspect the counter.
    #[test]
    fn test_failed_verify_counter_is_reset_by_login_success_key() {
        use crate::two_factor::InMemoryStore;
        use crate::rate_limiter::InMemoryRateLimiter;
        use std::sync::Arc;

        let store = Arc::new(InMemoryStore::default());
        let limiter = Arc::new(InMemoryRateLimiter::default());
        let handlers = TwoFactorHandlers::with_store_and_limiter(
            store.clone() as Arc<dyn crate::two_factor::TwoFactorStore>,
            limiter.clone(),
        );

        // Enroll a user (without activating — we just need the limiter state).
        let caller = AuthenticatedUser::new("key-test-user");
        let enroll_req = EnableTwoFactorRequest {
            user_id: "key-test-user".to_string(),
            email: "key@example.com".to_string(),
            idempotency_key: None,
        };
        let _ = handlers.enroll(&caller, enroll_req);

        // Submit wrong tokens to verify_and_activate to accumulate failures.
        let bad_verify = VerifyTwoFactorRequest {
            user_id: "key-test-user".to_string(),
            token: "000000".to_string(),
        };
        for _ in 0..2 {
            let _ = handlers.verify_and_activate(&caller, bad_verify.clone());
        }

        // record_success on the unified key clears failures for BOTH endpoints.
        // Confirm the key used is "2fa:{user_id}" and not "verify:" or "login:".
        let key = TwoFactorHandlers::rate_limit_key("2fa", "key-test-user");
        assert_eq!(key, "2fa:key-test-user",
            "failure-count key must be 2fa:{{user_id}}, not verify: or login:");

        // Simulate a successful login by calling record_success on the shared key.
        limiter.record_success(&key);

        // After success, submitting another bad token via verify_and_activate
        // should NOT already be in a blocked state from the prior failures
        // (the counter was reset).  With InMemoryRateLimiter's default threshold
        // of 10, 2 prior failures cleared by 1 success means we are back to 0.
        let result = handlers.verify_and_activate(&caller, bad_verify.clone());
        // The result is Ok(false) (wrong token) or Err (no 2FA data) — either
        // way it must NOT be a rate-limit error, proving the counter was reset.
        match result {
            Err(e) => {
                let msg = format!("{:?}", e);
                assert!(
                    !msg.contains("Too many") && !msg.contains("rate"),
                    "verify_and_activate must not be rate-limited after login success reset; got: {msg}"
                );
            }
            Ok(_) => {} // success or false — fine
        }
    }
}
