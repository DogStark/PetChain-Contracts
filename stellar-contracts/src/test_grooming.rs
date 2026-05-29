#[cfg(test)]
mod test_grooming {
    use crate::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    fn setup() -> (Env, Address, Address, u64) {
        let env = Env::default();
        env.mock_all_auths();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, PetChainContract);

        let owner = Address::generate(&env);
        let client = PetChainContractClient::new(&env, &contract_id);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Retriever"),
            &PrivacyLevel::Public,
        );

        (env, contract_id, owner, pet_id)
    }

    fn cid_for_index(env: &Env, idx: usize) -> String {
        let values = [
            "bafyphoto1",
            "bafyphoto2",
            "bafyphoto3",
            "bafyphoto4",
            "bafyphoto5",
        ];
        String::from_str(env, values[idx % values.len()])
    }

    #[test]
    #[should_panic(expected = "Record photo limit exceeded")]
    fn test_per_record_photo_limit_rejected() {
        let (env, contract_id, owner, pet_id) = setup();
        let client = PetChainContractClient::new(&env, &contract_id);
        let record_id = client.add_grooming_record(
            &pet_id,
            &owner,
            &String::from_str(&env, "Initial grooming"),
        );

        for idx in 0..5 {
            assert!(client.add_grooming_photo(&record_id, &cid_for_index(&env, idx)));
        }

        client.add_grooming_photo(&record_id, &String::from_str(&env, "bafyphoto6"));
    }

    #[test]
    fn test_remove_decrements_count_correctly() {
        let (env, contract_id, owner, pet_id) = setup();
        let client = PetChainContractClient::new(&env, &contract_id);
        let record_id =
            client.add_grooming_record(&pet_id, &owner, &String::from_str(&env, "Nail trim"));

        assert!(client.add_grooming_photo(&record_id, &String::from_str(&env, "bafyphoto1")));
        assert!(client.add_grooming_photo(&record_id, &String::from_str(&env, "bafyphoto2")));

        let record = client.get_grooming_record(&record_id).unwrap();
        assert_eq!(record.photos.len(), 2);

        assert!(client.remove_grooming_photo(&record_id, &String::from_str(&env, "bafyphoto1")));

        let record = client.get_grooming_record(&record_id).unwrap();
        assert_eq!(record.photos.len(), 1);

        assert!(client.add_grooming_photo(&record_id, &String::from_str(&env, "bafyphoto3")));

        let record = client.get_grooming_record(&record_id).unwrap();
        assert_eq!(record.photos.len(), 2);
    }

    #[test]
    #[should_panic(expected = "Invalid CID")]
    fn test_invalid_cid_rejected() {
        let (env, contract_id, owner, pet_id) = setup();
        let client = PetChainContractClient::new(&env, &contract_id);
        let record_id =
            client.add_grooming_record(&pet_id, &owner, &String::from_str(&env, "Bath"));

        client.add_grooming_photo(&record_id, &String::from_str(&env, ""));
    }

    #[test]
    #[should_panic(expected = "Pet photo limit exceeded")]
    fn test_per_pet_photo_limit_rejected() {
        let (env, contract_id, owner, pet_id) = setup();
        let client = PetChainContractClient::new(&env, &contract_id);

        for _ in 0..10 {
            let record_id = client.add_grooming_record(
                &pet_id,
                &owner,
                &String::from_str(&env, "Grooming session"),
            );
            for idx in 0..5 {
                assert!(client.add_grooming_photo(&record_id, &cid_for_index(&env, idx)));
            }
        }

        let extra_record = client.add_grooming_record(
            &pet_id,
            &owner,
            &String::from_str(&env, "Overflow session"),
        );
        client.add_grooming_photo(&extra_record, &String::from_str(&env, "bafyoverflow"));
    }

use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_add_grooming_record() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    let grooming_id = client.add_grooming_record(
        &pet_id,
        &String::from_str(&env, "Full Grooming"),
        &String::from_str(&env, "Pet Spa"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 2592000),
        &5000,
        &String::from_str(&env, "Haircut and bath"),
    );

    assert_eq!(grooming_id, 1);
}

#[test]
fn test_get_grooming_history() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    client.add_grooming_record(
        &pet_id,
        &String::from_str(&env, "Full Grooming"),
        &String::from_str(&env, "Pet Spa"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 2592000),
        &5000,
        &String::from_str(&env, "Haircut and bath"),
    );

    client.add_grooming_record(
        &pet_id,
        &String::from_str(&env, "Nail Trim"),
        &String::from_str(&env, "Pet Spa"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 1296000),
        &1500,
        &String::from_str(&env, "Nail trimming only"),
    );

    let history = client.get_grooming_history(&pet_id);
    assert_eq!(history.len(), 2);
}

#[test]
fn test_get_next_grooming_date() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    client.add_grooming_record(
        &pet_id,
        &String::from_str(&env, "Full Grooming"),
        &String::from_str(&env, "Pet Spa"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 2592000),
        &5000,
        &String::from_str(&env, "Haircut and bath"),
    );

    let next_date = client.get_next_grooming_date(&pet_id);
    assert!(next_date > 0);
}

#[test]
fn test_get_grooming_expenses() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    client.add_grooming_record(
        &pet_id,
        &String::from_str(&env, "Full Grooming"),
        &String::from_str(&env, "Pet Spa"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 2592000),
        &5000,
        &String::from_str(&env, "Haircut and bath"),
    );

    client.add_grooming_record(
        &pet_id,
        &String::from_str(&env, "Nail Trim"),
        &String::from_str(&env, "Pet Spa"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 1296000),
        &1500,
        &String::from_str(&env, "Nail trimming only"),
    );

    let total_expenses = client.get_grooming_expenses(&pet_id);
    assert_eq!(total_expenses, 6500);
}

#[test]
#[should_panic]
fn test_add_grooming_record_invalid_pet() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init_admin(&admin);

    client.add_grooming_record(
        &999,
        &String::from_str(&env, "Full Grooming"),
        &String::from_str(&env, "Pet Spa"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 2592000),
        &5000,
        &String::from_str(&env, "Haircut and bath"),
    );
}

#[test]
fn test_empty_grooming_history() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    let history = client.get_grooming_history(&pet_id);
    assert_eq!(history.len(), 0);

    let next_date = client.get_next_grooming_date(&pet_id);
    assert_eq!(next_date, 0);

    let expenses = client.get_grooming_expenses(&pet_id);
    assert_eq!(expenses, 0);
}

#[test]
fn test_get_grooming_record() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    let record_id = client.add_grooming_record(
        &pet_id,
        &String::from_str(&env, "Full Grooming"),
        &String::from_str(&env, "Pet Spa"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 2592000),
        &5000,
        &String::from_str(&env, "Haircut and bath"),
    );

    let record = client.get_grooming_record(&record_id).unwrap();
    assert_eq!(record.id, record_id);
    assert_eq!(record.pet_id, pet_id);
    assert_eq!(record.cost, 5000);
    assert_eq!(record.service_type, String::from_str(&env, "Full Grooming"));
}

#[test]
fn test_get_grooming_record_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let result = client.get_grooming_record(&999u64);
    assert!(result.is_none());
}

#[test]
fn test_get_grooming_count_zero() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    // No grooming records added — count should be 0
    let count = client.get_grooming_count(&pet_id);
    assert_eq!(count, 0);
}

#[test]
fn test_get_grooming_count_after_adding_records() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    client.add_grooming_record(
        &pet_id,
        &String::from_str(&env, "Full Grooming"),
        &String::from_str(&env, "Pet Spa"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 2592000),
        &5000,
        &String::from_str(&env, "Haircut and bath"),
    );

    assert_eq!(client.get_grooming_count(&pet_id), 1);

    client.add_grooming_record(
        &pet_id,
        &String::from_str(&env, "Nail Trim"),
        &String::from_str(&env, "Pet Spa"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 1296000),
        &1500,
        &String::from_str(&env, "Nail trimming only"),
    );

    assert_eq!(client.get_grooming_count(&pet_id), 2);
}

#[test]
fn test_get_grooming_count_matches_history_length() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2021-06-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &25,
        &None,
        &PrivacyLevel::Public,
    );

    for i in 0..3u64 {
        client.add_grooming_record(
            &pet_id,
            &String::from_str(&env, "Bath"),
            &String::from_str(&env, "Groomer"),
            &(env.ledger().timestamp() + i * 1000),
            &(env.ledger().timestamp() + i * 1000 + 2592000),
            &2000,
            &String::from_str(&env, ""),
        );
    }

    let count = client.get_grooming_count(&pet_id);
    let history = client.get_grooming_history(&pet_id);

    // Count must match the actual history length
    assert_eq!(count, history.len() as u64);
    assert_eq!(count, 3);
}

mod test_recurring_grooming {
    use crate::{Gender, GroomingFrequency, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    fn setup_pet(env: &Env, client: &PetChainContractClient) -> (Address, u64) {
        let owner = Address::generate(env);
        let admin = Address::generate(env);
        client.init_admin(&admin);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(env, "Buddy"),
            &String::from_str(env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(env, "Golden Retriever"),
            &String::from_str(env, "Golden"),
            &30,
            &None,
            &PrivacyLevel::Public,
        );
        (owner, pet_id)
    }

    #[test]
    fn test_create_grooming_schedule_generates_4_slots() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);
        let (_owner, pet_id) = setup_pet(&env, &client);

        let start = 1_000_000u64;
        let interval = 7 * 24 * 3600u64; // weekly
        let end = start + interval * 10;

        client.create_grooming_schedule(
            &pet_id,
            &GroomingFrequency::Weekly,
            &start,
            &end,
            &String::from_str(&env, "Groomer A"),
            &String::from_str(&env, "Bath"),
            &5000,
        );

        let history = client.get_grooming_history(&pet_id);
        assert_eq!(history.len(), 4);
    }

    #[test]
    fn test_advance_schedule_correct_date() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);
        let (_owner, pet_id) = setup_pet(&env, &client);

        let start = 1_000_000u64;
        let interval = 7 * 24 * 3600u64;
        let end = start + interval * 10;

        let schedule_id = client.create_grooming_schedule(
            &pet_id,
            &GroomingFrequency::Weekly,
            &start,
            &end,
            &String::from_str(&env, "Groomer A"),
            &String::from_str(&env, "Bath"),
            &5000,
        );

        // After 4 slots (indices 0..3), last_slot_date = start + 3*interval
        // advance should produce slot at start + 4*interval
        let record_id = client.advance_schedule(&schedule_id);
        assert!(record_id > 0);

        let record = client.get_grooming_record(&record_id).unwrap();
        assert_eq!(record.date, start + 4 * interval);
    }

    #[test]
    fn test_cancel_schedule_stops_generation() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);
        let (_owner, pet_id) = setup_pet(&env, &client);

        let start = 1_000_000u64;
        let interval = 7 * 24 * 3600u64;
        let end = start + interval * 10;

        let schedule_id = client.create_grooming_schedule(
            &pet_id,
            &GroomingFrequency::Weekly,
            &start,
            &end,
            &String::from_str(&env, "Groomer A"),
            &String::from_str(&env, "Bath"),
            &5000,
        );

        client.cancel_grooming_schedule(&schedule_id);

        let result = client.advance_schedule(&schedule_id);
        assert_eq!(result, 0);
    }
}
