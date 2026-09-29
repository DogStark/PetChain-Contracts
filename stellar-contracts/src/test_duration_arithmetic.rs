use crate::*;
use soroban_sdk::{
    testutils::Address as _,
    Address, Env, String, Vec,
};

const NOW: u64 = 1_700_000_000;

fn setup() -> (Env, PetChainContractClient<'static>, Address, Address, u64, u64) {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    env.ledger().set_timestamp(NOW);

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    client.init_admin(&admin);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Brown"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "VET123"),
        &String::from_str(&env, "General Practice"),
    );
    client.verify_vet(&admin, &vet);

    let vax_id = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabivax"),
        &NOW,
        &(NOW + 1000),
        &(NOW + 1000),
        &String::from_str(&env, "BATCH-1"),
    );

    (env, client, admin, vet, pet_id, vax_id)
}

#[test]
fn test_duration_window_end_never_overflows() {
    let end = duration_window_end(u64::MAX, u64::MAX);
    assert_eq!(end, u64::MAX);
}

#[test]
fn test_duration_window_end_zero_days() {
    let end = duration_window_end(NOW, 0);
    assert_eq!(end, NOW);
}

#[test]
fn test_duration_window_end_monotonic() {
    assert!(duration_window_end(NOW, 0) <= duration_window_end(NOW, 1));
    assert!(duration_window_end(NOW, 1000) <= duration_window_end(NOW, 2000));
}

#[test]
fn test_get_upcoming_vaccinations_no_panic_huge_days() {
    let (env, client, _admin, _vet, pet_id, _vax_id) = setup();

    // u64::MAX days must not panic; the saturated window returns at least the
    // vaccination whose next_due_date is within the capped window.
    let result = client.get_upcoming_vaccinations(&pet_id, &u64::MAX);
    assert!(!result.is_empty());
}

#[test]
fn test_get_upcoming_vaccinations_empty_when_overdue() {
    let (env, client, _admin, _vet, pet_id, _vax_id) = setup();

    // Advance time far past the dose due date; even with days_threshold=1 the
    // saturated arithmetic must classify the vaccination as overdue, not wrap.
    let far_future = u64::MAX - 86_400;
    env.ledger().with_mut(|l| l.timestamp = far_future);

    let result = client.get_upcoming_vaccinations(&pet_id, &1);
    assert!(result.is_empty());
}
