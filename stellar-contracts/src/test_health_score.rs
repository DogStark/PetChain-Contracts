use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env, String,
};

fn setup() -> (
    Env,
    PetChainContractClient<'static>,
    Address,
    Address,
    Address,
    u64,
) {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    client.init_admin(&admin);

    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Score"),
        &String::from_str(&env, "HEALTH-1"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Milo"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Mix"),
        &String::from_str(&env, "Brown"),
        &24u32,
        &None,
        &PrivacyLevel::Public,
    );

    (env, client, admin, owner, vet, pet_id)
}

#[test]
fn health_score_defaults_to_zero_when_components_are_missing() {
    let (env, client, _admin, _owner, _vet, pet_id) = setup();

    let score = client.get_health_score(&pet_id);
    assert_eq!(score.score, 0);
    assert_eq!(score.breakdown.vaccination, 0);
    assert_eq!(score.breakdown.lab_results, 0);
    assert_eq!(score.breakdown.activity, 0);
    assert_eq!(score.breakdown.insurance, 0);
    assert_eq!(score.computed_at, env.ledger().timestamp());
}

#[test]
fn health_score_uses_all_components_and_cache_ttl() {
    let (env, client, _admin, owner, vet, pet_id) = setup();

    env.ledger().set_timestamp(1_000);

    client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabies"),
        &1_000u64,
        &(1_000u64 + 90 * 86_400),
        &(1_000u64 + 90 * 86_400),
        &String::from_str(&env, "BATCH-1"),
    );

    client.add_lab_result(
        &pet_id,
        &vet,
        &String::from_str(&env, "CBC"),
        &String::from_str(&env, "All values within range"),
        &String::from_str(&env, "{}"),
        &None,
        &None,
    );

    env.ledger().set_timestamp(86_400 + 1_000);
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30u32,
        &4u32,
        &1_200u32,
        &String::from_str(&env, "Morning walk"),
    );

    env.ledger().set_timestamp(2 * 86_400 + 1_000);
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30u32,
        &4u32,
        &1_200u32,
        &String::from_str(&env, "Evening walk"),
    );

    env.ledger().set_timestamp(3 * 86_400 + 1_000);
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30u32,
        &4u32,
        &1_200u32,
        &String::from_str(&env, "Night walk"),
    );

    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-1"),
        &String::from_str(&env, "Good Pet Insurance"),
        &String::from_str(&env, "Premium"),
        &12_000u64,
        &100_000u64,
        &((3u64 * 86_400) + 1_000 + 90 * 86_400),
    );

    let initial = client.get_health_score(&pet_id);
    assert_eq!(initial.breakdown.vaccination, 100);
    assert_eq!(initial.breakdown.lab_results, 100);
    assert_eq!(initial.breakdown.activity, 10);
    assert_eq!(initial.breakdown.insurance, 100);
    assert_eq!(initial.score, 77);

    client.update_insurance_status(&owner, &pet_id, &String::from_str(&env, "POL-1"), &false);

    let cached = client.get_health_score(&pet_id);
    assert_eq!(cached.score, 77);
    assert_eq!(cached.computed_at, initial.computed_at);

    env.ledger().set_timestamp(initial.computed_at + 24 * 60 * 60 + 1);
    let recomputed = client.get_health_score(&pet_id);
    assert_eq!(recomputed.breakdown.insurance, 0);
    assert_eq!(recomputed.score, 52);
    assert!(recomputed.computed_at > initial.computed_at);
}
