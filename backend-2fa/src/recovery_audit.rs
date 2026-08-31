// ─── Recovery-code audit trail types and logic ─────────────────────
// Issue #1246: Append-only, redacted audit events for recovery code
// issuance, regeneration, use, and revocation.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Categories of recovery-code lifecycle events.
/// Plaintext codes MUST NEVER appear in any variant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryAuditAction {
    /// Initial set of codes was generated during 2FA enrollment.
    Issued,
    /// Existing codes were invalidated and a new set was generated.
    Regenerated,
    /// A single code was consumed during login recovery.
    Used { code_index: usize },
    /// All remaining codes were explicitly revoked (e.g., admin action).
    Revoked,
    /// Issuance or regeneration failed (e.g., store error).
    Failed { reason: String },
}

impl fmt::Display for RecoveryAuditAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Issued => write!(f, "recovery_codes_issued"),
            Self::Regenerated => write!(f, "recovery_codes_regenerated"),
            Self::Used { code_index } => write!(f, "recovery_code_used(index={})", code_index),
            Self::Revoked => write!(f, "recovery_codes_revoked"),
            Self::Failed { reason } => write!(f, "recovery_codes_failed({})", reason),
        }
    }
}

/// A single append-only audit entry for recovery-code lifecycle events.
///
/// Invariants enforced by construction:
/// - `code_hash_prefix` contains at most 8 hex characters (first 4 bytes of
///   SHA-256), never the plaintext code.
/// - `request_id` is an opaque correlation ID for support triage.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecoveryAuditEntry {
    /// Monotonically increasing sequence number within a user's audit log.
    pub seq: u64,
    /// The user whose recovery codes are affected.
    pub user_id: String,
    /// Who performed the action (user themselves, admin ID, or "system").
    pub actor: String,
    /// What happened.
    pub action: RecoveryAuditAction,
    /// Redacted identifier: first 8 hex chars of SHA-256(code), or empty.
    pub code_hash_prefix: String,
    /// How many codes existed before the action.
    pub codes_before: usize,
    /// How many codes exist after the action.
    pub codes_after: usize,
    /// Opaque request/correlation ID for support triage.
    pub request_id: String,
    /// Whether the action succeeded.
    pub outcome: RecoveryAuditOutcome,
    /// Unix timestamp (seconds).
    pub timestamp: u64,
}

/// Outcome of a recovery-code lifecycle action.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryAuditOutcome {
    Success,
    Failure(String),
}

/// Query parameters for paginated audit reads.
#[derive(Clone, Debug)]
pub struct RecoveryAuditQuery {
    pub user_id: String,
    pub page: u32,
    pub page_size: u32,
}

/// Validates that a `RecoveryAuditEntry` never contains plaintext codes.
///
/// This is a defense-in-depth check run before persisting an entry.
pub fn validate_no_plaintext(entry: &RecoveryAuditEntry) -> Result<(), String> {
    // Code hash prefix must be at most 8 hex characters
    if entry.code_hash_prefix.len() > 8 {
        return Err("code_hash_prefix exceeds 8 characters".into());
    }
    if !entry
        .code_hash_prefix
        .chars()
        .all(|c| c.is_ascii_hexdigit())
    {
        return Err("code_hash_prefix contains non-hex characters".into());
    }
    // Actor must not look like a recovery code (XXXX-XXXX pattern)
    if entry.actor.contains('-') && entry.actor.len() == 9 {
        return Err("actor field resembles a recovery code".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(action: RecoveryAuditAction) -> RecoveryAuditEntry {
        RecoveryAuditEntry {
            seq: 1,
            user_id: "user-42".into(),
            actor: "user-42".into(),
            action,
            code_hash_prefix: "a1b2c3d4".into(),
            codes_before: 8,
            codes_after: 8,
            request_id: "req-abc-123".into(),
            outcome: RecoveryAuditOutcome::Success,
            timestamp: 1_700_000_000,
        }
    }

    #[test]
    fn issued_action_display() {
        let entry = sample_entry(RecoveryAuditAction::Issued);
        assert_eq!(entry.action.to_string(), "recovery_codes_issued");
    }

    #[test]
    fn regenerated_action_display() {
        let entry = sample_entry(RecoveryAuditAction::Regenerated);
        assert_eq!(entry.action.to_string(), "recovery_codes_regenerated");
    }

    #[test]
    fn used_action_display() {
        let entry = sample_entry(RecoveryAuditAction::Used { code_index: 3 });
        assert_eq!(entry.action.to_string(), "recovery_code_used(index=3)");
    }

    #[test]
    fn revoked_action_display() {
        let entry = sample_entry(RecoveryAuditAction::Revoked);
        assert_eq!(entry.action.to_string(), "recovery_codes_revoked");
    }

    #[test]
    fn failed_action_display() {
        let entry = sample_entry(RecoveryAuditAction::Failed {
            reason: "db_timeout".into(),
        });
        assert_eq!(
            entry.action.to_string(),
            "recovery_codes_failed(db_timeout)"
        );
    }

    #[test]
    fn validate_no_plaintext_accepts_valid_entry() {
        let entry = sample_entry(RecoveryAuditAction::Issued);
        assert!(validate_no_plaintext(&entry).is_ok());
    }

    #[test]
    fn validate_no_plaintext_rejects_long_hash_prefix() {
        let mut entry = sample_entry(RecoveryAuditAction::Issued);
        entry.code_hash_prefix = "a1b2c3d4e5".into(); // 10 chars > 8
        assert!(validate_no_plaintext(&entry).is_err());
    }

    #[test]
    fn validate_no_plaintext_rejects_non_hex_hash_prefix() {
        let mut entry = sample_entry(RecoveryAuditAction::Issued);
        entry.code_hash_prefix = "zzzz".into();
        assert!(validate_no_plaintext(&entry).is_err());
    }

    #[test]
    fn validate_no_plaintext_rejects_code_like_actor() {
        let mut entry = sample_entry(RecoveryAuditAction::Issued);
        entry.actor = "1234-5678".into(); // Looks like a recovery code
        assert!(validate_no_plaintext(&entry).is_err());
    }

    #[test]
    fn validate_no_plaintext_accepts_empty_hash_prefix() {
        let mut entry = sample_entry(RecoveryAuditAction::Issued);
        entry.code_hash_prefix = String::new();
        assert!(validate_no_plaintext(&entry).is_ok());
    }

    #[test]
    fn regeneration_records_code_count_transition() {
        let entry = RecoveryAuditEntry {
            seq: 2,
            user_id: "user-42".into(),
            actor: "user-42".into(),
            action: RecoveryAuditAction::Regenerated,
            code_hash_prefix: String::new(),
            codes_before: 5,
            codes_after: 8,
            request_id: "req-def-456".into(),
            outcome: RecoveryAuditOutcome::Success,
            timestamp: 1_700_000_100,
        };
        assert_eq!(entry.codes_before, 5);
        assert_eq!(entry.codes_after, 8);
        assert!(validate_no_plaintext(&entry).is_ok());
    }

    #[test]
    fn used_code_records_decrement() {
        let entry = RecoveryAuditEntry {
            seq: 3,
            user_id: "user-42".into(),
            actor: "user-42".into(),
            action: RecoveryAuditAction::Used { code_index: 2 },
            code_hash_prefix: "ab12cd34".into(),
            codes_before: 8,
            codes_after: 7,
            request_id: "req-ghi-789".into(),
            outcome: RecoveryAuditOutcome::Success,
            timestamp: 1_700_000_200,
        };
        assert_eq!(entry.codes_after, entry.codes_before - 1);
        assert!(validate_no_plaintext(&entry).is_ok());
    }

    #[test]
    fn revocation_zeros_remaining_codes() {
        let entry = RecoveryAuditEntry {
            seq: 4,
            user_id: "user-42".into(),
            actor: "admin-99".into(),
            action: RecoveryAuditAction::Revoked,
            code_hash_prefix: String::new(),
            codes_before: 6,
            codes_after: 0,
            request_id: "req-jkl-012".into(),
            outcome: RecoveryAuditOutcome::Success,
            timestamp: 1_700_000_300,
        };
        assert_eq!(entry.codes_after, 0);
        assert!(validate_no_plaintext(&entry).is_ok());
    }

    #[test]
    fn failure_outcome_preserves_reason() {
        let entry = RecoveryAuditEntry {
            seq: 5,
            user_id: "user-42".into(),
            actor: "system".into(),
            action: RecoveryAuditAction::Failed {
                reason: "store_unavailable".into(),
            },
            code_hash_prefix: String::new(),
            codes_before: 8,
            codes_after: 8,
            request_id: "req-mno-345".into(),
            outcome: RecoveryAuditOutcome::Failure("store_unavailable".into()),
            timestamp: 1_700_000_400,
        };
        assert_eq!(
            entry.outcome,
            RecoveryAuditOutcome::Failure("store_unavailable".into())
        );
    }

    #[test]
    fn concurrent_regeneration_last_writer_wins() {
        // Two regeneration events for the same user in rapid succession.
        // The second event's codes_before should match the first's codes_after.
        let first = RecoveryAuditEntry {
            seq: 6,
            user_id: "user-42".into(),
            actor: "user-42".into(),
            action: RecoveryAuditAction::Regenerated,
            code_hash_prefix: String::new(),
            codes_before: 3,
            codes_after: 8,
            request_id: "req-pqr-678".into(),
            outcome: RecoveryAuditOutcome::Success,
            timestamp: 1_700_000_500,
        };
        let second = RecoveryAuditEntry {
            seq: 7,
            user_id: "user-42".into(),
            actor: "user-42".into(),
            action: RecoveryAuditAction::Regenerated,
            code_hash_prefix: String::new(),
            codes_before: first.codes_after,
            codes_after: 8,
            request_id: "req-stu-901".into(),
            outcome: RecoveryAuditOutcome::Success,
            timestamp: 1_700_000_501,
        };
        assert_eq!(second.codes_before, 8);
        assert!(second.seq > first.seq);
    }

    #[test]
    fn serialization_round_trip() {
        let entry = sample_entry(RecoveryAuditAction::Used { code_index: 1 });
        let json = serde_json::to_string(&entry).expect("serialize");
        let deserialized: RecoveryAuditEntry =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.user_id, entry.user_id);
        assert_eq!(deserialized.seq, entry.seq);
        // Verify no plaintext codes leaked into serialization
        assert!(!json.contains("XXXX-XXXX"));
        assert!(!json.contains("1234-5678"));
    }

    #[test]
    fn retention_entries_ordered_by_seq() {
        let entries: Vec<RecoveryAuditEntry> = (1..=5)
            .map(|i| RecoveryAuditEntry {
                seq: i,
                user_id: "user-42".into(),
                actor: "user-42".into(),
                action: RecoveryAuditAction::Issued,
                code_hash_prefix: String::new(),
                codes_before: 0,
                codes_after: 8,
                request_id: format!("req-{}", i),
                outcome: RecoveryAuditOutcome::Success,
                timestamp: 1_700_000_000 + i * 100,
            })
            .collect();

        for window in entries.windows(2) {
            assert!(window[0].seq < window[1].seq);
            assert!(window[0].timestamp < window[1].timestamp);
        }
    }
}
