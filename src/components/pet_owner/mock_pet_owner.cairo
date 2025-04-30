#[starknet::contract]
mod MockPetOwner {
    use petchain::base::types::{PetOwner};
    use starknet::{ContractAddress, get_caller_address};
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


    fn is_owner_registered(self: @ContractState, pet_owner: ContractAddress) -> bool {
        let registerd = self.pet_storage.is_owner_registered(pet_owner);
        registerd
    }

    fn is_pet_owner(self: @ContractState, pet_owner: ContractAddress) -> bool {
        let is_owner = self.pet_storage.is_pet_owner(pet_owner);
        is_owner
    }
    fn register_pet_owner(
        ref self: ContractState, name: ByteArray, email: ByteArray, emergency_contact: ByteArray,
    ) {
        let caller = get_caller_address();
        let is_owner = self.is_owner_registered(caller);
        assert(!is_owner, 'Already Registered');

        self.pet_storage.register_pet_owner(name, email, emergency_contact);
    }

    fn update_owner_profile(
        ref self: ContractState, name: ByteArray, email: ByteArray, emergency_contact: ByteArray,
    ) -> bool {
        let success = self.pet_storage.update_owner_profile(name, email, emergency_contact);
        success
    }

    fn get_pet_owner(ref self: ContractState, pet_owner_addr: ContractAddress) -> PetOwner {
        let pet_owner = self.pet_storage.get_pet_owner(pet_owner_addr);
        pet_owner
    }
}

