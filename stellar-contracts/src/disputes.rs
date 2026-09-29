//! Storage keys and value types for the dispute-resolution domain
//! (Issue #1146, phase 1 of the module split).
//!
//! Only the *data types* live here — the `#[contractimpl]` methods that
//! operate on them (`raise_dispute`, `vote_on_dispute`, `submit_evidence`,
//! `resolve_dispute`, ...) remain in `lib.rs`. Soroban's `#[contractimpl]`
//! macro does not currently support splitting a contract's methods across
//! more than one `impl` block/file for the same contract type (see
//! stellar/rs-soroban-sdk#1360, an open upstream feature request), so the
//! method bodies cannot be moved out without risking a build break that
//! can't be verified without a Rust toolchain. See
//! `docs/abi-migrations.md` for the full migration note.
//!
//! No public type path, field, or discriminant changed by this move: every
//! type below is re-exported at the crate root via `pub use disputes::*;`
//! in `lib.rs`, so `crate::Dispute`, `crate::DisputeKey`, etc. resolve
//! exactly as they did before the split.

use soroban_sdk::{contracttype, Address, BytesN, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArbitratorStats {
    pub address: Address,
    pub reputation: i64,
    pub total_rulings: u64,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DisputeStatus {
    Pending = 1,
    EvidencePhase = 2,
    ResolvedInFavorOfClaimer = 3,
    ResolvedInFavorOfTarget = 4,
    Cancelled = 5,
}

/// A stakeholder's vote on a dispute resolution.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DisputeVote {
    Approve = 1,
    Reject = 2,
}

/// A single recorded vote on a dispute, tracking who voted and how.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeVoteRecord {
    pub voter: Address,
    pub vote: DisputeVote,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dispute {
    pub dispute_id: u64,
    pub pet_id: u64,
    pub claimer: Address,
    pub target: Address,
    pub amount: u64,
    pub reason: String,
    pub evidence_hash: String,
    pub status: DisputeStatus,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evidence {
    pub evidence_id: u64,
    pub submitter: Address,
    pub cid: String,
    pub sha256_hash: BytesN<32>,
}

#[contracttype]
pub enum DisputeKey {
    Dispute(u64),
    DisputeCount,
    AppealWindow,
    Arbitrator,
    PetDisputesCount(u64),
    PetDisputesIndex((u64, u64)),
    DisputeEvidence(u64, u64),
    DisputeEvidenceCount(u64),
    PartyEvidenceCount(u64, Address),
    /// Vote cast by a given address on a given dispute.
    DisputeVoteByVoter(u64, Address),
    /// Ordered list of addresses that have voted on a dispute (for enumeration).
    DisputeVoters(u64),
}
