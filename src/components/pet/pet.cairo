use petchain::components::pet::interface::{IPet};
#[starknet::component]
pub mod PetComponent {
    use core::array::{Array, ArrayTrait};
    use petchain::components::pet::types::{Pet, Gender, Species};
    use starknet::{ContractAddress, get_block_timestamp, get_caller_address};
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess, Map};

    #[storage]
    pub struct Storage {
        pets: Map<u256, Pet>,
        pet_by_owners: Map<ContractAddress, u256>,
        owner_pet_index: Map<(ContractAddress, u256), u256>,
        pet_count_by_owner: Map<ContractAddress, u256>,
        total_pet_count: u256,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        PetCreated: PetCreated,
        PetUpdated: PetUpdated,
        PetActivated: PetActivated,
        PetDeActivated: PetDeActivated,
        PetOwnershipTransferred: PetOwnershipTransferred,
    }

    #[derive(Drop, starknet::Event)]
    struct PetCreated {
        #[key]
        owner: ContractAddress,
        pet_id: u256,
    }

    #[derive(Drop, starknet::Event)]
    struct PetUpdated {
        #[key]
        owner: ContractAddress,
        pet_id: u256,
    }

    #[derive(Drop, starknet::Event)]
    struct PetActivated {
        #[key]
        vet_id: u256,
    }

    #[derive(Drop, starknet::Event)]
    struct PetDeActivated {
        #[key]
        vet_id: u256,
    }

    #[derive(Drop, starknet::Event)]
    struct PetOwnershipTransferred {
        #[key]
        pet_id: u256,
        from: ContractAddress,
        to: ContractAddress,
    }

    #[embeddable_as(PetImpl)]
    impl PetComponentImpl<
        TContractState, +HasComponent<TContractState>,
    > of super::IPet<ComponentState<TContractState>> {
        fn register_pet(
            ref self: ComponentState<TContractState>,
            name: ByteArray,
            birthday: ByteArray,
            gender: Gender,
            species: Species,
            breed: felt252,
        ) -> u256 {
            assert(name != "", 'name is empty');
            assert(birthday != "", 'birthday is empty');
            let caller = get_caller_address();
            let total_pet_count = self.total_pet_count.read();
            let id: u256 = total_pet_count + 1;
            let pet = Pet {
                id,
                owner: caller,
                name,
                birthday,
                active: false,
                created_at: get_block_timestamp(),
                updated_at: get_block_timestamp(),
                new_owner: caller,
                species: species,
                gender: gender,
                breed: breed,
            };

            self.pets.write(id, pet);
            self.total_pet_count.write(id);
            self.pet_by_owners.write(caller, id);
            let owner_pet_count = self.pet_count_by_owner.read(caller) + 1;
            self.pet_count_by_owner.write(caller, owner_pet_count);
            self.owner_pet_index.write((caller, owner_pet_count), id);
            self.emit(Event::PetCreated(PetCreated { owner: caller, pet_id: id }));

            id
        }

        fn update_pet_profile(
            ref self: ComponentState<TContractState>,
            id: u256,
            name: ByteArray,
            birthday: ByteArray,
            gender: Gender,
            species: Species,
            breed: felt252,
        ) -> bool {
            assert(id > 0, 'invalid id');
            assert(name != "", 'name is empty');
            assert(birthday != "", 'birthday is empty');
            let pet = self.pets.read(id);
            assert(pet.id == id, 'Pet not found');
            let caller = get_caller_address();
            assert(pet.owner == caller, 'Only owner can update');

            let mut pet = self.pets.read(id);

            let active = pet.active;

            pet.name = name;
            pet.birthday = birthday;
            pet.active = active;
            pet.species = species;
            pet.gender = gender;
            pet.breed = breed;
            pet.updated_at = get_block_timestamp();

            self.pets.write(pet.id, pet);

            self.emit(Event::PetUpdated(PetUpdated { owner: caller, pet_id: id }));
            true
        }

        //TODO: Restrict to admin
        fn activate_pet(ref self: ComponentState<TContractState>, id: u256) {
            let mut pet = self.pets.read(id);
            assert(pet.id == id, 'Pet not found');

            let mut pet = self.pets.read(id);

            let timestamp = get_block_timestamp();

            assert(!pet.active, 'Pet is already active');

            pet.active = true;

            pet.updated_at = timestamp;

            self.pets.write(pet.id, pet);

            self.emit(Event::PetActivated(PetActivated { vet_id: id }));
        }

        //TODO: Restrict to admin
        fn deactivate_pet(ref self: ComponentState<TContractState>, id: u256) {
            let mut pet = self.pets.read(id);
            assert(pet.id == id, 'Pet not found');

            let timestamp = get_block_timestamp();

            assert(pet.active, 'Pet is not active');

            pet.active = false;

            pet.updated_at = timestamp;

            self.pets.write(pet.id, pet);

            self.emit(Event::PetDeActivated(PetDeActivated { vet_id: id }));
        }

        fn transfer_pet_ownership(
            ref self: ComponentState<TContractState>, id: u256, to: ContractAddress,
        ) {
            let mut pet = self.pets.read(id);
            let caller = get_caller_address();
            assert(pet.id == id, 'Pet not found');
            assert(pet.owner == caller, 'Not your pet');
            assert(pet.owner != to, 'Already the owner');

            pet.new_owner = to;
            pet.updated_at = get_block_timestamp();
            self.pets.write(id, pet);
        }

        fn accept_pet_transfer(ref self: ComponentState<TContractState>, id: u256) {
            let mut pet = self.pets.read(id);
            let caller = get_caller_address();
            assert(pet.id == id, 'Pet not found');
            assert(pet.new_owner == caller, 'Transfer not assigned to caller');

            // Old owner mappings
            let old_owner = pet.owner;
            let old_owner_count = self.pet_count_by_owner.read(old_owner);
            let new_owner_count = self.pet_count_by_owner.read(caller);

            // Update mappings
            self.pet_by_owners.write(caller, id);
            self.pet_count_by_owner.write(old_owner, old_owner_count - 1);
            self.pet_count_by_owner.write(caller, new_owner_count + 1);
            self.owner_pet_index.write((caller, new_owner_count + 1), id);

            // Update pet ownership
            pet.owner = caller;
            pet.new_owner = caller;
            pet.updated_at = get_block_timestamp();
            self.pets.write(id, pet);

            self
                .emit(
                    Event::PetOwnershipTransferred(
                        PetOwnershipTransferred { pet_id: id, from: old_owner, to: caller },
                    ),
                );
        }


        fn get_pet(self: @ComponentState<TContractState>, id: u256) -> Pet {
            let pet = self.pets.read(id);
            pet
        }

        fn is_pet_active(self: @ComponentState<TContractState>, id: u256) -> bool {
            let pet = self.pets.read(id);
            pet.active
        }

        fn get_pet_owner(self: @ComponentState<TContractState>, id: u256) -> ContractAddress {
            let pet = self.pets.read(id);
            pet.owner
        }

        fn get_pets_by_owner(
            self: @ComponentState<TContractState>, owner: ContractAddress,
        ) -> Array<Pet> {
            self.get_all_pets_by_owner(owner)
        }
        fn get_all_pets(self: @ComponentState<TContractState>) -> Array<Pet> {
            let mut all_pets = ArrayTrait::new();
            let total_pets = self.total_pet_count.read();

            for i in 1..total_pets {
                let pet = self.pets.read(i);
                all_pets.append(pet);
            };

            all_pets
        }
    }

    #[generate_trait]
    pub impl InternalImpl<
        TContractState, +HasComponent<TContractState>,
    > of InternalTrait<TContractState> {
        fn get_pet_ids_by_owner(
            self: @ComponentState<TContractState>, owner: ContractAddress,
        ) -> Array<u256> {
            let mut pet_ids = ArrayTrait::new();
            let count = self.pet_count_by_owner.read(owner);

            for i in 1..count {
                let pet_id = self.owner_pet_index.read((owner, i));
                pet_ids.append(pet_id);
            };
            pet_ids
        }
        fn get_all_pets_by_owner(
            self: @ComponentState<TContractState>, owner: ContractAddress,
        ) -> Array<Pet> {
            let pet_ids = self.get_pet_ids_by_owner(owner);
            let mut pets = ArrayTrait::new();

            for i in 0..pet_ids.len() {
                let pet_id = *pet_ids.at(i);
                let pet = self.pets.read(pet_id);
                pets.append(pet);
            };
            pets
        }
    }
}
