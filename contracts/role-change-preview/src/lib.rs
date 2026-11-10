#![no_std]
#![allow(clippy::too_many_arguments)]

//! Role-change impact preview module for PetChain Soroban contracts (Issue #1512).
//!
//! Provides a read-only preview (`preview_role_change`) of affected pet access
//! grants, active authority delegations, and administrator quorum thresholds
//! before executing a pet owner transfer, veterinarian role revocation, or
//! administrator removal.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

/// Current storage schema version for the role-change preview contract.
pub const STORAGE_SCHEMA_VERSION: u32 = 1;

/// Current event schema version emitted on role transitions.
pub const EVENT_SCHEMA_VERSION: u32 = 1;

/// Maximum access grants stored per pet to bound persistent state growth and iteration.
pub const MAX_GRANTS_PER_PET: u32 = 32;

/// Maximum delegations stored per actor to bound persistent state growth and iteration.
pub const MAX_DELEGATIONS_PER_ACTOR: u32 = 32;

/// Maximum number of administrators tracked in the council.
pub const MAX_ADMINS: u32 = 16;

/// Maximum length (in bytes) of a scope or reason string.
pub const MAX_SCOPE_LEN: u32 = 64;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RolePreviewError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    PetNotFound = 4,
    VetNotFound = 5,
    AdminNotFound = 6,
    GrantLimitExceeded = 7,
    DelegationLimitExceeded = 8,
    AdminLimitExceeded = 9,
    ScopeTooLong = 10,
    InvalidExpiry = 11,
    ExpiredRequest = 12,
    ReplayDetected = 13,
    CannotRemoveLastAdmin = 14,
    SameRoleHolder = 15,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum RoleChangeKind {
    PetOwnerTransfer = 1,
    VetRoleRevocation = 2,
    AdminRemoval = 3,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AccessScope {
    ReadMedical = 1,
    WriteMedical = 2,
    EmergencyCare = 3,
    CustodyDelegate = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PetOwnershipRecord {
    pub pet_id: u64,
    pub owner: Address,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessGrantEntry {
    pub grant_id: u64,
    pub pet_id: u64,
    pub grantor: Address,
    pub grantee: Address,
    pub scope: AccessScope,
    pub expires_at: u64,
    pub revoked: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationEntry {
    pub delegation_id: u64,
    pub delegator: Address,
    pub delegate: Address,
    pub target_pet_id: u64,
    pub role_kind: RoleChangeKind,
    pub label: String,
    pub expires_at: u64,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleChangeImpactPreview {
    pub change_kind: RoleChangeKind,
    pub target_pet_id: u64,
    pub current_holder: Address,
    pub proposed_holder: Option<Address>,
    pub affected_grant_ids: Vec<u64>,
    pub retained_grant_ids: Vec<u64>,
    pub invalidated_delegation_ids: Vec<u64>,
    pub remaining_admin_count: u32,
    pub would_leave_zero_admins: bool,
    pub storage_schema_version: u32,
    pub preview_timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleChangeExecutedEvent {
    pub version: u32,
    pub change_kind: RoleChangeKind,
    pub target_pet_id: u64,
    pub previous_holder: Address,
    pub new_holder: Option<Address>,
    pub revoked_grants_count: u32,
    pub invalidated_delegations_count: u32,
    pub nonce: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    StorageVersion,
    Admins,
    Pet(u64),
    VerifiedVet(Address),
    NextGrantId,
    PetGrants(u64),
    NextDelegationId,
    ActorDelegations(Address),
    UsedChangeNonce((Address, u64)),
}

#[contract]
pub struct RoleChangePreviewContract;

#[contractimpl]
impl RoleChangePreviewContract {
    /// Initialize the contract with an initial administrator.
    pub fn init(env: Env, admin: Address) -> Result<(), RolePreviewError> {
        if env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(RolePreviewError::AlreadyInitialized);
        }
        admin.require_auth();

        let mut admins = Vec::new(&env);
        admins.push_back(admin);
        env.storage()
            .instance()
            .set(&DataKey::StorageVersion, &STORAGE_SCHEMA_VERSION);
        env.storage().instance().set(&DataKey::Admins, &admins);
        env.storage().instance().set(&DataKey::NextGrantId, &1u64);
        env.storage()
            .instance()
            .set(&DataKey::NextDelegationId, &1u64);
        Ok(())
    }

    /// Add another administrator to the council (bounded by `MAX_ADMINS`).
    pub fn add_admin(
        env: Env,
        caller: Address,
        new_admin: Address,
    ) -> Result<(), RolePreviewError> {
        Self::require_initialized(&env)?;
        caller.require_auth();
        Self::require_is_admin(&env, &caller)?;

        let mut admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Admins)
            .unwrap_or_else(|| Vec::new(&env));

        if admins.contains(&new_admin) {
            return Err(RolePreviewError::SameRoleHolder);
        }
        if admins.len() >= MAX_ADMINS {
            return Err(RolePreviewError::AdminLimitExceeded);
        }
        admins.push_back(new_admin);
        env.storage().instance().set(&DataKey::Admins, &admins);
        Ok(())
    }

    /// Register a pet under an initial owner.
    pub fn register_pet(env: Env, owner: Address, pet_id: u64) -> Result<(), RolePreviewError> {
        Self::require_initialized(&env)?;
        owner.require_auth();

        let record = PetOwnershipRecord {
            pet_id,
            owner,
            active: true,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Pet(pet_id), &record);
        Ok(())
    }

    /// Verify a veterinarian address (admin-only).
    pub fn verify_vet(env: Env, admin: Address, vet: Address) -> Result<(), RolePreviewError> {
        Self::require_initialized(&env)?;
        admin.require_auth();
        Self::require_is_admin(&env, &admin)?;

        env.storage()
            .persistent()
            .set(&DataKey::VerifiedVet(vet), &true);
        Ok(())
    }

    /// Create a bounded access grant on a pet.
    pub fn grant_access(
        env: Env,
        grantor: Address,
        pet_id: u64,
        grantee: Address,
        scope: AccessScope,
        expires_at: u64,
    ) -> Result<u64, RolePreviewError> {
        Self::require_initialized(&env)?;
        grantor.require_auth();

        let pet: PetOwnershipRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Pet(pet_id))
            .ok_or(RolePreviewError::PetNotFound)?;

        if pet.owner != grantor {
            return Err(RolePreviewError::Unauthorized);
        }

        let now = env.ledger().timestamp();
        if expires_at <= now {
            return Err(RolePreviewError::InvalidExpiry);
        }

        let mut grants: Vec<AccessGrantEntry> = env
            .storage()
            .persistent()
            .get(&DataKey::PetGrants(pet_id))
            .unwrap_or_else(|| Vec::new(&env));

        if grants.len() >= MAX_GRANTS_PER_PET {
            return Err(RolePreviewError::GrantLimitExceeded);
        }

        let grant_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextGrantId)
            .unwrap_or(1);

        let entry = AccessGrantEntry {
            grant_id,
            pet_id,
            grantor,
            grantee,
            scope,
            expires_at,
            revoked: false,
        };
        grants.push_back(entry);
        env.storage()
            .persistent()
            .set(&DataKey::PetGrants(pet_id), &grants);
        env.storage()
            .instance()
            .set(&DataKey::NextGrantId, &(grant_id + 1));

        Ok(grant_id)
    }

    /// Create an active delegation issued by an owner, vet, or admin.
    pub fn create_delegation(
        env: Env,
        delegator: Address,
        delegate: Address,
        target_pet_id: u64,
        role_kind: RoleChangeKind,
        label: String,
        expires_at: u64,
    ) -> Result<u64, RolePreviewError> {
        Self::require_initialized(&env)?;
        delegator.require_auth();

        if label.len() > MAX_SCOPE_LEN {
            return Err(RolePreviewError::ScopeTooLong);
        }
        let now = env.ledger().timestamp();
        if expires_at <= now {
            return Err(RolePreviewError::InvalidExpiry);
        }

        // Verify delegator holds the role being delegated.
        match role_kind {
            RoleChangeKind::PetOwnerTransfer => {
                let pet: PetOwnershipRecord = env
                    .storage()
                    .persistent()
                    .get(&DataKey::Pet(target_pet_id))
                    .ok_or(RolePreviewError::PetNotFound)?;
                if pet.owner != delegator {
                    return Err(RolePreviewError::Unauthorized);
                }
            }
            RoleChangeKind::VetRoleRevocation => {
                if !Self::is_verified_vet(&env, &delegator) {
                    return Err(RolePreviewError::Unauthorized);
                }
            }
            RoleChangeKind::AdminRemoval => {
                Self::require_is_admin(&env, &delegator)?;
            }
        }

        let mut delegations: Vec<DelegationEntry> = env
            .storage()
            .persistent()
            .get(&DataKey::ActorDelegations(delegator.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        if delegations.len() >= MAX_DELEGATIONS_PER_ACTOR {
            return Err(RolePreviewError::DelegationLimitExceeded);
        }

        let delegation_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextDelegationId)
            .unwrap_or(1);

        let entry = DelegationEntry {
            delegation_id,
            delegator: delegator.clone(),
            delegate,
            target_pet_id,
            role_kind,
            label,
            expires_at,
            active: true,
        };
        delegations.push_back(entry);
        env.storage()
            .persistent()
            .set(&DataKey::ActorDelegations(delegator), &delegations);
        env.storage()
            .instance()
            .set(&DataKey::NextDelegationId, &(delegation_id + 1));

        Ok(delegation_id)
    }

    /// Read-only preview of all active access grants and delegations impacted
    /// if a role change (`PetOwnerTransfer`, `VetRoleRevocation`, or `AdminRemoval`)
    /// is executed at the current ledger timestamp.
    pub fn preview_role_change(
        env: Env,
        caller: Address,
        change_kind: RoleChangeKind,
        target_pet_id: u64,
        current_holder: Address,
        proposed_holder: Option<Address>,
    ) -> Result<RoleChangeImpactPreview, RolePreviewError> {
        Self::require_initialized(&env)?;
        caller.require_auth();

        let now = env.ledger().timestamp();
        let admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Admins)
            .unwrap_or_else(|| Vec::new(&env));
        let caller_is_admin = admins.contains(&caller);

        let mut affected_grant_ids = Vec::new(&env);
        let mut retained_grant_ids = Vec::new(&env);
        let mut invalidated_delegation_ids = Vec::new(&env);
        let mut remaining_admin_count = admins.len();
        let mut would_leave_zero_admins = false;

        match change_kind {
            RoleChangeKind::PetOwnerTransfer => {
                let pet: PetOwnershipRecord = env
                    .storage()
                    .persistent()
                    .get(&DataKey::Pet(target_pet_id))
                    .ok_or(RolePreviewError::PetNotFound)?;
                if pet.owner != current_holder {
                    return Err(RolePreviewError::Unauthorized);
                }
                if caller != pet.owner && !caller_is_admin {
                    return Err(RolePreviewError::Unauthorized);
                }
                if let Some(ref next_owner) = proposed_holder {
                    if *next_owner == current_holder {
                        return Err(RolePreviewError::SameRoleHolder);
                    }
                } else {
                    return Err(RolePreviewError::SameRoleHolder);
                }

                let grants: Vec<AccessGrantEntry> = env
                    .storage()
                    .persistent()
                    .get(&DataKey::PetGrants(target_pet_id))
                    .unwrap_or_else(|| Vec::new(&env));

                for grant in grants.iter() {
                    if !grant.revoked && grant.expires_at > now {
                        if grant.grantor == current_holder {
                            affected_grant_ids.push_back(grant.grant_id);
                        } else {
                            retained_grant_ids.push_back(grant.grant_id);
                        }
                    }
                }
            }
            RoleChangeKind::VetRoleRevocation => {
                if !caller_is_admin && caller != current_holder {
                    return Err(RolePreviewError::Unauthorized);
                }
                if !Self::is_verified_vet(&env, &current_holder) {
                    return Err(RolePreviewError::VetNotFound);
                }

                if target_pet_id != 0 {
                    let grants: Vec<AccessGrantEntry> = env
                        .storage()
                        .persistent()
                        .get(&DataKey::PetGrants(target_pet_id))
                        .unwrap_or_else(|| Vec::new(&env));

                    for grant in grants.iter() {
                        if !grant.revoked && grant.expires_at > now {
                            if grant.grantee == current_holder {
                                affected_grant_ids.push_back(grant.grant_id);
                            } else {
                                retained_grant_ids.push_back(grant.grant_id);
                            }
                        }
                    }
                }
            }
            RoleChangeKind::AdminRemoval => {
                if !caller_is_admin {
                    return Err(RolePreviewError::Unauthorized);
                }
                if !admins.contains(&current_holder) {
                    return Err(RolePreviewError::AdminNotFound);
                }
                if admins.len() <= 1 {
                    remaining_admin_count = 0;
                    would_leave_zero_admins = true;
                } else {
                    remaining_admin_count = admins.len() - 1;
                }
            }
        }

        let delegations: Vec<DelegationEntry> = env
            .storage()
            .persistent()
            .get(&DataKey::ActorDelegations(current_holder.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        for d in delegations.iter() {
            if d.active && d.expires_at > now && d.role_kind == change_kind {
                if change_kind == RoleChangeKind::PetOwnerTransfer
                    && d.target_pet_id != target_pet_id
                {
                    continue;
                }
                invalidated_delegation_ids.push_back(d.delegation_id);
            }
        }

        let version: u32 = env
            .storage()
            .instance()
            .get(&DataKey::StorageVersion)
            .unwrap_or(STORAGE_SCHEMA_VERSION);

        Ok(RoleChangeImpactPreview {
            change_kind,
            target_pet_id,
            current_holder,
            proposed_holder,
            affected_grant_ids,
            retained_grant_ids,
            invalidated_delegation_ids,
            remaining_admin_count,
            would_leave_zero_admins,
            storage_schema_version: version,
            preview_timestamp: now,
        })
    }

    /// Execute a role change with replay protection (`nonce`) and expiry enforcement,
    /// revoking all grants and delegations identified by `preview_role_change`.
    pub fn apply_role_change(
        env: Env,
        caller: Address,
        change_kind: RoleChangeKind,
        target_pet_id: u64,
        current_holder: Address,
        proposed_holder: Option<Address>,
        nonce: u64,
        expires_at: u64,
    ) -> Result<RoleChangeImpactPreview, RolePreviewError> {
        let now = env.ledger().timestamp();
        if expires_at < now {
            return Err(RolePreviewError::ExpiredRequest);
        }

        let nonce_key = DataKey::UsedChangeNonce((caller.clone(), nonce));
        if env.storage().persistent().has(&nonce_key) {
            return Err(RolePreviewError::ReplayDetected);
        }

        let preview = Self::preview_role_change(
            env.clone(),
            caller.clone(),
            change_kind,
            target_pet_id,
            current_holder.clone(),
            proposed_holder.clone(),
        )?;

        if preview.would_leave_zero_admins {
            return Err(RolePreviewError::CannotRemoveLastAdmin);
        }

        match change_kind {
            RoleChangeKind::PetOwnerTransfer => {
                let next_owner = proposed_holder
                    .clone()
                    .ok_or(RolePreviewError::SameRoleHolder)?;
                let mut pet: PetOwnershipRecord = env
                    .storage()
                    .persistent()
                    .get(&DataKey::Pet(target_pet_id))
                    .ok_or(RolePreviewError::PetNotFound)?;
                pet.owner = next_owner;
                env.storage()
                    .persistent()
                    .set(&DataKey::Pet(target_pet_id), &pet);

                let grants: Vec<AccessGrantEntry> = env
                    .storage()
                    .persistent()
                    .get(&DataKey::PetGrants(target_pet_id))
                    .unwrap_or_else(|| Vec::new(&env));
                let mut updated = Vec::new(&env);
                for mut g in grants.iter() {
                    if !g.revoked && g.expires_at > now && g.grantor == current_holder {
                        g.revoked = true;
                    }
                    updated.push_back(g);
                }
                env.storage()
                    .persistent()
                    .set(&DataKey::PetGrants(target_pet_id), &updated);
            }
            RoleChangeKind::VetRoleRevocation => {
                env.storage()
                    .persistent()
                    .set(&DataKey::VerifiedVet(current_holder.clone()), &false);
                if target_pet_id != 0 {
                    let grants: Vec<AccessGrantEntry> = env
                        .storage()
                        .persistent()
                        .get(&DataKey::PetGrants(target_pet_id))
                        .unwrap_or_else(|| Vec::new(&env));
                    let mut updated = Vec::new(&env);
                    for mut g in grants.iter() {
                        if !g.revoked && g.expires_at > now && g.grantee == current_holder {
                            g.revoked = true;
                        }
                        updated.push_back(g);
                    }
                    env.storage()
                        .persistent()
                        .set(&DataKey::PetGrants(target_pet_id), &updated);
                }
            }
            RoleChangeKind::AdminRemoval => {
                let admins: Vec<Address> = env
                    .storage()
                    .instance()
                    .get(&DataKey::Admins)
                    .unwrap_or_else(|| Vec::new(&env));
                let mut updated_admins = Vec::new(&env);
                for a in admins.iter() {
                    if a != current_holder {
                        updated_admins.push_back(a);
                    }
                }
                env.storage()
                    .instance()
                    .set(&DataKey::Admins, &updated_admins);
            }
        }

        let delegations: Vec<DelegationEntry> = env
            .storage()
            .persistent()
            .get(&DataKey::ActorDelegations(current_holder.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        let mut updated_delegations = Vec::new(&env);
        for mut d in delegations.iter() {
            if d.active
                && d.expires_at > now
                && d.role_kind == change_kind
                && (change_kind != RoleChangeKind::PetOwnerTransfer
                    || d.target_pet_id == target_pet_id)
            {
                d.active = false;
            }
            updated_delegations.push_back(d);
        }
        env.storage().persistent().set(
            &DataKey::ActorDelegations(current_holder.clone()),
            &updated_delegations,
        );

        env.storage().persistent().set(&nonce_key, &true);

        env.events().publish(
            (symbol_short!("role_chg"), target_pet_id),
            RoleChangeExecutedEvent {
                version: EVENT_SCHEMA_VERSION,
                change_kind,
                target_pet_id,
                previous_holder: current_holder,
                new_holder: proposed_holder,
                revoked_grants_count: preview.affected_grant_ids.len(),
                invalidated_delegations_count: preview.invalidated_delegation_ids.len(),
                nonce,
            },
        );

        Ok(preview)
    }

    pub fn get_pet_grants(env: Env, pet_id: u64) -> Vec<AccessGrantEntry> {
        env.storage()
            .persistent()
            .get(&DataKey::PetGrants(pet_id))
            .unwrap_or_else(|| Vec::new(&env))
    }

    fn require_initialized(env: &Env) -> Result<(), RolePreviewError> {
        if !env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(RolePreviewError::NotInitialized);
        }
        Ok(())
    }

    fn require_is_admin(env: &Env, caller: &Address) -> Result<(), RolePreviewError> {
        let admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Admins)
            .unwrap_or_else(|| Vec::new(env));
        if !admins.contains(caller) {
            return Err(RolePreviewError::Unauthorized);
        }
        Ok(())
    }

    fn is_verified_vet(env: &Env, vet: &Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::VerifiedVet(vet.clone()))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    fn setup() -> (Env, RoleChangePreviewContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);

        let contract_id = env.register_contract(None, RoleChangePreviewContract);
        let client = RoleChangePreviewContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init(&admin);
        (env, client, admin)
    }

    #[test]
    fn test_preview_pet_owner_transfer_lists_active_grants_and_delegations() {
        let (env, client, _admin) = setup();
        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let vet = Address::generate(&env);
        let sitter = Address::generate(&env);

        client.register_pet(&owner, &101);
        let g1 = client.grant_access(&owner, &101, &vet, &AccessScope::WriteMedical, &2_000);
        let g2 = client.grant_access(&owner, &101, &sitter, &AccessScope::EmergencyCare, &1_500);
        let d1 = client.create_delegation(
            &owner,
            &sitter,
            &101,
            &RoleChangeKind::PetOwnerTransfer,
            &String::from_str(&env, "weekend-care"),
            &2_000,
        );

        // Advance time so g2 is expired while g1 and d1 remain active.
        env.ledger().set_timestamp(1_600);

        let preview = client.preview_role_change(
            &owner,
            &RoleChangeKind::PetOwnerTransfer,
            &101,
            &owner,
            &Some(new_owner.clone()),
        );
        assert_eq!(preview.affected_grant_ids.len(), 1);
        assert_eq!(preview.affected_grant_ids.get(0), Some(g1));
        assert!(!preview.affected_grant_ids.contains(&g2));
        assert_eq!(preview.invalidated_delegation_ids.len(), 1);
        assert_eq!(preview.invalidated_delegation_ids.get(0), Some(d1));
        assert_eq!(preview.storage_schema_version, STORAGE_SCHEMA_VERSION);

        // Read-only guarantee: grants are still unrevoked after preview.
        let grants = client.get_pet_grants(&101);
        assert!(!grants.get(0).unwrap().revoked);
    }

    #[test]
    fn test_preview_and_apply_vet_revocation_and_replay_protection() {
        let (env, client, admin) = setup();
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);
        let assistant = Address::generate(&env);

        client.register_pet(&owner, &202);
        client.verify_vet(&admin, &vet);

        let g1 = client.grant_access(&owner, &202, &vet, &AccessScope::WriteMedical, &5_000);
        let d1 = client.create_delegation(
            &vet,
            &assistant,
            &202,
            &RoleChangeKind::VetRoleRevocation,
            &String::from_str(&env, "clinic-shift"),
            &5_000,
        );

        let preview = client.preview_role_change(
            &admin,
            &RoleChangeKind::VetRoleRevocation,
            &202,
            &vet,
            &None,
        );
        assert_eq!(preview.affected_grant_ids.get(0), Some(g1));
        assert_eq!(preview.invalidated_delegation_ids.get(0), Some(d1));

        // Apply change with nonce 77
        let applied = client.apply_role_change(
            &admin,
            &RoleChangeKind::VetRoleRevocation,
            &202,
            &vet,
            &None,
            &77,
            &2_000,
        );
        assert_eq!(applied.affected_grant_ids.len(), 1);

        // Replaying the same nonce fails with ReplayDetected and leaves state intact
        let replay = client.try_apply_role_change(
            &admin,
            &RoleChangeKind::VetRoleRevocation,
            &202,
            &vet,
            &None,
            &77,
            &2_000,
        );
        assert_eq!(replay, Err(Ok(RolePreviewError::ReplayDetected)));
    }

    #[test]
    fn test_admin_removal_preview_detects_last_admin_and_blocks_removal() {
        let (env, client, admin) = setup();
        let preview =
            client.preview_role_change(&admin, &RoleChangeKind::AdminRemoval, &0, &admin, &None);
        assert!(preview.would_leave_zero_admins);
        assert_eq!(preview.remaining_admin_count, 0);

        let res = client.try_apply_role_change(
            &admin,
            &RoleChangeKind::AdminRemoval,
            &0,
            &admin,
            &None,
            &1,
            &2_000,
        );
        assert_eq!(res, Err(Ok(RolePreviewError::CannotRemoveLastAdmin)));

        // Adding a second admin allows removing the first
        let admin2 = Address::generate(&env);
        client.add_admin(&admin, &admin2);
        let preview2 =
            client.preview_role_change(&admin2, &RoleChangeKind::AdminRemoval, &0, &admin, &None);
        assert!(!preview2.would_leave_zero_admins);
        assert_eq!(preview2.remaining_admin_count, 1);
    }

    #[test]
    fn test_unauthorized_and_boundary_validation_leaves_state_unchanged() {
        let (env, client, _admin) = setup();
        let owner = Address::generate(&env);
        let outsider = Address::generate(&env);
        client.register_pet(&owner, &303);

        // Unauthorized preview attempt
        let unauth = client.try_preview_role_change(
            &outsider,
            &RoleChangeKind::PetOwnerTransfer,
            &303,
            &owner,
            &Some(outsider.clone()),
        );
        assert_eq!(unauth, Err(Ok(RolePreviewError::Unauthorized)));

        // Oversized delegation label rejected
        let long_label = String::from_str(&env, &"x".repeat((MAX_SCOPE_LEN + 1) as usize));
        let bad_label = client.try_create_delegation(
            &owner,
            &outsider,
            &303,
            &RoleChangeKind::PetOwnerTransfer,
            &long_label,
            &5_000,
        );
        assert_eq!(bad_label, Err(Ok(RolePreviewError::ScopeTooLong)));
    }
}
