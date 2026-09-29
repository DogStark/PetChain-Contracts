//! Representative error codes for each public module (Issue #1255).
//!
//! Clients match on these numeric codes; they are also pinned in
//! `stellar-contracts/error-codes.json`. See docs/error-compatibility.md.

use crate::escrow::{self, EscrowError};
use crate::vet_registry::{self, VetRegistryContract, VetRegistryContractClient};
use crate::{ContractError, PetOwnershipContract, PetOwnershipContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, Error, Vec};

fn contract_error(code: u32) -> Error {
    Error::from_contract_error(code)
}

#[test]
fn transfer_module_errors_keep_their_codes() {
    let env = Env::default();
    env.mock_all_auths();
    let client =
        PetOwnershipContractClient::new(&env, &env.register_contract(None, PetOwnershipContract));
    let (owner, new_owner) = (Address::generate(&env), Address::generate(&env));

    assert_eq!(
        client.try_batch_initiate_transfer(&Vec::new(&env), &new_owner),
        Err(Ok(contract_error(10)))
    );
    assert_eq!(ContractError::EmptyBatch as u32, 10);

    client.create_pet(&1, &owner);
    client.initiate_transfer_with_timeout(&1, &new_owner, &7u32);
    assert_eq!(
        client.try_cancel_expired_transfer(&1),
        Err(Ok(contract_error(8)))
    );
    assert_eq!(ContractError::TransferNotExpired as u32, 8);
}

#[test]
fn vet_registry_errors_keep_their_codes() {
    let env = Env::default();
    env.mock_all_auths();
    let client =
        VetRegistryContractClient::new(&env, &env.register_contract(None, VetRegistryContract));
    let admin = Address::generate(&env);
    client.init(&admin);

    assert_eq!(client.try_init(&admin), Err(Ok(contract_error(0))));
    assert_eq!(vet_registry::ContractError::AlreadyInitialized as u32, 0);
    assert_eq!(vet_registry::ContractError::VetNotFound as u32, 3);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn escrow_errors_keep_their_codes() {
    assert_eq!(EscrowError::EscrowNotFound as u32, 4);
    let env = Env::default();
    let contract = env.register_contract(None, PetOwnershipContract);
    env.as_contract(&contract, || escrow::finalize_transfer(&env, 404));
}
