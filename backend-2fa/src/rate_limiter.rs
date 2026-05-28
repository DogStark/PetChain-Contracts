use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use redis::Commands;

/// Result returned when checking a rate limit for a given key.
#[derive(Debug, PartialEq)]
pub enum RateLimitResult {
    /// The attempt is allowed. Contains remaining attempts in the window.
    Allowed { remaining: u32 },
    /// The key is locked out. Contains seconds until the lockout expires.
    Blocked { retry_after_secs: u64 },
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
        Self { window_secs, max_failures, lockout_secs }
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
    fn sliding_window_add(&self, key: &str, now_ms: u64, cutoff_ms: u64, member: &str, ttl_secs: u64) -> u64;
    /// Set `key` with a TTL (seconds).
    fn set_ex(&self, key: &str, value: &str, ttl_secs: u64);
    /// Delete one or more keys.
    fn del(&self, keys: &[&str]);
}

// ---------------------------------------------------------------------------
// Live Redis backend
// ---------------------------------------------------------------------------

pub struct LiveRedisBackend {
    client: redis::Client,
}

impl LiveRedisBackend {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self { client: redis::Client::open(redis_url)? })
    }
}

impl RedisBackend for LiveRedisBackend {
    fn ttl(&self, key: &str) -> i64 {
        let mut con = match self.client.get_connection() {
            Ok(c) => c,
            Err(_) => return -2,
        };
        con.ttl(key).unwrap_or(-2)
    }

    fn sliding_window_add(&self, key: &str, now_ms: u64, cutoff_ms: u64, member: &str, ttl_secs: u64) -> u64 {
        let mut con = match self.client.get_connection() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[LiveRedisBackend] connection error: {e}");
                return 0;
            }
        };
        let result: redis::RedisResult<(u64,)> = (|| {
            let mut pipe = redis::pipe();
            pipe.cmd("ZREMRANGEBYSCORE").arg(key).arg(0u64).arg(cutoff_ms).ignore()
                .cmd("ZADD").arg(key).arg(now_ms).arg(member).ignore()
                .cmd("ZCARD").arg(key)
                .cmd("EXPIRE").arg(key).arg(ttl_secs).ignore();
            pipe.query(&mut con)
        })();
        match result {
            Ok((card,)) => card,
            Err(e) => {
                eprintln!("[LiveRedisBackend] pipeline error: {e}");
                0
            }
        }
    }

    fn set_ex(&self, key: &str, value: &str, ttl_secs: u64) {
        if let Ok(mut con) = self.client.get_connection() {
            let _: Result<(), _> = con.set_ex(key, value, ttl_secs);
        }
    }

    fn del(&self, keys: &[&str]) {
        if let Ok(mut con) = self.client.get_connection() {
            let _: Result<(), _> = redis::cmd("DEL").arg(keys).query(&mut con);
        }
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
}

/// In-process mock that faithfully implements the sorted-set sliding window.
pub struct MockRedisBackend {
    store: Mutex<HashMap<String, MockEntry>>,
    /// Injected "current time" for deterministic tests.
    now_ms: Mutex<u64>,
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
                    if now_ms >= exp_ms { -2 }
                    else { ((exp_ms - now_ms + 999) / 1_000) as i64 } // ceiling division → secs
                }
            },
        }
    }

    fn sliding_window_add(&self, key: &str, _now_ms: u64, cutoff_ms: u64, member: &str, ttl_secs: u64) -> u64 {
        let now_ms = self.current_ms();
        let mut store = self.store.lock().unwrap();
        let entry = store.entry(key.to_string()).or_insert(MockEntry {
            zset: Vec::new(),
            expires_at_ms: None,
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
        store.insert(key.to_string(), MockEntry {
            zset: vec![(0, value.to_string())],
            expires_at_ms: Some(now_ms + ttl_secs * 1_000),
        });
    }

    fn del(&self, keys: &[&str]) {
        let mut store = self.store.lock().unwrap();
        for k in keys {
            store.remove(*k);
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
        let mut records = self.records.lock().expect("rate limiter lock poisoned");
        let now = Instant::now();

        let record = records.entry(key.to_string()).or_insert(AttemptRecord {
            failures: 0,
            window_start: now,
            locked_until: None,
        });

        if let Some(locked_until) = record.locked_until {
            if now < locked_until {
                let retry_after_secs = (locked_until - now).as_secs().max(1);
                return RateLimitResult::Blocked { retry_after_secs };
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

        if record.failures >= self.max_failures {
            record.locked_until = Some(now + self.lockout);
            RateLimitResult::Blocked { retry_after_secs: self.lockout.as_secs() }
        } else {
            RateLimitResult::Allowed { remaining: self.max_failures - record.failures }
        }
    }

    fn record_success(&self, key: &str) {
        let mut records = self.records.lock().expect("rate limiter lock poisoned");
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
        Self { backend, default, endpoints: HashMap::new() }
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

        let lockout_ttl = self.backend.ttl(&lockout_key);
        if lockout_ttl > 0 {
            return RateLimitResult::Blocked { retry_after_secs: lockout_ttl as u64 };
        }

        let now_ms = Self::now_ms();
        let cutoff_ms = now_ms.saturating_sub(cfg.window_secs * 1_000);
        let member = Self::unique_member();

        let count = self.backend.sliding_window_add(&window_key, now_ms, cutoff_ms, &member, cfg.window_secs);

        if count >= cfg.max_failures as u64 {
            self.backend.set_ex(&lockout_key, "1", cfg.lockout_secs);
            return RateLimitResult::Blocked { retry_after_secs: cfg.lockout_secs };
        }

        RateLimitResult::Allowed { remaining: cfg.max_failures - count as u32 }
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
        Ok(Self { inner: SlidingWindowRateLimiter::new(backend, cfg) })
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
