use petchain::components::pet::IPet::{IPet};

#[starknet::contract]
mod PetChain {
    use super::IPet;
    use petchain::base::types::{Pet};
    use starknet::{ContractAddress, get_caller_address};
    use petchain::components::pet::pet_component;

    component!(path: pet_component::Pet_component, storage: pet_storage, event: PetEvent);

    #[storage]
    struct Storage {
        #[substorage(v0)]
        pet_storage: pet_component::Pet_component::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        PetEvent: pet_component::Pet_component::Event,
    }


    #[abi(embed_v0)]
    impl PetChainImpl of IPet<ContractState> {
        fn register_pet(
            ref self: ContractState, name: ByteArray, birthday: ByteArray, active: bool,
        ) -> u256 {
            let caller = get_caller_address();
            let id = self.pet_storage.register_pet(name, birthday, active);
            id
        }

        fn update_pet_profile(
            ref self: ContractState, id: u256, name: ByteArray, birthday: ByteArray, active: bool,
        ) -> bool {
            let success = self.pet_storage.update_pet_profile(id, name, birthday, active);
            success
        }

        fn get_pet(self: @ContractState, id: u256) -> Pet {
            let pet = self.pet_storage.get_pet(id);
            pet
        }

        fn is_pet_active(self: @ContractState, id: u256) -> bool {
            let status = self.pet_storage.is_pet_active(id);
            status
        }

        fn get_pet_owner(self: @ContractState, id: u256) -> ContractAddress {
            let owner = self.pet_storage.get_pet_owner(id);
            owner
        }

        fn get_pets_by_owner(self: @ContractState, owner: ContractAddress) -> Array<Pet> {
            let pets = self.pet_storage.get_pets_by_owner(owner);
            pets
        }
    }
}
