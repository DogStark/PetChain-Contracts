use crate::{ContractError, PetChainContract, PetChainContractClient};
use soroban_sdk::{testutils::Address as _, vec, Address, Env};

fn setup_multisig(env: &Env, threshold: u32) -> (PetChainContractClient, Address, Address, Address) {
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);

    let admin1 = Address::generate(env);
    let admin2 = Address::generate(env);
    let admin3 = Address::generate(env);

    let admins = vec![env, admin1.clone(), admin2.clone(), admin3.clone()];
    client.init_multisig(&admin1, &admins, &threshold);

    (client, admin1, admin2, admin3)
}

// Threshold starts at 2 (below the admin count of 3). Lowering it must still
// require all 3 admins to approve, not just the old threshold of 2 — a
// minimal quorum should not be able to unilaterally weaken governance.
#[test]
fn threshold_change_updates_only_after_every_admin_approves() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin1, admin2, admin3) = setup_multisig(&env, 2);

    client.set_threshold(&admin1, &1u32);
    client.set_threshold(&admin2, &1u32);
    assert_eq!(
        client.get_admin_threshold(),
        2u32,
        "must remain pending until every admin approves"
    );

    client.set_threshold(&admin3, &1u32);
    assert_eq!(client.get_admin_threshold(), 1u32);
}

#[test]
fn threshold_change_remains_pending_when_one_admin_abstains() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin1, admin2, _admin3) = setup_multisig(&env, 2);

    client.set_threshold(&admin1, &1u32);
    client.set_threshold(&admin2, &1u32);
    // admin3 never approves — the threshold must stay at its old value.
    assert_eq!(client.get_admin_threshold(), 2u32);
}

#[test]
fn threshold_change_same_admin_cannot_approve_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin1, _admin2, _admin3) = setup_multisig(&env, 2);

    client.set_threshold(&admin1, &1u32);
    let result = client.try_set_threshold(&admin1, &1u32);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::AdminAlreadyApproved.into()
    );
}

// A single-admin multisig is its own full quorum: one approval is enough.
#[test]
fn threshold_change_applies_immediately_with_single_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let admin1 = Address::generate(&env);
    let admins = vec![&env, admin1.clone()];
    client.init_multisig(&admin1, &admins, &1u32);

    client.set_threshold(&admin1, &1u32);
    assert_eq!(client.get_admin_threshold(), 1u32);
}
