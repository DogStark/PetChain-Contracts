use starknet::ContractAddress;
use petchain::base::types::{PetOwner};
#[starknet::interface]
pub trait IPetOwner<TContractState> {
    fn is_owner_registered(self: @TContractState, pet_owner: ContractAddress) -> bool;
    fn register_pet_owner(
        ref self: TContractState, name: ByteArray, email: ByteArray, emergency_contact: ByteArray,
    ) -> u256;
    fn is_pet_owner(self: @TContractState, id: u256, pet_owner: ContractAddress) -> bool;
    fn update_owner_profile(
        ref self: TContractState,
        id: u256,
        name: ByteArray,
        email: ByteArray,
        emergency_contact: ByteArray,
    ) -> bool;
    fn return_pet_owner_info(ref self: TContractState, pet_owner_addr: ContractAddress) -> PetOwner;
}
