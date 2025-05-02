use petchain::components::veterinary_professional::interface::{IVet};
#[starknet::component]
pub mod VetComponent {
    use super::IVet;
    use petchain::components::veterinary_professional::types::Vet;
    use starknet::{
        ContractAddress, get_block_timestamp, get_caller_address, contract_address_const,
    };
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess, Map};

    #[storage]
    pub struct Storage {
        vets: Map<ContractAddress, Vet>,
        vet_licences: Map<felt252, u256>,
        vet_ids: Map<u256, ContractAddress>,
        vet_count: u256,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        VetRegistered: VetRegistered,
        VetProfileUpdated: VetProfileUpdated,
        VetActivated: VetActivated,
        VetDeActivated: VetDeActivated,
        VetVerified: VetVerified,
    }

    #[derive(Drop, starknet::Event)]
    struct VetRegistered {
        #[key]
        vet_id: u256,
        vet_address: ContractAddress,
    }
    #[derive(Drop, starknet::Event)]
    struct VetProfileUpdated {
        #[key]
        vet_id: u256,
        vet_address: ContractAddress,
    }

    #[derive(Drop, starknet::Event)]
    struct VetActivated {
        #[key]
        vet_id: u256,
        vet_address: ContractAddress,
    }

    #[derive(Drop, starknet::Event)]
    struct VetDeActivated {
        #[key]
        vet_id: u256,
        vet_address: ContractAddress,
    }

    #[derive(Drop, starknet::Event)]
    struct VetVerified {
        #[key]
        vet_id: u256,
        vet_address: ContractAddress,
    }


    #[embeddable_as(VetComponentImpl)]
    impl VetComponenttImpl<
        TContractState, +HasComponent<TContractState>,
    > of super::IVet<ComponentState<TContractState>> {
        fn register_vet(
            ref self: ComponentState<TContractState>,
            name: ByteArray,
            email: ByteArray,
            emergency_contact: ByteArray,
            license_number: felt252,
            specialization: ByteArray,
        ) -> u256 {
            let caller = get_caller_address();
            let zero_address = contract_address_const::<'0'>();
            let timestamp = get_block_timestamp();

            assert(caller != zero_address, 'Zero Address detected');

            let existing_vet = self.get_vet(caller);
            assert(!existing_vet.registered, 'Already registered');

            let existing_license_owner = self.vet_licences.read(license_number);
            assert(existing_license_owner == 0, 'License already registered');

            let id = self.vet_count.read() + 1;

            let new_vet = Vet {
                vet_id: id,
                address: caller,
                name,
                email,
                emergency_contact,
                license_number,
                registered: true,
                specialization,
                is_verified: false,
                is_active: false,
                created_at: timestamp,
                updated_at: timestamp,
            };

            self.vet_licences.write(license_number, id);
            self.vet_ids.write(id, caller);
            self.vets.write(caller, new_vet.clone());
            self.vet_count.write(id);

            self.emit(Event::VetRegistered(VetRegistered { vet_id: id, vet_address: caller }));

            id
        }

        fn update_vet_profile(
            ref self: ComponentState<TContractState>,
            name: ByteArray,
            email: ByteArray,
            emergency_contact: ByteArray,
            license_number: felt252,
            specialization: ByteArray,
        ) -> bool {
            let caller = get_caller_address();
            let zero_address = contract_address_const::<'0x0'>();
            let timestamp = get_block_timestamp();

            assert(caller != zero_address, 'Zero Address detected');

            let mut vet = self.vets.read(caller);
            assert(vet.registered, 'Not registered');
            assert(caller == vet.address, 'Only owner can update');

            // Handle license change
            if vet.license_number != license_number {
                // Invalidate old license
                self.vet_licences.write(vet.license_number, 0);
            }

            // Update profile
            vet.name = name;
            vet.email = email;
            vet.emergency_contact = emergency_contact;
            vet.license_number = license_number;
            vet.specialization = specialization;
            vet.updated_at = timestamp;

            let id = vet.vet_id;

            // Write updated state
            self.vet_licences.write(license_number, id);
            self.vets.write(caller, vet);

            // Emit event
            self
                .emit(
                    Event::VetProfileUpdated(VetProfileUpdated { vet_id: id, vet_address: caller }),
                );

            true
        }

        //TODO: Restrict to admin
        fn activate_vet(ref self: ComponentState<TContractState>, address: ContractAddress) {
            let caller = get_caller_address();
            let zero_address = contract_address_const::<'0x0'>();
            assert(caller != zero_address, 'Zero Address detected');

            let timestamp = get_block_timestamp();

            let mut vet = self.vets.read(address);

            assert(!vet.is_active, 'Vet is already active');

            vet.is_active = true;

            vet.updated_at = timestamp;

            let id = vet.vet_id;

            self.vet_licences.write(vet.license_number, id);
            self.vet_ids.write(id, caller);
            self.vets.write(vet.address, vet);

            self.emit(Event::VetActivated(VetActivated { vet_id: id, vet_address: caller }));
        }

        //TODO: Restrict to admin
        fn deactivate_vet(ref self: ComponentState<TContractState>, address: ContractAddress) {
            let caller = get_caller_address();
            let zero_address = contract_address_const::<'0x0'>();
            assert(caller != zero_address, 'Zero Address detected');

            let timestamp = get_block_timestamp();

            let mut vet = self.vets.read(address);

            assert(vet.is_active, 'Vet is already inactive');

            vet.is_active = false;

            vet.updated_at = timestamp;

            let id = vet.vet_id;

            self.vet_licences.write(vet.license_number, id);
            self.vet_ids.write(id, caller);
            self.vets.write(vet.address, vet);

            self.emit(Event::VetDeActivated(VetDeActivated { vet_id: id, vet_address: caller }));
        }

        //TODO: Restrict to admin
        fn verify_vet(ref self: ComponentState<TContractState>, address: ContractAddress) {
            let caller = get_caller_address();
            let zero_address = contract_address_const::<'0x0'>();
            assert(caller != zero_address, 'Zero Address detected');

            let timestamp = get_block_timestamp();

            let mut vet = self.vets.read(address);

            assert(!vet.is_verified, 'Vet is already Verified');

            vet.is_verified = true;

            vet.updated_at = timestamp;

            let id = vet.vet_id;

            self.vet_licences.write(vet.license_number, id);
            self.vet_ids.write(id, caller);
            self.vets.write(vet.address, vet);

            self.emit(Event::VetVerified(VetVerified { vet_id: id, vet_address: caller }));
        }

        fn is_vet_active(
            ref self: ComponentState<TContractState>, address: ContractAddress,
        ) -> bool {
            let vet = self.vets.read(address);

            let is_active = vet.is_active;

            is_active
        }

        fn is_vet_verified(
            ref self: ComponentState<TContractState>, address: ContractAddress,
        ) -> bool {
            let vet = self.vets.read(address);

            let is_verified = vet.is_verified;

            is_verified
        }

        fn get_vet(ref self: ComponentState<TContractState>, address: ContractAddress) -> Vet {
            let vet = self.vets.read(address);
            vet
        }

        fn get_vet_by_id(ref self: ComponentState<TContractState>, vet_id: u256) -> Vet {
            let vet_address = self.vet_ids.read(vet_id);
            let vet = self.vets.read(vet_address);
            vet
        }

        fn get_vet_by_license_number(
            ref self: ComponentState<TContractState>, license_number: felt252,
        ) -> Vet {
            let vet_id = self.vet_licences.read(license_number);
            let vet_address = self.vet_ids.read(vet_id);
            let vet = self.vets.read(vet_address);
            vet
        }
    }
}