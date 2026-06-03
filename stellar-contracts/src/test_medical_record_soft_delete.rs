use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup(env: &Env) -> (PetChainContractClient<'_>, Address, Address, u64) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let vet = Address::generate(env);
    let owner = Address::generate(env);

    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(env, "Dr. Soft"),
        &String::from_str(env, "LIC-SOFT"),
        &String::from_str(env, "General"),
    );
    client.verify_vet(&admin, &vet);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(env, "Buddy"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Labrador"),
        &String::from_str(env, "Brown"),
        &25,
        &None,
        &PrivacyLevel::Public,
    );

    (client, vet, owner, pet_id)
}

fn add_record(
    client: &PetChainContractClient<'_>,
    env: &Env,
    vet: &Address,
    pet_id: u64,
    diagnosis: &str,
) -> u64 {
    client.add_medical_record(
        &pet_id,
        vet,
        &String::from_str(env, diagnosis),
        &String::from_str(env, "Treatment"),
        &Vec::new(env),
        &String::from_str(env, "notes for soft delete test"),
    )
}

// ---------------------------------------------------------------------------
// Soft-delete basics
// ---------------------------------------------------------------------------

#[test]
fn test_soft_delete_hides_record_from_get() {
    let env = Env::default();
    let (client, vet, owner, pet_id) = setup(&env);
    let rid = add_record(&client, &env, &vet, pet_id, "Cough");

    assert!(client.get_medical_record(&rid).is_some());
    client.delete_medical_record(&pet_id, &rid, &owner);
    assert!(client.get_medical_record(&rid).is_none());
}

#[test]
fn test_soft_delete_hides_record_from_pagination() {
    let env = Env::default();
    let (client, vet, owner, pet_id) = setup(&env);
    let _r1 = add_record(&client, &env, &vet, pet_id, "Fever");
    let r2 = add_record(&client, &env, &vet, pet_id, "Cough");
    let _r3 = add_record(&client, &env, &vet, pet_id, "Rash");

    client.delete_medical_record(&pet_id, &r2, &owner);

    let records = client.get_pet_medical_records(&pet_id, &0u64, &10u32);
    assert_eq!(records.len(), 2);
    for r in records.iter() {
        assert_ne!(r.id, r2);
    }
}

#[test]
fn test_soft_delete_hides_record_from_search() {
    let env = Env::default();
    let (client, vet, owner, pet_id) = setup(&env);
    let rid = add_record(&client, &env, &vet, pet_id, "Allergy");

    let filter = MedicalRecordFilter {
        vet_address: None,
        from_date: None,
        to_date: None,
        diagnosis_keyword: None,
    };
    assert_eq!(
        client
            .search_medical_records(&pet_id, &filter, &0u64, &10u32)
            .len(),
        1
    );

    client.delete_medical_record(&pet_id, &rid, &owner);

    assert_eq!(
        client
            .search_medical_records(&pet_id, &filter, &0u64, &10u32)
            .len(),
        0
    );
}

#[test]
fn test_soft_delete_hides_record_from_keyword_search() {
    let env = Env::default();
    let (client, vet, owner, pet_id) = setup(&env);
    let rid = add_record(&client, &env, &vet, pet_id, "Dermatitis");

    assert_eq!(
        client
            .search_by_keyword(&pet_id, &String::from_str(&env, "soft"))
            .len(),
        1
    );

    client.delete_medical_record(&pet_id, &rid, &owner);

    assert_eq!(
        client
            .search_by_keyword(&pet_id, &String::from_str(&env, "soft"))
            .len(),
        0
    );
}

#[test]
fn test_vet_can_soft_delete_own_record() {
    let env = Env::default();
    let (client, vet, _owner, pet_id) = setup(&env);
    let rid = add_record(&client, &env, &vet, pet_id, "Checkup");

    assert!(client.delete_medical_record(&pet_id, &rid, &vet));
    assert!(client.get_medical_record(&rid).is_none());
}

// ---------------------------------------------------------------------------
// Double-delete rejected
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #160)")]
fn test_double_soft_delete_rejected() {
    let env = Env::default();
    let (client, vet, owner, pet_id) = setup(&env);
    let rid = add_record(&client, &env, &vet, pet_id, "Flu");

    client.delete_medical_record(&pet_id, &rid, &owner);
    client.delete_medical_record(&pet_id, &rid, &owner); // should panic
}

// ---------------------------------------------------------------------------
// Unauthorised delete rejected
// ---------------------------------------------------------------------------

#[test]
#[should_panic]
fn test_unauthorised_delete_rejected() {
    let env = Env::default();
    let (client, vet, _owner, pet_id) = setup(&env);
    let rid = add_record(&client, &env, &vet, pet_id, "Flu");

    let stranger = Address::generate(&env);
    client.delete_medical_record(&pet_id, &rid, &stranger);
}

// ---------------------------------------------------------------------------
// Purge before retention period rejected
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #162)")]
fn test_purge_before_retention_rejected() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let (client, vet, owner, pet_id) = setup(&env);
    let rid = add_record(&client, &env, &vet, pet_id, "Sprain");

    client.delete_medical_record(&pet_id, &rid, &owner);

    // Advance only 1 day — default retention is 30 days
    env.ledger().with_mut(|l| l.timestamp = 1_000_000 + 86_400);
    client.purge_expired_records(&pet_id, &owner); // should panic
}

// ---------------------------------------------------------------------------
// Purge after retention period succeeds (owner)
// ---------------------------------------------------------------------------

#[test]
fn test_owner_purge_after_retention_succeeds() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let (client, vet, owner, pet_id) = setup(&env);
    let rid = add_record(&client, &env, &vet, pet_id, "Fracture");

    client.delete_medical_record(&pet_id, &rid, &owner);

    // Advance past default 30-day retention
    env.ledger()
        .with_mut(|l| l.timestamp = 1_000_000 + 31 * 86_400);
    let purged = client.purge_expired_records(&pet_id, &owner);
    assert_eq!(purged, 1);

    // Record is gone from storage entirely
    assert!(client.get_medical_record(&rid).is_none());
}

// ---------------------------------------------------------------------------
// Admin purge tested
// ---------------------------------------------------------------------------

#[test]
fn test_admin_purge_after_retention_succeeds() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let (client, vet, owner, pet_id) = setup(&env);
    let rid = add_record(&client, &env, &vet, pet_id, "Infection");

    client.delete_medical_record(&pet_id, &rid, &owner);

    env.ledger()
        .with_mut(|l| l.timestamp = 1_000_000 + 31 * 86_400);

    // Use the admin address (init_admin was called in setup with a generated admin)
    // Re-derive admin by re-running setup logic — easier to use owner here since
    // we need the admin address. Use a fresh env to get the admin.
    let env2 = Env::default();
    env2.mock_all_auths();
    let contract_id2 = env2.register_contract(None, PetChainContract);
    let client2 = PetChainContractClient::new(&env2, &contract_id2);
    let admin2 = Address::generate(&env2);
    let vet2 = Address::generate(&env2);
    let owner2 = Address::generate(&env2);
    client2.init_admin(&admin2);
    client2.register_vet(
        &vet2,
        &String::from_str(&env2, "Dr. Admin"),
        &String::from_str(&env2, "LIC-ADM"),
        &String::from_str(&env2, "General"),
    );
    client2.verify_vet(&admin2, &vet2);
    let pet2 = client2.register_pet(
        &owner2,
        &String::from_str(&env2, "Rex"),
        &String::from_str(&env2, "2019-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env2, "Husky"),
        &String::from_str(&env2, "White"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );
    env2.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let rid2 = client2.add_medical_record(
        &pet2,
        &vet2,
        &String::from_str(&env2, "Infection"),
        &String::from_str(&env2, "Antibiotics"),
        &Vec::new(&env2),
        &String::from_str(&env2, "admin purge test"),
    );
    client2.delete_medical_record(&pet2, &rid2, &owner2);
    env2.ledger()
        .with_mut(|l| l.timestamp = 1_000_000 + 31 * 86_400);

    let purged = client2.purge_expired_records(&pet2, &admin2);
    assert_eq!(purged, 1);
    assert!(client2.get_medical_record(&rid2).is_none());
}

// ---------------------------------------------------------------------------
// Configurable retention period
// ---------------------------------------------------------------------------

#[test]
fn test_custom_retention_period_respected() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let contract_id = env.register_contract(None, PetChainContract);
    env.mock_all_auths();
    let client = PetChainContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let vet = Address::generate(&env);
    let owner = Address::generate(&env);

    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Custom"),
        &String::from_str(&env, "LIC-CUST"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Fido"),
        &String::from_str(&env, "2021-01-01"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "Poodle"),
        &String::from_str(&env, "Black"),
        &10,
        &None,
        &PrivacyLevel::Public,
    );

    // Set retention to 1 day
    client.set_retention_period(&admin, &86_400u64);

    let rid = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "OK"),
        &Vec::new(&env),
        &String::from_str(&env, "custom retention notes"),
    );
    client.delete_medical_record(&pet_id, &rid, &owner);

    // Advance 2 days — past the 1-day custom retention
    env.ledger()
        .with_mut(|l| l.timestamp = 1_000_000 + 2 * 86_400);
    let purged = client.purge_expired_records(&pet_id, &owner);
    assert_eq!(purged, 1);
}

// ---------------------------------------------------------------------------
// Purge only removes deleted records; active records untouched
// ---------------------------------------------------------------------------

#[test]
fn test_purge_leaves_active_records_intact() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let (client, vet, owner, pet_id) = setup(&env);

    let r_active = add_record(&client, &env, &vet, pet_id, "Active");
    let r_deleted = add_record(&client, &env, &vet, pet_id, "Deleted");

    client.delete_medical_record(&pet_id, &r_deleted, &owner);

    env.ledger()
        .with_mut(|l| l.timestamp = 1_000_000 + 31 * 86_400);
    let purged = client.purge_expired_records(&pet_id, &owner);
    assert_eq!(purged, 1);

    // Active record still visible
    assert!(client.get_medical_record(&r_active).is_some());
    let records = client.get_pet_medical_records(&pet_id, &0u64, &10u32);
    assert_eq!(records.len(), 1);
    assert_eq!(records.get(0).unwrap().id, r_active);
}

// ---------------------------------------------------------------------------
// Purge with no deleted records returns 0
// ---------------------------------------------------------------------------

#[test]
fn test_purge_no_deleted_records_returns_zero() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let (client, vet, owner, pet_id) = setup(&env);
    add_record(&client, &env, &vet, pet_id, "Healthy");

    env.ledger()
        .with_mut(|l| l.timestamp = 1_000_000 + 31 * 86_400);
    let purged = client.purge_expired_records(&pet_id, &owner);
    assert_eq!(purged, 0);
}

// ---------------------------------------------------------------------------
// Unauthorised purge rejected
// ---------------------------------------------------------------------------

#[test]
#[should_panic]
fn test_unauthorised_purge_rejected() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let (client, vet, owner, pet_id) = setup(&env);
    let rid = add_record(&client, &env, &vet, pet_id, "Flu");
    client.delete_medical_record(&pet_id, &rid, &owner);

    env.ledger()
        .with_mut(|l| l.timestamp = 1_000_000 + 31 * 86_400);
    let stranger = Address::generate(&env);
    client.purge_expired_records(&pet_id, &stranger);
}

// ---------------------------------------------------------------------------
// Window boundary: purge at exactly retention boundary succeeds
// ---------------------------------------------------------------------------

#[test]
fn test_purge_at_exact_retention_boundary_succeeds() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let (client, vet, owner, pet_id) = setup(&env);
    let rid = add_record(&client, &env, &vet, pet_id, "Boundary");

    client.delete_medical_record(&pet_id, &rid, &owner);

    // Advance exactly 30 days (default retention)
    env.ledger()
        .with_mut(|l| l.timestamp = 1_000_000 + 30 * 86_400);
    let purged = client.purge_expired_records(&pet_id, &owner);
    assert_eq!(purged, 1);
}
