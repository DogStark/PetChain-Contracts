use crate::{Allele, Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{testutils::Address as _, Address, Env, Map, String};

fn setup() -> (Env, PetChainContractClient<'static>, Address, u64, u64) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);

    let sire_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Sire"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Black"),
        &String::from_str(&env, "Lab"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );
    let dam_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Dam"),
        &String::from_str(&env, "2020-06-01"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "Brown"),
        &String::from_str(&env, "Lab"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );
    (env, client, owner, sire_id, dam_id)
}

#[test]
fn test_set_and_get_pet_traits() {
    let (env, client, _owner, sire_id, _dam_id) = setup();

    let mut traits: Map<String, Allele> = Map::new(&env);
    traits.set(String::from_str(&env, "coat_color"), Allele::Dominant);
    traits.set(String::from_str(&env, "eye_color"), Allele::Recessive);

    client.set_pet_traits(&sire_id, &traits);

    let stored = client.get_pet_traits(&sire_id);
    assert_eq!(
        stored.get(String::from_str(&env, "coat_color")),
        Some(Allele::Dominant)
    );
    assert_eq!(
        stored.get(String::from_str(&env, "eye_color")),
        Some(Allele::Recessive)
    );
}

#[test]
fn test_get_pet_traits_empty_when_not_set() {
    let (env, client, _owner, sire_id, _dam_id) = setup();
    let traits = client.get_pet_traits(&sire_id);
    assert_eq!(traits.len(), 0);
}

#[test]
fn test_dominant_dominant_gives_100_percent() {
    let (env, client, _owner, sire_id, dam_id) = setup();

    let mut sire_traits: Map<String, Allele> = Map::new(&env);
    sire_traits.set(String::from_str(&env, "coat_color"), Allele::Dominant);
    client.set_pet_traits(&sire_id, &sire_traits);

    let mut dam_traits: Map<String, Allele> = Map::new(&env);
    dam_traits.set(String::from_str(&env, "coat_color"), Allele::Dominant);
    client.set_pet_traits(&dam_id, &dam_traits);

    let record_id = client.add_breeding_record(
        &sire_id,
        &dam_id,
        &1_000_000u64,
        &String::from_str(&env, "test litter"),
    );

    let predicted = client.compute_offspring_traits(&record_id);
    assert_eq!(
        predicted.get(String::from_str(&env, "coat_color")),
        Some(10000u32)
    );
}

#[test]
fn test_dominant_recessive_gives_75_percent() {
    let (env, client, _owner, sire_id, dam_id) = setup();

    let mut sire_traits: Map<String, Allele> = Map::new(&env);
    sire_traits.set(String::from_str(&env, "size"), Allele::Dominant);
    client.set_pet_traits(&sire_id, &sire_traits);

    let mut dam_traits: Map<String, Allele> = Map::new(&env);
    dam_traits.set(String::from_str(&env, "size"), Allele::Recessive);
    client.set_pet_traits(&dam_id, &dam_traits);

    let record_id = client.add_breeding_record(
        &sire_id,
        &dam_id,
        &1_000_000u64,
        &String::from_str(&env, "test litter"),
    );

    let predicted = client.compute_offspring_traits(&record_id);
    assert_eq!(
        predicted.get(String::from_str(&env, "size")),
        Some(7500u32)
    );
}

#[test]
fn test_recessive_recessive_gives_0_percent() {
    let (env, client, _owner, sire_id, dam_id) = setup();

    let mut sire_traits: Map<String, Allele> = Map::new(&env);
    sire_traits.set(String::from_str(&env, "tail_length"), Allele::Recessive);
    client.set_pet_traits(&sire_id, &sire_traits);

    let mut dam_traits: Map<String, Allele> = Map::new(&env);
    dam_traits.set(String::from_str(&env, "tail_length"), Allele::Recessive);
    client.set_pet_traits(&dam_id, &dam_traits);

    let record_id = client.add_breeding_record(
        &sire_id,
        &dam_id,
        &1_000_000u64,
        &String::from_str(&env, "test litter"),
    );

    let predicted = client.compute_offspring_traits(&record_id);
    assert_eq!(
        predicted.get(String::from_str(&env, "tail_length")),
        Some(0u32)
    );
}

#[test]
fn test_get_trait_probability_known_trait() {
    let (env, client, _owner, sire_id, dam_id) = setup();

    let mut sire_traits: Map<String, Allele> = Map::new(&env);
    sire_traits.set(String::from_str(&env, "fur_type"), Allele::Dominant);
    client.set_pet_traits(&sire_id, &sire_traits);

    let mut dam_traits: Map<String, Allele> = Map::new(&env);
    dam_traits.set(String::from_str(&env, "fur_type"), Allele::Recessive);
    client.set_pet_traits(&dam_id, &dam_traits);

    let record_id = client.add_breeding_record(
        &sire_id,
        &dam_id,
        &1_000_000u64,
        &String::from_str(&env, "test litter"),
    );
    client.compute_offspring_traits(&record_id);

    let prob = client.get_trait_probability(&record_id, &String::from_str(&env, "fur_type"));
    assert_eq!(prob, Some(7500u32));
}

#[test]
fn test_get_trait_probability_unknown_trait_returns_none() {
    let (env, client, _owner, sire_id, dam_id) = setup();

    let record_id = client.add_breeding_record(
        &sire_id,
        &dam_id,
        &1_000_000u64,
        &String::from_str(&env, "test litter"),
    );
    // No traits set, no compute called
    let prob = client.get_trait_probability(&record_id, &String::from_str(&env, "unknown_trait"));
    assert_eq!(prob, None);
}

#[test]
fn test_trait_only_in_dam_uses_recessive_for_sire() {
    let (env, client, _owner, sire_id, dam_id) = setup();

    // Only dam has this trait; sire defaults to Recessive
    let mut dam_traits: Map<String, Allele> = Map::new(&env);
    dam_traits.set(String::from_str(&env, "ear_shape"), Allele::Dominant);
    client.set_pet_traits(&dam_id, &dam_traits);

    let record_id = client.add_breeding_record(
        &sire_id,
        &dam_id,
        &1_000_000u64,
        &String::from_str(&env, "test litter"),
    );

    let predicted = client.compute_offspring_traits(&record_id);
    // Recessive × Dominant → 7500
    assert_eq!(
        predicted.get(String::from_str(&env, "ear_shape")),
        Some(7500u32)
    );
}

#[test]
fn test_multiple_traits_computed_correctly() {
    let (env, client, _owner, sire_id, dam_id) = setup();

    let mut sire_traits: Map<String, Allele> = Map::new(&env);
    sire_traits.set(String::from_str(&env, "coat_color"), Allele::Dominant);
    sire_traits.set(String::from_str(&env, "size"), Allele::Recessive);
    client.set_pet_traits(&sire_id, &sire_traits);

    let mut dam_traits: Map<String, Allele> = Map::new(&env);
    dam_traits.set(String::from_str(&env, "coat_color"), Allele::Recessive);
    dam_traits.set(String::from_str(&env, "size"), Allele::Recessive);
    client.set_pet_traits(&dam_id, &dam_traits);

    let record_id = client.add_breeding_record(
        &sire_id,
        &dam_id,
        &1_000_000u64,
        &String::from_str(&env, "multi-trait litter"),
    );

    let predicted = client.compute_offspring_traits(&record_id);
    assert_eq!(
        predicted.get(String::from_str(&env, "coat_color")),
        Some(7500u32)
    );
    assert_eq!(
        predicted.get(String::from_str(&env, "size")),
        Some(0u32)
    );
}
