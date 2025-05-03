use starknet::ContractAddress;

/// @notice Pet strcut
#[derive(Drop, Serde, starknet::Store)]
pub struct Pet {
    #[key]
    pub id: u256,
    pub owner: ContractAddress,
    pub name: ByteArray,
    pub birthday: ByteArray,
    pub active: bool,
    pub created_at: u64,
    pub updated_at: u64,
    pub new_owner: ContractAddress,
    pub species: Species,
    pub gender: Gender,
    pub breed: felt252,
}

#[derive(Serde, Copy, Drop, PartialEq, Debug, starknet::Store)]
pub enum Species {
    #[default]
    Other,
    Dog,
    Cat,
    Bird,
}

#[derive(Serde, Copy, Drop, PartialEq, Debug, starknet::Store)]
pub enum Gender {
    #[default]
    NotSpecified,
    Male,
    Female,
}

