#[starknet::contract]
mod MockPet {
    use petchain::components::pet::pet::PetComponent;

    component!(path: PetComponent, storage: pet_storage, event: PetEvent);

    #[storage]
    struct Storage {
        #[substorage(v0)]
        pet_storage: PetComponent::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        PetEvent: PetComponent::Event,
    }


    #[abi(embed_v0)]
    impl PetImpl = PetComponent::PetImpl<ContractState>;
}

