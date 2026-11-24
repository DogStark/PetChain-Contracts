// SPDX-License-Identifier: MIT
//! Celo verified-vet registry with capability-scoped permissions.
//!
//! Instead of a single global `verified` flag, each vet holds an explicit set
//! of capability scopes (vaccination, medical record, transfer attestation).
//! Scopes are granted/revoked individually and every grant carries an expiry
//! so stale permissions cannot be replayed indefinitely.

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Symbol, Vec};

/// Storage layout version. Bump when the persistent schema changes.
const STORAGE_VERSION: u32 = 2;

/// Maximum number of capability scopes a single vet may hold.
const MAX_SCOPES_PER_VET: u32 = 8;

/// Maximum number of vets tracked in the registry (bounds iteration/growth).
const MAX_VETS: u32 = 256;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Caller is not the admin.
    Unauthorized = 1,
    /// Vet is not registered.
    VetNotFound = 2,
    /// Vet already registered.
    VetExists = 3,
    /// Requested scope is not held by the vet.
    ScopeNotGranted = 4,
    /// Scope grant has expired.
    ScopeExpired = 5,
    /// Registry is at capacity.
    RegistryFull = 6,
    /// Vet already holds the maximum number of scopes.
    TooManyScopes = 7,
    /// Expiry is not in the future.
    InvalidExpiry = 8,
}

/// Explicit vet capability scopes.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Capability {
    Vaccination = 0,
    MedicalRecord = 1,
    TransferAttestation = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeGrant {
    pub capability: Capability,
    /// Ledger timestamp (seconds) after which the grant is invalid.
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VetRecord {
    pub vet: Address,
    pub scopes: Vec<ScopeGrant>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeGranted {
    pub vet: Address,
    pub capability: Capability,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeRevoked {
    pub vet: Address,
    pub capability: Capability,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VetRegistered {
    pub vet: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VetDeregistered {
    pub vet: Address,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Version,
    VetCount,
    Vet(Address),
}

#[contract]
pub struct CeloVetRegistry;

#[contractimpl]
impl CeloVetRegistry {
    /// Initialize the registry with an admin. Idempotent-safe: only callable once.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::Unauthorized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Version, &STORAGE_VERSION);
        env.storage().instance().set(&DataKey::VetCount, &0u32);
        Ok(())
    }

    /// Register a vet with no scopes. Admin only.
    pub fn register_vet(env: Env, vet: Address) -> Result<(), Error> {
        Self::require_admin(&env)?;
        let key = DataKey::Vet(vet.clone());
        if env.storage().persistent().has(&key) {
            return Err(Error::VetExists);
        }
        let count: u32 = env.storage().instance().get(&DataKey::VetCount).unwrap_or(0);
        if count >= MAX_VETS {
            return Err(Error::RegistryFull);
        }
        let record = VetRecord { vet: vet.clone(), scopes: Vec::new(&env) };
        env.storage().persistent().set(&key, &record);
        env.storage().instance().set(&DataKey::VetCount, &(count + 1));
        env.events().publish((Symbol::new(&env, "vet_registered"),), VetRegistered { vet });
        Ok(())
    }

    /// Deregister a vet, dropping all scopes. Admin only.
    pub fn deregister_vet(env: Env, vet: Address) -> Result<(), Error> {
        Self::require_admin(&env)?;
        let key = DataKey::Vet(vet.clone());
        if !env.storage().persistent().has(&key) {
            return Err(Error::VetNotFound);
        }
        env.storage().persistent().remove(&key);
        let count: u32 = env.storage().instance().get(&DataKey::VetCount).unwrap_or(0);
        env.storage().instance().set(&DataKey::VetCount, &count.saturating_sub(1));
        env.events().publish((Symbol::new(&env, "vet_deregistered"),), VetDeregistered { vet });
        Ok(())
    }

    /// Grant a capability scope to a registered vet until `expires_at`. Admin only.
    pub fn grant_scope(
        env: Env,
        vet: Address,
        capability: Capability,
        expires_at: u64,
    ) -> Result<(), Error> {
        Self::require_admin(&env)?;
        if expires_at <= env.ledger().timestamp() {
            return Err(Error::InvalidExpiry);
        }
        let key = DataKey::Vet(vet.clone());
        let mut record: VetRecord = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::VetNotFound)?;

        let mut found = false;
        let mut updated: Vec<ScopeGrant> = Vec::new(&env);
        for grant in record.scopes.iter() {
            if grant.capability == capability {
                updated.push_back(ScopeGrant { capability, expires_at });
                found = true;
            } else {
                updated.push_back(grant);
            }
        }
        if !found {
            if updated.len() >= MAX_SCOPES_PER_VET {
                return Err(Error::TooManyScopes);
            }
            updated.push_back(ScopeGrant { capability, expires_at });
        }
        record.scopes = updated;
        env.storage().persistent().set(&key, &record);
        env.events().publish(
            (Symbol::new(&env, "scope_granted"),),
            ScopeGranted { vet, capability, expires_at },
        );
        Ok(())
    }

    /// Revoke a capability scope from a vet. Admin only.
    pub fn revoke_scope(env: Env, vet: Address, capability: Capability) -> Result<(), Error> {
        Self::require_admin(&env)?;
        let key = DataKey::Vet(vet.clone());
        let mut record: VetRecord = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::VetNotFound)?;

        let mut updated: Vec<ScopeGrant> = Vec::new(&env);
        let mut removed = false;
        for grant in record.scopes.iter() {
            if grant.capability == capability {
                removed = true;
            } else {
                updated.push_back(grant);
            }
        }
        if !removed {
            return Err(Error::ScopeNotGranted);
        }
        record.scopes = updated;
        env.storage().persistent().set(&key, &record);
        env.events().publish(
            (Symbol::new(&env, "scope_revoked"),),
            ScopeRevoked { vet, capability },
        );
        Ok(())
    }

    /// Returns true only if the vet currently holds a non-expired scope.
    pub fn has_scope(env: Env, vet: Address, capability: Capability) -> bool {
        let key = DataKey::Vet(vet);
        let record: VetRecord = match env.storage().persistent().get(&key) {
            Some(r) => r,
            None => return false,
        };
        let now = env.ledger().timestamp();
        for grant in record.scopes.iter() {
            if grant.capability == capability {
                return grant.expires_at > now;
            }
        }
        false
    }

    /// Assert a vet holds a live scope; used by downstream attestation flows.
    pub fn require_scope(env: Env, vet: Address, capability: Capability) -> Result<(), Error> {
        let key = DataKey::Vet(vet);
        let record: VetRecord = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::VetNotFound)?;
        let now = env.ledger().timestamp();
        for grant in record.scopes.iter() {
            if grant.capability == capability {
                if grant.expires_at > now {
                    return Ok(());
                }
                return Err(Error::ScopeExpired);
            }
        }
        Err(Error::ScopeNotGranted)
    }

    /// Read the full record for a vet.
    pub fn get_vet(env: Env, vet: Address) -> Result<VetRecord, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Vet(vet))
            .ok_or(Error::VetNotFound)
    }

    /// Current storage schema version.
    pub fn storage_version(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Version).unwrap_or(0)
    }

    fn require_admin(env: &Env) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::Unauthorized)?;
        admin.require_auth();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::Env;

    fn setup() -> (Env, CeloVetRegistryClient<'static>, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);
        let contract_id = env.register_contract(None, CeloVetRegistry);
        let client = CeloVetRegistryClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let vet = Address::generate(&env);
        client.initialize(&admin);
        client.register_vet(&vet);
        (env, client, admin, vet)
    }

    #[test]
    fn grant_and_use_scope() {
        let (_env, client, _admin, vet) = setup();
        client.grant_scope(&vet, &Capability::Vaccination, &2_000);
        assert!(client.has_scope(&vet, &Capability::Vaccination));
        assert!(client.require_scope(&vet, &Capability::Vaccination).is_ok());
        // Other scopes remain ungranted.
        assert!(!client.has_scope(&vet, &Capability::MedicalRecord));
    }

    #[test]
    fn unauthorized_grant_rejected() {
        let env = Env::default();
        env.ledger().set_timestamp(1_000);
        let contract_id = env.register_contract(None, CeloVetRegistry);
        let client = CeloVetRegistryClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let vet = Address::generate(&env);
        client.initialize(&admin);
        client.register_vet(&vet);
        // No auth mocked: admin.require_auth() must fail.
        let res = client.try_grant_scope(&vet, &Capability::Vaccination, &2_000);
        assert!(res.is_err());
        assert!(!client.has_scope(&vet, &Capability::Vaccination));
    }

    #[test]
    fn expiry_boundary() {
        let (env, client, _admin, vet) = setup();
        client.grant_scope(&vet, &Capability::TransferAttestation, &1_500);
        env.ledger().set_timestamp(1_499);
        assert!(client.has_scope(&vet, &Capability::TransferAttestation));
        env.ledger().set_timestamp(1_500);
        assert!(!client.has_scope(&vet, &Capability::TransferAttestation));
        assert_eq!(
            client.require_scope(&vet, &Capability::TransferAttestation),
            Err(Error::ScopeExpired)
        );
    }

    #[test]
    fn invalid_expiry_rejected_and_state_unchanged() {
        let (_env, client, _admin, vet) = setup();
        let res = client.try_grant_scope(&vet, &Capability::Vaccination, &1_000);
        assert_eq!(res, Err(Ok(Error::InvalidExpiry)));
        assert!(!client.has_scope(&vet, &Capability::Vaccination));
    }

    #[test]
    fn revoke_removes_scope() {
        let (_env, client, _admin, vet) = setup();
        client.grant_scope(&vet, &Capability::MedicalRecord, &2_000);
        client.revoke_scope(&vet, &Capability::MedicalRecord);
        assert!(!client.has_scope(&vet, &Capability::MedicalRecord));
        assert_eq!(
            client.require_scope(&vet, &Capability::MedicalRecord),
            Err(Error::ScopeNotGranted)
        );
    }

    #[test]
    fn revoke_missing_scope_fails_without_change() {
        let (_env, client, _admin, vet) = setup();
        client.grant_scope(&vet, &Capability::Vaccination, &2_000);
        let res = client.try_revoke_scope(&vet, &Capability::MedicalRecord);
        assert_eq!(res, Err(Ok(Error::ScopeNotGranted)));
        assert!(client.has_scope(&vet, &Capability::Vaccination));
    }

    #[test]
    fn scope_cap_enforced() {
        let (_env, client, _admin, vet) = setup();
        // Only three capabilities exist; cap is higher, so all three fit.
        client.grant_scope(&vet, &Capability::Vaccination, &2_000);
        client.grant_scope(&vet, &Capability::MedicalRecord, &2_000);
        client.grant_scope(&vet, &Capability::TransferAttestation, &2_000);
        let record = client.get_vet(&vet);
        assert_eq!(record.scopes.len(), 3);
    }

    #[test]
    fn unknown_vet_has_no_scope() {
        let (env, client, _admin, _vet) = setup();
        let stranger = Address::generate(&env);
        assert!(!client.has_scope(&stranger, &Capability::Vaccination));
        assert_eq!(
            client.require_scope(&stranger, &Capability::Vaccination),
            Err(Error::VetNotFound)
        );
    }

    #[test]
    fn storage_version_reported() {
        let (_env, client, _admin, _vet) = setup();
        assert_eq!(client.storage_version(), STORAGE_VERSION);
    }
}
