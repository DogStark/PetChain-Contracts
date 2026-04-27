use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_add_insurance_policy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2021-05-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden"),
        &String::from_str(&env, "Golden Retriever"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31536000; // 1 year
    let result = client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "INS-2024-001"),
        &String::from_str(&env, "PetGuard Insurance"),
        &String::from_str(&env, "Premium"),
        &2500,
        &100000,
        &expiry,
    );

    assert_eq!(result, true);
}

#[test]
fn test_get_pet_insurance() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2022-03-10"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "White"),
        &String::from_str(&env, "Persian"),
        &5,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31536000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "CAT-INS-789"),
        &String::from_str(&env, "Feline Care Plus"),
        &String::from_str(&env, "Basic"),
        &1500,
        &25000,
        &expiry,
    );

    let policy = client.get_pet_insurance(&pet_id);
    assert!(policy.is_some());

    let policy = policy.unwrap();
    assert_eq!(policy.policy_id, String::from_str(&env, "CAT-INS-789"));
    assert_eq!(policy.provider, String::from_str(&env, "Feline Care Plus"));
    assert_eq!(policy.coverage_type, String::from_str(&env, "Basic"));
    assert_eq!(policy.premium, 1500);
    assert_eq!(policy.coverage_limit, 25000);
    assert_eq!(policy.expiry_date, expiry);
    assert_eq!(policy.active, true);
}

#[test]
fn test_update_insurance_status() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Charlie"),
        &String::from_str(&env, "2020-08-20"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Black"),
        &String::from_str(&env, "Labrador"),
        &28,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31536000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "DOG-2024-456"),
        &String::from_str(&env, "Canine Coverage"),
        &String::from_str(&env, "Comprehensive"),
        &3000,
        &150000,
        &expiry,
    );

    // Deactivate insurance
    let result = client.update_insurance_status(
        &owner,
        &pet_id,
        &String::from_str(&env, "DOG-2024-456"),
        &false,
    );
    assert_eq!(result, true);

    let policy = client.get_pet_insurance(&pet_id).unwrap();
    assert_eq!(policy.active, false);

    // Reactivate insurance
    let result = client.update_insurance_status(
        &owner,
        &pet_id,
        &String::from_str(&env, "DOG-2024-456"),
        &true,
    );
    assert_eq!(result, true);

    let policy = client.get_pet_insurance(&pet_id).unwrap();
    assert_eq!(policy.active, true);

    // Wrong policy ID returns false
    let result = client.update_insurance_status(
        &owner,
        &pet_id,
        &String::from_str(&env, "NONEXISTENT-ID"),
        &false,
    );
    assert_eq!(result, false);
}

#[test]
fn test_insurance_for_nonexistent_pet() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let expiry = env.ledger().timestamp() + 31536000;
    let result = client.add_insurance_policy(
        &999,
        &String::from_str(&env, "FAKE-001"),
        &String::from_str(&env, "No Provider"),
        &String::from_str(&env, "None"),
        &1000,
        &10000,
        &expiry,
    );

    assert_eq!(result, false);
}

#[test]
fn test_get_insurance_for_pet_without_policy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Bella"),
        &String::from_str(&env, "2023-01-01"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "Brown"),
        &String::from_str(&env, "Beagle"),
        &12,
        &None,
        &PrivacyLevel::Public,
    );

    let policy = client.get_pet_insurance(&pet_id);
    assert!(policy.is_none());
}

#[test]
fn test_update_nonexistent_insurance() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rocky"),
        &String::from_str(&env, "2022-06-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Gray"),
        &String::from_str(&env, "Husky"),
        &25,
        &None,
        &PrivacyLevel::Public,
    );

    let result = client.update_insurance_status(
        &owner,
        &pet_id,
        &String::from_str(&env, "FAKE-POLICY"),
        &false,
    );
    assert_eq!(result, false);
}

#[test]
fn test_insurance_policy_fields() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Milo"),
        &String::from_str(&env, "2021-11-30"),
        &Gender::Male,
        &Species::Cat,
        &String::from_str(&env, "Orange"),
        &String::from_str(&env, "Tabby"),
        &8,
        &None,
        &PrivacyLevel::Public,
    );

    let start_time = env.ledger().timestamp();
    let expiry = start_time + 31536000;

    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POLICY-XYZ-123"),
        &String::from_str(&env, "Pet Health Co"),
        &String::from_str(&env, "Accident & Illness"),
        &1800,
        &75000,
        &expiry,
    );

    let policy = client.get_pet_insurance(&pet_id).unwrap();

    // Verify all fields
    assert_eq!(policy.policy_id, String::from_str(&env, "POLICY-XYZ-123"));
    assert_eq!(policy.provider, String::from_str(&env, "Pet Health Co"));
    assert_eq!(
        policy.coverage_type,
        String::from_str(&env, "Accident & Illness")
    );
    assert_eq!(policy.premium, 1800);
    assert_eq!(policy.coverage_limit, 75000);
    assert_eq!(policy.start_date, start_time);
    assert_eq!(policy.expiry_date, expiry);
    assert_eq!(policy.active, true);
}

#[test]
fn test_multiple_policies_per_pet() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Brown"),
        &String::from_str(&env, "Poodle"),
        &10,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31536000;

    // Add first policy
    let r1 = client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-001"),
        &String::from_str(&env, "Provider A"),
        &String::from_str(&env, "Basic"),
        &1000,
        &50000,
        &expiry,
    );
    assert_eq!(r1, true);

    // Add second policy
    let r2 = client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-002"),
        &String::from_str(&env, "Provider B"),
        &String::from_str(&env, "Premium"),
        &2000,
        &100000,
        &expiry,
    );
    assert_eq!(r2, true);

    // Add third policy
    let r3 = client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-003"),
        &String::from_str(&env, "Provider C"),
        &String::from_str(&env, "Comprehensive"),
        &3000,
        &200000,
        &expiry,
    );
    assert_eq!(r3, true);

    // get_all_pet_policies returns all three
    let policies = client.get_all_pet_policies(&pet_id);
    assert_eq!(policies.len(), 3);

    assert_eq!(
        policies.get(0).unwrap().policy_id,
        String::from_str(&env, "POL-001")
    );
    assert_eq!(
        policies.get(1).unwrap().policy_id,
        String::from_str(&env, "POL-002")
    );
    assert_eq!(
        policies.get(2).unwrap().policy_id,
        String::from_str(&env, "POL-003")
    );
}

#[test]
fn test_get_pet_insurance_returns_latest_policy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Daisy"),
        &String::from_str(&env, "2021-07-04"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Calico"),
        &String::from_str(&env, "Domestic"),
        &6,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31536000;

    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "FIRST-001"),
        &String::from_str(&env, "Old Provider"),
        &String::from_str(&env, "Basic"),
        &500,
        &10000,
        &expiry,
    );

    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "LATEST-002"),
        &String::from_str(&env, "New Provider"),
        &String::from_str(&env, "Premium"),
        &1500,
        &80000,
        &expiry,
    );

    // get_pet_insurance returns the most recently added policy
    let policy = client.get_pet_insurance(&pet_id).unwrap();
    assert_eq!(policy.policy_id, String::from_str(&env, "LATEST-002"));
    assert_eq!(policy.provider, String::from_str(&env, "New Provider"));
}

#[test]
fn test_get_all_pet_policies_empty() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Nemo"),
        &String::from_str(&env, "2023-03-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "White"),
        &String::from_str(&env, "Dalmatian"),
        &20,
        &None,
        &PrivacyLevel::Public,
    );

    let policies = client.get_all_pet_policies(&pet_id);
    assert_eq!(policies.len(), 0);
}

#[test]
fn test_policies_independent_across_pets() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let expiry = env.ledger().timestamp() + 31536000;

    let pet1 = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2019-05-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Black"),
        &String::from_str(&env, "Rottweiler"),
        &40,
        &None,
        &PrivacyLevel::Public,
    );

    let pet2 = client.register_pet(
        &owner,
        &String::from_str(&env, "Whiskers"),
        &String::from_str(&env, "2022-09-10"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Gray"),
        &String::from_str(&env, "Siamese"),
        &4,
        &None,
        &PrivacyLevel::Public,
    );

    client.add_insurance_policy(
        &pet1,
        &String::from_str(&env, "DOG-POL-1"),
        &String::from_str(&env, "Dog Insurer"),
        &String::from_str(&env, "Basic"),
        &1000,
        &50000,
        &expiry,
    );
    client.add_insurance_policy(
        &pet1,
        &String::from_str(&env, "DOG-POL-2"),
        &String::from_str(&env, "Dog Insurer"),
        &String::from_str(&env, "Premium"),
        &2000,
        &100000,
        &expiry,
    );

    client.add_insurance_policy(
        &pet2,
        &String::from_str(&env, "CAT-POL-1"),
        &String::from_str(&env, "Cat Insurer"),
        &String::from_str(&env, "Basic"),
        &800,
        &30000,
        &expiry,
    );

    let pet1_policies = client.get_all_pet_policies(&pet1);
    let pet2_policies = client.get_all_pet_policies(&pet2);

    assert_eq!(pet1_policies.len(), 2);
    assert_eq!(pet2_policies.len(), 1);

    assert_eq!(
        pet1_policies.get(0).unwrap().policy_id,
        String::from_str(&env, "DOG-POL-1")
    );
    assert_eq!(
        pet1_policies.get(1).unwrap().policy_id,
        String::from_str(&env, "DOG-POL-2")
    );
    assert_eq!(
        pet2_policies.get(0).unwrap().policy_id,
        String::from_str(&env, "CAT-POL-1")
    );
}

#[test]
fn test_update_insurance_status_targets_specific_policy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Spot"),
        &String::from_str(&env, "2021-04-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "White"),
        &String::from_str(&env, "Dalmatian"),
        &22,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31536000;

    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-A"),
        &String::from_str(&env, "Provider A"),
        &String::from_str(&env, "Basic"),
        &1000,
        &50000,
        &expiry,
    );
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-B"),
        &String::from_str(&env, "Provider B"),
        &String::from_str(&env, "Premium"),
        &2000,
        &100000,
        &expiry,
    );

    // Deactivate only POL-A; POL-B should remain active
    let result = client.update_insurance_status(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-A"),
        &false,
    );
    assert_eq!(result, true);

    let all = client.get_all_pet_policies(&pet_id);
    let pol_a = all.get(0).unwrap();
    let pol_b = all.get(1).unwrap();
    assert_eq!(pol_a.active, false);
    assert_eq!(pol_b.active, true);

    // Deactivate POL-B independently
    let result = client.update_insurance_status(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-B"),
        &false,
    );
    assert_eq!(result, true);

    let all = client.get_all_pet_policies(&pet_id);
    assert_eq!(all.get(0).unwrap().active, false);
    assert_eq!(all.get(1).unwrap().active, false);

    // Reactivate POL-A only
    let result = client.update_insurance_status(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-A"),
        &true,
    );
    assert_eq!(result, true);

    let all = client.get_all_pet_policies(&pet_id);
    assert_eq!(all.get(0).unwrap().active, true);
    assert_eq!(all.get(1).unwrap().active, false);
}

#[test]
fn test_update_insurance_status_unknown_policy_id_returns_false() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Fido"),
        &String::from_str(&env, "2020-06-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Brown"),
        &String::from_str(&env, "Mutt"),
        &15,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31536000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "REAL-POL"),
        &String::from_str(&env, "Provider"),
        &String::from_str(&env, "Basic"),
        &1000,
        &50000,
        &expiry,
    );

    let result = client.update_insurance_status(
        &owner,
        &pet_id,
        &String::from_str(&env, "WRONG-ID"),
        &false,
    );
    assert_eq!(result, false);

    // Original policy untouched
    let policy = client.get_pet_insurance(&pet_id).unwrap();
    assert_eq!(policy.active, true);
}
