#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

/// Storage keys for the foster placement module.
///
/// Layout is versioned via `DataKey::Version` so future migrations can detect
/// the on-disk schema without breaking existing placements.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Version,
    Owner,
    AuthorizedShelter,
    Placement(u64),
    NextPlacementId,
}

/// Current storage schema version.
const STORAGE_VERSION: u32 = 1;

/// Maximum number of renewals allowed per placement.
const MAX_RENEWALS: u32 = 3;

/// Maximum total placement duration (initial term + all renewals), in seconds.
/// 365 days.
const MAX_TOTAL_DURATION: u64 = 31_536_000;

/// Maximum length of the free-form reason string attached to a renewal.
const MAX_REASON_LEN: u32 = 128;

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum PlacementStatus {
    Active,
    Ended,
}

#[contracttype]
#[derive(Clone)]
pub struct Placement {
    pub id: u64,
    pub foster_owner: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub renewals: u32,
    pub status: PlacementStatus,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FosterError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    PlacementNotFound = 4,
    PlacementNotActive = 5,
    InvalidDuration = 6,
    MaxRenewalsExceeded = 7,
    MaxTotalDurationExceeded = 8,
    ReasonTooLong = 9,
}

#[contract]
pub struct FosterContract;

#[contractimpl]
impl FosterContract {
    /// One-time initialization. Sets the owner and the authorized shelter that
    /// may co-approve renewals.
    pub fn initialize(env: Env, owner: Address, shelter: Address) -> Result<(), FosterError> {
        if env.storage().instance().has(&DataKey::Version) {
            return Err(FosterError::AlreadyInitialized);
        }
        owner.require_auth();
        env.storage().instance().set(&DataKey::Version, &STORAGE_VERSION);
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::AuthorizedShelter, &shelter);
        env.storage().instance().set(&DataKey::NextPlacementId, &1u64);
        Ok(())
    }

    /// Opens a new foster placement for `foster_owner` starting now and lasting
    /// `duration` seconds. Only the contract owner may open placements.
    pub fn open_placement(
        env: Env,
        foster_owner: Address,
        duration: u64,
    ) -> Result<u64, FosterError> {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(FosterError::NotInitialized)?;
        owner.require_auth();

        if duration == 0 || duration > MAX_TOTAL_DURATION {
            return Err(FosterError::InvalidDuration);
        }

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextPlacementId)
            .unwrap_or(1);
        let now = env.ledger().timestamp();

        let placement = Placement {
            id,
            foster_owner: foster_owner.clone(),
            start_time: now,
            end_time: now + duration,
            renewals: 0,
            status: PlacementStatus::Active,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Placement(id), &placement);
        env.storage()
            .instance()
            .set(&DataKey::NextPlacementId, &(id + 1));

        env.events().publish(
            (symbol_short!("placement"), symbol_short!("opened")),
            (id, foster_owner, now, now + duration),
        );

        Ok(id)
    }

    /// Extends an active placement by `extra_duration` seconds.
    ///
    /// Requires authorization from both the placement's foster owner and the
    /// authorized shelter. Enforces `MAX_RENEWALS` and `MAX_TOTAL_DURATION`.
    /// On any failure the placement is left untouched.
    pub fn renew_placement(
        env: Env,
        placement_id: u64,
        extra_duration: u64,
        reason: Symbol,
    ) -> Result<u64, FosterError> {
        let shelter: Address = env
            .storage()
            .instance()
            .get(&DataKey::AuthorizedShelter)
            .ok_or(FosterError::NotInitialized)?;

        let mut placement: Placement = env
            .storage()
            .persistent()
            .get(&DataKey::Placement(placement_id))
            .ok_or(FosterError::PlacementNotFound)?;

        if placement.status != PlacementStatus::Active {
            return Err(FosterError::PlacementNotActive);
        }

        // Both the foster owner and the authorized shelter must approve.
        placement.foster_owner.require_auth();
        shelter.require_auth();

        if extra_duration == 0 {
            return Err(FosterError::InvalidDuration);
        }

        if reason.len() > MAX_REASON_LEN {
            return Err(FosterError::ReasonTooLong);
        }

        if placement.renewals >= MAX_RENEWALS {
            return Err(FosterError::MaxRenewalsExceeded);
        }

        let new_end = placement
            .end_time
            .checked_add(extra_duration)
            .ok_or(FosterError::InvalidDuration)?;

        let total = new_end
            .checked_sub(placement.start_time)
            .ok_or(FosterError::InvalidDuration)?;

        if total > MAX_TOTAL_DURATION {
            return Err(FosterError::MaxTotalDurationExceeded);
        }

        placement.end_time = new_end;
        placement.renewals += 1;

        env.storage()
            .persistent()
            .set(&DataKey::Placement(placement_id), &placement);

        env.events().publish(
            (symbol_short!("placement"), symbol_short!("renewed")),
            (placement_id, new_end, placement.renewals, reason),
        );

        Ok(new_end)
    }

    /// Ends an active placement. Only the contract owner may end placements.
    pub fn end_placement(env: Env, placement_id: u64) -> Result<(), FosterError> {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(FosterError::NotInitialized)?;
        owner.require_auth();

        let mut placement: Placement = env
            .storage()
            .persistent()
            .get(&DataKey::Placement(placement_id))
            .ok_or(FosterError::PlacementNotFound)?;

        if placement.status != PlacementStatus::Active {
            return Err(FosterError::PlacementNotActive);
        }

        placement.status = PlacementStatus::Ended;
        env.storage()
            .persistent()
            .set(&DataKey::Placement(placement_id), &placement);

        env.events().publish(
            (symbol_short!("placement"), symbol_short!("ended")),
            placement_id,
        );

        Ok(())
    }

    /// Read-only accessor for a placement.
    pub fn get_placement(env: Env, placement_id: u64) -> Result<Placement, FosterError> {
        env.storage()
            .persistent()
            .get(&DataKey::Placement(placement_id))
            .ok_or(FosterError::PlacementNotFound)
    }

    /// Exposes the configured limits so clients can pre-validate renewals.
    pub fn limits(_env: Env) -> (u32, u64) {
        (MAX_RENEWALS, MAX_TOTAL_DURATION)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, testutils::Ledger, Env};

    fn setup(env: &Env) -> (FosterContractClient, Address, Address) {
        env.mock_all_auths();
        let contract_id = env.register_contract(None, FosterContract);
        let client = FosterContractClient::new(env, &contract_id);
        let owner = Address::generate(env);
        let shelter = Address::generate(env);
        client.initialize(&owner, &shelter);
        (client, owner, shelter)
    }

    #[test]
    fn renew_success() {
        let env = Env::default();
        let (client, _owner, _shelter) = setup(&env);
        let foster = Address::generate(&env);
        let id = client.open_placement(&foster, &1_000);
        let new_end = client.renew_placement(&id, &500, &symbol_short!("ok"));
        assert_eq!(new_end, 1_500);
        let p = client.get_placement(&id);
        assert_eq!(p.renewals, 1);
        assert_eq!(p.end_time, 1_500);
    }

    #[test]
    fn renew_unauthorized_fails() {
        let env = Env::default();
        let (client, _owner, _shelter) = setup(&env);
        let foster = Address::generate(&env);
        let id = client.open_placement(&foster, &1_000);
        // Drop all auth mocks: the foster owner / shelter auth will fail.
        env.set_auths(&[]);
        let res = client.try_renew_placement(&id, &500, &symbol_short!("no"));
        assert!(res.is_err());
        // State unchanged.
        let p = client.get_placement(&id);
        assert_eq!(p.renewals, 0);
        assert_eq!(p.end_time, 1_000);
    }

    #[test]
    fn renew_max_count_boundary() {
        let env = Env::default();
        let (client, _owner, _shelter) = setup(&env);
        let foster = Address::generate(&env);
        let id = client.open_placement(&foster, &1_000);
        for _ in 0..MAX_RENEWALS {
            client.renew_placement(&id, &100, &symbol_short!("ok"));
        }
        let res = client.try_renew_placement(&id, &100, &symbol_short!("ok"));
        assert_eq!(res, Err(Ok(FosterError::MaxRenewalsExceeded)));
        let p = client.get_placement(&id);
        assert_eq!(p.renewals, MAX_RENEWALS);
    }

    #[test]
    fn renew_max_total_duration_boundary() {
        let env = Env::default();
        let (client, _owner, _shelter) = setup(&env);
        let foster = Address::generate(&env);
        let id = client.open_placement(&foster, &MAX_TOTAL_DURATION);
        let res = client.try_renew_placement(&id, &1, &symbol_short!("ok"));
        assert_eq!(res, Err(Ok(FosterError::MaxTotalDurationExceeded)));
        let p = client.get_placement(&id);
        assert_eq!(p.renewals, 0);
    }

    #[test]
    fn renew_zero_duration_rejected() {
        let env = Env::default();
        let (client, _owner, _shelter) = setup(&env);
        let foster = Address::generate(&env);
        let id = client.open_placement(&foster, &1_000);
        let res = client.try_renew_placement(&id, &0, &symbol_short!("ok"));
        assert_eq!(res, Err(Ok(FosterError::InvalidDuration)));
    }

    #[test]
    fn renew_ended_placement_rejected() {
        let env = Env::default();
        let (client, _owner, _shelter) = setup(&env);
        let foster = Address::generate(&env);
        let id = client.open_placement(&foster, &1_000);
        client.end_placement(&id);
        let res = client.try_renew_placement(&id, &100, &symbol_short!("ok"));
        assert_eq!(res, Err(Ok(FosterError::PlacementNotActive)));
    }

    #[test]
    fn renew_missing_placement_rejected() {
        let env = Env::default();
        let (client, _owner, _shelter) = setup(&env);
        let res = client.try_renew_placement(&999, &100, &symbol_short!("ok"));
        assert_eq!(res, Err(Ok(FosterError::PlacementNotFound)));
    }
}
