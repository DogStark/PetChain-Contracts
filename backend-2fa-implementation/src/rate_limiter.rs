use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

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
