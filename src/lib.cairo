#[starknet::interface]
pub trait IPetChain<TContractState> {}

#[starknet::contract]
mod PetChain {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl PetChainImpl of super::IPetChain<ContractState> {}
}
