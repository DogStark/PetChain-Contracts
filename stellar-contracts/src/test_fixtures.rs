//! # Shared Deterministic Test Fixtures — Issue #689
//!
//! Builder functions for creating test contracts, pets, vets, and admins with
//! deterministic, reproducible addresses and IDs. All test modules should use
//! these helpers instead of duplicating setup code.
//!
//! ## Usage
//!
//! ```rust
//! use crate::test_fixtures::{TestEnv, create_test_pet, create_test_vet, create_test_admin};
//!
//! let te = TestEnv::new();
//! let pet_id = create_test_pet(&te.env, &te.client, &te.owner);
//! let vet    = create_test_vet(&te.env, &te.client);
//! ```
//!
//! ## Determinism guarantee
//!
//! `TestEnv::new()` seeds the environment with a fixed timestamp (1_700_000_000)
//! and sequence number (1_000) so that ledger-timestamp-dependent calculations
//! (age, expiry, TTL) produce the same result across every test run.

use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, Env, String, Vec,
};

// ─── Fixed seed values ────────────────────────────────────────────────────────

/// Baseline ledger timestamp used by all fixtures.
/// Corresponds to 2023-11-14 22:13:20 UTC — a stable reference point.
pub const BASE_TIMESTAMP: u64 = 1_700_000_000;

/// Fixed protocol version used in test ledger state.
pub const PROTOCOL_VERSION: u32 = 22;

/// Fixed sequence number for all test environments.
pub const BASE_SEQUENCE: u32 = 1_000;

// ─── TestEnv ──────────────────────────────────────────────────────────────────

/// A fully initialised PetChain test environment with a two-of-two multisig
/// admin and a pre-registered owner address.
///
/// Create with [`TestEnv::new()`]. Fields are public so individual tests can
/// reach in without extra boilerplate.
pub struct TestEnv<'a> {
    pub env: Env,
    pub client: PetChainContractClient<'a>,
    /// First multisig admin (also used as the default proposer in governance tests).
    pub admin1: Address,
    /// Second multisig admin (provides the second approval to reach quorum).
    pub admin2: Address,
    /// Pre-registered pet owner.
    pub owner: Address,
}

impl<'a> TestEnv<'a> {
    /// Create a deterministic test environment with a two-of-two multisig and
    /// a pre-registered owner. The ledger timestamp is set to `BASE_TIMESTAMP`.
    pub fn new() -> TestEnv<'a> {
        let env = Env::default();
        env.mock_all_auths();
        env.budget().reset_unlimited();

        // Seed ledger with a fixed timestamp so tests are reproducible.
        env.ledger().set(LedgerInfo {
            timestamp: BASE_TIMESTAMP,
            protocol_version: PROTOCOL_VERSION,
            sequence_number: BASE_SEQUENCE,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100,
            min_persistent_entry_ttl: 100,
            max_entry_ttl: 9_999_999,
        });

        let contract_id = env.register(PetChainContract, ());
        let client = PetChainContractClient::new(&env, &contract_id);

        let admin1 = Address::generate(&env);
        let admin2 = Address::generate(&env);
        let mut admins = Vec::new(&env);
        admins.push_back(admin1.clone());
        admins.push_back(admin2.clone());
        client.init_multisig(&admin1, &admins, &2);

        let owner = Address::generate(&env);

        TestEnv { env, client, admin1, admin2, owner }
    }

    /// Advance the ledger timestamp by `secs` seconds.
    pub fn advance_time(&self, secs: u64) {
        let current = self.env.ledger().timestamp();
        self.env.ledger().set(LedgerInfo {
            timestamp: current + secs,
            protocol_version: PROTOCOL_VERSION,
            sequence_number: self.env.ledger().sequence() + 1,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100,
            min_persistent_entry_ttl: 100,
            max_entry_ttl: 9_999_999,
        });
    }
}

// ─── Builder: pet ─────────────────────────────────────────────────────────────

/// Create and register a test pet owned by `owner`.
///
/// Returns the new pet's on-chain ID (starts at 1, increments per call).
///
/// # Defaults
/// | Field    | Value                |
/// |----------|----------------------|
/// | name     | "TestPet"            |
/// | birthday | "2020-01-01"         |
/// | gender   | Male                 |
/// | species  | Dog                  |
/// | breed    | "Labrador"           |
/// | color    | "Yellow"             |
/// | weight   | 25                   |
/// | privacy  | Public               |
pub fn create_test_pet(
    env: &Env,
    client: &PetChainContractClient,
    owner: &Address,
) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, "TestPet"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Labrador"),
        &String::from_str(env, "Yellow"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    )
}

/// Create a test pet with a custom name (all other fields use the same
/// defaults as [`create_test_pet`]).
pub fn create_named_pet(
    env: &Env,
    client: &PetChainContractClient,
    owner: &Address,
    name: &str,
) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, name),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Labrador"),
        &String::from_str(env, "Yellow"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    )
}

// ─── Builder: vet ─────────────────────────────────────────────────────────────

/// Create and register a test vet with a deterministic licence number.
///
/// Returns the vet's address.
///
/// # Defaults
/// | Field          | Value              |
/// |----------------|--------------------|
/// | name           | "Dr. TestVet"      |
/// | license_number | "VET-TEST-001"     |
/// | specialization | "General Practice" |
pub fn create_test_vet(env: &Env, client: &PetChainContractClient) -> Address {
    let vet_address = Address::generate(env);
    client.register_vet(
        &vet_address,
        &String::from_str(env, "Dr. TestVet"),
        &String::from_str(env, "VET-TEST-001"),
        &String::from_str(env, "General Practice"),
    );
    vet_address
}

/// Create and register a test vet with a custom licence number to avoid
/// collisions when multiple vets are needed in one test.
pub fn create_vet_with_license(
    env: &Env,
    client: &PetChainContractClient,
    license: &str,
) -> Address {
    let vet_address = Address::generate(env);
    client.register_vet(
        &vet_address,
        &String::from_str(env, "Dr. TestVet"),
        &String::from_str(env, license),
        &String::from_str(env, "General Practice"),
    );
    vet_address
}

// ─── Builder: admin setup ─────────────────────────────────────────────────────

/// Return a `(admin1, admin2, client)` triple backed by a fresh multisig
/// environment, mirroring the legacy `setup()` pattern used in several test
/// files. Prefer `TestEnv::new()` for new tests.
pub fn create_test_admin(env: &Env) -> (Address, Address, PetChainContractClient) {
    env.mock_all_auths();
    let contract_id = env.register(PetChainContract, ());
    let client = PetChainContractClient::new(env, &contract_id);

    let admin1 = Address::generate(env);
    let admin2 = Address::generate(env);
    let mut admins = Vec::new(env);
    admins.push_back(admin1.clone());
    admins.push_back(admin2.clone());
    client.init_multisig(&admin1, &admins, &2);

    (admin1, admin2, client)
}

// ─── Self-tests ───────────────────────────────────────────────────────────────

#[test]
fn fixture_test_env_initialises_correctly() {
    let te = TestEnv::new();
    // Ledger timestamp should match BASE_TIMESTAMP
    assert_eq!(te.env.ledger().timestamp(), BASE_TIMESTAMP);
    // Two admins should be registered
    let admins = te.client.get_admins();
    assert_eq!(admins.len(), 2);
}

#[test]
fn fixture_create_test_pet_returns_incrementing_ids() {
    let te = TestEnv::new();
    let id1 = create_test_pet(&te.env, &te.client, &te.owner);
    let id2 = create_test_pet(&te.env, &te.client, &te.owner);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[test]
fn fixture_create_test_vet_is_registered() {
    let te = TestEnv::new();
    let vet = create_test_vet(&te.env, &te.client);
    assert!(te.client.is_vet_registered(&vet));
}

#[test]
fn fixture_advance_time_updates_ledger() {
    let te = TestEnv::new();
    te.advance_time(3_600);
    assert_eq!(te.env.ledger().timestamp(), BASE_TIMESTAMP + 3_600);
}

#[test]
fn fixture_create_admin_returns_two_admins() {
    let env = Env::default();
    let (admin1, admin2, client) = create_test_admin(&env);
    let admins = client.get_admins();
    assert!(admins.contains(&admin1));
    assert!(admins.contains(&admin2));
}

#[test]
fn fixture_create_vet_with_license_avoids_collision() {
    let te = TestEnv::new();
    let v1 = create_vet_with_license(&te.env, &te.client, "VET-A");
    let v2 = create_vet_with_license(&te.env, &te.client, "VET-B");
    assert_ne!(v1, v2);
    assert!(te.client.is_vet_registered(&v1));
    assert!(te.client.is_vet_registered(&v2));
}
