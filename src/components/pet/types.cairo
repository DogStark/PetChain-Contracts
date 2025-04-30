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
}
