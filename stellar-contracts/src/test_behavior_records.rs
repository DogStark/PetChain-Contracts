use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup() -> (Env, PetChainContractClient<'static>, Address, Address, u64) {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );

    (env, client, owner, admin, pet_id)
}

#[test]
fn test_get_behavior_records_pagination() {
    let (env, client, owner, _admin, pet_id) = setup();

    for i in 0..5u32 {
        client.add_behavior_record(
            &pet_id,
            &owner,
            &BehaviorType::Training,
            &(i % 10),
            &String::from_str(&env, "Training session"),
        );
    }

    // Page 0, size 2 → first 2 records
    let page0 = client.get_behavior_records(&pet_id, &owner, &0u32, &2u32, &None);
    assert_eq!(page0.items.len(), 2);
    assert_eq!(page0.total, 5);
    assert_eq!(page0.page, 0);
    assert_eq!(page0.page_size, 2);

    // Page 2, size 2 → records 5 (only one left)
    let page2 = client.get_behavior_records(&pet_id, &owner, &2u32, &2u32, &None);
    assert_eq!(page2.items.len(), 1);

    // Page 3, size 2 → empty
    let page3 = client.get_behavior_records(&pet_id, &owner, &3u32, &2u32, &None);
    assert_eq!(page3.items.len(), 0);
}

#[test]
fn test_get_behavior_records_type_filter() {
    let (env, client, owner, _admin, pet_id) = setup();

    client.add_behavior_record(
        &pet_id,
        &owner,
        &BehaviorType::Training,
        &5,
        &String::from_str(&env, "Sit"),
    );
    client.add_behavior_record(
        &pet_id,
        &owner,
        &BehaviorType::Aggression,
        &7,
        &String::from_str(&env, "Barking"),
    );
    client.add_behavior_record(
        &pet_id,
        &owner,
        &BehaviorType::Training,
        &4,
        &String::from_str(&env, "Stay"),
    );

    let training = client.get_behavior_records(
        &pet_id,
        &owner,
        &0u32,
        &10u32,
        &Some(BehaviorType::Training),
    );
    assert_eq!(training.items.len(), 2);
    assert_eq!(training.total, 2);
    for i in 0..training.items.len() {
        assert_eq!(training.items.get(i).unwrap().behavior_type, BehaviorType::Training);
    }

    let aggression = client.get_behavior_records(
        &pet_id,
        &owner,
        &0u32,
        &10u32,
        &Some(BehaviorType::Aggression),
    );
    assert_eq!(aggression.items.len(), 1);
    assert_eq!(aggression.total, 1);
}

#[test]
fn test_get_behavior_records_pagination_with_type_filter() {
    let (env, client, owner, _admin, pet_id) = setup();

    // 4 training records, 2 anxiety records
    for i in 0..4u32 {
        client.add_behavior_record(
            &pet_id,
            &owner,
            &BehaviorType::Training,
            &i,
            &String::from_str(&env, "Training"),
        );
    }
    for i in 0..2u32 {
        client.add_behavior_record(
            &pet_id,
            &owner,
            &BehaviorType::Anxiety,
            &i,
            &String::from_str(&env, "Anxiety"),
        );
    }

    // Page size 3 on Training → 3 of 4
    let page = client.get_behavior_records(
        &pet_id,
        &owner,
        &0u32,
        &3u32,
        &Some(BehaviorType::Training),
    );
    assert_eq!(page.items.len(), 3);
    assert_eq!(page.total, 4);

    // Second page of Training → 1 remaining
    let page2 = client.get_behavior_records(
        &pet_id,
        &owner,
        &1u32,
        &3u32,
        &Some(BehaviorType::Training),
    );
    assert_eq!(page2.items.len(), 1);
    assert_eq!(page2.total, 4);
}

#[test]
fn test_get_behavior_records_page_size_capped() {
    let (env, client, owner, _admin, pet_id) = setup();

    for _ in 0..60u32 {
        client.add_behavior_record(
            &pet_id,
            &owner,
            &BehaviorType::Other,
            &1,
            &String::from_str(&env, "Record"),
        );
    }

    let page = client.get_behavior_records(&pet_id, &owner, &0u32, &100u32, &None);
    assert_eq!(page.items.len(), 50);
    assert_eq!(page.page_size, 50);
    assert_eq!(page.total, 60);
}
