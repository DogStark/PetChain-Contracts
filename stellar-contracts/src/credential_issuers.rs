//! Storage keys and value types for versioned vet-credential issuers and
//! their key-rotation policy (Issue #1336). Policy documentation lives in
//! `docs/vet-credential-issuers.md`; the contract methods live in `lib.rs`.

use soroban_sdk::{contracttype, Address, BytesN};

/// Longest overlap window a rotation may grant, during which the previous
/// key version may still mint new credentials (7 days).
pub const MAX_ISSUER_ROTATION_OVERLAP_SECS: u64 = 7 * 24 * 60 * 60;

#[contracttype]
pub enum IssuerKey {
    /// issuer -> CredentialIssuer
    Issuer(Address),
    /// (issuer, version) -> IssuerKeyVersion
    KeyVersion((Address, u32)),
    /// (issuer, public_key) -> version that registered it (prevents key reuse)
    KeyInUse((Address, BytesN<32>)),
    /// credential id -> VetCredential
    Credential(u64),
    CredentialCount,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialIssuer {
    pub issuer: Address,
    /// Newest key version; the only one that can mint outside an overlap.
    pub current_version: u32,
    pub registered_at: u64,
    /// Set when the whole issuer is revoked. Retroactive: every credential
    /// the issuer ever minted stops verifying.
    pub revoked_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuerKeyVersion {
    pub version: u32,
    pub public_key: BytesN<32>,
    pub activated_at: u64,
    /// Hard expiry of the key. Credentials minted under it may not outlive it.
    pub expires_at: u64,
    /// Set when the version is rotated out: minting with it is allowed until
    /// (exclusive) this timestamp. Existing credentials are unaffected.
    pub mint_until: Option<u64>,
    /// Set when the version is revoked (key compromise). Retroactive: every
    /// credential minted under this version stops verifying.
    pub revoked_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VetCredential {
    pub id: u64,
    pub issuer: Address,
    pub key_version: u32,
    pub vet: Address,
    pub issued_at: u64,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VetCredentialStatus {
    Valid = 1,
    NotFound = 2,
    Expired = 3,
    IssuerRevoked = 4,
    KeyVersionRevoked = 5,
}
