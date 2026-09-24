//! Direct-transfer state machine tests (Issue #1190).

use crate::{PetOwnershipContract, PetOwnershipContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

/// Escrows a transfer of pet 1 from `owner` to `to` and disputes it.
fn disputed(env: &Env) -> (PetOwnershipContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(env, &contract_id);
    let (owner, to) = (Address::generate(env), Address::generate(env));
    client.create_pet(&1, &owner);
    client.initiate_transfer(&1, &to);
    client.accept_transfer(&1);
    client.raise_dispute(&1, &to);
    (client, owner, to)
}

// Backward transition: a new transfer accepted during a dispute replaces the
// disputed escrow with an undisputed one.
#[test]
fn disputed_escrow_is_replaced_by_a_new_transfer() {
    let env = Env::default();
    let (client, _, _) = disputed(&env);
    let other = Address::generate(&env);
    client.initiate_transfer(&1, &other);
    client.accept_transfer(&1);
    assert!(!client.get_escrowed_transfer(&1).unwrap().disputed);
}

// Ownership moves while the pet's escrowed transfer is under dispute, leaving
// an escrow whose `from` is no longer the owner.
#[test]
fn escrowed_pet_can_be_batch_transferred() {
    let env = Env::default();
    let (client, owner, _) = disputed(&env);
    let other = Address::generate(&env);
    client.batch_transfer(&Vec::from_array(&env, [1]), &other);
    assert_eq!(client.get_current_owner(&1), other);
    assert_eq!(client.get_escrowed_transfer(&1).unwrap().from, owner);
}
