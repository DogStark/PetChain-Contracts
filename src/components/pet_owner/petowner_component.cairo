use petchain::components::pet_owner::IPetOwner::{IPetOwner};
#[starknet::component]
pub mod PetOwnerComponent {
    use petchain::base::types::PetOwner;
    use starknet::{
        ContractAddress, get_block_timestamp, get_caller_address, contract_address_const,
    };
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess, Map};


    #[storage]
    pub struct Storage {
        pet_owner: Map<ContractAddress, PetOwner>,
        pet_owner_id_address: Map<u256, ContractAddress>,
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
    }

    #[derive(Drop, starknet::Event)]
    struct PetOwnerUpdated {
        #[key]
        owner_address: ContractAddress,
    }


    #[embeddable_as(PetOwnerImpl)]
    impl PetOwnerComponentImpl<
        TContractState, +HasComponent<TContractState>,
    > of super::IPetOwner<ComponentState<TContractState>> {
        fn is_owner_registered(
            self: @ComponentState<TContractState>, pet_owner: ContractAddress,
        ) -> bool {
            let pet_owner = self.pet_owner.read(pet_owner);
            pet_owner.is_pet_owner
        }

        fn is_pet_owner(self: @ComponentState<TContractState>, pet_owner: ContractAddress) -> bool {
            let pet_owner = self.pet_owner.read(pet_owner);
            pet_owner.is_pet_owner
        }

        fn register_pet_owner(
            ref self: ComponentState<TContractState>,
            name: ByteArray,
            email: ByteArray,
            emergency_contact: ByteArray,
        ) {
            let caller = get_caller_address();
            let zero_address = contract_address_const::<'0x0'>();
            assert(caller != zero_address, 'Zero Address detected');
            let is_registered = self.is_owner_registered(caller);
            assert(!is_registered, 'Already Registered');

            let pet_owner = PetOwner {
                owners_address: caller,
                name,
                email,
                emergency_contact,
                created_at: get_block_timestamp(),
                updated_at: get_block_timestamp(),
                is_pet_owner: true,
            };

            self.pet_owner.write(caller, pet_owner);

            self.emit(Event::PetOwnerCreated(PetOwnerCreated { owner_address: caller }));
        }

        fn update_owner_profile(
            ref self: ComponentState<TContractState>,
            name: ByteArray,
            email: ByteArray,
            emergency_contact: ByteArray,
        ) -> bool {
            let caller = get_caller_address();
            let is_registered = self.is_owner_registered(caller);
            assert(is_registered, 'Not Registered');

            let mut pet_owner = self.pet_owner.read(caller);

            pet_owner.name = name;
            pet_owner.email = email;
            pet_owner.emergency_contact = emergency_contact;
            pet_owner.updated_at = get_block_timestamp();

            self.pet_owner.write(caller, pet_owner);

            self.emit(Event::PetOwnerUpdated(PetOwnerUpdated { owner_address: caller }));

            true
        }

        fn get_pet_owner(
            ref self: ComponentState<TContractState>, pet_owner_addr: ContractAddress,
        ) -> PetOwner {
            self.pet_owner.read(pet_owner_addr)
        }
    }
}
