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
///
/// Implement this trait to back the limiter with Redis, a database, or
/// any other store. The in-process [`InMemoryRateLimiter`] is provided
/// for development and testing.
pub trait RateLimiter: Send + Sync {
    /// Record a failed attempt for `key` and return whether further
    /// attempts are currently allowed.
    fn record_failure(&self, key: &str) -> RateLimitResult;

    /// Clear the failure counter for `key` on a successful attempt.
    fn record_success(&self, key: &str);
}

// ---------------------------------------------------------------------------
// In-memory implementation
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct AttemptRecord {
    failures: u32,
    window_start: Instant,
    locked_until: Option<Instant>,
}

/// Thread-safe in-memory rate limiter using a sliding window + lockout.
///
/// Configuration:
/// - `max_failures`  — max failed attempts before lockout (default 5)
/// - `window`        — rolling window for counting failures (default 60 s)
/// - `lockout`       — how long to block after hitting the limit (default 300 s)
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
    /// Sensible production defaults: 5 failures / 60 s → 300 s lockout.
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

        // Already locked out?
        if let Some(locked_until) = record.locked_until {
            if now < locked_until {
                let retry_after_secs = (locked_until - now).as_secs().max(1);
                return RateLimitResult::Blocked { retry_after_secs };
            } else {
                // Lockout expired — reset
                record.failures = 0;
                record.window_start = now;
                record.locked_until = None;
            }
        }

        // Roll the window if it has elapsed
        if now.duration_since(record.window_start) >= self.window {
            record.failures = 0;
            record.window_start = now;
        }

        record.failures += 1;

        if record.failures >= self.max_failures {
            record.locked_until = Some(now + self.lockout);
            RateLimitResult::Blocked {
                retry_after_secs: self.lockout.as_secs(),
            }
        } else {
            let remaining = self.max_failures - record.failures;
            RateLimitResult::Allowed { remaining }
        }
    }

    fn record_success(&self, key: &str) {
        let mut records = self.records.lock().expect("rate limiter lock poisoned");
        records.remove(key);
    }
}

// ---------------------------------------------------------------------------
// Redis-backed implementation
// ---------------------------------------------------------------------------

/// Redis-backed rate limiter using the INCR + EXPIRE pattern.
///
/// State survives server restarts and is shared across multiple processes,
/// making this suitable for production deployments.
///
/// Key schema (for a given `key`):
/// - `rate:{key}:failures` — integer counter, TTL = `window_secs`
/// - `rate:{key}:lockout`  — exists while locked out, TTL = `lockout_secs`
///
/// On any Redis connectivity error the limiter **fails open** (returns
/// `Allowed`) to avoid locking out users during an outage.
pub struct RedisRateLimiter {
    client: redis::Client,
    max_failures: u32,
    window_secs: u64,
    lockout_secs: u64,
}

impl RedisRateLimiter {
    /// Create a new `RedisRateLimiter`.
    ///
    /// `redis_url` must be a valid Redis URL such as `redis://127.0.0.1:6379`.
    /// This validates the URL format but does not open a connection immediately.
    pub fn new(
        redis_url: &str,
        max_failures: u32,
        window_secs: u64,
        lockout_secs: u64,
    ) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            client,
            max_failures,
            window_secs,
            lockout_secs,
        })
    }
}

impl RateLimiter for RedisRateLimiter {
    fn record_failure(&self, key: &str) -> RateLimitResult {
        let mut con = match self.client.get_connection() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[RedisRateLimiter] connection error: {e}");
                return RateLimitResult::Allowed {
                    remaining: self.max_failures,
                };
            }
        };

        let lockout_key = format!("rate:{key}:lockout");
        let failure_key = format!("rate:{key}:failures");

        // Check active lockout.
        let lockout_ttl: i64 = con.ttl(&lockout_key).unwrap_or(-2);
        if lockout_ttl > 0 {
            return RateLimitResult::Blocked {
                retry_after_secs: lockout_ttl as u64,
            };
        }

        // Increment failure counter.
        let count: u64 = match con.incr(&failure_key, 1u64) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("[RedisRateLimiter] incr error: {e}");
                return RateLimitResult::Allowed {
                    remaining: self.max_failures,
                };
            }
        };

        // On first failure start the sliding window TTL.
        if count == 1 {
            let _: Result<(), _> = con.expire(&failure_key, self.window_secs as i64);
        }

        // Impose lockout when the threshold is reached.
        if count >= self.max_failures as u64 {
            let _: Result<(), _> =
                con.set_ex(&lockout_key, "1", self.lockout_secs);
            return RateLimitResult::Blocked {
                retry_after_secs: self.lockout_secs,
            };
        }

        RateLimitResult::Allowed {
            remaining: self.max_failures - count as u32,
        }
    }

    fn record_success(&self, key: &str) {
        let mut con = match self.client.get_connection() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[RedisRateLimiter] connection error on success: {e}");
                return;
            }
        };

        let lockout_key = format!("rate:{key}:lockout");
        let failure_key = format!("rate:{key}:failures");

        // Delete both keys with a single DEL command (atomic, no-op for missing keys).
        let _: Result<(), _> = redis::cmd("DEL")
            .arg(&lockout_key)
            .arg(&failure_key)
            .query(&mut con);
    }
}
