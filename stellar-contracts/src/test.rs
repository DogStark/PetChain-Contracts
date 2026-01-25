

#[test]
fn test_register_pet_owner() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let name = String::from_str(&env, "John Doe");
    let email = String::from_str(&env, "john@example.com");
    let emergency = String::from_str(&env, "555-1234");


}
