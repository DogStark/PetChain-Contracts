// ============================================================
// CANONICAL MEDICAL-RECORD HASHING TESTS (Issue #1169)
//
// `get_medical_record_hash` computes a deterministic, versioned commitment
// over a medical record's clinical fields (see
// `PetChainContract::canonical_medical_record_preimage` for the exact
// encoding). These tests document:
//   - Determinism: hashing the same record twice yields the same digest.
//   - Sensitivity: changing a clinical field (e.g. diagnosis) changes the
//     hash.
//   - Stability under non-clinical mutation: soft-deleting a record (which
//     only touches `deleted_at`/`updated_at`) does NOT change its hash.
// ============================================================

use crate::{
    Gender, Medication, PetChainContract, PetChainContractClient, PrivacyLevel, Species,
};
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

fn setup(env: &Env) -> (PetChainContractClient<'static>, Address, Address, u64) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let vet = Address::generate(env);
    let owner = Address::generate(env);
    let mut admins: Vec<Address> = Vec::new(env);
    admins.push_back(admin.clone());
    client.init_multisig(&admin, &admins, &1u32);

    client.register_vet(
        &vet,
        &String::from_str(env, "Dr. Hash"),
        &String::from_str(env, "LIC-HASH-1"),
        &String::from_str(env, "General"),
    );
    client.verify_vet(&admin, &vet);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(env, "Hashy"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Mixed"),
        &String::from_str(env, "Brown"),
        &20,
        &None,
        &PrivacyLevel::Public,
    );

    (client, vet, owner, pet_id)
}

fn add_record(
    env: &Env,
    client: &PetChainContractClient<'static>,
    vet: &Address,
    pet_id: u64,
    diagnosis: &str,
) -> u64 {
    client.add_medical_record(
        &pet_id,
        vet,
        &String::from_str(env, diagnosis),
        &String::from_str(env, "Rest and fluids"),
        &Vec::<Medication>::new(env),
        &String::from_str(env, "routine checkup"),
    )
}

/// Hashing the same stored record twice yields an identical digest.
#[test]
fn test_hash_is_deterministic() {
    let env = Env::default();
    let (client, vet, _owner, pet_id) = setup(&env);
    let record_id = add_record(&env, &client, &vet, pet_id, "Ear infection");

    let hash1 = client.get_medical_record_hash(&record_id);
    let hash2 = client.get_medical_record_hash(&record_id);
    assert_eq!(hash1, hash2);
}

/// Two records with different clinical content (diagnosis) hash to
/// different digests.
#[test]
fn test_hash_differs_for_different_diagnosis() {
    let env = Env::default();
    let (client, vet, _owner, pet_id) = setup(&env);

    let record_a = add_record(&env, &client, &vet, pet_id, "Ear infection");
    let record_b = add_record(&env, &client, &vet, pet_id, "Broken leg");

    let hash_a = client.get_medical_record_hash(&record_a);
    let hash_b = client.get_medical_record_hash(&record_b);
    assert_ne!(hash_a, hash_b);
}

/// Soft-deleting a record only touches `deleted_at`/`updated_at`
/// bookkeeping fields, which are intentionally excluded from the
/// canonical commitment, so the hash must not change.
#[test]
fn test_hash_stable_across_soft_delete() {
    let env = Env::default();
    let (client, vet, owner, pet_id) = setup(&env);
    let record_id = add_record(&env, &client, &vet, pet_id, "Ear infection");

    let hash_before = client.get_medical_record_hash(&record_id);
    client.delete_medical_record(&pet_id, &record_id, &owner);
    let hash_after = client.get_medical_record_hash(&record_id);

    assert_eq!(
        hash_before, hash_after,
        "soft-delete must not change the clinical-content hash"
    );
}
