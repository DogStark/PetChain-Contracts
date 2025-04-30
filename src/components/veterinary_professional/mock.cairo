#[starknet::contract]
mod MockVeterinaryProfessional {
    use petchain::components::veterinary_professional::vet::VeterinaryProfessionalComponent;

    component!(
        path: VeterinaryProfessionalComponent,
        storage: vet_storage,
        event: VeterinaryProfessionalEvent,
    );

    #[storage]
    struct Storage {
        #[substorage(v0)]
        vet_storage: VeterinaryProfessionalComponent::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        VeterinaryProfessionalEvent: VeterinaryProfessionalComponent::Event,
    }


    #[abi(embed_v0)]
    impl VeterinaryProfessionalImpl =
        VeterinaryProfessionalComponent::VeterinaryProfessionalComponentImpl<ContractState>;
}

