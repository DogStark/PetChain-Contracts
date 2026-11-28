//! Canonical consent purpose/scope encoding and versioned consent hashes
//! (Issue #1337).
//!
//! Canonical encoding (format version 1):
//!
//! * purpose: fixed `u32` code per [`ConsentType`] (see [`purpose_code`]).
//! * scopes: a set, encoded as a `u32` bitmask (see [`scope_bit`]). Order and
//!   duplicates in the caller's list are irrelevant, so `[ReadLab, ReadMedical]`
//!   and `[ReadMedical, ReadLab, ReadLab]` encode identically, while any two
//!   different sets differ in at least one bit and cannot collide.
//! * consent hash:
//!   `sha256(domain || u64be(pet_id) || xdr(subject) || u32be(purpose)
//!           || u32be(scope_mask) || u32be(version))`
//!
//! A consent *line* is `(pet_id, subject, purpose)`. Each distinct set of
//! terms granted on a line gets the next version number, which is bound into
//! the hash, so a revoked version can never be resurrected under the same
//! hash. Codes and bits are pinned here explicitly (never derived from enum
//! declaration order) so reordering the enums cannot change any hash.
//!
//! Offline verification (Issue #1445): the consent hash preimage above is the
//! canonical payload for offline verifiers. See `docs/offline-verification.md`
//! for the field-level spec and test vectors. The byte layout is pinned by
//! [`offline_consent_payload`], which must match the documented format.

use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contracttype, Address, Bytes, BytesN, Env, Vec};

use crate::{ConsentScope, ConsentType};

/// Domain tag for [`canonical_consent_hash`].
pub const CONSENT_HASH_DOMAIN: &[u8] = b"petchain:consent:v1";

/// Upper bound on the raw (pre-normalisation) scope list length.
pub const MAX_CONSENT_SCOPE_INPUT: u32 = 16;

#[contracttype]
pub enum ConsentCanonKey {
    /// (pet_id, subject, purpose_code) -> latest version number on that line
    Line((u64, Address, u32)),
    /// (pet_id, subject, purpose_code, version) -> consent hash
    Version((u64, Address, u32, u32)),
    /// consent hash -> CanonicalConsent
    Record(BytesN<32>),
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CanonicalConsentStatus {
    Active = 1,
    /// Replaced by a later version with different terms.
    Superseded = 2,
    Revoked = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalConsent {
    pub consent_hash: BytesN<32>,
    pub pet_id: u64,
    pub owner: Address,
    pub subject: Address,
    pub purpose: ConsentType,
    pub scope_mask: u32,
    pub version: u32,
    pub status: CanonicalConsentStatus,
    pub granted_at: u64,
    /// Set when the version stops being active (superseded or revoked).
    pub ended_at: Option<u64>,
}

pub fn purpose_code(purpose: &ConsentType) -> u32 {
    match purpose {
        ConsentType::Insurance => 1,
        ConsentType::Research => 2,
        ConsentType::PublicHealth => 3,
        ConsentType::Other => 4,
        ConsentType::DataSharing => 5,
    }
}

pub fn scope_bit(scope: &ConsentScope) -> u32 {
    match scope {
        ConsentScope::ReadMedical => 1 << 0,
        ConsentScope::WriteMedical => 1 << 1,
        ConsentScope::ReadLab => 1 << 2,
        ConsentScope::EmergencyOnly => 1 << 3,
    }
}

/// Normalise a scope list into its canonical bitmask. Returns `None` for an
/// empty or oversized list.
pub fn canonical_scope_mask(scopes: &Vec<ConsentScope>) -> Option<u32> {
    if scopes.is_empty() || scopes.len() > MAX_CONSENT_SCOPE_INPUT {
        return None;
    }
    let mut mask = 0u32;
    for scope in scopes.iter() {
        mask |= scope_bit(&scope);
    }
    Some(mask)
}

/// Expand a canonical bitmask back into scopes in ascending bit order.
pub fn scopes_from_mask(env: &Env, mask: u32) -> Vec<ConsentScope> {
    let mut out = Vec::new(env);
    for scope in [
        ConsentScope::ReadMedical,
        ConsentScope::WriteMedical,
        ConsentScope::ReadLab,
        ConsentScope::EmergencyOnly,
    ] {
        if mask & scope_bit(&scope) != 0 {
            out.push_back(scope);
        }
    }
    out
}

pub fn canonical_consent_hash(
    env: &Env,
    pet_id: u64,
    subject: &Address,
    purpose: u32,
    scope_mask: u32,
    version: u32,
) -> BytesN<32> {
    let preimage = offline_consent_payload(env, pet_id, subject, purpose, scope_mask, version);
    env.crypto().sha256(&preimage).into()
}

/// Build the canonical preimage bytes (domain-separated, pre-hash) for a
/// consent version. This is the exact payload an offline verifier must
/// reconstruct before checking the signature / hash.
pub fn offline_consent_payload(
    env: &Env,
    pet_id: u64,
    subject: &Address,
    purpose: u32,
    scope_mask: u32,
    version: u32,
) -> Bytes {
    let mut preimage = Bytes::from_slice(env, CONSENT_HASH_DOMAIN);
    preimage.extend_from_slice(&pet_id.to_be_bytes());
    preimage.append(&subject.clone().to_xdr(env));
    preimage.extend_from_slice(&purpose.to_be_bytes());
    preimage.extend_from_slice(&scope_mask.to_be_bytes());
    preimage.extend_from_slice(&version.to_be_bytes());
    preimage
}
