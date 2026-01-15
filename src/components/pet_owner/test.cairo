#[cfg(test)]
mod tests {
    use petchain::components::pet_owner::interface::{IPetOwnerDispatcher, IPetOwnerDispatcherTrait};
    use snforge_std::{
        ContractClassTrait, DeclareResultTrait, declare, start_cheat_caller_address,
        stop_cheat_caller_address,
    };
    use stellar::{ContractAddress};

    fn setup() -> ContractAddress {
        let declare_result = declare("MockPetOwner");
        assert(declare_result.is_ok(), 'Contract declaration failed');
        // Deploy PetChain
        let contract_class = declare_result.unwrap().contract_class();
        let mut calldata = array![];
        let (contract_address, _) = contract_class.deploy(@calldata).unwrap();

        contract_address
    }


    #[test]
    fn test_register_pet_owner() {
        let contract_address = setup();
        let dispatcher = IPetOwnerDispatcher { contract_address };

        let owner_address: ContractAddress = 12345.try_into().unwrap();

        // Test input values
        let name: ByteArray = "John";
        let email: ByteArray = "John@yahoo.com";
        let emergency_contact: ByteArray = "1234567890";

        start_cheat_caller_address(contract_address, owner_address);
        dispatcher.register_pet_owner(name, email, emergency_contact);
        stop_cheat_caller_address(owner_address);

        // Retrieve the account to verify it was stored correctly
        let owner_account = dispatcher.get_pet_owner(owner_address);

        assert(owner_account.is_pet_owner, 'registration failed');
        assert(owner_account.name == "John", 'name mismatch');
        assert(owner_account.email == "John@yahoo.com", 'email mismatch');
        assert(owner_account.emergency_contact == "1234567890", 'emergency_contact mismatch');
    }


    #[test]
    #[should_panic(expected: ('Already Registered',))]
    fn test_register_pet_owner_twice() {
        let contract_address = setup();
        let dispatcher = IPetOwnerDispatcher { contract_address };

        let owner_address: ContractAddress = 12345.try_into().unwrap();

        // Test input values
        let name: ByteArray = "John";
        let email: ByteArray = "John@yahoo.com";
        let emergency_contact: ByteArray = "1234567890";
        let name1: ByteArray = "John";
        let email1: ByteArray = "John@yahoo.com";
        let emergency_contact1: ByteArray = "1234567890";

        start_cheat_caller_address(contract_address, owner_address);
        dispatcher.register_pet_owner(name, email, emergency_contact);
        dispatcher.register_pet_owner(name1, email1, emergency_contact1);
        stop_cheat_caller_address(owner_address);
    }

    #[test]
    fn test_update_owner_profile() {
        let contract_address = setup();
        let dispatcher = IPetOwnerDispatcher { contract_address };

        let owner_address: ContractAddress = 12345.try_into().unwrap();

        // Test input values
        let name: ByteArray = "John";
        let email: ByteArray = "John@yahoo.com";
        let emergency_contact: ByteArray = "1234567890";

        let name1: ByteArray = "James";
        let email1: ByteArray = "James@yahoo.com";
        let emergency_contact1: ByteArray = "10987654321";

        start_cheat_caller_address(contract_address, owner_address);
        dispatcher.register_pet_owner(name, email, emergency_contact);
        // Retrieve the account to verify it was stored correctly
        let owner_account = dispatcher.get_pet_owner(owner_address);

        assert(owner_account.name == "John", 'name mismatch');
        assert(owner_account.email == "John@yahoo.com", 'email mismatch');
        assert(owner_account.emergency_contact == "1234567890", 'emergency_contact mismatch');

        start_cheat_caller_address(contract_address, owner_address);
        let success = dispatcher.update_owner_profile(name1, email1, emergency_contact1);
        stop_cheat_caller_address(owner_address);

        // Retrieve the account to verify it was stored correctly
        let owner_account_1 = dispatcher.get_pet_owner(owner_address);
        assert(success, 'Profile no updated');
        assert(owner_account_1.name == "James", 'name mismatch');
        assert(owner_account_1.email == "James@yahoo.com", 'email mismatch');
        assert(owner_account_1.emergency_contact == "10987654321", 'emergency_contact mismatch');
    }
    #[test]
    fn test_register_two_profiles() {
        let contract_address = setup();
        let dispatcher = IPetOwnerDispatcher { contract_address };

        let owner_address: ContractAddress = 12345.try_into().unwrap();

        let another_address: ContractAddress = 467372.try_into().unwrap();

        // Test input values
        let name: ByteArray = "John";
        let email: ByteArray = "John@yahoo.com";
        let emergency_contact: ByteArray = "1234567890";

        let name1: ByteArray = "James";
        let email1: ByteArray = "James@yahoo.com";
        let emergency_contact1: ByteArray = "10987654321";

        start_cheat_caller_address(contract_address, owner_address);
        dispatcher.register_pet_owner(name, email, emergency_contact);
        stop_cheat_caller_address(owner_address);

        start_cheat_caller_address(contract_address, another_address);
        dispatcher.register_pet_owner(name1, email1, emergency_contact1);
        stop_cheat_caller_address(another_address);
    }


    #[test]
    #[should_panic(expected: ('Not Registered',))]
    fn test_update_profile_without_registering() {
        let contract_address = setup();
        let dispatcher = IPetOwnerDispatcher { contract_address };

        let owner_address: ContractAddress = 12345.try_into().unwrap();
        let another_address: ContractAddress = 467372.try_into().unwrap();

        // Test input values
        let name: ByteArray = "John";
        let email: ByteArray = "John@yahoo.com";
        let emergency_contact: ByteArray = "1234567890";

        let name1: ByteArray = "James";
        let email1: ByteArray = "James@yahoo.com";
        let emergency_contact1: ByteArray = "10987654321";

        start_cheat_caller_address(contract_address, owner_address);
        dispatcher.register_pet_owner(name, email, emergency_contact);
        stop_cheat_caller_address(owner_address);

        start_cheat_caller_address(contract_address, another_address);
        let _success = dispatcher.update_owner_profile(name1, email1, emergency_contact1);
        stop_cheat_caller_address(another_address);
    }
}
