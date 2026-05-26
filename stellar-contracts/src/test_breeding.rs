use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn register_pet(env: &Env, client: &PetChainContractClient, owner: &Address, name: &str) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, name),
        &String::from_str(env, "2021-01-01"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(env, "Brown"),
        &String::from_str(env, "Mixed"),
        &20,
        &None,
        &PrivacyLevel::Public,
    )
}

fn setup() -> (
    Env,
    PetChainContractClient<'static>,
    Address,
    u64,
    u64,
    u64,
    u64,
) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let sire = register_pet(&env, &client, &owner, "Sire");
    let dam = register_pet(&env, &client, &owner, "Dam");
    let offspring = register_pet(&env, &client, &owner, "Offspring");
    let grandchild = register_pet(&env, &client, &owner, "Grandchild");
    (env, client, owner, sire, dam, offspring, grandchild)
}

#[test]
#[should_panic(expected = "Error(Contract, #20)")]
fn test_self_reference_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let sire = register_pet(&env, &client, &owner, "Sire");
    let dam = register_pet(&env, &client, &owner, "Dam");
    let record_id = client.add_breeding_record(
        &sire,
        &dam,
        &env.ledger().timestamp(),
        &String::from_str(&env, "bad"),
    );

    client.add_offspring(&record_id, &sire);
}

#[test]
#[should_panic(expected = "Error(Contract, #21)")]
fn test_cycle_insertion_rejected() {
    let (env, client, _owner, sire, dam, offspring, _grandchild) = setup();
    let record_id = client.add_breeding_record(
        &sire,
        &dam,
        &env.ledger().timestamp(),
        &String::from_str(&env, "first"),
    );
    assert!(client.add_offspring(&record_id, &offspring));

    let cycle_record = client.add_breeding_record(
        &offspring,
        &dam,
        &env.ledger().timestamp(),
        &String::from_str(&env, "cycle"),
    );
    client.add_offspring(&cycle_record, &sire);
}

#[test]
fn test_get_lineage_returns_parent_tree() {
    let (env, client, _owner, sire, dam, offspring, grandchild) = setup();
    let first = client.add_breeding_record(
        &sire,
        &dam,
        &env.ledger().timestamp(),
        &String::from_str(&env, "first"),
    );
    assert!(client.add_offspring(&first, &offspring));

    let second = client.add_breeding_record(
        &offspring,
        &dam,
        &env.ledger().timestamp(),
        &String::from_str(&env, "second"),
    );
    assert!(client.add_offspring(&second, &grandchild));

    let lineage = client.get_lineage(&grandchild, &2);
    assert_eq!(lineage.len(), 4);
    assert_eq!(lineage.get(0).unwrap(), offspring);
    assert_eq!(lineage.get(1).unwrap(), dam);
    assert_eq!(lineage.get(2).unwrap(), sire);
    assert_eq!(lineage.get(3).unwrap(), dam);
}
