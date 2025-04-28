// use starknet::ContractAddress;
// use petchain::base::types::{PetOwner};
use petchain::contracts::interface::IPetOwner;


#[starknet::component]
pub mod PetOwner_component {
    use petchain::base::types::PetOwner;
    use starknet::{ContractAddress, get_block_timestamp, get_caller_address};
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess, Map};
    // use core::num::traits::Zero;

    #[storage]
    pub struct Storage {
        pet_owner: Map<ContractAddress, PetOwner>,
        pet_owner_id_address: Map<u256, ContractAddress>,
        petowner_ids: u256,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        PetOwnerCreated: PetOwnerCreated,
        PetOwnerUpdated: PetOwnerUpdated,
    }

    #[derive(Drop, starknet::Event)]
    struct PetOwnerCreated {
        #[key]
        owner_address: ContractAddress,
        owner_id: u256,
    }

    #[derive(Drop, starknet::Event)]
    struct PetOwnerUpdated {
        #[key]
        owner_address: ContractAddress,
        owner_id: u256,
    }

    #[embeddable_as(PetOwnerComponent)]
    impl PetOwnerImpl<
        TContractState, +HasComponent<TContractState>,
    > of super::IPetOwner<ComponentState<TContractState>> {
        fn is_owner_registered(
            self: @ComponentState<TContractState>, pet_owner: ContractAddress,
        ) -> bool {
            let pet_owner = self.pet_owner.read(pet_owner);
            pet_owner.is_pet_owner
        }

        fn is_pet_owner(
            self: @ComponentState<TContractState>, id: u256, pet_owner: ContractAddress,
        ) -> bool {
            let pet_owner = self.pet_owner.read(pet_owner);
            id == pet_owner.id
        }

        fn register_pet_owner(
            ref self: ComponentState<TContractState>,
            name: ByteArray,
            email: ByteArray,
            emergency_contact: ByteArray,
        ) -> u256 {
            let caller = get_caller_address();
            let is_registered = self.is_owner_registered(caller);
            assert(!is_registered, 'Already Registered');

            let cur_id = self.petowner_ids.read() + 1;
            let pet_owner = PetOwner {
                owners_address: caller,
                id: cur_id,
                name,
                email,
                emergency_contact,
                created_at: get_block_timestamp(),
                updated_at: get_block_timestamp(),
                is_pet_owner: true,
            };

            self.pet_owner.write(caller, pet_owner);
            self.petowner_ids.write(cur_id);
            self.pet_owner_id_address.write(cur_id, caller);

            self
                .emit(
                    Event::PetOwnerCreated(
                        PetOwnerCreated { owner_address: caller, owner_id: cur_id },
                    ),
                );

            cur_id
        }

        fn update_owner_profile(
            ref self: ComponentState<TContractState>,
            id: u256,
            name: ByteArray,
            email: ByteArray,
            emergency_contact: ByteArray,
        ) -> bool {
            let caller = get_caller_address();
            let is_registered = self.is_owner_registered(caller);
            assert(is_registered, 'Not Registered');

            let is_correct_owner = self.is_pet_owner(id, caller);
            assert(is_correct_owner, 'Not Your Profile');

            let mut pet_owner = self.pet_owner.read(caller);

            pet_owner.name = name;
            pet_owner.email = email;
            pet_owner.emergency_contact = emergency_contact;
            pet_owner.updated_at = get_block_timestamp();

            let id = pet_owner.id;
            self.pet_owner.write(caller, pet_owner);
            self.pet_owner_id_address.write(id, caller);

            self
                .emit(
                    Event::PetOwnerUpdated(PetOwnerUpdated { owner_address: caller, owner_id: id }),
                );

            true
        }

        fn return_pet_owner_info(
            ref self: ComponentState<TContractState>, pet_owner_addr: ContractAddress,
        ) -> PetOwner {
            self.pet_owner.read(pet_owner_addr)
        }
    }
}
