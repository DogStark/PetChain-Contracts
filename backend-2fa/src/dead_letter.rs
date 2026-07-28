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

use crate::webhooks::WebhookPayload;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum number of entries retained in the dead-letter queue.
///
/// When the DLQ is full, the **oldest** entry is evicted to make room for
/// the newest, keeping the DLQ bounded regardless of how many permanent
/// failures occur.
pub const MAX_DLQ_SIZE: usize = 500;

/// A single entry in the Dead-Letter Queue.
///
/// Populated when a webhook delivery exhausts all retry attempts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqEntry {
    /// Unique, monotonically-increasing identifier within a single process
    /// lifetime.  Not persistent across restarts.
    pub id: usize,
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
}

impl DlqEntry {
    /// Construct a new `DlqEntry` from the components available at the point
    /// of permanent failure inside `deliver_one`.
    pub fn new(
        id: usize,
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
            url: url.into(),
            payload,
            body: body.into(),
            failure_reason: failure_reason.into(),
            failed_at,
            replay_attempts: 0,
        }
    }
}
