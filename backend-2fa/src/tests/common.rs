#![allow(dead_code)]
use crate::error::ApiError;
use crate::handlers::{
    clear_two_factor_store_for_tests, get_two_factor_data_for_tests,
    get_two_factor_store_for_tests, overwrite_two_factor_data_for_tests, AdminDashboardHandlers,
    AdminRecoveryHandlers, AdminWebhookHandlers, AuthenticatedAdmin, AuthenticatedUser,
    CanaryHandlers, ConfigureWebhookRequest, CreateCanaryRequest, DisableTwoFactorRequest,
    EnableTwoFactorRequest, LoginWithTwoFactorRequest, PoolMetricsHandlers, ProvisionTenantRequest,
    RecoverWithBackupRequest, TenantProvisioningHandlers, TwoFactorHandlers,
    UpgradeAlgorithmRequest, VerifyTwoFactorRequest,
};
use crate::rate_limiter::{
    progressive_delay_secs, DistributedRateLimiter, EndpointConfig, InMemoryRateLimiter,
    MockRedisBackend, RateLimitResult, RateLimiter, RedisRateLimiter, RedisTwoFactorFailureCounter,
    SlidingWindowRateLimiter,
};
use crate::two_factor::{
    InMemoryStore, MockStoreConfig, MockStoreFailure, MockTwoFactorStore, TenantConfig,
    TenantRegistry, TenantScopedStore, TotpConfig, TwoFactorAuth, TwoFactorData, TwoFactorStore,
};
use crate::webhooks::{DefaultHttpClient, HttpClient, SecurityEventType, WebhookManager};
use actix_web::ResponseError;
use std::collections::{BTreeMap, HashMap};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, Secret, TOTP};

pub(crate) fn caller(id: &str) -> AuthenticatedUser {
    AuthenticatedUser::new(id)
}

pub(crate) fn generate_token(secret: &str) -> String {
    use totp_rs::{Algorithm, Secret, TOTP};
    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded(secret.to_string()).to_bytes().unwrap(),
        None,
        String::new(),
    )
    .unwrap()
    .generate_current()
    .unwrap()
}

pub(crate) fn admin() -> AuthenticatedAdmin {
    AuthenticatedAdmin::new("super-admin")
}

/// Returns a unique key per test invocation to prevent cross-test pollution.
pub(crate) fn unique_key(label: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    format!("test:{label}:{nanos}")
}

pub(crate) fn redis_url() -> Option<String> {
    std::env::var("REDIS_URL").ok()
}

pub(crate) fn make_limiter(
    max_failures: u32,
    window_secs: u64,
    lockout_secs: u64,
) -> Option<RedisRateLimiter> {
    redis_url()
        .and_then(|url| RedisRateLimiter::new(&url, max_failures, window_secs, lockout_secs).ok())
}

#[derive(Clone, Default)]
pub(crate) struct MockRedisState {
    /// sorted set: key → BTreeMap<score_ms, member_ms>
    zsets: HashMap<String, BTreeMap<u64, u64>>,
    /// string keys (lockout)
    strings: HashMap<String, u64>, // value = TTL remaining (secs)
}

impl MockRedisState {
    fn zremrangebyscore(&mut self, key: &str, min: u64, max: u64) {
        if let Some(set) = self.zsets.get_mut(key) {
            set.retain(|&score, _| score < min || score > max);
        }
    }

    fn zadd(&mut self, key: &str, score: u64, member: u64) {
        self.zsets
            .entry(key.to_string())
            .or_default()
            .insert(score, member);
    }

    fn zcard(&self, key: &str) -> u64 {
        self.zsets.get(key).map(|s| s.len() as u64).unwrap_or(0)
    }

    fn set_ex(&mut self, key: &str, ttl: u64) {
        self.strings.insert(key.to_string(), ttl);
    }

    fn ttl(&self, key: &str) -> i64 {
        match self.strings.get(key) {
            Some(&t) if t > 0 => t as i64,
            _ => -2,
        }
    }

    fn del(&mut self, keys: &[&str]) {
        for k in keys {
            self.zsets.remove(*k);
            self.strings.remove(*k);
        }
    }
}

/// Simulates the sliding-window logic of `RedisRateLimiter::record_failure`
/// using the mock state, so we can assert on the algorithm without Redis.
pub(crate) fn mock_record_failure(
    state: &Arc<Mutex<MockRedisState>>,
    key: &str,
    now_ms: u64,
    max_failures: u32,
    window_secs: u64,
    lockout_secs: u64,
) -> RateLimitResult {
    let mut s = state.lock().unwrap();

    let lockout_key = format!("rate:{key}:lockout");
    let window_key = format!("rate:{key}:window");

    if s.ttl(&lockout_key) > 0 {
        return RateLimitResult::Blocked {
            limit: 0,
            remaining: 0,
            reset_at: 0,
            retry_after_secs: s.ttl(&lockout_key) as u64,
        };
    }

    let cutoff_ms = now_ms.saturating_sub(window_secs * 1_000);
    s.zremrangebyscore(&window_key, 0, cutoff_ms);
    s.zadd(&window_key, now_ms, now_ms);
    let count = s.zcard(&window_key);

    if count >= max_failures as u64 {
        s.set_ex(&lockout_key, lockout_secs);
        return RateLimitResult::Blocked {
            limit: 0,
            remaining: 0,
            reset_at: 0,
            retry_after_secs: lockout_secs,
        };
    }

    RateLimitResult::Allowed {
        limit: 0,
        remaining: max_failures - count as u32,
        reset_at: 0,
    }
}

pub(crate) fn mock_record_success(state: &Arc<Mutex<MockRedisState>>, key: &str) {
    let mut s = state.lock().unwrap();
    s.del(&[
        &format!("rate:{key}:lockout"),
        &format!("rate:{key}:window"),
    ]);
}

pub(crate) fn limiter(
    max: u32,
    window_secs: u64,
    lockout_secs: u64,
) -> SlidingWindowRateLimiter<MockRedisBackend> {
    SlidingWindowRateLimiter::new(
        MockRedisBackend::new(),
        EndpointConfig::new(window_secs, max, lockout_secs),
    )
}

impl crate::rate_limiter::SlidingWindowRateLimiter<crate::rate_limiter::MockRedisBackend> {
    pub(crate) fn backend_advance_ms(&self, ms: u64) {
        // Access the backend field directly (same crate, so pub(crate) is fine).
        self.backend.advance_ms(ms);
    }
}

pub(crate) fn setup_user(user_id: &str) {
    let store = get_two_factor_store_for_tests();
    let _ = store.save(
        user_id,
        TwoFactorData {
            secret: "JBSWY3DPEHPK3PXP".to_string(),
            backup_codes: vec![],
            enabled: true,
            algorithm: Algorithm::SHA1,
            last_used_step: None,
        },
    );
}

pub(crate) struct RecordingHttpClient {
    calls: Arc<Mutex<Vec<String>>>,
}

impl RecordingHttpClient {
    fn new() -> (Self, Arc<Mutex<Vec<String>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                calls: calls.clone(),
            },
            calls,
        )
    }
}

impl HttpClient for RecordingHttpClient {
    fn post(&self, url: &str, body: &str, _signature_header: &str) -> Result<(), String> {
        self.calls.lock().unwrap().push(format!("{}:{}", url, body));
        Ok(())
    }
}

pub(crate) fn make_canary_handlers() -> (CanaryHandlers, Arc<Mutex<Vec<String>>>) {
    let (http_client, calls) = RecordingHttpClient::new();
    let wm = Arc::new(WebhookManager::new_with_http_allowed(Arc::new(http_client)));
    wm.configure(
        SecurityEventType::CanaryTriggered,
        "http://alert.example.com/hook".to_string(),
    )
    .unwrap();
    (CanaryHandlers::new(wm), calls)
}

pub(crate) fn make_handlers() -> AdminWebhookHandlers {
    let manager = Arc::new(WebhookManager::new_with_http_allowed(Arc::new(
        DefaultHttpClient,
    )));
    AdminWebhookHandlers::new(manager)
}

pub(crate) fn provision_req(tenant_id: &str) -> ProvisionTenantRequest {
    ProvisionTenantRequest {
        tenant_id: tenant_id.to_string(),
        name: format!("{tenant_id} Inc"),
        max_users: 50,
        totp_issuer: "AcmeCo".to_string(),
        rate_limit_max_failures: 7,
    }
}

pub(crate) const ALGORITHMS: [Algorithm; 3] =
    [Algorithm::SHA1, Algorithm::SHA256, Algorithm::SHA512];

pub(crate) const WINDOWS: [u8; 4] = [0, 1, 2, 5];

pub(crate) fn make_data(secret: &str) -> TwoFactorData {
    TwoFactorData {
        secret: secret.to_string(),
        backup_codes: vec!["0000-1111".to_string()],
        enabled: true,
        algorithm: Algorithm::SHA1,
        last_used_step: None,
    }
}
