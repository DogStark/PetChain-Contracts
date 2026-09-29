//! Soroban authorization grant scope matrix.
//!
//! Access grants cover multiple record domains. This module encodes a single
//! executable policy that decides, for every public read/write, whether an
//! actor may proceed given their scope, actor role, expiry, and revocation
//! state. The matrix is table-driven so contract tests can be generated
//! directly from it.

use soroban_sdk::{contracttype, Address, Env, Symbol};

/// Record domains that a grant can cover.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Domain {
    MedicalRecord,
    Vaccination,
    Prescription,
    LabResult,
}

/// Actor roles recognized by the policy.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Actor {
    Owner,
    Vet,
    Emergency,
    Delegated,
}

/// The kind of operation being attempted against a domain.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Access {
    Read,
    Write,
}

/// A grant of access from an owner to an actor over a set of domains.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grant {
    pub actor: Actor,
    pub domains: u32,
    pub expires_at: u64,
    pub revoked: bool,
}

/// The outcome of a scope decision.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Decision {
    Allow,
    Deny,
}

/// Bit positions for each domain inside `Grant::domains`.
const DOMAIN_MEDICAL_RECORD: u32 = 1 << 0;
const DOMAIN_VACCINATION: u32 = 1 << 1;
const DOMAIN_PRESCRIPTION: u32 = 1 << 2;
const DOMAIN_LAB_RESULT: u32 = 1 << 3;

fn domain_bit(domain: Domain) -> u32 {
    match domain {
        Domain::MedicalRecord => DOMAIN_MEDICAL_RECORD,
        Domain::Vaccination => DOMAIN_VACCINATION,
        Domain::Prescription => DOMAIN_PRESCRIPTION,
        Domain::LabResult => DOMAIN_LAB_RESULT,
    }
}

/// Returns true when `grant` covers `domain`.
pub fn covers(grant: &Grant, domain: Domain) -> bool {
    grant.domains & domain_bit(domain) != 0
}

/// Core scope decision.
///
/// Precedence is explicit and ordered so that expiry and revocation always
/// take precedence over broader grants:
///
/// 1. Revocation denies everything, regardless of actor or domain.
/// 2. Expiry denies everything, regardless of actor or domain.
/// 3. Cross-pet access (a grant that does not cover the requested domain)
///    is denied.
/// 4. Owner may read and write any covered domain.
/// 5. Vet may read and write any covered domain.
/// 6. Emergency may read any covered domain but never write.
/// 7. Delegated may read any covered domain but never write.
///
/// Denied decisions never carry protected data; callers must not surface
/// record contents when `Decision::Deny` is returned.
pub fn decide(env: &Env, grant: &Grant, domain: Domain, access: Access) -> Decision {
    // 1. Revocation takes precedence over every broader grant.
    if grant.revoked {
        return Decision::Deny;
    }

    // 2. Expiry takes precedence over every broader grant.
    if env.ledger().timestamp() >= grant.expires_at {
        return Decision::Deny;
    }

    // 3. Cross-pet access: the grant must cover the requested domain.
    if !covers(grant, domain) {
        return Decision::Deny;
    }

    // 4-7. Actor-specific scope decisions.
    match (grant.actor, access) {
        (Actor::Owner, Access::Read) | (Actor::Owner, Access::Write) => Decision::Allow,
        (Actor::Vet, Access::Read) | (Actor::Vet, Access::Write) => Decision::Allow,
        (Actor::Emergency, Access::Read) => Decision::Allow,
        (Actor::Emergency, Access::Write) => Decision::Deny,
        (Actor::Delegated, Access::Read) => Decision::Allow,
        (Actor::Delegated, Access::Write) => Decision::Deny,
    }
}

/// A single row of the documented scope matrix.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeRow {
    pub actor: Actor,
    pub access: Access,
    pub expired: bool,
    pub revoked: bool,
    pub cross_pet: bool,
    pub expected: Decision,
}

/// The full documented scope matrix.
///
/// Every public read/write has a row here, so contract tests can be generated
/// table-driven from this single source of truth.
pub fn scope_matrix() -> [ScopeRow; 16] {
    use Access::{Read, Write};
    use Actor::{Delegated, Emergency, Owner, Vet};
    use Decision::{Allow, Deny};

    [
        // Owner: full read/write on covered domains.
        ScopeRow { actor: Owner, access: Read, expired: false, revoked: false, cross_pet: false, expected: Allow },
        ScopeRow { actor: Owner, access: Write, expired: false, revoked: false, cross_pet: false, expected: Allow },
        // Vet: full read/write on covered domains.
        ScopeRow { actor: Vet, access: Read, expired: false, revoked: false, cross_pet: false, expected: Allow },
        ScopeRow { actor: Vet, access: Write, expired: false, revoked: false, cross_pet: false, expected: Allow },
        // Emergency: read-only.
        ScopeRow { actor: Emergency, access: Read, expired: false, revoked: false, cross_pet: false, expected: Allow },
        ScopeRow { actor: Emergency, access: Write, expired: false, revoked: false, cross_pet: false, expected: Deny },
        // Delegated: read-only.
        ScopeRow { actor: Delegated, access: Read, expired: false, revoked: false, cross_pet: false, expected: Allow },
        ScopeRow { actor: Delegated, access: Write, expired: false, revoked: false, cross_pet: false, expected: Deny },
        // Expiry takes precedence over broader grants.
        ScopeRow { actor: Owner, access: Read, expired: true, revoked: false, cross_pet: false, expected: Deny },
        ScopeRow { actor: Vet, access: Write, expired: true, revoked: false, cross_pet: false, expected: Deny },
        // Revocation takes precedence over broader grants.
        ScopeRow { actor: Owner, access: Read, expired: false, revoked: true, cross_pet: false, expected: Deny },
        ScopeRow { actor: Vet, access: Write, expired: false, revoked: true, cross_pet: false, expected: Deny },
        // Cross-pet access is denied even for otherwise broad grants.
        ScopeRow { actor: Owner, access: Read, expired: false, revoked: false, cross_pet: true, expected: Deny },
        ScopeRow { actor: Vet, access: Write, expired: false, revoked: false, cross_pet: true, expected: Deny },
        // Revocation outranks expiry when both are present.
        ScopeRow { actor: Owner, access: Write, expired: true, revoked: true, cross_pet: false, expected: Deny },
        ScopeRow { actor: Emergency, access: Read, expired: true, revoked: true, cross_pet: false, expected: Deny },
    ]
}

/// Convenience helper used by contract entrypoints to enforce the matrix.
///
/// Returns `Ok(())` when access is allowed and `Err(Symbol)` otherwise. The
/// error carries no protected data, so denied calls reveal nothing about the
/// underlying record.
pub fn require_access(
    env: &Env,
    grant: &Grant,
    domain: Domain,
    access: Access,
) -> Result<(), Symbol> {
    match decide(env, grant, domain, access) {
        Decision::Allow => Ok(()),
        Decision::Deny => Err(Symbol::new(env, "access_denied")),
    }
}

/// Builds a grant for a given actor and domain set.
///
/// `expires_at` is an absolute ledger timestamp; `revoked` starts false.
pub fn grant_for(actor: Actor, domains: &[Domain], expires_at: u64) -> Grant {
    let mut bits: u32 = 0;
    for domain in domains.iter() {
        bits |= domain_bit(*domain);
    }
    Grant {
        actor,
        domains: bits,
        expires_at,
        revoked: false,
    }
}

/// Marks a grant as revoked. Revocation always wins over broader grants.
pub fn revoke(grant: &mut Grant) {
    grant.revoked = true;
}

/// Returns the address associated with a grant's actor for audit purposes.
///
/// This is intentionally a pure mapping so tests can assert that denied calls
/// never expose the protected address.
pub fn actor_address(actor: Actor, owner: &Address, delegate: &Address) -> Address {
    match actor {
        Actor::Owner => owner.clone(),
        Actor::Vet | Actor::Emergency | Actor::Delegated => delegate.clone(),
    }
}
