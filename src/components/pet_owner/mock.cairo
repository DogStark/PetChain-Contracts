#[starknet::contract]
mod MockPetOwner {
    use petchain::components::pet_owner::pet_owner::PetOwnerComponent;

    component!(path: PetOwnerComponent, storage: pet_storage, event: PetOwnerEvent);

    #[storage]
    struct Storage {
        #[substorage(v0)]
        pet_storage: PetOwnerComponent::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        PetOwnerEvent: PetOwnerComponent::Event,
    }


    #[abi(embed_v0)]
    impl PetOwnerImpl = PetOwnerComponent::PetOwnerImpl<ContractState>;
}

