use starknet::ContractAddress;

/// @notice Struct containing all data for a Vet
#[derive(Clone, Drop, Serde, starknet::Store)]
pub struct Vet {
    #[key]
    pub vet_id: u256,
    pub address: ContractAddress,
    pub name: ByteArray,
    pub email: ByteArray,
    pub emergency_contact: ByteArray,
    pub license_number: felt252,
    pub registered: bool,
    pub specialization: ByteArray,
    pub is_verified: bool,
    pub is_active: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

