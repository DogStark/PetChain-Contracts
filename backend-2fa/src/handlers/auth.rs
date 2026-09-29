//! Core 2FA authentication handlers.
//!
//! Contains [`TwoFactorHandlers`] (enroll, verify, login, disable, recover,
//! upgrade) and all associated request/response types.
#[cfg(not(test))]
use crate::db::PostgresTwoFactorStore;
use crate::error::ApiError;
use crate::rate_limiter::{InMemoryRateLimiter, RateLimiter};
use crate::two_factor::{
    HmacAlgorithm, InMemoryStore, TenantConfig, TenantScopedStore, TotpConfig, TwoFactorAuth,
    TwoFactorData, TwoFactorStore,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
#[cfg(not(test))]
use std::sync::OnceLock;

pub(crate) fn verification_config(algorithm: HmacAlgorithm) -> TotpConfig {
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
pub(crate) fn test_two_factor_store() -> Arc<InMemoryStore> {
    std::thread_local! {
        static STORE: Arc<InMemoryStore> = Arc::new(InMemoryStore::default());
    }

    STORE.with(|store| store.clone())
}

#[cfg(test)]
pub(crate) fn two_factor_store() -> Arc<dyn TwoFactorStore> {
    test_two_factor_store()
}

#[cfg(not(test))]
pub(crate) fn two_factor_store() -> Arc<dyn TwoFactorStore> {
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
const MAX_FIELD_LENGTH: usize = 255;

/// Validate that a string is non-empty and within max length
pub(crate) fn validate_non_empty_max_length(
    field_name: &str,
    value: &str,
) -> Result<(), ApiError> {
    if value.is_empty() {
        return Err(ApiError::bad_request(
            format!("{} must not be empty", field_name),
            None,
        ));
    }
    if value.len() > MAX_FIELD_LENGTH {
        return Err(ApiError::bad_request(
            format!(
                "{} must not exceed {} characters",
                field_name, MAX_FIELD_LENGTH
            ),
            None,
        ));
    }
    Ok(())
}

/// Validate that a token is exactly 6-8 decimal digits
pub(crate) fn validate_token(token: &str) -> Result<(), ApiError> {
    if token.len() < 6 || token.len() > 8 {
        return Err(ApiError::bad_request(
            "token must be exactly 6-8 decimal digits",
            None,
        ));
    }
    if !token.chars().all(|c| c.is_ascii_digit()) {
        return Err(ApiError::bad_request(
            "token must contain only decimal digits",
            None,
        ));
    }
    Ok(())
}

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

// ---------------------------------------------------------------------------
// Admin JWT scope check helper
// ---------------------------------------------------------------------------

/// Represents an authenticated admin caller (must have `admin` scope in JWT).
/// In a real HTTP layer the JWT would be validated by middleware; here we model
/// the scope as a field so handlers can enforce it without depending on a web
/// framework.
///
/// Defined here (rather than in [`super::admin`]) so that [`RecoveryLogCaller`]
/// can reference it without creating a circular module dependency.
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
    pub(crate) limiter: Arc<dyn RateLimiter>,
    pub(crate) store: Arc<dyn TwoFactorStore>,
    pub(crate) issuer: String,
    /// Serialises the check-then-act read/save sequence in `enroll()` so
    /// that two concurrent enrollment requests for the same user cannot
    /// both observe "not enabled" and both proceed to `save()`.
    pub(crate) enroll_lock: Arc<Mutex<()>>,
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

    /// Create a `TwoFactorHandlers` instance for single-tenant deployments.
    ///
    /// **DEPRECATED**: Use [`TwoFactorHandlers::for_tenant`] for multi-tenant
    /// deployments to ensure proper tenant isolation. This constructor uses a
    /// shared singleton store, which can lead to cross-tenant data leakage if
    /// user IDs are not properly namespaced by callers.
    #[deprecated(
        since = "0.2.0",
        note = "Use TwoFactorHandlers::for_tenant() for multi-tenant deployments to ensure proper tenant isolation"
    )]
    pub fn new() -> Self {
        Self::new_with_optional_limiter(None)
    }

    /// Create a `TwoFactorHandlers` instance scoped to a specific tenant.
    ///
    /// This constructor wraps the singleton store in a [`TenantScopedStore`],
    /// which automatically prefixes all user IDs with the tenant ID to ensure
    /// complete data isolation between tenants. This is the recommended approach
    /// for multi-tenant deployments.
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - The unique identifier for this tenant
    ///
    /// # Example
    ///
    /// ```ignore
    /// use backend_2fa::handlers::TwoFactorHandlers;
    ///
    /// let handlers = TwoFactorHandlers::for_tenant("tenant-123");
    /// // All operations through this handler are scoped to tenant-123
    /// ```
    pub fn for_tenant(tenant_id: &str) -> Self {
        let config = TenantConfig::new(tenant_id);
        let scoped_store = TenantScopedStore::new(two_factor_store(), config);
        Self {
            limiter: Arc::new(InMemoryRateLimiter::default()),
            store: Arc::new(scoped_store),
            issuer: "PetChain".to_string(),
            enroll_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn new_with_defaults() -> Self {
        Self {
            limiter: Arc::new(InMemoryRateLimiter::default()),
            store: two_factor_store(),
            issuer: default_issuer(),
            enroll_lock: Arc::new(Mutex::new(())),
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
            enroll_lock: Arc::new(Mutex::new(())),
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
            enroll_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn with_store(store: Arc<dyn TwoFactorStore>) -> Self {
        Self {
            limiter: Arc::new(InMemoryRateLimiter::default()),
            store,
            issuer: default_issuer(),
            enroll_lock: Arc::new(Mutex::new(())),
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
            enroll_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Create a `TwoFactorHandlers` instance with both a custom [`RateLimiter`]
    /// and a custom [`TwoFactorStore`], without requiring a custom issuer.
    ///
    /// Intended for integration tests that need to inject a mock store and a
    /// mock limiter simultaneously (e.g. asserting rate-limit behavior against
    /// a controlled store), which previously required chaining workarounds.
    pub fn with_limiter_and_store(
        limiter: Arc<dyn RateLimiter>,
        store: Arc<dyn TwoFactorStore>,
    ) -> Self {
        Self {
            limiter,
            store,
            issuer: "PetChain".to_string(),
            enroll_lock: Arc::new(Mutex::new(())),
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
        validate_non_empty_max_length("user_id", &req.user_id)?;
        validate_non_empty_max_length("email", &req.email)?;
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

        // Persist Argon2id hashes only — the plaintext codes are returned to
        // the caller once below and never stored.
        let hashed_backup_codes = TwoFactorAuth::hash_backup_codes(&setup.backup_codes)
            .map_err(|e| ApiError::internal_error(e, None))?;

        self.store
            .save(
                &req.user_id,
                TwoFactorData {
                    secret: setup.secret.clone(),
                    backup_codes: hashed_backup_codes,
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
        validate_non_empty_max_length("user_id", &req.user_id)?;
        validate_token(&req.token)?;
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
        validate_non_empty_max_length("user_id", &req.user_id)?;
        validate_token(&req.token)?;
        caller.authorize(&req.user_id)?;

        self.ensure_not_locked(&req.user_id)?;

        if let Err(e) = self.store.check_retry_after(&req.user_id) {
            if e.starts_with("retry_after:") {
                let retry_secs: u64 = e
                    .strip_prefix("retry_after:")
                    .unwrap_or("60")
                    .parse()
                    .unwrap_or(60);
                return Err(ApiError::rate_limited(
                    format!(
                        "Progressive delay in effect. Retry after {} seconds.",
                        retry_secs
                    ),
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
        validate_non_empty_max_length("user_id", &req.user_id)?;
        validate_token(&req.token)?;
        caller.authorize(&req.user_id)?;

        self.ensure_not_locked(&req.user_id)?;

        if let Err(e) = self.store.check_retry_after(&req.user_id) {
            if e.starts_with("retry_after:") {
                let retry_secs: u64 = e
                    .strip_prefix("retry_after:")
                    .unwrap_or("60")
                    .parse()
                    .unwrap_or(60);
                return Err(ApiError::rate_limited(
                    format!(
                        "Progressive delay in effect. Retry after {} seconds.",
                        retry_secs
                    ),
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
        validate_non_empty_max_length("user_id", &req.user_id)?;
        caller.authorize(&req.user_id)?;

        let data = self.store_get(&req.user_id)?;

        if !data.enabled {
            return Err(ApiError::bad_request("2FA not enabled for user", None));
        }

        // Find the index of the provided backup code
        let Some(index) = TwoFactorAuth::verify_backup_code(&data.backup_codes, &req.backup_code)
        else {
            return Err(ApiError::bad_request("InvalidRecoveryCode", None));
        };
        let code_index = index as i32;

        // Atomically consume the matched code. `data` may be stale if a parallel
        // request already used this code, so only the request that actually
        // removes it from storage may proceed (issue #1226).
        let consumed = self
            .store
            .remove_backup_code(&req.user_id, &data.backup_codes[index])
            .map_err(|e| ApiError::internal_error(e, None))?;
        if !consumed {
            return Err(ApiError::bad_request("InvalidRecoveryCode", None));
        }

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

        let setup = TwoFactorAuth::setup("recovery", &self.issuer)
            .map_err(|e| ApiError::internal_error(e, None))?;

        // Persist Argon2id hashes only — the plaintext codes are returned to
        // the caller once below and never stored. This also fully rotates
        // away any legacy plaintext codes that may have been on the record.
        let hashed_backup_codes = TwoFactorAuth::hash_backup_codes(&setup.backup_codes)
            .map_err(|e| ApiError::internal_error(e, None))?;

        self.store
            .save(
                &req.user_id,
                TwoFactorData {
                    secret: setup.secret.clone(),
                    backup_codes: hashed_backup_codes,
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
        validate_non_empty_max_length("user_id", &req.user_id)?;
        validate_token(&req.token)?;
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

        // Persist Argon2id hashes only — the plaintext codes are returned to
        // the caller once below and never stored.
        let hashed_backup_codes = TwoFactorAuth::hash_backup_codes(&setup.backup_codes)
            .map_err(|e| ApiError::internal_error(e, None))?;

        // Save new secret and backup codes, immediately invalidate old secret
        self.store
            .save(
                &req.user_id,
                TwoFactorData {
                    secret: setup.secret.clone(),
                    backup_codes: hashed_backup_codes,
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

#[cfg(test)]
pub(crate) fn get_two_factor_data_for_tests(
    user_id: &str,
) -> Option<TwoFactorData> {
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

#[cfg(test)]
pub(crate) fn get_two_factor_store_for_tests() -> Arc<InMemoryStore> {
    test_two_factor_store()
}

/// Issue #1226: parallel recovery requests must not both redeem one backup code.
///
/// Assumption: every request reads a (possibly stale) snapshot, verifies the
/// code against it, and may only proceed after the store's atomic
/// `remove_backup_code` succeeds. Exactly one of N concurrent requests
/// calling `remove_backup_code` with the same code hash will get `true`; all
/// others must be rejected.
#[cfg(test)]
mod backup_code_race_tests {
    use super::*;
    use std::sync::Barrier;

    const USER: &str = "race-user";
    const CODES: [&str; 3] = ["1111-1111", "2222-2222", "3333-3333"];

    fn seeded_store() -> Arc<InMemoryStore> {
        let codes: Vec<String> = CODES.iter().map(|c| c.to_string()).collect();
        let store = Arc::new(InMemoryStore::default());
        store
            .save(
                USER,
                TwoFactorData {
                    secret: TwoFactorAuth::generate_secret(),
                    backup_codes: TwoFactorAuth::hash_backup_codes(&codes).unwrap(),
                    enabled: true,
                    algorithm: HmacAlgorithm::SHA256,
                    last_used_step: None,
                },
            )
            .unwrap();
        store
    }

    fn recover(handlers: &TwoFactorHandlers, caller: &str, code: &str) -> Result<(), ApiError> {
        handlers
            .recover(
                &AuthenticatedUser::new(caller),
                RecoverWithBackupRequest {
                    user_id: USER.to_string(),
                    backup_code: code.to_string(),
                },
                None,
            )
            .map(|_| ())
    }

    #[test]
    fn remove_backup_code_is_compare_and_delete() {
        let store = seeded_store();
        let target = store.get(USER).unwrap().backup_codes[1].clone();
        let barrier = Arc::new(Barrier::new(16));

        let wins = (0..16)
            .map(|_| {
                let (store, target, barrier) = (store.clone(), target.clone(), barrier.clone());
                std::thread::spawn(move || {
                    barrier.wait();
                    store.remove_backup_code(USER, &target).unwrap()
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|h| h.join().unwrap())
            .filter(|won| *won)
            .count();

        assert_eq!(wins, 1);
        let remaining = store.get(USER).unwrap().backup_codes;
        assert_eq!(remaining.len(), CODES.len() - 1);
        assert!(!remaining.contains(&target));
    }

    #[test]
    fn remove_backup_code_missing_user_or_code() {
        let store = seeded_store();
        assert!(store.remove_backup_code("unknown", "x").is_err());
        assert_eq!(store.remove_backup_code(USER, "not-stored"), Ok(false));
        assert_eq!(store.get(USER).unwrap().backup_codes.len(), CODES.len());
    }

    #[test]
    fn concurrent_recovery_with_same_code_succeeds_once() {
        let handlers = Arc::new(TwoFactorHandlers::with_store(seeded_store()));
        let barrier = Arc::new(Barrier::new(8));

        let results: Vec<_> = (0..8)
            .map(|_| {
                let (handlers, barrier) = (handlers.clone(), barrier.clone());
                std::thread::spawn(move || {
                    barrier.wait();
                    recover(&handlers, USER, CODES[0])
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect();

        assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
        for err in results.into_iter().filter_map(Result::err) {
            assert_eq!(err.message, "InvalidRecoveryCode");
        }
    }

    #[test]
    fn replayed_code_is_rejected_after_success() {
        let handlers = TwoFactorHandlers::with_store(seeded_store());
        assert!(recover(&handlers, USER, CODES[0]).is_ok());
        let err = recover(&handlers, USER, CODES[0]).unwrap_err();
        assert_eq!(err.message, "InvalidRecoveryCode");
        // Codes from the rotated-away set are rejected too.
        assert!(recover(&handlers, USER, CODES[1]).is_err());
    }

    #[test]
    fn unauthorized_or_invalid_requests_do_not_consume_codes() {
        let store = seeded_store();
        let handlers = TwoFactorHandlers::with_store(store.clone());

        assert!(recover(&handlers, "someone-else", CODES[0]).is_err());
        assert!(recover(&handlers, USER, "9999-9999").is_err());
        assert!(recover(&handlers, USER, "").is_err());
        assert_eq!(store.get(USER).unwrap().backup_codes.len(), CODES.len());

        assert!(recover(&handlers, USER, CODES[0]).is_ok());
    }
}
