#[starknet::contract]
mod PetChain {
    use petchain::contracts::interface::IPetChain;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl PetChainImpl of IPetChain<ContractState> {}
}
