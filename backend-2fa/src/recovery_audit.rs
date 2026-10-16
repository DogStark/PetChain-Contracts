// ─── Recovery-code audit trail types and logic ─────────────────────
// Issue #1246: Append-only, redacted audit events for recovery code
// issuance, regeneration, use, and revocation.
// Issue #1460: Paginated audit export view exposing actors and event IDs
// while excluding applicant private data.

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

/// Maximum number of entries returned in a single audit export page.
pub const MAX_AUDIT_PAGE_SIZE: u32 = 100;

/// Error codes for the adoption case audit export view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditExportError {
    /// Caller is not authorized to read the audit log.
    Unauthorized,
    /// Requested page size is zero or exceeds `MAX_AUDIT_PAGE_SIZE`.
    InvalidPageSize,
    /// Requested page is out of range for the available entries.
    PageOutOfRange,
}

impl fmt::Display for AuditExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized => write!(f, "audit_export_unauthorized"),
            Self::InvalidPageSize => write!(f, "audit_export_invalid_page_size"),
            Self::PageOutOfRange => write!(f, "audit_export_page_out_of_range"),
        }
    }
}

/// A single redacted row in the adoption case audit export.
///
/// Exposes only the transition, actor, and event ID. Applicant private data
/// (names, contact details, free-text notes) is intentionally omitted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditExportRow {
    /// Stable event ID for this transition (equals the entry `seq`).
    pub event_id: u64,
    /// Actor who performed the transition.
    pub actor: String,
    /// Human-readable transition label.
    pub transition: String,
    /// Unix timestamp (seconds) of the transition.
    pub timestamp: u64,
}

/// A paginated page of redacted audit export rows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditExportPage {
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
    pub rows: Vec<AuditExportRow>,
}

/// Builds a paginated, redacted audit export for a user's case transitions.
///
/// Authorization: `caller` must equal `query.user_id` or be the literal
/// `"admin"` actor. Unauthorized callers receive `AuditExportError::Unauthorized`
/// and no rows are produced, leaving state unchanged.
///
/// Bounds: `page_size` must be in `1..=MAX_AUDIT_PAGE_SIZE`; iteration is
/// limited to the requested page window so persistent state growth is bounded.
pub fn export_case_audit(
    caller: &str,
    query: &RecoveryAuditQuery,
    entries: &[RecoveryAuditEntry],
) -> Result<AuditExportPage, AuditExportError> {
    if caller != query.user_id && caller != "admin" {
        return Err(AuditExportError::Unauthorized);
    }
    if query.page_size == 0 || query.page_size > MAX_AUDIT_PAGE_SIZE {
        return Err(AuditExportError::InvalidPageSize);
    }

    let total = entries.len() as u64;
    let start = (query.page as u64) * (query.page_size as u64);
    if start > total {
        return Err(AuditExportError::PageOutOfRange);
    }
    let end = (start + query.page_size as u64).min(total);

    let rows = entries[start as usize..end as usize]
        .iter()
        .map(|entry| AuditExportRow {
            event_id: entry.seq,
            actor: entry.actor.clone(),
            transition: entry.action.to_string(),
            timestamp: entry.timestamp,
        })
        .collect();

    Ok(AuditExportPage {
        page: query.page,
        page_size: query.page_size,
        total,
        rows,
    })
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

    fn sample_entries(n: u64) -> Vec<RecoveryAuditEntry> {
        (0..n)
            .map(|i| RecoveryAuditEntry {
                seq: i + 1,
                user_id: "user-42".into(),
                actor: "user-42".into(),
                action: RecoveryAuditAction::Used { code_index: i as usize },
                code_hash_prefix: "a1b2c3d4".into(),
                codes_before: 8,
                codes_after: 7,
                request_id: format!("req-{}", i),
                outcome: RecoveryAuditOutcome::Success,
                timestamp: 1_700_000_000 + i,
            })
            .collect()
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
            actor: "admin".into(),
            action: RecoveryAuditAction::Revoked,
            code_hash_prefix: String::new(),
            codes_before: 7,
            codes_after: 0,
            request_id: "req-jkl-012".into(),
            outcome: RecoveryAuditOutcome::Success,
            timestamp: 1_700_000_300,
        };
        assert_eq!(entry.codes_after, 0);
        assert!(validate_no_plaintext(&entry).is_ok());
    }

    #[test]
    fn export_returns_redacted_rows_for_owner() {
        let entries = sample_entries(3);
        let query = RecoveryAuditQuery {
            user_id: "user-42".into(),
            page: 0,
            page_size: 10,
        };
        let page = export_case_audit("user-42", &query, &entries).unwrap();
        assert_eq!(page.total, 3);
        assert_eq!(page.rows.len(), 3);
        assert_eq!(page.rows[0].event_id, 1);
        assert_eq!(page.rows[0].actor, "user-42");
        assert_eq!(page.rows[0].transition, "recovery_code_used(index=0)");
    }

    #[test]
    fn export_allows_admin_actor() {
        let entries = sample_entries(2);
        let query = RecoveryAuditQuery {
            user_id: "user-42".into(),
            page: 0,
            page_size: 10,
        };
        assert!(export_case_audit("admin", &query, &entries).is_ok());
    }

    #[test]
    fn export_rejects_unauthorized_caller() {
        let entries = sample_entries(2);
        let query = RecoveryAuditQuery {
            user_id: "user-42".into(),
            page: 0,
            page_size: 10,
        };
        assert_eq!(
            export_case_audit("user-99", &query, &entries),
            Err(AuditExportError::Unauthorized)
        );
    }

    #[test]
    fn export_rejects_zero_page_size() {
        let entries = sample_entries(2);
        let query = RecoveryAuditQuery {
            user_id: "user-42".into(),
            page: 0,
            page_size: 0,
        };
        assert_eq!(
            export_case_audit("user-42", &query, &entries),
            Err(AuditExportError::InvalidPageSize)
        );
    }

    #[test]
    fn export_rejects_oversized_page_size() {
        let entries = sample_entries(2);
        let query = RecoveryAuditQuery {
            user_id: "user-42".into(),
            page: 0,
            page_size: MAX_AUDIT_PAGE_SIZE + 1,
        };
        assert_eq!(
            export_case_audit("user-42", &query, &entries),
            Err(AuditExportError::InvalidPageSize)
        );
    }

    #[test]
    fn export_paginates_within_bounds() {
        let entries = sample_entries(5);
        let query = RecoveryAuditQuery {
            user_id: "user-42".into(),
            page: 1,
            page_size: 2,
        };
        let page = export_case_audit("user-42", &query, &entries).unwrap();
        assert_eq!(page.total, 5);
        assert_eq!(page.rows.len(), 2);
        assert_eq!(page.rows[0].event_id, 3);
        assert_eq!(page.rows[1].event_id, 4);
    }

    #[test]
    fn export_rejects_out_of_range_page() {
        let entries = sample_entries(2);
        let query = RecoveryAuditQuery {
            user_id: "user-42".into(),
            page: 5,
            page_size: 10,
        };
        assert_eq!(
            export_case_audit("user-42", &query, &entries),
            Err(AuditExportError::PageOutOfRange)
        );
    }

    #[test]
    fn export_failed_call_leaves_entries_unchanged() {
        let entries = sample_entries(2);
        let before = entries.clone();
        let query = RecoveryAuditQuery {
            user_id: "user-42".into(),
            page: 0,
            page_size: 0,
        };
        assert!(export_case_audit("user-42", &query, &entries).is_err());
        assert_eq!(entries.len(), before.len());
        assert_eq!(entries[0].seq, before[0].seq);
        assert_eq!(entries[1].seq, before[1].seq);
    }

    #[test]
    fn export_empty_log_returns_empty_page() {
        let entries: Vec<RecoveryAuditEntry> = Vec::new();
        let query = RecoveryAuditQuery {
            user_id: "user-42".into(),
            page: 0,
            page_size: 10,
        };
        let page = export_case_audit("user-42", &query, &entries).unwrap();
        assert_eq!(page.total, 0);
        assert!(page.rows.is_empty());
    }
}
