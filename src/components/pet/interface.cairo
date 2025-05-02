use starknet::ContractAddress;
use petchain::components::pet::types::{Pet, Gender};
#[starknet::interface]
pub trait IPet<TContractState> {
    fn register_pet(
        ref self: TContractState,
        name: ByteArray,
        birthday: ByteArray,
        gender: Gender,
        species: felt252,
    ) -> u256;
    fn update_pet_profile(
        ref self: TContractState,
        id: u256,
        name: ByteArray,
        birthday: ByteArray,
        gender: Gender,
        species: felt252,
    ) -> bool;
    fn get_pet(self: @TContractState, id: u256) -> Pet;
    fn is_pet_active(self: @TContractState, id: u256) -> bool;
    fn get_pet_owner(self: @TContractState, id: u256) -> ContractAddress;
    fn get_pets_by_owner(self: @TContractState, owner: ContractAddress) -> Array<Pet>;
    fn get_all_pets(self: @TContractState) -> Array<Pet>;
    fn deactivate_pet(ref self: TContractState, id: u256);
    fn activate_pet(ref self: TContractState, id: u256);
    fn transfer_pet_ownership(ref self: TContractState, id: u256, to: ContractAddress);
    fn accept_pet_transfer(ref self: TContractState, id: u256);
}

