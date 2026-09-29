//! Storage keys and value types for claim-document status/version
//! semantics (Issue #1341).
//!
//! Only the *data types* live here; the `#[contractimpl]` methods that
//! operate on them (`submit_claim_document`, `supersede_claim_document`,
//! `revoke_claim_document`, `approve_claim`, ...) stay in `lib.rs` because
//! a contract's exported methods must share one `#[contractimpl]` block
//! (see `docs/module-boundaries.md`).
//!
//! Lifecycle (see `docs/claim-document-revocation.md`):
//!
//! ```text
//!   submit ──► Active ──supersede──► Superseded ──revoke──► Revoked
//!                 │                                            ▲
//!                 └──────────────────revoke────────────────────┘
//! ```
//!
//! `Revoked` is terminal. Only `Active` documents can back a new approval.

use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

/// Maximum number of document versions (including superseded and revoked
/// ones) that may be registered against a single claim.
pub const MAX_CLAIM_DOCUMENTS: u32 = 20;

/// Maximum byte length of a revocation reason.
pub const MAX_CLAIM_REVOCATION_REASON_LEN: u32 = 256;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimDocKey {
    /// claim_id -> pet_id the claim's documents are bound to.
    ClaimPet(u64),
    /// (claim_id, doc_index) -> ClaimDocumentRecord.
    Document((u64, u32)),
    /// claim_id -> number of document records registered (next index).
    DocumentCount(u64),
    /// claim_id -> ClaimSettlement (immutable once written).
    Settlement(u64),
    /// digest -> (claim_id, doc_index) of the revocation that burned it.
    RevokedDigest(BytesN<32>),
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimDocumentStatus {
    /// Current version; may back a new approval.
    Active,
    /// Replaced by a newer version (`superseded_by`); auditable only.
    Superseded,
    /// Invalidated; auditable only. Terminal.
    Revoked,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimDocumentRecord {
    pub claim_id: u64,
    pub doc_index: u32,
    pub digest: BytesN<32>,
    /// 1 for an original submission, previous version + 1 for a supersession.
    pub version: u32,
    /// Index of the document this version replaced, if any.
    pub supersedes: Option<u32>,
    /// Index of the document that replaced this one, if any.
    pub superseded_by: Option<u32>,
    pub status: ClaimDocumentStatus,
    pub submitted_by: Address,
    pub submitted_at: u64,
    pub status_changed_by: Option<Address>,
    pub status_changed_at: Option<u64>,
    pub revocation_reason: Option<String>,
}

/// Immutable snapshot of the documents that backed a claim approval.
///
/// Historical behavior: a settlement is never unwound by a later
/// revocation. The digests and versions captured here are exactly what the
/// approver accepted; `settlement_has_revoked_documents` lets auditors flag
/// settlements whose evidence was revoked afterwards.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimSettlement {
    pub claim_id: u64,
    pub pet_id: u64,
    pub approver: Address,
    pub approved_at: u64,
    pub doc_indices: Vec<u32>,
    pub doc_digests: Vec<BytesN<32>>,
    pub doc_versions: Vec<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimDocumentStatusEvent {
    pub version: u32,
    pub claim_id: u64,
    pub doc_index: u32,
    pub digest: BytesN<32>,
    pub doc_version: u32,
    pub status: ClaimDocumentStatus,
    pub actor: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimApprovedEvent {
    pub version: u32,
    pub claim_id: u64,
    pub pet_id: u64,
    pub approver: Address,
    pub doc_count: u32,
    pub timestamp: u64,
}
