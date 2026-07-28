use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Vec,
};

const DAY: u64 = 86_400;

struct TestContext {
    env: Env,
    client: PetChainContractClient<'static>,
    admin: Address,
    vet1: Address,
    vet2: Address,
    outsider: Address,
    pets: Vec<u64>,
}

fn setup() -> TestContext {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let vet1 = Address::generate(&env);
    let vet2 = Address::generate(&env);
    let outsider = Address::generate(&env);

    client.init_admin(&admin);
    register_verified_vet(&env, &client, &admin, &vet1, "LIC-VET-1");
    register_verified_vet(&env, &client, &admin, &vet2, "LIC-VET-2");

    let mut pets = Vec::new(&env);
    for name in ["Aster", "Bram", "Cleo", "Dune"] {
        pets.push_back(client.register_pet(
            &owner,
            &String::from_str(&env, name),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Female,
            &Species::Dog,
            &String::from_str(&env, "Retriever"),
            &String::from_str(&env, "Brown"),
            &20u32,
            &None,
            &PrivacyLevel::Public,
        ));
    }

    let now = 1_000_000;
    env.ledger().with_mut(|ledger| ledger.timestamp = now);
    let past = now.saturating_sub(DAY);
    let future = now.saturating_add(DAY);

    // Pet 0: overdue vax from vet1 only
    add_vaccination(&env, &client, pets.get(0).unwrap(), &vet1, past, past);

    // Pet 1: overdue vax from vet2, future vax from vet1
    add_vaccination(&env, &client, pets.get(1).unwrap(), &vet2, past, past);
    add_vaccination(&env, &client, pets.get(1).unwrap(), &vet1, past, future);

    // Pet 2: overdue vax from vet2, medical record from vet1
    add_vaccination(&env, &client, pets.get(2).unwrap(), &vet2, past, past);
    add_medical_record(&env, &client, pets.get(2).unwrap(), &vet1);

    // Pet 3: overdue vax from vet2 only
    add_vaccination(&env, &client, pets.get(3).unwrap(), &vet2, past, past);

    TestContext {
        env,
        client,
        admin,
        vet1,
        vet2,
        outsider,
        pets,
    }
}

fn register_verified_vet(
    env: &Env,
    client: &PetChainContractClient,
    admin: &Address,
    vet: &Address,
    license: &str,
) {
    client.register_vet(
        vet,
        &String::from_str(env, "Dr. Scope"),
        &String::from_str(env, license),
        &String::from_str(env, "General"),
    );
    client.verify_vet(admin, vet);
}

fn add_vaccination(
    env: &Env,
    client: &PetChainContractClient,
    pet_id: u64,
    vet: &Address,
    administered_at: u64,
    next_due_date: u64,
) {
    client.add_vaccination(
        &pet_id,
        vet,
        &VaccineType::Rabies,
        &String::from_str(env, "Rabies"),
        &administered_at,
        &next_due_date,
        &next_due_date,
        &String::from_str(env, "BATCH"),
    );
}

fn add_medical_record(
    env: &Env,
    client: &PetChainContractClient,
    pet_id: u64,
    vet: &Address,
) {
    client.add_medical_record(
        &pet_id,
        vet,
        &String::from_str(env, "Diagnosis"),
        &String::from_str(env, "Treatment"),
        &Vec::new(env),
        &String::from_str(env, "notes"),
    );
}

// --- Authorization tests ---

#[test]
#[should_panic(expected = "Error(Contract, #28)")]
fn non_admin_cannot_request_global_view() {
    let ctx = setup();
    ctx.client
        .get_pets_overdue_vaccinations(&ctx.vet1, &None, &0, &10);
}

#[test]
#[should_panic(expected = "Error(Contract, #28)")]
fn non_admin_cannot_filter_by_another_vet() {
    let ctx = setup();
    ctx.client
        .get_pets_overdue_vaccinations(&ctx.vet1, &Some(ctx.vet2.clone()), &0, &10);
}

#[test]
#[should_panic(expected = "Error(Contract, #28)")]
fn non_vet_cannot_filter_by_self() {
    let ctx = setup();
    ctx.client.get_pets_overdue_vaccinations(
        &ctx.outsider,
        &Some(ctx.outsider.clone()),
        &0,
        &10,
    );
}

// --- Vet-scoped filter tests ---

#[test]
fn vet1_sees_only_pets_with_vet1_care_records() {
    let ctx = setup();
    // Vet1 has: vax on pet0 (overdue), future vax on pet1, medical record on pet2
    // Overdue pets with vet1 care: pet0 (overdue vax from vet1),
    //   pet1 (overdue globally, vet1 has future vax = care record),
    //   pet2 (overdue globally, vet1 has medical record)
    let result =
        ctx.client
            .get_pets_overdue_vaccinations(&ctx.vet1, &Some(ctx.vet1.clone()), &0, &10);
    assert_eq!(result.len(), 3);
    assert!(result.contains(ctx.pets.get(0).unwrap()));
    assert!(result.contains(ctx.pets.get(1).unwrap()));
    assert!(result.contains(ctx.pets.get(2).unwrap()));
}

#[test]
fn vet2_sees_only_pets_with_vet2_care_records() {
    let ctx = setup();
    // Vet2 has: vax on pet1, pet2, pet3 (all overdue)
    let result =
        ctx.client
            .get_pets_overdue_vaccinations(&ctx.vet2, &Some(ctx.vet2.clone()), &0, &10);
    assert_eq!(result.len(), 3);
    assert!(result.contains(ctx.pets.get(1).unwrap()));
    assert!(result.contains(ctx.pets.get(2).unwrap()));
    assert!(result.contains(ctx.pets.get(3).unwrap()));
}

// --- Admin-scoped tests ---

#[test]
fn admin_none_filter_returns_all_overdue_pets() {
    let ctx = setup();
    // All 4 pets have at least one overdue vaccination
    let all = ctx
        .client
        .get_pets_overdue_vaccinations(&ctx.admin, &None, &0, &10);
    assert_eq!(all.len(), 4);
}

#[test]
fn admin_can_filter_by_any_vet() {
    let ctx = setup();
    let filtered =
        ctx.client
            .get_pets_overdue_vaccinations(&ctx.admin, &Some(ctx.vet1.clone()), &0, &10);
    // Admin filtering by vet1: pets 0, 1, 2
    assert_eq!(filtered.len(), 3);
    assert!(filtered.contains(ctx.pets.get(0).unwrap()));
    assert!(filtered.contains(ctx.pets.get(1).unwrap()));
    assert!(filtered.contains(ctx.pets.get(2).unwrap()));
}

// --- Pagination tests ---

#[test]
fn zero_limit_returns_empty() {
    let ctx = setup();
    let result =
        ctx.client
            .get_pets_overdue_vaccinations(&ctx.admin, &None, &0, &0);
    assert!(result.is_empty());
}

#[test]
fn pagination_offset_and_limit_work() {
    let ctx = setup();
    // All 4 overdue pets; skip first 2, take 1
    let page =
        ctx.client
            .get_pets_overdue_vaccinations(&ctx.admin, &None, &2, &1);
    assert_eq!(page.len(), 1);
}
