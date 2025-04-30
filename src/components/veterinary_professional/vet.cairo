use petchain::components::veterinary_professional::interface::{IVeterinary_professional};
#[starknet::component]
pub mod VeterinaryProfessionalComponent {
    use petchain::components::veterinary_professional::types::Vet;
    use starknet::{
        ContractAddress, get_block_timestamp, get_caller_address, contract_address_const,
    };
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess, Map};

    #[storage]
    pub struct Storage {
        vet: Map<ContractAddress, Vet>,
        vets: Map<u256, Vet>,
        vet_licences: Map<felt252, Vet>,
        vets_id: u256,
        admin_address: ContractAddress,
        vet_id: u256,
        vet_address: ContractAddress,
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


    #[embeddable_as(VeterinaryProfessionalComponentImpl)]
    impl VeterinaryProfessionalComponenttImpl<
        TContractState, +HasComponent<TContractState>,
    > of super::IVeterinary_professional<ComponentState<TContractState>> {
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

            let existing_vet = self.get_vet_by_license_number(license_number);
            assert(!existing_vet.registered, ' already registered');

            assert(caller != zero_address, 'Zero Address detected');

            let id = self.vets_id.read() + 1;

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

            self.vet_licences.write(license_number, new_vet.clone());
            self.vets.write(id, new_vet.clone());
            self.vet.write(caller, new_vet);
            self.vets_id.write(id);

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

            let existing_vet = self.vet.read(caller);

            assert(existing_vet.registered, ' not registered');

            let mut vet = existing_vet;

            vet.name = name;
            vet.email = email;
            vet.emergency_contact = emergency_contact;
            vet.license_number = license_number;
            vet.specialization = specialization;
            vet.updated_at = timestamp;

            let id = vet.vet_id;

            self.vet_licences.write(license_number, vet.clone());
            self.vets.write(id, vet.clone());
            self.vet.write(caller, vet);

            self
                .emit(
                    Event::VetProfileUpdated(VetProfileUpdated { vet_id: id, vet_address: caller }),
                );

            true
        }

        fn activate_vet(ref self: ComponentState<TContractState>, address: ContractAddress) {
            let caller = get_caller_address();
            let zero_address = contract_address_const::<'0x0'>();
            assert(caller != zero_address, 'Zero Address detected');

            let admin = self.admin_address.read();
            assert(caller == admin, 'insufficient Permission');

            let timestamp = get_block_timestamp();

            let mut vet = self.vet.read(address);

            assert(!vet.is_active, 'Vet is already active');

            vet.is_active = true;

            vet.updated_at = timestamp;

            let id = vet.vet_id;

            self.vet_licences.write(vet.license_number, vet.clone());
            self.vets.write(id, vet.clone());
            self.vet.write(vet.address, vet);

            self.emit(Event::VetActivated(VetActivated { vet_id: id, vet_address: caller }));
        }

        fn deactivate_vet(ref self: ComponentState<TContractState>, address: ContractAddress) {
            let caller = get_caller_address();
            let zero_address = contract_address_const::<'0x0'>();
            assert(caller != zero_address, 'Zero Address detected');

            let admin = self.admin_address.read();
            assert(caller == admin, 'insufficient Permission');

            let timestamp = get_block_timestamp();

            let mut vet = self.vet.read(address);

            assert(vet.is_active, 'Vet is already inactive');

            vet.is_active = false;

            vet.updated_at = timestamp;

            let id = vet.vet_id;

            self.vet_licences.write(vet.license_number, vet.clone());
            self.vets.write(id, vet.clone());
            self.vet.write(vet.address, vet);

            self.emit(Event::VetDeActivated(VetDeActivated { vet_id: id, vet_address: caller }));
        }

        fn verify_vet(ref self: ComponentState<TContractState>, address: ContractAddress) {
            let caller = get_caller_address();
            let zero_address = contract_address_const::<'0x0'>();
            assert(caller != zero_address, 'Zero Address detected');

            let admin = self.admin_address.read();
            assert(caller == admin, 'insufficient Permission');

            let timestamp = get_block_timestamp();

            let mut vet = self.vet.read(address);

            assert(!vet.is_verified, 'Vet is already Verified');

            vet.is_verified = true;

            vet.updated_at = timestamp;

            let id = vet.vet_id;

            self.vet_licences.write(vet.license_number, vet.clone());
            self.vets.write(id, vet.clone());
            self.vet.write(vet.address, vet);

            self.emit(Event::VetVerified(VetVerified { vet_id: id, vet_address: caller }));
        }

        fn is_vet_active(
            ref self: ComponentState<TContractState>, address: ContractAddress,
        ) -> bool {
            let vet = self.vet.read(address);

            let is_active = vet.is_active;

            is_active
        }

        fn is_vet_verified(
            ref self: ComponentState<TContractState>, address: ContractAddress,
        ) -> bool {
            let vet = self.vet.read(address);

            let is_verified = vet.is_verified;

            is_verified
        }

        fn get_vet(ref self: ComponentState<TContractState>, address: ContractAddress) -> Vet {
            let vet = self.vet.read(address);
            vet
        }

        fn get_vet_by_id(ref self: ComponentState<TContractState>, vet_id: u256) -> Vet {
            let vet = self.vets.read(vet_id);
            vet
        }

        fn get_vet_by_license_number(
            ref self: ComponentState<TContractState>, license_number: felt252,
        ) -> Vet {
            let vet = self.vet_licences.read(license_number);
            vet
        }
        // FOR TESTING
        fn init(ref self: ComponentState<TContractState>, admin: ContractAddress) {
            self.admin_address.write(admin);
        }
    }
}
