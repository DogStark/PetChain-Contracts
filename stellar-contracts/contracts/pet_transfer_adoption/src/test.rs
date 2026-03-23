#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Env, String};

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let words = client.hello(&String::from_str(&env, "Dev"));
    assert_eq!(
        words,
        vec![
            &env,
            String::from_str(&env, "Hello"),
            String::from_str(&env, "Dev"),
        ]
    );
}

/// ======================================================
/// VET REGISTRY INITIALIZATION TESTS
/// ======================================================

#[cfg(test)]
mod vet_registry_init_tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_init_success() {
        let env = Env::default();
        let vet_registry_contract_id = env.register_contract(None, vet_registry::VetRegistryContract);
        let vet_registry_client = vet_registry::VetRegistryContractClient::new(&env, &vet_registry_contract_id);

        let admin = env.accounts().generate(None);

        // Should successfully initialize
        vet_registry_client.init(&admin);

        // Verify initialization was successful by checking admin is set
        // (Note: There's no getter for admin in current implementation,
        // but this test ensures the init call completes without error)
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #0)")]
    fn test_init_already_initialized() {
        let env = Env::default();
        let vet_registry_contract_id = env.register_contract(None, vet_registry::VetRegistryContract);
        let vet_registry_client = vet_registry::VetRegistryContractClient::new(&env, &vet_registry_contract_id);

        let admin = env.accounts().generate(None);

        // First initialization should succeed
        vet_registry_client.init(&admin);

        // Second initialization should fail with AlreadyInitialized error
        // AlreadyInitialized = 0, so it should panic with error code 0
        vet_registry_client.init(&admin);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #0)")]
    fn test_init_already_initialized_assert_error_code() {
        let env = Env::default();
        let vet_registry_contract_id = env.register_contract(None, vet_registry::VetRegistryContract);
        let vet_registry_client = vet_registry::VetRegistryContractClient::new(&env, &vet_registry_contract_id);

        let admin = env.accounts().generate(None);
        let admin2 = env.accounts().generate(None);

        // First initialization
        vet_registry_client.init(&admin);

        // Try to initialize again with different admin - should fail with error code 0
        vet_registry_client.init(&admin2);
    }
}
