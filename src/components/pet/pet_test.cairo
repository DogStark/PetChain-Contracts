#[cfg(test)]
mod tests {
    use petchain::components::pet::IPet::{IPetDispatcher, IPetDispatcherTrait};
    use snforge_std::{
        ContractClassTrait, DeclareResultTrait, declare, start_cheat_caller_address,
        stop_cheat_caller_address,
    };
    use starknet::{ContractAddress};

    fn setup() -> ContractAddress {
        let declare_result = declare("PetChain");
        assert(declare_result.is_ok(), 'Contract declaration failed');

        let contract_class = declare_result.unwrap().contract_class();
        let mut calldata = array![];
        let (contract_address, _) = contract_class.deploy(@calldata).unwrap();

        contract_address
    }


    #[test]
    fn test_register_pet() {
        let contract_address = setup();
        let dispatcher = IPetDispatcher { contract_address };

        let owner: ContractAddress = 12345.try_into().unwrap();

        let name: ByteArray = "Pablo";
        let birthday: ByteArray = "20-10-2024";
        let active: bool = true;

        start_cheat_caller_address(contract_address, owner);
        let pet_id = dispatcher.register_pet(name, birthday, active);
        stop_cheat_caller_address(owner);

        assert(pet_id == 1, 'pet_id should start from 1');

        let pet = dispatcher.get_pet(pet_id);

        assert(pet.name == "Pablo", 'name mismatch');
        assert(pet.birthday == "20-10-2024", 'birthday mismatch');
    }


    #[test]
    fn test_update_pet_profile() {
        let contract_address = setup();
        let dispatcher = IPetDispatcher { contract_address };

        let owner: ContractAddress = 12345.try_into().unwrap();

        let name: ByteArray = "Jake";
        let birthday: ByteArray = "02-02-2023";
        let active: bool = true;

        let name1: ByteArray = "Rambo";
        let birthday1: ByteArray = "01-01-2025";
        let active1: bool = false;

        start_cheat_caller_address(contract_address, owner);
        let pet_id = dispatcher.register_pet(name, birthday, active);

        let pet = dispatcher.get_pet(pet_id);
        assert(pet_id == 1, 'Pet not updated');
        assert(pet.name == "Jake", 'name mismatch');
        assert(pet.birthday == "02-02-2023", 'birthday mismatch');
        start_cheat_caller_address(contract_address, owner);
        let success = dispatcher.update_pet_profile(pet_id, name1, birthday1, active1);
        stop_cheat_caller_address(owner);

        let pet_1 = dispatcher.get_pet(pet.id);
        assert(success, 'Pet no updated');
        assert(pet_1.name == "Rambo", 'name mismatch');
        assert(pet_1.birthday == "01-01-2025", 'birthday mismatch');
    }
    #[test]
    fn test_multiple_pet_registrations() {
        let contract_address = setup();
        let dispatcher = IPetDispatcher { contract_address };

        let owner: ContractAddress = 12345.try_into().unwrap();

        let another_address: ContractAddress = 467372.try_into().unwrap();

        let name: ByteArray = "Jake";
        let birthday: ByteArray = "02-02-2023";
        let active: bool = true;

        let name1: ByteArray = "Rambo";
        let birthday1: ByteArray = "01-01-2025";
        let active1: bool = false;

        start_cheat_caller_address(contract_address, owner);
        let _id = dispatcher.register_pet(name, birthday, active);
        stop_cheat_caller_address(owner);

        start_cheat_caller_address(contract_address, another_address);
        let pet_id_2 = dispatcher.register_pet(name1, birthday1, active1);
        stop_cheat_caller_address(another_address);
        assert(pet_id_2 == 2, 'Id did not increment');
    }
    #[test]
    #[should_panic(expected: ('Only owner can update',))]
    fn test_update_pet_profile_with_wrong_caller() {
        let contract_address = setup();
        let dispatcher = IPetDispatcher { contract_address };

        let owner: ContractAddress = 12345.try_into().unwrap();

        let another_address: ContractAddress = 467372.try_into().unwrap();

        let name: ByteArray = "Jake";
        let birthday: ByteArray = "02-02-2023";
        let active: bool = true;

        let name1: ByteArray = "Rambo";
        let birthday1: ByteArray = "01-01-2025";
        let active1: bool = false;

        let name2: ByteArray = "Samba";
        let birthday2: ByteArray = "30-12-2019";
        let active2: bool = true;

        start_cheat_caller_address(contract_address, owner);
        let pet_id = dispatcher.register_pet(name, birthday, active);
        stop_cheat_caller_address(owner);

        start_cheat_caller_address(contract_address, another_address);
        let _pet_id1 = dispatcher.register_pet(name1, birthday1, active1);
        stop_cheat_caller_address(another_address);

        start_cheat_caller_address(contract_address, another_address);
        let _pet2 = dispatcher.update_pet_profile(pet_id, name2, birthday2, active2);
        stop_cheat_caller_address(another_address);
    }

    #[test]
    #[should_panic(expected: ('Pet not found',))]
    fn test_update_pet_profile_with_wrong_pet_id() {
        let contract_address = setup();
        let dispatcher = IPetDispatcher { contract_address };

        let owner: ContractAddress = 12345.try_into().unwrap();
        let another_address: ContractAddress = 467372.try_into().unwrap();

        let name: ByteArray = "Jake";
        let birthday: ByteArray = "02-02-2023";
        let active: bool = true;

        let name1: ByteArray = "Rambo";
        let birthday1: ByteArray = "01-01-2025";
        let active1: bool = false;

        start_cheat_caller_address(contract_address, owner);
        let pet_id = dispatcher.register_pet(name, birthday, active);
        stop_cheat_caller_address(owner);

        start_cheat_caller_address(contract_address, another_address);
        let _success = dispatcher.update_pet_profile(pet_id + 1, name1, birthday1, active1);
        stop_cheat_caller_address(another_address);
    }
}
