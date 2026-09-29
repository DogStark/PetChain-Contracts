// ---------------------------------------------------------------------------
// Dead-Letter Queue for webhook deliveries (Issue #1058)
// ---------------------------------------------------------------------------
//
// When a webhook delivery exhausts all retry attempts and reaches
// `PermanentFailure`, the failed payload is written into a bounded
// `VecDeque` called the Dead-Letter Queue (DLQ).  The DLQ can be inspected
// via `GET /admin/webhooks/dead-letter` and replayed via
// `POST /admin/webhooks/dead-letter/replay`.
//
// Key properties
// ──────────────
// • Bounded at `MAX_DLQ_SIZE` entries.  When full, the oldest entry is
//   evicted before the newest is appended (ring-buffer semantics).
// • Entries carry the original serialised payload body, the target URL,
//   the failure reason, and the Unix timestamp at which permanent failure
//   was declared.
// • Replay re-queues each entry through the normal `deliver_one` path; on
//   success the entry is removed from the DLQ, on failure it stays (and its
//   `replay_attempts` counter is incremented).
//
// Delivery guarantees (Issue #1262)
// ─────────────────────────────────
// Security events are persisted to a durable outbox *before* any delivery
// attempt, so a crash between commit and send cannot lose an event.  Each
// event carries an idempotent `delivery_key`; consumers dedupe on that key
// and acknowledge via `acknowledge`, which never mutates the source audit
// record.  Retry/backoff is bounded by `MAX_DELIVERY_ATTEMPTS` and
// `MAX_BACKOFF_SECS`; once exhausted the event is dead-lettered and retained
// for at most `DLQ_RETENTION_SECS` before it becomes eligible for replay or
// eviction.

use crate::webhooks::WebhookPayload;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum number of entries retained in the dead-letter queue.
///
/// When the DLQ is full, the **oldest** entry is evicted to make room for
/// the newest, keeping the DLQ bounded regardless of how many permanent
/// failures occur.
pub const MAX_DLQ_SIZE: usize = 500;

/// Maximum number of delivery attempts (initial send + retries) before an
/// event is declared permanently failed and dead-lettered.
pub const MAX_DELIVERY_ATTEMPTS: u32 = 8;

/// Upper bound on the exponential backoff delay, in seconds.  Backoff is
/// `min(base * 2^attempt, MAX_BACKOFF_SECS)` so retries stay bounded.
pub const MAX_BACKOFF_SECS: u64 = 3600;

/// How long a dead-lettered event is retained before it becomes eligible for
/// eviction.  Replay is always permitted while the entry is present.
pub const DLQ_RETENTION_SECS: u64 = 7 * 24 * 3600;

/// Compute the bounded exponential backoff delay for a given attempt.
///
/// `attempt` is zero-based (0 = first retry).  The result never exceeds
/// `MAX_BACKOFF_SECS`, keeping retry behaviour bounded and documented.
pub fn backoff_secs(attempt: u32) -> u64 {
    let base: u64 = 1;
    base.saturating_mul(1u64 << attempt.min(32)).min(MAX_BACKOFF_SECS)
}

/// A single entry in the Dead-Letter Queue.
///
/// Populated when a webhook delivery exhausts all retry attempts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqEntry {
    /// Unique, monotonically-increasing identifier within a single process
    /// lifetime.  Not persistent across restarts.
    pub id: usize,
    /// Idempotent delivery key for the originating security event.  Consumers
    /// dedupe on this key so duplicate deliveries are harmless.
    pub delivery_key: String,
    /// URL that the delivery was targeting when it failed permanently.
    pub url: String,
    /// The original payload that failed to be delivered.
    pub payload: WebhookPayload,
    /// The raw JSON body that was sent (or attempted), kept so replay can
    /// retransmit exactly the same bytes without re-serialising.
    pub body: String,
    /// Last error message returned by the HTTP client.
    pub failure_reason: String,
    /// Unix timestamp (seconds) when permanent failure was recorded.
    pub failed_at: u64,
    /// How many times this entry has been attempted via the replay endpoint.
    /// Starts at 0; incremented on every replay attempt regardless of result.
    pub replay_attempts: u32,
    /// Whether a consumer has acknowledged this event.  Acknowledgement is
    /// recorded here only; the source audit record is never modified.
    pub acknowledged: bool,
}

impl DlqEntry {
    /// Construct a new `DlqEntry` from the components available at the point
    /// of permanent failure inside `deliver_one`.
    pub fn new(
        id: usize,
        delivery_key: impl Into<String>,
        url: impl Into<String>,
        payload: WebhookPayload,
        body: impl Into<String>,
        failure_reason: impl Into<String>,
    ) -> Self {
        let failed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            id,
            delivery_key: delivery_key.into(),
            url: url.into(),
            payload,
            body: body.into(),
            failure_reason: failure_reason.into(),
            failed_at,
            replay_attempts: 0,
            acknowledged: false,
        }
    }

    /// Mark this event as acknowledged by a consumer.
    ///
    /// Acknowledgement only flips a flag on the outbox/DLQ entry; the source
    /// audit record is left untouched, satisfying the "ack without mutating
    /// the audit record" guarantee.
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }

    /// Whether this entry has exceeded `DLQ_RETENTION_SECS` and is therefore
    /// eligible for eviction.
    pub fn is_expired(&self, now: u64) -> bool {
        now.saturating_sub(self.failed_at) >= DLQ_RETENTION_SECS
    }
}
