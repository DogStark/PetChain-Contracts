use petchain::components::pet_owner::IPetOwner::{IPetOwner};

#[starknet::contract]
mod PetChain {
    use super::IPetOwner;
    use petchain::base::types::{PetOwner};
    use starknet::{ContractAddress, get_caller_address};
    use starknet::storage::{Map};
    use petchain::components::pet_owner::petowner_component;

    component!(
        path: petowner_component::PetOwner_component, storage: pet_storage, event: PetOwnerEvent,
    );

    #[storage]
    struct Storage {
       
        #[substorage(v0)]
        pet_storage: petowner_component::PetOwner_component::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        PetOwnerEvent: petowner_component::PetOwner_component::Event,
    }


    #[abi(embed_v0)]
    impl PetChainImpl of IPetOwner<ContractState> {
        fn is_owner_registered(self: @ContractState, pet_owner: ContractAddress) -> bool {
            let registerd = self.pet_storage.is_owner_registered(pet_owner);
            registerd
        }

        fn is_pet_owner(self: @ContractState, id: u256, pet_owner: ContractAddress) -> bool {
            let is_owner = self.pet_storage.is_pet_owner(id, pet_owner);
            is_owner
        }
        fn register_pet_owner(
            ref self: ContractState,
            name: ByteArray,
            email: ByteArray,
            emergency_contact: ByteArray,
        ) -> u256 {
            let caller = get_caller_address();
            let is_owner = self.is_owner_registered(caller);
            assert(!is_owner, 'Already Registered');

            let id = self.pet_storage.register_pet_owner(name, email, emergency_contact);
            id
        }

        fn update_owner_profile(
            ref self: ContractState,
            id: u256,
            name: ByteArray,
            email: ByteArray,
            emergency_contact: ByteArray,
        ) -> bool {
            let success = self.pet_storage.update_owner_profile(id, name, email, emergency_contact);
            success
        }

        fn return_pet_owner_info(
            ref self: ContractState, pet_owner_addr: ContractAddress,
        ) -> PetOwner {
            let pet_owner = self.pet_storage.return_pet_owner_info(pet_owner_addr);
            pet_owner
        }
    }
}
