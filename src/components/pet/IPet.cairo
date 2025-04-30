use starknet::ContractAddress;
use petchain::base::types::{Pet};
#[starknet::interface]
pub trait IPet<TContractState> {
    fn register_pet(
        ref self: TContractState, name: ByteArray, birthday: ByteArray, active: bool,
    ) -> u256;
    fn update_pet_profile(
        ref self: TContractState, id: u256, name: ByteArray, birthday: ByteArray, active: bool,
    ) -> bool;
    fn get_pet(self: @TContractState, id: u256) -> Pet;
    fn is_pet_active(self: @TContractState, id: u256) -> bool;
    fn get_pet_owner(self: @TContractState, id: u256) -> ContractAddress;
    fn get_pets_by_owner(self: @TContractState, owner: ContractAddress) -> Array<Pet>;
}
