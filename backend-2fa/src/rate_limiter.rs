use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use redis::Commands;

#[derive(Debug, PartialEq, Clone)]
pub enum RateLimitResult {
    /// The attempt is allowed.
    Allowed {
        limit: u32,
        remaining: u32,
        reset_at: u64,
    },
    /// The key is locked out.
    Blocked {
        limit: u32,
        remaining: u32,
        reset_at: u64,
        retry_after_secs: u64,
    },
}

impl RateLimitResult {
    pub fn limit(&self) -> u32 {
        match self {
            RateLimitResult::Allowed { limit, .. } => *limit,
            RateLimitResult::Blocked { limit, .. } => *limit,
        }
    }

    pub fn remaining(&self) -> u32 {
        match self {
            RateLimitResult::Allowed { remaining, .. } => *remaining,
            RateLimitResult::Blocked { remaining, .. } => *remaining,
        }
    }

    pub fn reset_at(&self) -> u64 {
        match self {
            RateLimitResult::Allowed { reset_at, .. } => *reset_at,
            RateLimitResult::Blocked { reset_at, .. } => *reset_at,
        }
    }

    pub fn is_blocked(&self) -> bool {
        matches!(self, RateLimitResult::Blocked { .. })
    }

    pub fn retry_after_secs(&self) -> u64 {
        match self {
            RateLimitResult::Allowed { .. } => 0,
            RateLimitResult::Blocked { retry_after_secs, .. } => *retry_after_secs,
        }
    }
}

/// A pluggable rate limiter interface.
pub trait RateLimiter: Send + Sync {
    fn record_failure(&self, key: &str) -> RateLimitResult;
    fn record_success(&self, key: &str);
}

// ---------------------------------------------------------------------------
// Per-endpoint window configuration
// ---------------------------------------------------------------------------

/// Window size and request limit for a single endpoint.
#[derive(Clone, Debug)]
pub struct EndpointConfig {
    pub window_secs: u64,
    pub max_failures: u32,
    pub lockout_secs: u64,
}

impl EndpointConfig {
    pub fn new(window_secs: u64, max_failures: u32, lockout_secs: u64) -> Self {
        Self {
            window_secs,
            max_failures,
            lockout_secs,
        }
    }
}

// ---------------------------------------------------------------------------
// Redis backend abstraction (enables mock injection)
// ---------------------------------------------------------------------------

/// Minimal Redis operations needed by the sliding-window limiter.
/// Implement this trait to swap in a mock for tests.
pub trait RedisBackend: Send + Sync {
    /// Returns the TTL of `key` in seconds, or -2 if the key does not exist.
    fn ttl(&self, key: &str) -> i64;
    /// Atomically: remove members with score in [0, cutoff_ms], add a new
    /// member with score `now_ms`, return the cardinality, and refresh the TTL.
    fn sliding_window_add(
        &self,
        key: &str,
        now_ms: u64,
        cutoff_ms: u64,
        member: &str,
        ttl_secs: u64,
    ) -> u64;
    /// Set `key` with a TTL (seconds).
    fn set_ex(&self, key: &str, value: &str, ttl_secs: u64);
    /// Delete one or more keys.
    fn del(&self, keys: &[&str]);
    /// Increment a string counter and set TTL on first write.
    fn incr_with_ttl(&self, key: &str, ttl_secs: u64) -> u64 {
        let _ = (key, ttl_secs);
        0
    }
    /// Read a string counter.
    fn get_u64(&self, key: &str) -> Option<u64> {
        let _ = key;
        None
    }
}

// ---------------------------------------------------------------------------
// Tenant-scoped rate-limit key helper (Issue #854)
// ---------------------------------------------------------------------------

/// A typed helper that builds rate-limit keys with guaranteed tenant isolation.
///
/// Using `TenantRateLimitKey` instead of ad-hoc `format!` strings ensures that
/// every rate-limit key is prefixed by the tenant ID, preventing cross-tenant
/// bucket sharing.
///
/// # Examples
/// ```
/// use petchain_2fa::rate_limiter::TenantRateLimitKey;
///
/// let key = TenantRateLimitKey::new("tenant_a", "verify", "user42");
/// assert_eq!(key.as_str(), "tenant_a::verify::user42");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TenantRateLimitKey {
    inner: String,
}

impl TenantRateLimitKey {
    /// Build a tenant-scoped rate-limit key.
    ///
    /// * `tenant_id` — the tenant identifier (must not be empty)
    /// * `action`    — the action being rate-limited (e.g. `"verify"`, `"login"`)
    /// * `user_id`   — the user within the tenant
    pub fn new(tenant_id: &str, action: &str, user_id: &str) -> Self {
        debug_assert!(!tenant_id.is_empty(), "tenant_id must not be empty");
        debug_assert!(!action.is_empty(), "action must not be empty");
        Self {
            inner: format!("{tenant_id}::{action}::{user_id}"),
        }
    }

    /// Return the key as a string slice, suitable for passing to
    /// `RateLimiter::record_failure` / `record_success`.
    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl std::fmt::Display for TenantRateLimitKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

impl AsRef<str> for TenantRateLimitKey {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

pub fn progressive_delay_secs(attempt: u32) -> Option<u64> {
    if attempt == 0 || attempt >= 10 {
        None
    } else {
        Some(1u64 << (attempt - 1).min(8))
    }
}

pub struct RedisTwoFactorFailureCounter<B: RedisBackend> {
    backend: B,
    key_prefix: String,
    ttl_secs: u64,
}

impl<B: RedisBackend> RedisTwoFactorFailureCounter<B> {
    pub fn new(backend: B, key_prefix: impl Into<String>, ttl_secs: u64) -> Self {
        Self {
            backend,
            key_prefix: key_prefix.into(),
            ttl_secs,
        }
    }

    fn key(&self, user_id: &str) -> String {
        format!("{}2fa:failures:{}", self.key_prefix, user_id)
    }

    pub fn record_failure(&self, user_id: &str) -> u32 {
        self.backend
            .incr_with_ttl(&self.key(user_id), self.ttl_secs)
            .min(u32::MAX as u64) as u32
    }

    pub fn get_failures(&self, user_id: &str) -> u32 {
        self.backend
            .get_u64(&self.key(user_id))
            .unwrap_or(0)
            .min(u32::MAX as u64) as u32
    }

    pub fn reset(&self, user_id: &str) {
        let key = self.key(user_id);
        self.backend.del(&[&key]);
    }
}

// ---------------------------------------------------------------------------
// Live Redis backend
// ---------------------------------------------------------------------------

pub struct LiveRedisBackend {
    client: redis::Client,
}

impl LiveRedisBackend {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
        })
    }

    fn get_connection(&self) -> redis::RedisResult<redis::Connection> {
        self.client.get_connection()
    }
}

impl RedisBackend for LiveRedisBackend {
    fn ttl(&self, key: &str) -> i64 {
        let mut con = match self.get_connection() {
            Ok(c) => c,
            Err(_) => return -2,
        };
        con.ttl(key).unwrap_or(-2)
    }

    fn sliding_window_add(
        &self,
        key: &str,
        now_ms: u64,
        cutoff_ms: u64,
        member: &str,
        ttl_secs: u64,
    ) -> u64 {
        let mut con = match self.client.get_connection() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(key = key, error = %e, "[LiveRedisBackend] connection error");
                return 0;
            }
        };
        let result: redis::RedisResult<(u64,)> = (|| {
            let mut pipe = redis::pipe();
            pipe.cmd("ZREMRANGEBYSCORE")
                .arg(key)
                .arg(0u64)
                .arg(cutoff_ms)
                .ignore()
                .cmd("ZADD")
                .arg(key)
                .arg(now_ms)
                .arg(member)
                .ignore()
                .cmd("ZCARD")
                .arg(key)
                .cmd("EXPIRE")
                .arg(key)
                .arg(ttl_secs)
                .ignore();
            pipe.query(&mut con)
        })();
        match result {
            Ok((card,)) => card,
            Err(e) => {
                tracing::error!(key = key, error = %e, "[LiveRedisBackend] pipeline error");
                0
            }
        }
    }

    fn set_ex(&self, key: &str, value: &str, ttl_secs: u64) {
        if let Ok(mut con) = self.get_connection() {
            let _: Result<(), _> = con.set_ex(key, value, ttl_secs);
        }
    }

    fn del(&self, keys: &[&str]) {
        if let Ok(mut con) = self.get_connection() {
            let _: Result<(), _> = redis::cmd("DEL").arg(keys).query(&mut con);
        }
    }

    fn incr_with_ttl(&self, key: &str, ttl_secs: u64) -> u64 {
        let mut con = match self.get_connection() {
            Ok(c) => c,
            Err(_) => return 0,
        };
        let count: u64 = redis::cmd("INCR").arg(key).query(&mut con).unwrap_or(0);
        if count == 1 {
            let _: Result<(), _> = redis::cmd("EXPIRE").arg(key).arg(ttl_secs).query(&mut con);
        }
        count
    }

    fn get_u64(&self, key: &str) -> Option<u64> {
        let mut con = self.get_connection().ok()?;
        redis::cmd("GET").arg(key).query(&mut con).ok()
    }
}

// ---------------------------------------------------------------------------
// Mock Redis backend (in-process, for tests)
// ---------------------------------------------------------------------------

struct MockEntry {
    /// Sorted set: (score_ms, member)
    zset: Vec<(u64, String)>,
    /// Expiry in mock-clock milliseconds (None = no expiry)
    expires_at_ms: Option<u64>,
    value: Option<u64>,
}

/// In-process mock that faithfully implements the sorted-set sliding window.
pub struct MockRedisBackend {
    store: Mutex<HashMap<String, MockEntry>>,
    /// Injected "current time" for deterministic tests.
    now_ms: Mutex<u64>,
}

// --- Simple per-user quota store used by admin handlers in tests ---
#[derive(Default, Clone)]
pub struct UserQuotaStore {
    inner: Arc<Mutex<HashMap<String, (u32, Option<u64>)>>>,
}

impl UserQuotaStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_quota(&self, user_id: &str, requests_per_minute: u32) {
        let mut m = self.inner.lock().unwrap();
        m.insert(user_id.to_string(), (requests_per_minute, None));
    }

    pub fn grant_unlimited(&self, user_id: &str, expires_at: u64) {
        let mut m = self.inner.lock().unwrap();
        let entry = m.entry(user_id.to_string()).or_insert((0, None));
        entry.1 = Some(expires_at);
    }
}

impl MockRedisBackend {
    pub fn new() -> Self {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            store: Mutex::new(HashMap::new()),
            now_ms: Mutex::new(now_ms),
        }
    }

    /// Advance the mock clock by `ms` milliseconds (for deterministic tests).
    pub fn advance_ms(&self, ms: u64) {
        *self.now_ms.lock().unwrap() += ms;
    }

    fn current_ms(&self) -> u64 {
        *self.now_ms.lock().unwrap()
    }
}

impl Default for MockRedisBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RedisBackend for MockRedisBackend {
    fn ttl(&self, key: &str) -> i64 {
        let now_ms = self.current_ms();
        let store = self.store.lock().unwrap();
        match store.get(key) {
            None => -2,
            Some(entry) => match entry.expires_at_ms {
                None => -1,
                Some(exp_ms) => {
                    if now_ms >= exp_ms {
                        -2
                    } else {
                        ((exp_ms - now_ms + 999) / 1_000) as i64
                    } // ceiling division → secs
                }
            },
        }
    }

    fn sliding_window_add(
        &self,
        key: &str,
        _now_ms: u64,
        cutoff_ms: u64,
        member: &str,
        ttl_secs: u64,
    ) -> u64 {
        let now_ms = self.current_ms();
        let mut store = self.store.lock().unwrap();
        let entry = store.entry(key.to_string()).or_insert(MockEntry {
            zset: Vec::new(),
            expires_at_ms: None,
            value: None,
        });
        // Evict if the key itself has expired
        if let Some(exp) = entry.expires_at_ms {
            if now_ms >= exp {
                entry.zset.clear();
                entry.expires_at_ms = None;
            }
        }
        // ZREMRANGEBYSCORE: remove scores <= cutoff_ms
        entry.zset.retain(|(score, _)| *score > cutoff_ms);
        // ZADD
        entry.zset.push((now_ms, member.to_string()));
        // EXPIRE
        entry.expires_at_ms = Some(now_ms + ttl_secs * 1_000);
        entry.zset.len() as u64
    }

    fn set_ex(&self, key: &str, value: &str, ttl_secs: u64) {
        let now_ms = self.current_ms();
        let mut store = self.store.lock().unwrap();
        store.insert(
            key.to_string(),
            MockEntry {
                zset: vec![(0, value.to_string())],
                expires_at_ms: Some(now_ms + ttl_secs * 1_000),
                value: None,
            },
        );
    }

    fn del(&self, keys: &[&str]) {
        let mut store = self.store.lock().unwrap();
        for k in keys {
            store.remove(*k);
        }
    }

    fn incr_with_ttl(&self, key: &str, ttl_secs: u64) -> u64 {
        let now_ms = self.current_ms();
        let mut store = self.store.lock().unwrap();
        let entry = store.entry(key.to_string()).or_insert(MockEntry {
            zset: Vec::new(),
            expires_at_ms: Some(now_ms + ttl_secs * 1_000),
            value: Some(0),
        });
        if entry
            .expires_at_ms
            .map(|exp| now_ms >= exp)
            .unwrap_or(false)
        {
            entry.value = Some(0);
        }
        let value = entry.value.unwrap_or(0).saturating_add(1);
        entry.value = Some(value);
        entry.expires_at_ms = Some(now_ms + ttl_secs * 1_000);
        value
    }

    fn get_u64(&self, key: &str) -> Option<u64> {
        let now_ms = self.current_ms();
        let store = self.store.lock().unwrap();
        let entry = store.get(key)?;
        if entry
            .expires_at_ms
            .map(|exp| now_ms >= exp)
            .unwrap_or(false)
        {
            None
        } else {
            entry.value
        }
    }
}

// ---------------------------------------------------------------------------
// In-memory implementation (unchanged)
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct AttemptRecord {
    failures: u32,
    window_start: Instant,
    locked_until: Option<Instant>,
}

/// Thread-safe in-memory rate limiter using a sliding window + lockout.
pub struct InMemoryRateLimiter {
    max_failures: u32,
    window: Duration,
    lockout: Duration,
    records: Mutex<HashMap<String, AttemptRecord>>,
}

impl InMemoryRateLimiter {
    pub fn new(max_failures: u32, window_secs: u64, lockout_secs: u64) -> Self {
        Self {
            max_failures,
            window: Duration::from_secs(window_secs),
            lockout: Duration::from_secs(lockout_secs),
            records: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryRateLimiter {
    fn default() -> Self {
        Self::new(5, 60, 300)
    }
}

impl RateLimiter for InMemoryRateLimiter {
    fn record_failure(&self, key: &str) -> RateLimitResult {
        // Recover from poisoned lock: data is still valid, just recover the Mutex
        let mut records = self.records.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let now = Instant::now();
        let unix_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let record = records.entry(key.to_string()).or_insert(AttemptRecord {
            failures: 0,
            window_start: now,
            locked_until: None,
        });

        if let Some(locked_until) = record.locked_until {
            if now < locked_until {
                let retry_after_secs = (locked_until - now).as_secs().max(1);
                let reset_at = unix_now + retry_after_secs;
                return RateLimitResult::Blocked {
                    limit: self.max_failures,
                    remaining: 0,
                    reset_at,
                    retry_after_secs,
                };
            } else {
                record.failures = 0;
                record.window_start = now;
                record.locked_until = None;
            }
        }

        if now.duration_since(record.window_start) >= self.window {
            record.failures = 0;
            record.window_start = now;
        }

        record.failures += 1;

        // reset_at = end of the current window
        let elapsed_secs = now.duration_since(record.window_start).as_secs();
        let window_remaining_secs = self.window.as_secs().saturating_sub(elapsed_secs);
        let reset_at = unix_now + window_remaining_secs;

        if record.failures > self.max_failures {
            record.locked_until = Some(now + self.lockout);
            RateLimitResult::Blocked {
                limit: self.max_failures,
                remaining: 0,
                reset_at: unix_now + self.lockout.as_secs(),
                retry_after_secs: self.lockout.as_secs(),
            }
        } else {
            RateLimitResult::Allowed {
                limit: self.max_failures,
                remaining: self.max_failures.saturating_sub(record.failures),
                reset_at,
            }
        }
    }

    fn record_success(&self, key: &str) {
        // Recover from poisoned lock: data is still valid, just recover the Mutex
        let mut records = self.records.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        records.remove(key);
    }
}

// ---------------------------------------------------------------------------
// Redis-backed sliding window rate limiter (generic over backend)
// ---------------------------------------------------------------------------

/// Redis-backed rate limiter using a sorted-set sliding window.
///
/// Accepts any [`RedisBackend`] — use [`LiveRedisBackend`] in production and
/// [`MockRedisBackend`] in tests.  Per-endpoint configuration is supported via
/// [`EndpointConfig`]: pass the endpoint name as part of the `key` (e.g.
/// `"login:user:42"`) or supply separate limiters per endpoint.
///
/// On any backend error the limiter **fails open** (returns `Allowed`) to
/// avoid locking out users during an outage.
pub struct SlidingWindowRateLimiter<B: RedisBackend> {
    pub(crate) backend: B,
    /// Default config, used when no per-endpoint override matches.
    default: EndpointConfig,
    /// Optional per-endpoint overrides keyed by endpoint prefix.
    endpoints: HashMap<String, EndpointConfig>,
}

impl<B: RedisBackend> SlidingWindowRateLimiter<B> {
    pub fn new(backend: B, default: EndpointConfig) -> Self {
        Self {
            backend,
            default,
            endpoints: HashMap::new(),
        }
    }

    /// Register a per-endpoint config.  The `endpoint` string is matched as a
    /// prefix of the rate-limit key (e.g. `"login"` matches `"login:user:42"`).
    pub fn with_endpoint(mut self, endpoint: impl Into<String>, config: EndpointConfig) -> Self {
        self.endpoints.insert(endpoint.into(), config);
        self
    }

    fn config_for(&self, key: &str) -> &EndpointConfig {
        self.endpoints
            .iter()
            .find(|(prefix, _)| key.starts_with(prefix.as_str()))
            .map(|(_, cfg)| cfg)
            .unwrap_or(&self.default)
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    fn unique_member() -> String {
        let d = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        format!("{}:{}", d.as_millis(), d.subsec_nanos())
    }
}

impl<B: RedisBackend> RateLimiter for SlidingWindowRateLimiter<B> {
    fn record_failure(&self, key: &str) -> RateLimitResult {
        let cfg = self.config_for(key);
        let lockout_key = format!("rate:{key}:lockout");
        let window_key = format!("rate:{key}:window");

        let unix_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let lockout_ttl = self.backend.ttl(&lockout_key);
        if lockout_ttl > 0 {
            return RateLimitResult::Blocked {
                limit: cfg.max_failures,
                remaining: 0,
                reset_at: unix_now + lockout_ttl as u64,
                retry_after_secs: lockout_ttl as u64,
            };
        }

        let now_ms = Self::now_ms();
        let cutoff_ms = now_ms.saturating_sub(cfg.window_secs * 1_000);
        let member = Self::unique_member();

        let count = self.backend.sliding_window_add(
            &window_key,
            now_ms,
            cutoff_ms,
            &member,
            cfg.window_secs,
        );

        if count > cfg.max_failures as u64 {
            self.backend.set_ex(&lockout_key, "1", cfg.lockout_secs);
            return RateLimitResult::Blocked {
                limit: cfg.max_failures,
                remaining: 0,
                reset_at: unix_now + cfg.lockout_secs,
                retry_after_secs: cfg.lockout_secs,
            };
        }

        RateLimitResult::Allowed {
            limit: cfg.max_failures,
            remaining: cfg.max_failures.saturating_sub(count as u32),
            reset_at: unix_now + cfg.window_secs,
        }
    }

    fn record_success(&self, key: &str) {
        let lockout_key = format!("rate:{key}:lockout");
        let window_key = format!("rate:{key}:window");
        self.backend.del(&[&lockout_key, &window_key]);
    }
}

// ---------------------------------------------------------------------------
// Legacy RedisRateLimiter (kept for backwards compatibility)
// ---------------------------------------------------------------------------

/// Redis-backed rate limiter using a sorted-set sliding window.
///
/// Prefer [`SlidingWindowRateLimiter`] with [`LiveRedisBackend`] for new code.
pub struct RedisRateLimiter {
    inner: SlidingWindowRateLimiter<LiveRedisBackend>,
}

impl RedisRateLimiter {
    pub fn new(
        redis_url: &str,
        max_failures: u32,
        window_secs: u64,
        lockout_secs: u64,
    ) -> Result<Self, redis::RedisError> {
        let backend = LiveRedisBackend::new(redis_url)?;
        let cfg = EndpointConfig::new(window_secs, max_failures, lockout_secs);
        Ok(Self {
            inner: SlidingWindowRateLimiter::new(backend, cfg),
        })
    }
}

impl RateLimiter for RedisRateLimiter {
    fn record_failure(&self, key: &str) -> RateLimitResult {
        self.inner.record_failure(key)
    }
    fn record_success(&self, key: &str) {
        self.inner.record_success(key)
    }
}

// ---------------------------------------------------------------------------
// DistributedRateLimiter — atomic INCR+EXPIRE via Lua, fallback to in-memory
// ---------------------------------------------------------------------------

/// Lua script: atomically increment a counter and set TTL on first use.
/// Returns {count, ttl} so callers can compute reset_at.
/// KEYS[1] = rate-limit key, ARGV[1] = window_secs, ARGV[2] = max_requests
const INCR_EXPIRE_SCRIPT: &str = r#"
local current = redis.call('INCR', KEYS[1])
if current == 1 then
    redis.call('EXPIRE', KEYS[1], ARGV[1])
end
local ttl = redis.call('TTL', KEYS[1])
return {current, ttl}
"#;

/// Distributed rate limiter using Redis INCR + EXPIRE (Lua atomic script).
///
/// Falls back to an in-memory limiter with a warning log when Redis is
/// unavailable.  A configurable `key_prefix` isolates counters per service
/// instance (e.g. `"svc-a:"` vs `"svc-b:"`).
pub struct DistributedRateLimiter {
    client: Option<redis::Client>,
    fallback: InMemoryRateLimiter,
    max_requests: u32,
    window_secs: u64,
    key_prefix: String,
}

impl DistributedRateLimiter {
    /// Create a new `DistributedRateLimiter`.
    ///
    /// * `redis_url`    — Redis connection URL; `None` forces in-memory fallback.
    /// * `max_requests` — Maximum requests allowed per window.
    /// * `window_secs`  — Sliding window duration in seconds.
    /// * `key_prefix`   — Prefix prepended to every Redis key (e.g. `"api:"`).
    pub fn new(
        redis_url: Option<&str>,
        max_requests: u32,
        window_secs: u64,
        key_prefix: impl Into<String>,
    ) -> Self {
        let client = redis_url.and_then(|url| redis::Client::open(url).ok());
        Self {
            client,
            fallback: InMemoryRateLimiter::new(max_requests, window_secs, window_secs),
            max_requests,
            window_secs,
            key_prefix: key_prefix.into(),
        }
    }

    fn redis_key(&self, key: &str) -> String {
        format!("{}rl:{}", self.key_prefix, key)
    }

    fn try_redis(&self, key: &str) -> Option<RateLimitResult> {
        let client = self.client.as_ref()?;
        let mut con = client.get_connection().ok()?;

        let unix_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let redis_key = self.redis_key(key);
        let (count, ttl): (u64, i64) = redis::Script::new(INCR_EXPIRE_SCRIPT)
            .key(&redis_key)
            .arg(self.window_secs)
            .arg(self.max_requests)
            .invoke(&mut con)
            .ok()?;

        // ttl may be -1 (no expiry) or -2 (key missing); fall back to window_secs
        let ttl_secs = if ttl > 0 { ttl as u64 } else { self.window_secs };
        let reset_at = unix_now + ttl_secs;

        if count > self.max_requests as u64 {
            Some(RateLimitResult::Blocked {
                limit: self.max_requests,
                remaining: 0,
                reset_at,
                retry_after_secs: self.window_secs,
            })
        } else {
            Some(RateLimitResult::Allowed {
                limit: self.max_requests,
                remaining: self.max_requests.saturating_sub(count as u32),
                reset_at,
            })
        }
    }
}

impl RateLimiter for DistributedRateLimiter {
    fn record_failure(&self, key: &str) -> RateLimitResult {
        let result = match self.try_redis(key) {
            Some(result) => result,
            None => {
                tracing::warn!(
                    key = key,
                    "[DistributedRateLimiter] Redis unavailable, falling back to in-memory"
                );
                crate::metrics::record_redis_fallback();
                self.fallback.record_failure(key)
            }
        };
        if matches!(result, RateLimitResult::Blocked { .. }) {
            let endpoint = key.split(':').next().unwrap_or(key);
            crate::metrics::record_rate_limit_hit(endpoint, "limit_exceeded");
        }
        result
    }

    fn record_success(&self, key: &str) {
        if let Some(client) = &self.client {
            if let Ok(mut con) = client.get_connection() {
                let redis_key = self.redis_key(key);
                let _: Result<(), _> = redis::cmd("DEL").arg(&redis_key).query(&mut con);
                return;
            }
        }
        self.fallback.record_success(key);
    }
}

// ---------------------------------------------------------------------------
// Tests for TenantRateLimitKey (Issue #854)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tenant_key_tests {
    use super::*;

    #[test]
    fn test_tenant_key_format() {
        let key = TenantRateLimitKey::new("tenant_a", "verify", "user42");
        assert_eq!(key.as_str(), "tenant_a::verify::user42");
    }

    #[test]
    fn test_tenant_key_display() {
        let key = TenantRateLimitKey::new("org1", "login", "bob");
        assert_eq!(format!("{key}"), "org1::login::bob");
    }

    #[test]
    fn test_tenant_key_as_ref() {
        let key = TenantRateLimitKey::new("t1", "action", "u1");
        let s: &str = key.as_ref();
        assert_eq!(s, "t1::action::u1");
    }

    #[test]
    fn test_different_tenants_same_user_get_different_keys() {
        let key_a = TenantRateLimitKey::new("tenant_a", "verify", "user1");
        let key_b = TenantRateLimitKey::new("tenant_b", "verify", "user1");
        assert_ne!(key_a, key_b);
        assert_ne!(key_a.as_str(), key_b.as_str());
    }

    #[test]
    fn test_cross_tenant_isolation_with_in_memory_limiter() {
        // Two tenants with the same user_id must have independent rate-limit state.
        let limiter = InMemoryRateLimiter::new(2, 60, 300);

        let key_a = TenantRateLimitKey::new("tenant_a", "verify", "shared_user");
        let key_b = TenantRateLimitKey::new("tenant_b", "verify", "shared_user");

        // Exhaust tenant_a's limit
        limiter.record_failure(key_a.as_str());
        limiter.record_failure(key_a.as_str());
        let result_a = limiter.record_failure(key_a.as_str());
        assert!(matches!(result_a, RateLimitResult::Blocked { .. }));

        // tenant_b should still be allowed
        let result_b = limiter.record_failure(key_b.as_str());
        assert!(matches!(result_b, RateLimitResult::Allowed { .. }));
    }

    #[test]
    fn test_same_tenant_different_actions_are_independent() {
        let limiter = InMemoryRateLimiter::new(1, 60, 300);

        let verify_key = TenantRateLimitKey::new("t1", "verify", "user1");
        let disable_key = TenantRateLimitKey::new("t1", "disable", "user1");

        // Exhaust verify limit
        limiter.record_failure(verify_key.as_str());
        let result_v = limiter.record_failure(verify_key.as_str());
        assert!(matches!(result_v, RateLimitResult::Blocked { .. }));

        // disable action should still be allowed
        let result_d = limiter.record_failure(disable_key.as_str());
        assert!(matches!(result_d, RateLimitResult::Allowed { .. }));
    }
}
