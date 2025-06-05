#[cfg(test)]
mod tests {
    use petchain::components::medical_record::interface::{
        IMedicalRecordsDispatcher, IMedicalRecordsDispatcherTrait,
    };

    use petchain::components::medical_record::types::{Medication, MedicalRecordType};
    use snforge_std::{
        ContractClassTrait, DeclareResultTrait, declare, start_cheat_caller_address,
        stop_cheat_caller_address,
    };
    use starknet::{ContractAddress};

    fn setup() -> ContractAddress {
        let declare_result = declare("MockMedicalRecordsComponent");
        assert(declare_result.is_ok(), 'Contract declaration failed');
        // Deploy PetChain
        let contract_class = declare_result.unwrap().contract_class();
        let mut calldata = array![];
        let (contract_address, _) = contract_class.deploy(@calldata).unwrap();

        contract_address
    }

    #[test]
    fn test_create_medical_record() {
        let contract_address = setup();
        let dispatcher = IMedicalRecordsDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();

        let pet_id: u256 = 1;
        let record_type = MedicalRecordType::Checkup;
        let title: ByteArray = "Initial Checkup";
        let description: ByteArray = "Routine health check.";
        let diagnosis: ByteArray = "Healthy";
        let treatment: ByteArray = "None";
        let medications: Array<Medication> = array![];
        let next_visit_date: u64 = 1722455450;
        let is_emergency = false;
        let is_follow_up_required = false;
        let visit_cost = 250_u256;

        start_cheat_caller_address(contract_address, vet_address);
        let record_id = dispatcher
            .create_medical_record(
                pet_id,
                record_type,
                title.clone(),
                description.clone(),
                diagnosis.clone(),
                treatment.clone(),
                medications,
                next_visit_date,
                is_emergency,
                is_follow_up_required,
                visit_cost,
            );
        stop_cheat_caller_address(vet_address);

        assert(record_id == 1, 'record_id incorrect');
        // -------------------------------------------------------
        // FetchMedical History and Assert get_pet_medical_history
        // -------------------------------------------------------
        let record = dispatcher.get_medical_record(record_id);

        let history = dispatcher.get_pet_medical_history(pet_id);

        let history_record = history.at(0);

        assert(record.id == *history_record.id, 'record id mismatch');
        assert(record.pet_id == *history_record.pet_id, 'pet id mismatch');
        assert(record.title.clone() == history_record.title.clone(), 'title mismatch');
        assert(
            record.description.clone() == history_record.description.clone(),
            'description mismatch',
        );
        assert(record.diagnosis.clone() == history_record.diagnosis.clone(), 'diagnosis mismatch');
        assert(record.treatment.clone() == history_record.treatment.clone(), 'treatment mismatch');
        assert(record.next_visit_date == *history_record.next_visit_date, 'next visit mismatch');
        assert(record.is_emergency == *history_record.is_emergency, 'emergency status mismatch');
        assert(
            record.is_follow_up_required == *history_record.is_follow_up_required,
            'follow-up status mismatch',
        );
        assert(record.visit_cost == *history_record.visit_cost, 'visit cost mismatch');

        // testing emergency update
        let emergency_count = dispatcher.get_emergency_count();
        assert(emergency_count == 0, 'emergency_count error');

        // testing require_follow_up
        let require_follow_up = dispatcher.get_is_follow_up_required(record_id);
        assert(!require_follow_up, 'emergency_count error');
    }


    #[test]
    fn test_create_two_medical_records() {
        let contract_address = setup();
        let dispatcher = IMedicalRecordsDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();

        let pet_id: u256 = 1;
        let record_type = MedicalRecordType::Checkup;
        let record_type2 = MedicalRecordType::Surgery;
        let title1: ByteArray = "Initial Checkup";
        let description1: ByteArray = "Routine health check.";
        let diagnosis1: ByteArray = "Healthy";
        let treatment1: ByteArray = "None";
        let medications1: Array<Medication> = array![];
        let next_visit_date1: u64 = 1722455450;
        let visit_cost1 = 250_u256;

        let title2: ByteArray = "Follow-up Check";
        let description2: ByteArray = "Second visit checkup.";
        let diagnosis2: ByteArray = "Still Healthy";
        let treatment2: ByteArray = "None";
        let medications2: Array<Medication> = array![];
        let next_visit_date2: u64 = 1723055450;
        let visit_cost2 = 300_u256;

        start_cheat_caller_address(contract_address, vet_address);

        // Create first record
        let _record_id1 = dispatcher
            .create_medical_record(
                pet_id,
                record_type,
                title1.clone(),
                description1.clone(),
                diagnosis1.clone(),
                treatment1.clone(),
                medications1,
                next_visit_date1,
                false,
                false,
                visit_cost1,
            );

        // Create second record
        let record_id2 = dispatcher
            .create_medical_record(
                pet_id,
                record_type2,
                title2.clone(),
                description2.clone(),
                diagnosis2.clone(),
                treatment2.clone(),
                medications2,
                next_visit_date2,
                true,
                true,
                visit_cost2,
            );

        stop_cheat_caller_address(vet_address);

        let record = dispatcher.get_medical_record(record_id2);
        // Second record assertions
        let history = dispatcher.get_pet_medical_history(pet_id);

        let history_record = history.at(1);

        assert(record.id == *history_record.id, 'record id mismatch');
        assert(record.pet_id == *history_record.pet_id, 'pet id mismatch');
        assert(record.title.clone() == history_record.title.clone(), 'title mismatch');
        assert(
            record.description.clone() == history_record.description.clone(),
            'description mismatch',
        );
        assert(record.diagnosis.clone() == history_record.diagnosis.clone(), 'diagnosis mismatch');
        assert(record.treatment.clone() == history_record.treatment.clone(), 'treatment mismatch');
        assert(record.next_visit_date == *history_record.next_visit_date, 'next visit mismatch');
        assert(record.is_emergency == *history_record.is_emergency, 'emergency status mismatch');
        assert(
            record.is_follow_up_required == *history_record.is_follow_up_required,
            'follow-up status mismatch',
        );
        assert(record.visit_cost == *history_record.visit_cost, 'visit cost mismatch');
        assert(record_id2 == 2, 'record_id2 incorrect');
        // testing emergency update
        let emergency_count = dispatcher.get_emergency_count();
        assert(emergency_count == 1, 'emergency_count error');

        // testing require_follow_up
        let require_follow_up = dispatcher.get_is_follow_up_required(record_id2);
        assert(require_follow_up, 'emergency_count error');
    }
}
