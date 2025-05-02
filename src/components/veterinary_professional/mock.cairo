#[starknet::contract]
mod MockVeterinaryProfessional {
    use petchain::components::veterinary_professional::vet::VetComponent;

    component!(path: VetComponent, storage: vet_storage, event: VeterinaryProfessionalEvent);

    #[storage]
    struct Storage {
        #[substorage(v0)]
        vet_storage: VetComponent::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        VeterinaryProfessionalEvent: VetComponent::Event,
    }


    #[abi(embed_v0)]
    impl Vet = VetComponent::VetComponentImpl<ContractState>;
}

