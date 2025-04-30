use starknet::ContractAddress;
use petchain::base::types::{PetOwner};
#[starknet::interface]
pub trait IPetOwner<TContractState> {
    fn is_owner_registered(self: @TContractState, pet_owner: ContractAddress) -> bool;
    fn register_pet_owner(
        ref self: TContractState, name: ByteArray, email: ByteArray, emergency_contact: ByteArray,
    );
    fn is_pet_owner(self: @TContractState, pet_owner: ContractAddress) -> bool;
    fn update_owner_profile(
        ref self: TContractState, name: ByteArray, email: ByteArray, emergency_contact: ByteArray,
    ) -> bool;
    fn get_pet_owner(ref self: TContractState, pet_owner_addr: ContractAddress) -> PetOwner;
}
