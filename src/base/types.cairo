use starknet::ContractAddress;

/// @notice Struct containing all data for a pet owner
#[derive(Drop, Serde, starknet::Store)]
pub struct PetOwner {
    #[key]
    pub owners_address: ContractAddress,
    pub name: ByteArray, // Owner's full name
    pub email: ByteArray, // Email address
    pub emergency_contact: ByteArray, // Emergency contact
    pub created_at: u64, // Timestamp of registration
    pub updated_at: u64,
    pub is_pet_owner: bool,
}

