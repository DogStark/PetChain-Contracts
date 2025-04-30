use starknet::ContractAddress;
use petchain::components::veterinary_professional::types::{Vet};
#[starknet::interface]
pub trait IVeterinary_professional<TContractState> {
    fn register_vet(
        ref self: TContractState,
        name: ByteArray,
        email: ByteArray,
        emergency_contact: ByteArray,
        license_number: felt252,
        specialization: ByteArray,
    ) -> u256;

    fn update_vet_profile(
        ref self: TContractState,
        name: ByteArray,
        email: ByteArray,
        emergency_contact: ByteArray,
        license_number: felt252,
        specialization: ByteArray,
    ) -> bool;

    fn deactivate_vet(ref self: TContractState, address: ContractAddress);

    fn activate_vet(ref self: TContractState, address: ContractAddress);

    fn get_vet(ref self: TContractState, address: ContractAddress) -> Vet;

    fn get_vet_by_id(ref self: TContractState, vet_id: u256) -> Vet;

    fn get_vet_by_license_number(ref self: TContractState, license_number: felt252) -> Vet;

    fn is_vet_verified(ref self: TContractState, address: ContractAddress) -> bool;

    fn is_vet_active(ref self: TContractState, address: ContractAddress) -> bool;

    fn verify_vet(ref self: TContractState, address: ContractAddress);

    // for TESTING
    fn init(ref self: TContractState, admin: ContractAddress);
}

