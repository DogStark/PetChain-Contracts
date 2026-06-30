use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

fn setup() -> (Env, PetChainContractClient<'static>, Address, u64) {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );

    (env, client, owner, pet_id)
}

fn ingredient(env: &Env, name: &str, calories: u32) -> Ingredient {
    Ingredient {
        name: String::from_str(env, name),
        calories,
    }
}

#[test]
fn test_add_nutrition_plan_exact_calories() {
    let (env, client, _owner, pet_id) = setup();

    let mut ingredients = Vec::new(&env);
    ingredients.push_back(ingredient(&env, "Chicken", 200));
    ingredients.push_back(ingredient(&env, "Rice", 150));
    ingredients.push_back(ingredient(&env, "Carrots", 50));

    let plan_id = client.add_nutrition_plan(
        &pet_id,
        &String::from_str(&env, "Balanced Meal"),
        &ingredients,
        &400u32,
    );

    let plan = client.get_nutrition_plan(&plan_id).unwrap();
    assert_eq!(plan.total_calories, 400u32);
    assert_eq!(plan.ingredients.len(), 3);
}

#[test]
fn test_add_nutrition_plan_within_tolerance() {
    let (env, client, _owner, pet_id) = setup();

    let mut ingredients = Vec::new(&env);
    ingredients.push_back(ingredient(&env, "Chicken", 200));
    ingredients.push_back(ingredient(&env, "Rice", 150));
    ingredients.push_back(ingredient(&env, "Carrots", 50));

    // Declared total is 3 kcal off the actual sum (400) → within ±5 tolerance
    let plan_id = client.add_nutrition_plan(
        &pet_id,
        &String::from_str(&env, "Balanced Meal"),
        &ingredients,
        &397u32,
    );

    let plan = client.get_nutrition_plan(&plan_id).unwrap();
    assert_eq!(plan.total_calories, 397u32);
}

#[test]
#[should_panic]
fn test_add_nutrition_plan_outside_tolerance() {
    let (env, client, _owner, pet_id) = setup();

    let mut ingredients = Vec::new(&env);
    ingredients.push_back(ingredient(&env, "Chicken", 200));
    ingredients.push_back(ingredient(&env, "Rice", 150));
    ingredients.push_back(ingredient(&env, "Carrots", 50));

    // Declared total is 10 kcal off the actual sum (400) → outside ±5 tolerance
    client.add_nutrition_plan(
        &pet_id,
        &String::from_str(&env, "Balanced Meal"),
        &ingredients,
        &410u32,
    );
}
