#[cfg(test)]
mod tests {
    use petchain::components::veterinary_professional::interface::{
        IVetDispatcher, IVetDispatcherTrait,
    };
    use snforge_std::{
        ContractClassTrait, DeclareResultTrait, declare, start_cheat_caller_address,
        stop_cheat_caller_address,
    };
    use starknet::{ContractAddress};

    fn setup() -> ContractAddress {
        let declare_result = declare("MockVeterinaryProfessional");
        assert(declare_result.is_ok(), 'Contract declaration failed');
        // Deploy PetChain
        let contract_class = declare_result.unwrap().contract_class();
        let mut calldata = array![];
        let (contract_address, _) = contract_class.deploy(@calldata).unwrap();

        contract_address
    }

    #[test]
    fn test_register_vet() {
        let contract_address = setup();
        let dispatcher = IVetDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();

        // Test input values
        let name: ByteArray = "John";
        let email: ByteArray = "John@yahoo.com";
        let emergency_contact: ByteArray = "1234567890";
        let license_number = 'JOS1234';
        let specialization: ByteArray = "Sugery";

        start_cheat_caller_address(contract_address, vet_address);
        let vet_id = dispatcher
            .register_vet(name, email, emergency_contact, license_number, specialization);
        stop_cheat_caller_address(vet_address);

        // Retrieve the account to verify it was stored correctly
        let vet = dispatcher.get_vet(vet_address);

        assert(vet_id == 1, 'wrong initialization');
        assert(vet.name == "John", 'name mismatch');
        assert(vet.email == "John@yahoo.com", 'email mismatch');
        assert(vet.emergency_contact == "1234567890", 'emergency_contact mismatch');
        assert(vet.license_number == license_number, 'license_number failed');
        assert(vet.specialization == "Sugery", 'specialization failed');
    }

    #[test]
    fn test_register_two_vets() {
        let contract_address = setup();
        let dispatcher = IVetDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();
        let second_vet_address: ContractAddress = 54321.try_into().unwrap();

        // Test input values
        let name: ByteArray = "John";
        let email: ByteArray = "John@yahoo.com";
        let emergency_contact: ByteArray = "1234567890";
        let license_number = 'JOS1234';
        let specialization: ByteArray = "Sugery";

        start_cheat_caller_address(contract_address, vet_address);
        let _vet_id = dispatcher
            .register_vet(name, email, emergency_contact, license_number, specialization);
        stop_cheat_caller_address(vet_address);

        let name1: ByteArray = "James";
        let email1: ByteArray = "James@yahoo.com";
        let emergency_contact1: ByteArray = "1234567890";
        let license_number1 = 'LAGOS1234';
        let specialization1: ByteArray = "Anmial Husbandry";
        start_cheat_caller_address(contract_address, second_vet_address);
        let vet_id_1 = dispatcher
            .register_vet(name1, email1, emergency_contact1, license_number1, specialization1);
        stop_cheat_caller_address(vet_address);

        // Retrieve the account to verify it was stored correctly
        let vet = dispatcher.get_vet(second_vet_address);

        assert(vet_id_1 == 2, 'wrong initialization');
        assert(vet.name == "James", 'name mismatch');
        assert(vet.email == "James@yahoo.com", 'email mismatch');
        assert(vet.emergency_contact == "1234567890", 'emergency_contact mismatch');
        assert(vet.license_number == license_number1, 'license_number failed');
        assert(vet.specialization == "Anmial Husbandry", 'specialization failed');
    }
    #[test]
    #[should_panic(expected: ('License already registered',))]
    fn test_register_two_vets_same_license() {
        let contract_address = setup();
        let dispatcher = IVetDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();

        let malicious_address: ContractAddress = 21563.try_into().unwrap();

        // Test input values
        let name: ByteArray = "John";
        let email: ByteArray = "John@yahoo.com";
        let emergency_contact: ByteArray = "1234567890";
        let name1: ByteArray = "John";
        let email1: ByteArray = "John@yahoo.com";
        let emergency_contact1: ByteArray = "1234567890";
        let license_number = 'JOS1234';
        let specialization: ByteArray = "Sugery";
        let specialization1: ByteArray = "Anmial Husbandry";

        start_cheat_caller_address(contract_address, vet_address);
        dispatcher.register_vet(name, email, emergency_contact, license_number, specialization);
        start_cheat_caller_address(contract_address, malicious_address);
        dispatcher.register_vet(name1, email1, emergency_contact1, license_number, specialization1);
        stop_cheat_caller_address(vet_address);
    }
    #[test]
    #[should_panic(expected: ('Already registered',))]
    fn test_register_vet_twice_with_same_license_numbers() {
        let contract_address = setup();
        let dispatcher = IVetDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();

        // Test input values
        let name: ByteArray = "John";
        let email: ByteArray = "John@yahoo.com";
        let emergency_contact: ByteArray = "1234567890";
        let name1: ByteArray = "John";
        let email1: ByteArray = "John@yahoo.com";
        let emergency_contact1: ByteArray = "1234567890";
        let license_number = 'KAD1234';
        let license_number1 = 'KAD1234';
        let specialization: ByteArray = "Sugery";
        let specialization1: ByteArray = "Anmial Husbandry";

        start_cheat_caller_address(contract_address, vet_address);
        dispatcher.register_vet(name, email, emergency_contact, license_number, specialization);

        dispatcher
            .register_vet(name1, email1, emergency_contact1, license_number1, specialization1);
        stop_cheat_caller_address(vet_address);
    }

    #[test]
    #[should_panic(expected: ('Already registered',))]
    fn test_register_same_vet_different_license_numbers() {
        let contract_address = setup();
        let dispatcher = IVetDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();

        // Test input values
        let name: ByteArray = "John";
        let email: ByteArray = "John@yahoo.com";
        let emergency_contact: ByteArray = "1234567890";
        let name1: ByteArray = "John";
        let email1: ByteArray = "John@yahoo.com";
        let emergency_contact1: ByteArray = "1234567890";
        let license_number = 'KAD1234';
        let license_number1 = 'Lagos1234';
        let specialization: ByteArray = "Sugery";
        let specialization1: ByteArray = "Anmial Husbandry";

        start_cheat_caller_address(contract_address, vet_address);
        dispatcher.register_vet(name, email, emergency_contact, license_number, specialization);
        dispatcher
            .register_vet(name1, email1, emergency_contact1, license_number1, specialization1);
        stop_cheat_caller_address(vet_address);
    }
    #[test]
    fn test_update_vet_profile_ok() {
        let contract_address = setup();
        let dispatcher = IVetDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();

        // Test input values
        let name: ByteArray = "John";
        let email: ByteArray = "John@yahoo.com";
        let emergency_contact: ByteArray = "1234567890";
        let license_number = 'KAD1234';
        let specialization: ByteArray = "Sugery";
        let license_number1 = 'JOS1234';
        let specialization1: ByteArray = "Care";

        let name1: ByteArray = "James";
        let email1: ByteArray = "James@yahoo.com";
        let emergency_contact1: ByteArray = "10987654321";

        start_cheat_caller_address(contract_address, vet_address);
        let _vet_id = dispatcher
            .register_vet(name, email, emergency_contact, license_number, specialization);
        // Retrieve the account to verify it was stored correctly
        let owner_account = dispatcher.get_vet(vet_address);

        assert(owner_account.name == "John", 'name mismatch');
        assert(owner_account.email == "John@yahoo.com", 'email mismatch');
        assert(owner_account.emergency_contact == "1234567890", 'emergency_contact mismatch');

        start_cheat_caller_address(contract_address, vet_address);
        let success = dispatcher
            .update_vet_profile(
                name1, email1, emergency_contact1, license_number1, specialization1,
            );
        stop_cheat_caller_address(vet_address);

        // Retrieve the account to verify it was stored correctly
        let vet = dispatcher.get_vet(vet_address);

        let vet_0 = dispatcher.get_vet_by_license_number(license_number);
        assert(vet_0.vet_id == 0, 'vet id should be 0');

        let vet_1 = dispatcher.get_vet_by_license_number(license_number1);
        assert(vet_1.vet_id == vet.vet_id, 'vet id mismatch');

        assert(success, 'Profile no updated');
        assert(vet.name == "James", 'name mismatch');
        assert(vet.email == "James@yahoo.com", 'email mismatch');
        assert(vet.emergency_contact == "10987654321", 'emergency_contact mismatch');
        assert(vet.license_number == 'JOS1234', 'license failed');
        assert(vet.specialization == "Care", 'specialization failed')
    }


    #[test]
    #[should_panic(expected: ('Not registered',))]
    fn test_update_vet_profile_without_registering() {
        let contract_address = setup();
        let dispatcher = IVetDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();

        // Test input value
        let license_number1 = 'JOS1234';
        let specialization1: ByteArray = "Care";

        let name1: ByteArray = "James";
        let email1: ByteArray = "James@yahoo.com";
        let emergency_contact1: ByteArray = "10987654321";

        start_cheat_caller_address(contract_address, vet_address);
        let _success = dispatcher
            .update_vet_profile(
                name1, email1, emergency_contact1, license_number1, specialization1,
            );
        stop_cheat_caller_address(vet_address);
    }

    #[test]
    fn test_activate_vet_profile() {
        let contract_address = setup();
        let dispatcher = IVetDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();

        // Test input values
        let name: ByteArray = "John";
        let email: ByteArray = "John@yahoo.com";
        let emergency_contact: ByteArray = "1234567890";
        let license_number = 'JOS1234';
        let specialization: ByteArray = "Sugery";

        start_cheat_caller_address(contract_address, vet_address);
        let _vet_id = dispatcher
            .register_vet(name, email, emergency_contact, license_number, specialization);
        stop_cheat_caller_address(contract_address);

        dispatcher.activate_vet(vet_address);

        // Retrieve the account to verify it was stored correctly
        let vet = dispatcher.get_vet(vet_address);
        assert(vet.is_active, 'activation failed');
    }


    #[test]
    fn test_deactivate_vet_profile() {
        let contract_address = setup();
        let dispatcher = IVetDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();

        // Test input values
        let name: ByteArray = "John";
        let email: ByteArray = "John@yahoo.com";
        let emergency_contact: ByteArray = "1234567890";
        let license_number = 'JOS1234';
        let specialization: ByteArray = "Sugery";

        start_cheat_caller_address(contract_address, vet_address);
        let _vet_id = dispatcher
            .register_vet(name, email, emergency_contact, license_number, specialization);
        stop_cheat_caller_address(vet_address);

        let _success = dispatcher.activate_vet(vet_address);
        let vet1 = dispatcher.get_vet(vet_address);
        assert(vet1.is_active, 'activation failed');

        let _success = dispatcher.deactivate_vet(vet_address);
        stop_cheat_caller_address(vet_address);
        // Retrieve the account to verify it was stored correctly
        let vet = dispatcher.get_vet(vet_address);
        assert(!vet.is_active, 'deactivation failed');
    }


    #[test]
    fn test_verify_vet_profile() {
        let contract_address = setup();
        let dispatcher = IVetDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();

        // Test input values
        let name: ByteArray = "John";
        let email: ByteArray = "John@yahoo.com";
        let emergency_contact: ByteArray = "1234567890";
        let license_number = 'JOS1234';
        let specialization: ByteArray = "Sugery";

        start_cheat_caller_address(contract_address, vet_address);
        let _vet_id = dispatcher
            .register_vet(name, email, emergency_contact, license_number, specialization);
        stop_cheat_caller_address(contract_address);

        dispatcher.verify_vet(vet_address);
        stop_cheat_caller_address(contract_address);

        // Retrieve the account to verify it was stored correctly
        let vet = dispatcher.get_vet(vet_address);
        assert(vet.is_verified, 'activation failed');
    }
}
