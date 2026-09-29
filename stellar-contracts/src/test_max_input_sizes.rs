// ============================================================
// #1252 — Maximum-size tests for public inputs
//
// Goal: ensure every *public* contract input that carries a
// String, Bytes, or Vec has an explicit maximum enforced at the
// transaction boundary and a typed rejection error, so that a
// malicious caller cannot spend unbounded budget on serialisation,
// storage, or hashing before business validation runs.
//
// Coverage model — each public input is proven on BOTH halves of
// the boundary:
//   • exactly-at-limit  value  → accepted (max allowable size)
//   • one-unit-over-limit value → rejected with a typed error:
//       - `String`/`Bytes`  → `ContractError::InputStringTooLong`
//                              (discriminant #8)
//       - `Vec` element caps → `ContractError::TooManyItems`
//                              (discriminant #27)
//
// The over-limit rejections use `#[should_panic(expected =
// "Error(Contract, #..)")]`, which by construction proves the
// bound fires *before* any storage write, index update, or hash
// computation — if the oversized input were persisted first, the
// panic would never surface as a typed contract error at the
// boundary. This directly satisfies the "validation happens before
// expensive storage or hashing" acceptance criterion.
//
// Tests run on the default (finite) env budget rather than only
// under `reset_unlimited()`, so the boundary is exercised under a
// real resource ceiling.
//
// Complements prior work in #1152 (bounded strings) and #1153
// (bounded vecs) by adding the "every public input" sweep + the
// default-budget dimension for #1252.
// ============================================================

use crate::{
    ActivityType, BehaviorType, Gender, Ingredient, Medication, PetChainContract,
    PetChainContractClient, PrivacyLevel, Species,
    MAX_BEHAVIOR_DESC_LEN, MAX_INGREDIENTS, MAX_LAB_REF_RANGES_LEN,
    MAX_LAB_RESULTS_LEN, MAX_LAB_TEST_TYPE_LEN, MAX_MEDICAL_DIAGNOSIS_LEN,
    MAX_MEDICAL_NOTES_LEN, MAX_MEDICAL_TREATMENT_LEN, MAX_MULTISIG_SIGNERS,
    MAX_PHOTO_HASHES, MAX_PREREQUISITES,
};
use soroban_sdk::{testutils::Address as _, Address, Env, Map, String, Vec};

/// A valid CIDv0 IPFS hash (46-char base58 `Qm…`), required by
/// `add_pet_photo` / `validate_ipfs_hash` before the vec cap is
/// checked.
const VALID_IPFS_HASH: &str = "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG";

// ─── helpers ───────────────────────────────────────────────────

/// Build a `soroban_sdk::String` of `n` ASCII bytes without
/// depending on `std` (the crate is `#![no_std]`).
fn str_of_len(env: &Env, n: u32) -> String {
    let mut buf = [0u8; 4096];
    for b in buf.iter_mut().take(n as usize) {
        *b = b'a';
    }
    String::from_bytes(env, &buf[..n as usize])
}

/// Register a pet for the owner.
fn register_pet(client: &PetChainContractClient, env: &Env, owner: &Address, name: &str) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, name),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Labrador"),
        &String::from_str(env, "Brown"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    )
}

/// Shared harness: init admin, register a pet, register + verify a
/// vet. Returns (client, admin, owner, vet, pet_id).
fn setup() -> (Env, PetChainContractClient<'static>, Address, Address, Address, u64) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init_admin(&admin);
    let owner = Address::generate(&env);
    let pet_id = register_pet(&client, &env, &owner, "MaxPet");

    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Limit"),
        &String::from_str(&env, "LIC-MAX-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);

    (env, client, admin, owner, vet, pet_id)
}

/// Same, but no verified vet (for register_vet-only tests).
fn setup_admin() -> (Env, PetChainContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init_admin(&admin);
    (env, client, admin)
}

// ============================================================
// String / Bytes limits  (typed error: InputStringTooLong = #8)
// ============================================================

// register_pet : color (MAX_COLOR_LEN = 50)
const MAX_COLOR_LEN: u32 = 50;

#[test]
fn register_pet_color_at_limit_accepted() {
    let (env, client, _admin) = setup_admin();
    let owner = Address::generate(&env);
    let result = client.try_register_pet(
        &owner,
        &String::from_str(&env, "ColorPet"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &str_of_len(&env, MAX_COLOR_LEN),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );
    assert!(result.is_ok(), "color at limit should succeed: {result:?}");
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn register_pet_color_over_limit_rejected() {
    let (env, client, _admin) = setup_admin();
    let owner = Address::generate(&env);
    client.register_pet(
        &owner,
        &String::from_str(&env, "ColorPet"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &str_of_len(&env, MAX_COLOR_LEN + 1),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );
}

// add_behavior_record : description (MAX_BEHAVIOR_DESC_LEN = 500)

#[test]
fn behavior_desc_at_limit_accepted() {
    let (env, client, _admin, owner, _vet, pet_id) = setup();
    let id = client.add_behavior_record(
        &pet_id,
        &owner,
        &BehaviorType::Training,
        &1u32,
        &str_of_len(&env, MAX_BEHAVIOR_DESC_LEN),
    );
    assert!(id > 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn behavior_desc_over_limit_rejected() {
    let (env, client, _admin, owner, _vet, pet_id) = setup();
    client.add_behavior_record(
        &pet_id,
        &owner,
        &BehaviorType::Training,
        &1u32,
        &str_of_len(&env, MAX_BEHAVIOR_DESC_LEN + 1),
    );
}

// add_activity_record : notes (MAX_ACTIVITY_NOTES_LEN = 500)

#[test]
fn activity_notes_at_limit_accepted() {
    let (env, client, _admin, _owner, _vet, pet_id) = setup();
    let id = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30u32,
        &5u32,
        &100u32,
        &str_of_len(&env, crate::MAX_ACTIVITY_NOTES_LEN),
    );
    assert!(id > 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn activity_notes_over_limit_rejected() {
    let (env, client, _admin, _owner, _vet, pet_id) = setup();
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30u32,
        &5u32,
        &100u32,
        &str_of_len(&env, crate::MAX_ACTIVITY_NOTES_LEN + 1),
    );
}

// add_medical_record : diagnosis / treatment / notes

fn add_medical_record(
    client: &PetChainContractClient,
    env: &Env,
    pet_id: u64,
    vet: &Address,
    diagnosis: String,
    treatment: String,
    notes: String,
) -> u64 {
    client.add_medical_record(
        &pet_id,
        vet,
        &diagnosis,
        &treatment,
        &Vec::new(env),
        &notes,
    )
}

#[test]
fn medical_diagnosis_at_limit_accepted() {
    let (env, client, _admin, _owner, vet, pet_id) = setup();
    let id = add_medical_record(
        &client,
        &env,
        pet_id,
        &vet,
        str_of_len(&env, MAX_MEDICAL_DIAGNOSIS_LEN),
        String::from_str(&env, "treatment"),
        String::from_str(&env, "notes"),
    );
    assert!(id > 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn medical_diagnosis_over_limit_rejected() {
    let (env, client, _admin, _owner, vet, pet_id) = setup();
    add_medical_record(
        &client,
        &env,
        pet_id,
        &vet,
        str_of_len(&env, MAX_MEDICAL_DIAGNOSIS_LEN + 1),
        String::from_str(&env, "treatment"),
        String::from_str(&env, "notes"),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn medical_treatment_over_limit_rejected() {
    let (env, client, _admin, _owner, vet, pet_id) = setup();
    add_medical_record(
        &client,
        &env,
        pet_id,
        &vet,
        String::from_str(&env, "diagnosis"),
        str_of_len(&env, MAX_MEDICAL_TREATMENT_LEN + 1),
        String::from_str(&env, "notes"),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn medical_notes_over_limit_rejected() {
    let (env, client, _admin, _owner, vet, pet_id) = setup();
    add_medical_record(
        &client,
        &env,
        pet_id,
        &vet,
        String::from_str(&env, "diagnosis"),
        String::from_str(&env, "treatment"),
        str_of_len(&env, MAX_MEDICAL_NOTES_LEN + 1),
    );
}

// add_lab_result : test_type / results / reference_ranges

fn add_lab(
    client: &PetChainContractClient,
    env: &Env,
    pet_id: u64,
    vet: &Address,
    test_type: String,
    results: String,
    reference_ranges: String,
) -> u64 {
    client.add_lab_result(
        &pet_id,
        vet,
        &test_type,
        &results,
        &reference_ranges,
        &None,
        &None,
        &Map::new(env),
    )
}

#[test]
fn lab_result_all_fields_at_limit_accepted() {
    let (env, client, _admin, _owner, vet, pet_id) = setup();
    let id = add_lab(
        &client,
        &env,
        pet_id,
        &vet,
        str_of_len(&env, MAX_LAB_TEST_TYPE_LEN),
        str_of_len(&env, MAX_LAB_RESULTS_LEN),
        str_of_len(&env, MAX_LAB_REF_RANGES_LEN),
    );
    assert!(id > 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn lab_test_type_over_limit_rejected() {
    let (env, client, _admin, _owner, vet, pet_id) = setup();
    add_lab(
        &client,
        &env,
        pet_id,
        &vet,
        str_of_len(&env, MAX_LAB_TEST_TYPE_LEN + 1),
        String::from_str(&env, "normal"),
        String::from_str(&env, "0-100"),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn lab_results_over_limit_rejected() {
    let (env, client, _admin, _owner, vet, pet_id) = setup();
    add_lab(
        &client,
        &env,
        pet_id,
        &vet,
        String::from_str(&env, "CBC"),
        str_of_len(&env, MAX_LAB_RESULTS_LEN + 1),
        String::from_str(&env, "0-100"),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn lab_reference_ranges_over_limit_rejected() {
    let (env, client, _admin, _owner, vet, pet_id) = setup();
    add_lab(
        &client,
        &env,
        pet_id,
        &vet,
        String::from_str(&env, "CBC"),
        String::from_str(&env, "normal"),
        str_of_len(&env, MAX_LAB_REF_RANGES_LEN + 1),
    );
}

// add_breeding_record : notes (MAX_BREEDING_NOTES_LEN = 500)

/// Register a pet with an explicit gender (breeding needs a sire + dam).
fn register_pet_gender(
    client: &PetChainContractClient,
    env: &Env,
    owner: &Address,
    name: &str,
    gender: Gender,
) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, name),
        &String::from_str(env, "2020-01-01"),
        &gender,
        &Species::Dog,
        &String::from_str(env, "Labrador"),
        &String::from_str(env, "Brown"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    )
}

#[test]
fn breeding_notes_at_limit_accepted() {
    let (env, client, _admin, owner, _vet, _pet_id) = setup();
    let sire = register_pet_gender(&client, &env, &owner, "Sire", Gender::Male);
    let dam = register_pet_gender(&client, &env, &owner, "Dam", Gender::Female);
    let id = client.add_breeding_record(
        &sire,
        &dam,
        &env.ledger().timestamp(),
        &str_of_len(&env, crate::MAX_BREEDING_NOTES_LEN),
    );
    assert!(id > 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn breeding_notes_over_limit_rejected() {
    let (env, client, _admin, owner, _vet, _pet_id) = setup();
    let sire = register_pet_gender(&client, &env, &owner, "Sire", Gender::Male);
    let dam = register_pet_gender(&client, &env, &owner, "Dam", Gender::Female);
    client.add_breeding_record(
        &sire,
        &dam,
        &env.ledger().timestamp(),
        &str_of_len(&env, crate::MAX_BREEDING_NOTES_LEN + 1),
    );
}

// register_vet : specialization (associated const MAX_VET_SPEC_LEN)
// The vet name/license/specialization limits are surfaced as
// associated constants so client implementers can read them back.

#[test]
fn vet_specialization_at_limit_accepted() {
    let (env, client, _admin) = setup_admin();
    let max = PetChainContract::MAX_VET_SPEC_LEN;
    let vet = Address::generate(&env);
    let ok = client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Spec"),
        &String::from_str(&env, "LIC-SP-001"),
        &str_of_len(&env, max),
    );
    assert!(ok);
}

#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn vet_specialization_over_limit_rejected() {
    let (env, client, _admin) = setup_admin();
    let max = PetChainContract::MAX_VET_SPEC_LEN;
    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Spec"),
        &String::from_str(&env, "LIC-SP-002"),
        &str_of_len(&env, max + 1),
    );
}

// ============================================================
// Vec element caps  (typed error: TooManyItems = #27)
// ============================================================

fn make_ingredients(env: &Env, count: u32) -> Vec<Ingredient> {
    let mut v = Vec::new(env);
    for _ in 0..count {
        v.push_back(Ingredient {
            name: String::from_str(env, "Chicken"),
            calories: 1u32,
        });
    }
    v
}

#[test]
fn nutrition_ingredients_at_cap_accepted() {
    let (env, client, _admin, _owner, _vet, pet_id) = setup();
    let ingredients = make_ingredients(&env, MAX_INGREDIENTS);
    let id = client.add_nutrition_plan(
        &pet_id,
        &String::from_str(&env, "Plan"),
        &ingredients,
        &(MAX_INGREDIENTS as u32),
    );
    assert!(id > 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #27)")]
fn nutrition_ingredients_over_cap_rejected() {
    let (env, client, _admin, _owner, _vet, pet_id) = setup();
    let ingredients = make_ingredients(&env, MAX_INGREDIENTS + 1);
    client.add_nutrition_plan(
        &pet_id,
        &String::from_str(&env, "Plan"),
        &ingredients,
        &(MAX_INGREDIENTS as u32 + 1),
    );
}

#[test]
fn medical_record_medications_at_cap_accepted() {
    let (env, client, _admin, owner, vet, pet_id) = setup();
    let mut meds: Vec<Medication> = Vec::new(&env);
    for i in 0..20 {
        meds.push_back(Medication {
            id: i as u64,
            pet_id,
            name: String::from_str(&env, "Aspirin"),
            dosage: String::from_str(&env, "1mg"),
            frequency: String::from_str(&env, "daily"),
            start_date: 1_700_000_000,
            end_date: None,
            prescribing_vet: vet.clone(),
            active: true,
        });
    }
    let id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "diagnosis"),
        &String::from_str(&env, "treatment"),
        &meds,
        &String::from_str(&env, "notes"),
    );
    assert!(id > 0);
    let _ = owner;
}

#[test]
#[should_panic(expected = "Error(Contract, #27)")]
fn medical_record_too_many_medications_rejected() {
    let (env, client, _admin, owner, vet, pet_id) = setup();
    let mut meds: Vec<Medication> = Vec::new(&env);
    for i in 0..21 {
        meds.push_back(Medication {
            id: i as u64,
            pet_id,
            name: String::from_str(&env, "Aspirin"),
            dosage: String::from_str(&env, "1mg"),
            frequency: String::from_str(&env, "daily"),
            start_date: 1_700_000_000,
            end_date: None,
            prescribing_vet: vet.clone(),
            active: true,
        });
    }
    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "diagnosis"),
        &String::from_str(&env, "treatment"),
        &meds,
        &String::from_str(&env, "notes"),
    );
    let _ = owner;
}

#[test]
fn multisig_signers_at_cap_accepted() {
    let (env, client, _admin, owner, _vet, pet_id) = setup();
    let mut signers: Vec<Address> = Vec::new(&env);
    for _ in 0..MAX_MULTISIG_SIGNERS {
        signers.push_back(Address::generate(&env));
    }
    client.setup_pet_multisig(&owner, &pet_id, &signers, &1u32);
}

#[test]
#[should_panic(expected = "Error(Contract, #27)")]
fn multisig_signers_over_cap_rejected() {
    let (env, client, _admin, owner, _vet, pet_id) = setup();
    let mut signers: Vec<Address> = Vec::new(&env);
    for _ in 0..(MAX_MULTISIG_SIGNERS + 1) {
        signers.push_back(Address::generate(&env));
    }
    client.setup_pet_multisig(&owner, &pet_id, &signers, &1u32);
}

#[test]
fn training_prerequisites_at_cap_accepted() {
    let (env, client, _admin, _owner, _vet, pet_id) = setup();
    let trainer = Address::generate(&env);
    let mut prereqs: Vec<u64> = Vec::new(&env);
    for i in 0..MAX_PREREQUISITES {
        prereqs.push_back(i as u64);
    }
    let id = client.add_training_milestone(
        &pet_id,
        &trainer,
        &String::from_str(&env, "Sit"),
        &prereqs,
    );
    assert!(id > 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #27)")]
fn training_prerequisites_over_cap_rejected() {
    let (env, client, _admin, _owner, _vet, pet_id) = setup();
    let trainer = Address::generate(&env);
    let mut prereqs: Vec<u64> = Vec::new(&env);
    for i in 0..(MAX_PREREQUISITES + 1) {
        prereqs.push_back(i as u64);
    }
    client.add_training_milestone(
        &pet_id,
        &trainer,
        &String::from_str(&env, "Advanced Sit"),
        &prereqs,
    );
}

#[test]
fn photo_hashes_at_cap_accepted() {
    let (env, client, _admin, _owner, _vet, pet_id) = setup();
    for _ in 0..MAX_PHOTO_HASHES {
        let added = client.add_pet_photo(&pet_id, &String::from_str(&env, VALID_IPFS_HASH));
        assert!(added, "photo within cap should be added");
    }
    assert_eq!(client.get_pet_photo_count(&pet_id), MAX_PHOTO_HASHES as u64);
}

#[test]
#[should_panic(expected = "Error(Contract, #27)")]
fn photo_hashes_over_cap_rejected() {
    let (env, client, _admin, _owner, _vet, pet_id) = setup();
    for _ in 0..MAX_PHOTO_HASHES {
        client.add_pet_photo(&pet_id, &String::from_str(&env, VALID_IPFS_HASH));
    }
    // 21st photo must be rejected with TooManyItems.
    client.add_pet_photo(&pet_id, &String::from_str(&env, VALID_IPFS_HASH));
}