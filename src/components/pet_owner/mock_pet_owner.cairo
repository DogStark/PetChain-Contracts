#[starknet::contract]
mod MockPetOwner {
    use petchain::components::pet_owner::petowner_component;

    component!(
        path: petowner_component::PetOwnerComponent, storage: pet_storage, event: PetOwnerEvent,
    );

    #[storage]
    struct Storage {
        #[substorage(v0)]
        pet_storage: petowner_component::PetOwnerComponent::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        PetOwnerEvent: petowner_component::PetOwnerComponent::Event,
    }


    #[abi(embed_v0)]
    impl PetOwnerImpl =
        petowner_component::PetOwnerComponent::PetOwnerImpl<ContractState>;
}

