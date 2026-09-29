//! Storage keys and value types for confirmed, audited data-retention
//! purges (Issue #1344).
//!
//! `purge_records_confirmed` (in `lib.rs`) is irreversible, so it demands
//! more than an ordinary record update:
//!
//! - **Roles**: only the pet's current owner (`PurgeRole::Owner`) or an
//!   admin (`PurgeRole::Admin`) may purge. Vets, access-grant holders and
//!   custodians cannot, even for records they authored.
//! - **Nonce**: each caller has a monotonic purge nonce; the call must
//!   present the current value, so a purge can never be replayed.
//! - **Confirmation**: the caller must supply
//!   `compute_purge_confirmation(caller, pet_id, record_ids, nonce)`, which
//!   binds contract, account, pet, exact record set and nonce. A
//!   confirmation for one scope cannot authorize another.
//! - **Scope**: every record must belong to `pet_id`, be soft-deleted, be
//!   past the retention period, and not be under a purge hold. Validation
//!   is all-or-nothing.
//! - **Audit**: the stored entry and event carry only a count and a digest
//!   of `(record_id, canonical record hash)` pairs — never record contents.

use soroban_sdk::{contracttype, Address, BytesN};

pub const MAX_PURGE_BATCH: u32 = 25;
pub const PURGE_CONFIRMATION_DOMAIN: &[u8] = b"petchain:purge-confirmation:v1";
pub const PURGE_RECORDS_DIGEST_DOMAIN: &[u8] = b"petchain:purge-records:v1";

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PurgeKey {
    /// caller -> next expected purge nonce (starts at 0).
    Nonce(Address),
    /// record_id -> true while the record is under a purge hold.
    Hold(u64),
    /// Number of purge audit entries.
    AuditCount,
    /// audit_id -> PurgeAuditEntry. audit_id starts at 1.
    Audit(u64),
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PurgeRole {
    Owner,
    Admin,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurgeAuditEntry {
    pub audit_id: u64,
    pub pet_id: u64,
    pub purged_by: Address,
    pub role: PurgeRole,
    pub record_count: u32,
    /// sha256(domain || pet_id || count || (record_id || record_hash)*).
    pub records_digest: BytesN<32>,
    pub nonce: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurgeAuditEvent {
    pub version: u32,
    pub audit_id: u64,
    pub pet_id: u64,
    pub purged_by: Address,
    pub role: PurgeRole,
    pub record_count: u32,
    pub records_digest: BytesN<32>,
    pub nonce: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurgeHoldChangedEvent {
    pub version: u32,
    pub record_id: u64,
    pub held: bool,
    pub admin: Address,
    pub timestamp: u64,
}
