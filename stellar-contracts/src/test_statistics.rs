use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup_env() -> (Env, PetChainContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    // Initialize admin
    let admin = Address::generate(&env);
    client.init_admin(&admin);

    (env, client, admin)
}

fn register_pet_with_species(
    client: &PetChainContractClient,
    env: &Env,
    owner: &Address,
    species: Species,
) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, "Pet"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &species,
        &String::from_str(env, "Breed"),
        &String::from_str(env, "Color"),
        &10u32,
        &None,
        &PrivacyLevel::Public,
    )
}

#[test]
fn test_get_total_pets() {
    let (env, client, _admin) = setup_env();
    let owner = Address::generate(&env);

    assert_eq!(client.get_total_pets(), 0);

    register_pet_with_species(&client, &env, &owner, Species::Dog);
    assert_eq!(client.get_total_pets(), 1);

    register_pet_with_species(&client, &env, &owner, Species::Cat);
    assert_eq!(client.get_total_pets(), 2);
}

#[test]
fn test_get_species_count() {
    let (env, client, _admin) = setup_env();
    let owner = Address::generate(&env);

    register_pet_with_species(&client, &env, &owner, Species::Dog);
    register_pet_with_species(&client, &env, &owner, Species::Dog);
    register_pet_with_species(&client, &env, &owner, Species::Cat);

    assert_eq!(client.get_species_count(&String::from_str(&env, "Dog")), 2);
    assert_eq!(client.get_species_count(&String::from_str(&env, "Cat")), 1);
    assert_eq!(client.get_species_count(&String::from_str(&env, "Bird")), 0);
}

#[test]
fn test_get_pets_by_species_pagination() {
    let (env, client, _admin) = setup_env();
    let owner = Address::generate(&env);

    register_pet_with_species(&client, &env, &owner, Species::Dog);
    register_pet_with_species(&client, &env, &owner, Species::Dog);
    register_pet_with_species(&client, &env, &owner, Species::Dog);

    let dogs_all = client.get_pets_by_species(&String::from_str(&env, "Dog"), &0u64, &10u32);
    assert_eq!(dogs_all.len(), 3);

    let dogs_page = client.get_pets_by_species(&String::from_str(&env, "Dog"), &1u64, &1u32);
    assert_eq!(dogs_page.len(), 1);

    let dogs_empty = client.get_pets_by_species(&String::from_str(&env, "Dog"), &5u64, &1u32);
    assert_eq!(dogs_empty.len(), 0);
}

#[test]
fn test_get_active_pets_count() {
    let (env, client, _admin) = setup_env();
    let owner = Address::generate(&env);

    let id1 = register_pet_with_species(&client, &env, &owner, Species::Dog);
    let id2 = register_pet_with_species(&client, &env, &owner, Species::Cat);

    assert_eq!(client.get_active_pets_count(), 0);

    client.activate_pet(&id1);
    assert_eq!(client.get_active_pets_count(), 1);

    client.activate_pet(&id2);
    assert_eq!(client.get_active_pets_count(), 2);

    client.deactivate_pet(&id1);
    assert_eq!(client.get_active_pets_count(), 1);

    // Activating an already-active pet must not double-count.
    client.activate_pet(&id2);
    assert_eq!(client.get_active_pets_count(), 1);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_activate_pet_requires_owner_auth() {
    let env = Env::default();
    env.mock_all_auths(); // Mock auth for registration
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let _non_owner = Address::generate(&env);

    let pet_id = register_pet_with_species(&client, &env, &owner, Species::Dog);

    // Clear auth mocking so real auth checks apply
    env.set_auths(&[]);

    // Attempting to activate pet should panic due to missing auth
    client.activate_pet(&pet_id);
}

#[test]
fn test_get_vet_stats_initial_state() {
    let (env, client, _admin) = setup_env();
    let vet = Address::generate(&env);

    let stats = client.get_vet_stats(&vet);
    assert_eq!(stats.total_records, 0);
    assert_eq!(stats.total_vaccinations, 0);
    assert_eq!(stats.total_treatments, 0);
    assert_eq!(stats.pets_treated, 0);
}

#[test]
fn test_vet_stats_update_after_vaccination() {
    let (env, client, admin) = setup_env();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    // Register vet
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "VET123"),
        &String::from_str(&env, "Animal Clinic"),
    );
    client.verify_vet(&admin, &vet);

    // Register pet
    let pet_id = register_pet_with_species(&client, &env, &owner, Species::Dog);

    // Initial stats should be zero
    let initial_stats = client.get_vet_stats(&vet);
    assert_eq!(initial_stats.total_records, 0);
    assert_eq!(initial_stats.total_vaccinations, 0);
    assert_eq!(initial_stats.pets_treated, 0);

    // Add vaccination
    client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabies"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 365 * 24 * 60 * 60),
        &String::from_str(&env, "BATCH123"),
    );

    // Check stats after vaccination
    let stats = client.get_vet_stats(&vet);
    assert_eq!(stats.total_records, 1);
    assert_eq!(stats.total_vaccinations, 1);
    assert_eq!(stats.total_treatments, 0);
    assert_eq!(stats.pets_treated, 1);
}

#[test]
fn test_vet_stats_update_after_medical_record() {
    let (env, client, admin) = setup_env();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    // Register vet
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Jones"),
        &String::from_str(&env, "VET456"),
        &String::from_str(&env, "Pet Hospital"),
    );
    client.verify_vet(&admin, &vet);

    // Register pet
    let pet_id = register_pet_with_species(&client, &env, &owner, Species::Cat);

    // Add medical record
    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "Healthy"),
        &Vec::new(&env),
        &String::from_str(&env, "Regular checkup"),
    );

    // Check stats after medical record
    let stats = client.get_vet_stats(&vet);
    assert_eq!(stats.total_records, 1);
    assert_eq!(stats.total_vaccinations, 0);
    assert_eq!(stats.total_treatments, 1);
    assert_eq!(stats.pets_treated, 1);
}

#[test]
fn test_vet_stats_multiple_operations_same_pet() {
    let (env, client, admin) = setup_env();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    // Register vet
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Brown"),
        &String::from_str(&env, "VET789"),
        &String::from_str(&env, "Vet Clinic"),
    );
    client.verify_vet(&admin, &vet);

    // Register pet
    let pet_id = register_pet_with_species(&client, &env, &owner, Species::Dog);

    // Add vaccination
    client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Bordetella,
        &String::from_str(&env, "Bordetella"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 365 * 24 * 60 * 60),
        &String::from_str(&env, "BATCH456"),
    );

    // Add medical record
    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Treatment"),
        &String::from_str(&env, "Treated"),
        &Vec::new(&env),
        &String::from_str(&env, "Treatment notes"),
    );

    // Check stats - pets_treated should still be 1 (same pet)
    let stats = client.get_vet_stats(&vet);
    assert_eq!(stats.total_records, 2);
    assert_eq!(stats.total_vaccinations, 1);
    assert_eq!(stats.total_treatments, 1);
    assert_eq!(stats.pets_treated, 1);
}

#[test]
fn test_vet_stats_multiple_pets() {
    let (env, client, admin) = setup_env();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    // Register vet
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Wilson"),
        &String::from_str(&env, "VET101"),
        &String::from_str(&env, "Animal Care"),
    );
    client.verify_vet(&admin, &vet);

    // Register two pets
    let pet_id1 = register_pet_with_species(&client, &env, &owner, Species::Dog);
    let pet_id2 = register_pet_with_species(&client, &env, &owner, Species::Cat);

    // Add vaccination for first pet
    client.add_vaccination(
        &pet_id1,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabies"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 365 * 24 * 60 * 60),
        &String::from_str(&env, "BATCH789"),
    );

    // Add medical record for second pet
    client.add_medical_record(
        &pet_id2,
        &vet,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "Healthy"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes"),
    );

    // Check stats - should have 2 pets treated
    let stats = client.get_vet_stats(&vet);
    assert_eq!(stats.total_records, 2);
    assert_eq!(stats.total_vaccinations, 1);
    assert_eq!(stats.total_treatments, 1);
    assert_eq!(stats.pets_treated, 2);
}

// ── Vet Review Tests ──────────────────────────────────────────────────────────

fn setup_vet_and_pet(
    client: &PetChainContractClient,
    env: &Env,
    admin: &Address,
) -> (Address, Address, u64) {
    let owner = Address::generate(env);
    let vet = Address::generate(env);

    // Register vet
    client.register_vet(
        &vet,
        &String::from_str(env, "Dr. Smith"),
        &String::from_str(env, "VET123"),
        &String::from_str(env, "Animal Clinic"),
    );
    client.verify_vet(admin, &vet);

    // Register pet
    let pet_id = register_pet_with_species(client, env, &owner, Species::Dog);

    (owner, vet, pet_id)
}

#[test]
fn test_get_vet_reviews_empty_initially() {
    let (env, client, admin) = setup_env();
    let (_owner, vet, _pet_id) = setup_vet_and_pet(&client, &env, &admin);

    let reviews = client.get_vet_reviews(&vet, &0u64, &10u32);
    assert_eq!(reviews.len(), 0);
}

#[test]
fn test_add_and_get_vet_review() {
    let (env, client, admin) = setup_env();
    let (owner, vet, _pet_id) = setup_vet_and_pet(&client, &env, &admin);

    // Add a review
    let review_id = client.add_vet_review(&owner, &vet, &5, &String::from_str(&env, "Excellent vet!"));
    assert!(review_id > 0);

    // Get reviews
    let reviews = client.get_vet_reviews(&vet, &0u64, &10u32);
    assert_eq!(reviews.len(), 1);
    assert_eq!(reviews.get(0).unwrap().rating, 5);
}

#[test]
fn test_get_vet_reviews_pagination() {
    let (env, client, admin) = setup_env();
    let (owner, vet, _pet_id) = setup_vet_and_pet(&client, &env, &admin);

    // Add 5 reviews from different owners
    for i in 1..=5 {
        let review_owner = Address::generate(&env);
        client.add_vet_review(&review_owner, &vet, &i, &String::from_str(&env, "Review"));
    }

    // Get all reviews
    let all_reviews = client.get_vet_reviews(&vet, &0u64, &10u32);
    assert_eq!(all_reviews.len(), 5);

    // Get first page (2 reviews)
    let page1 = client.get_vet_reviews(&vet, &0u64, &2u32);
    assert_eq!(page1.len(), 2);
    assert_eq!(page1.get(0).unwrap().rating, 1);
    assert_eq!(page1.get(1).unwrap().rating, 2);

    // Get second page (2 reviews)
    let page2 = client.get_vet_reviews(&vet, &2u64, &2u32);
    assert_eq!(page2.len(), 2);
    assert_eq!(page2.get(0).unwrap().rating, 3);
    assert_eq!(page2.get(1).unwrap().rating, 4);

    // Get last page (1 review)
    let page3 = client.get_vet_reviews(&vet, &4u64, &2u32);
    assert_eq!(page3.len(), 1);
    assert_eq!(page3.get(0).unwrap().rating, 5);

    // Get beyond available reviews
    let empty = client.get_vet_reviews(&vet, &10u64, &5u32);
    assert_eq!(empty.len(), 0);
}

#[test]
fn test_get_vet_average_rating() {
    let (env, client, admin) = setup_env();
    let (owner, vet, _pet_id) = setup_vet_and_pet(&client, &env, &admin);

    // No reviews - should return 0
    let avg = client.get_vet_average_rating(&vet);
    assert_eq!(avg, 0);

    // Add review with rating 5
    let owner1 = Address::generate(&env);
    client.add_vet_review(&owner1, &vet, &5, &String::from_str(&env, "Great"));
    let avg = client.get_vet_average_rating(&vet);
    assert_eq!(avg, 500); // 5.0 * 100 = 500

    // Add review with rating 3
    let owner2 = Address::generate(&env);
    client.add_vet_review(&owner2, &vet, &3, &String::from_str(&env, "Good"));
    let avg = client.get_vet_average_rating(&vet);
    assert_eq!(avg, 400); // (5+3)/2 = 4.0 * 100 = 400

    // Add review with rating 4
    let owner3 = Address::generate(&env);
    client.add_vet_review(&owner3, &vet, &4, &String::from_str(&env, "Very good"));
    let avg = client.get_vet_average_rating(&vet);
    assert_eq!(avg, 400); // (5+3+4)/3 = 4.0 * 100 = 400
}

#[test]
fn test_get_vet_average_rating_with_fractional() {
    let (env, client, admin) = setup_env();
    let (owner, vet, _pet_id) = setup_vet_and_pet(&client, &env, &admin);

    // Add reviews: 5, 4, 4 = avg 4.333... -> 433
    let owner1 = Address::generate(&env);
    client.add_vet_review(&owner1, &vet, &5, &String::from_str(&env, "Excellent"));
    
    let owner2 = Address::generate(&env);
    client.add_vet_review(&owner2, &vet, &4, &String::from_str(&env, "Good"));
    
    let owner3 = Address::generate(&env);
    client.add_vet_review(&owner3, &vet, &4, &String::from_str(&env, "Good"));

    let avg = client.get_vet_average_rating(&vet);
    assert_eq!(avg, 433); // (5+4+4)/3 = 4.333... * 100 = 433 (integer division)
}

#[test]
fn test_duplicate_review_prevented() {
    let (env, client, admin) = setup_env();
    let (owner, vet, _pet_id) = setup_vet_and_pet(&client, &env, &admin);

    // First review should succeed
    let result1 = client.try_add_vet_review(&owner, &vet, &5, &String::from_str(&env, "First review"));
    assert!(result1.is_ok());

    // Second review from same owner should fail
    let result2 = client.try_add_vet_review(&owner, &vet, &4, &String::from_str(&env, "Second review"));
    assert!(result2.is_err());
}

#[test]
fn test_get_pet_count_by_owner_zero_for_new_owner() {
    let (env, client, _admin) = setup_env();
    let owner = Address::generate(&env);
    assert_eq!(client.get_pet_count_by_owner(&owner), 0);
}

#[test]
fn test_get_pet_count_by_owner_increments_on_register() {
    let (env, client, _admin) = setup_env();
    let owner = Address::generate(&env);

    register_pet_with_species(&client, &env, &owner, Species::Dog);
    assert_eq!(client.get_pet_count_by_owner(&owner), 1);

    register_pet_with_species(&client, &env, &owner, Species::Cat);
    assert_eq!(client.get_pet_count_by_owner(&owner), 2);
}

#[test]
fn test_get_pet_count_by_owner_independent_per_owner() {
    let (env, client, _admin) = setup_env();
    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);

    register_pet_with_species(&client, &env, &owner1, Species::Dog);
    register_pet_with_species(&client, &env, &owner1, Species::Dog);
    register_pet_with_species(&client, &env, &owner2, Species::Cat);

    assert_eq!(client.get_pet_count_by_owner(&owner1), 2);
    assert_eq!(client.get_pet_count_by_owner(&owner2), 1);
}
