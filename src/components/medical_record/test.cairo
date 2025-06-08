#[cfg(test)]
mod tests {
    use petchain::components::medical_record::interface::{
        IMedicalRecordsDispatcher, IMedicalRecordsDispatcherTrait,
    };

    use petchain::components::medical_record::types::{Medication, MedicalRecordType};
    use snforge_std::{
        ContractClassTrait, DeclareResultTrait, declare, start_cheat_caller_address,
        stop_cheat_caller_address, start_cheat_block_timestamp_global,
    };
    use starknet::{ContractAddress, get_block_timestamp};


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

    #[test]
    fn test_update_medication_successfully_with_all_valid_fields() {
        let contract_address = setup();
        let med_1 = new_medication(
            0, // id
            0, // medical_record_id
            0, // pet_id
            "Med A", // name
            "10mg", // dosage
            "Once daily", // frequency
            10, // duration_days
            "Take after meals", // instructions
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let med_2 = new_medication(
            2,
            102,
            1002,
            "Med B",
            "5mg",
            "Twice daily",
            7,
            "Take before sleep",
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let med_3 = new_medication(
            0,
            0,
            0,
            "Med C",
            "20mg",
            "Once daily",
            14,
            "Avoid alcohol",
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let med_4 = new_medication(
            0,
            0,
            0,
            "Med D",
            "15mg",
            "Three times daily",
            5,
            "Do not skip doses",
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let med_5 = new_medication(
            0,
            0,
            0,
            "Med E",
            "50mg",
            "Once weekly",
            15,
            "Store in cool place",
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let dispatcher = IMedicalRecordsDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();

        let pet_id: u256 = 1;
        let record_type = MedicalRecordType::Checkup;
        let title: ByteArray = "Initial Checkup";
        let description: ByteArray = "Routine health check.";
        let diagnosis: ByteArray = "Healthy";
        let treatment: ByteArray = "None";
        let medications: Array<Medication> = array![med_1, med_2, med_3, med_4, med_5];
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
                medications.clone(),
                next_visit_date,
                is_emergency,
                is_follow_up_required,
                visit_cost,
            );
        stop_cheat_caller_address(vet_address);

        let total_medication_count = dispatcher.get_medical_record_medications(record_id);
        assert(total_medication_count.len() == 5, 'global med count error');

        let success = dispatcher
            .update_medication(
                5, "Med E1", "100mg", "once a day", 30, "keep away from sunlight", 0, true,
            );
        let med1 = dispatcher.get_medication(5);

        assert(success, 'update failed');

        assert(med1.id == 5, 'get med error');
        assert(med1.medical_record_id == record_id, 'record id mismatch');
        assert(med1.pet_id == pet_id, 'pet id mismatch');
        assert(med1.name == "Med E1", 'name mismatch');
        assert(med1.dosage == "100mg", 'dosage mismatch');
        assert(med1.frequency == "once a day", 'frequency mismatch');
        assert(med1.instructions == "keep away from sunlight", 'instructions mismatch');
        assert(!med1.is_completed, 'id mismatch');
        assert(med1.is_active, 'is active mismatch');
        assert(med1.duration_days == 30, 'duration days mismatch');
        assert(med1.prescribed_date == 0, 'prescribed mismatch');
        assert(med1.start_date == 0, 'start date mismatch');
        assert(med1.end_date == calculate_seconds_in_day(30), 'end date mismatch');
    }


    #[test]
    fn test_medications_complete() {
        let contract_address = setup();
        let med_1 = new_medication(
            0, // id
            0, // medical_record_id
            0, // pet_id
            "Med A", // name
            "10mg", // dosage
            "Once daily", // frequency
            10, // duration_days
            "Take after meals", // instructions
            0, // prescribed_date
            0, // start_date
            0 // created_at
        );

        let dispatcher = IMedicalRecordsDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();

        let pet_id: u256 = 1;
        let record_type = MedicalRecordType::Checkup;
        let title: ByteArray = "Initial Checkup";
        let description: ByteArray = "Routine health check.";
        let diagnosis: ByteArray = "Healthy";
        let treatment: ByteArray = "None";
        let medications: Array<Medication> = array![med_1];
        let next_visit_date: u64 = 1722455450;
        let is_emergency = false;
        let is_follow_up_required = false;
        let visit_cost = 250_u256;

        start_cheat_caller_address(contract_address, vet_address);
        let _record_id = dispatcher
            .create_medical_record(
                pet_id,
                record_type,
                title.clone(),
                description.clone(),
                diagnosis.clone(),
                treatment.clone(),
                medications.clone(),
                next_visit_date,
                is_emergency,
                is_follow_up_required,
                visit_cost,
            );
        stop_cheat_caller_address(vet_address);

        // Warp time forward by 12 days (half the duration)
        let time_passed = calculate_seconds_in_day(12);
        println!("Time to warp forward: {} seconds", time_passed);
        start_cheat_block_timestamp_global(time_passed);
        println!("New timestamp after warp: {}", get_block_timestamp());

        let med1 = dispatcher.get_medication(1);

        assert(!med1.is_active, 'is active mismatch');
        assert(med1.is_completed, 'completed error');
    }

    #[test]
    fn test_get_medical_record_medications() {
        let contract_address = setup();
        let med_1 = new_medication(
            0, // id
            0, // medical_record_id
            0, // pet_id
            "Med A", // name
            "10mg", // dosage
            "Once daily", // frequency
            10, // duration_days
            "Take after meals", // instructions
            0, // prescribed_date
            0, // start_date
            0 // created_at
        );

        let med_2 = new_medication(
            2,
            102,
            1002,
            "Med B",
            "5mg",
            "Twice daily",
            7,
            "Take before sleep",
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let med_3 = new_medication(
            0,
            0,
            0,
            "Med C",
            "20mg",
            "Once daily",
            14,
            "Avoid alcohol",
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );

        let dispatcher = IMedicalRecordsDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();

        let pet_id: u256 = 1;
        let record_type = MedicalRecordType::Checkup;
        let title: ByteArray = "Initial Checkup";
        let description: ByteArray = "Routine health check.";
        let diagnosis: ByteArray = "Healthy";
        let treatment: ByteArray = "None";
        let medications: Array<Medication> = array![med_1, med_2, med_3];
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
                medications.clone(),
                next_visit_date,
                is_emergency,
                is_follow_up_required,
                visit_cost,
            );
        stop_cheat_caller_address(vet_address);

        let med = dispatcher.get_medical_record_medications(record_id);
        assert(*med.at(0).id == 1, 'med1 id mismatch');
        assert(*med.at(1).id == 2, 'med2 id mismatch');
        assert(*med.at(2).id == 3, 'med3 id mismatch');
    }

    fn calculate_seconds_in_day(day: u64) -> u64 {
        day * 86400
    }

    #[test]
    fn test_pet_medication_count() {
        let contract_address = setup();
        let med_1 = new_medication(
            0, // id
            0, // medical_record_id
            0, // pet_id
            "Med A", // name
            "10mg", // dosage
            "Once daily", // frequency
            10, // duration_days
            "Take after meals", // instructions
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let med_2 = new_medication(
            2,
            102,
            1002,
            "Med B",
            "5mg",
            "Twice daily",
            7,
            "Take before sleep",
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let med_3 = new_medication(
            0,
            0,
            0,
            "Med C",
            "20mg",
            "Once daily",
            14,
            "Avoid alcohol",
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let med_4 = new_medication(
            0,
            0,
            0,
            "Med D",
            "15mg",
            "Three times daily",
            5,
            "Do not skip doses",
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let med_5 = new_medication(
            0,
            0,
            0,
            "Med E",
            "50mg",
            "Once daily",
            3,
            "Store in cool place",
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let dispatcher = IMedicalRecordsDispatcher { contract_address };

        let vet_address: ContractAddress = 12345.try_into().unwrap();

        let pet_id: u256 = 1;
        let record_type = MedicalRecordType::Checkup;
        let title: ByteArray = "Initial Checkup";
        let description: ByteArray = "Routine health check.";
        let diagnosis: ByteArray = "Healthy";
        let treatment: ByteArray = "None";
        let medications: Array<Medication> = array![med_1, med_2, med_3, med_4, med_5];
        let next_visit_date: u64 = 1722455450;
        let is_emergency = false;
        let is_follow_up_required = false;
        let visit_cost = 250_u256;

        start_cheat_caller_address(contract_address, vet_address);
        let _record_id = dispatcher
            .create_medical_record(
                pet_id,
                record_type,
                title.clone(),
                description.clone(),
                diagnosis.clone(),
                treatment.clone(),
                medications.clone(),
                next_visit_date,
                is_emergency,
                is_follow_up_required,
                visit_cost,
            );
        stop_cheat_caller_address(vet_address);

        let pet_medication1 = dispatcher.get_pet_active_medications(pet_id);
        assert(pet_medication1.len() == 5, 'pet_medication before error');

        // Warp time forward by 4 days (half the duration)
        let time_passed = calculate_seconds_in_day(4);
        start_cheat_block_timestamp_global(time_passed);

        let pet_medication = dispatcher.get_pet_active_medications(pet_id);
        assert(pet_medication.len() == 4, 'pet_medication error');

        assert(*pet_medication.at(0).id == 1, 'med1 id mismatch');
        assert(*pet_medication.at(1).id == 2, 'med2 id mismatch');
        assert(*pet_medication.at(2).id == 3, 'med3 id mismatch');
        assert(*pet_medication.at(3).id == 4, 'med4 id mismatch');

        assert(pet_medication.at(0).name.clone() == "Med A", 'name A id mismatch');
        assert(pet_medication.at(1).name.clone() == "Med B", 'name B id mismatch');
        assert(pet_medication.at(2).name.clone() == "Med C", 'name C id mismatch');
        assert(pet_medication.at(3).name.clone() == "Med D", 'name D id mismatch');
    }


    fn new_medication(
        id: u256,
        medical_record_id: u256,
        pet_id: u256,
        name: ByteArray,
        dosage: ByteArray,
        frequency: ByteArray,
        duration_days: u64,
        instructions: ByteArray,
        prescribed_date: u64,
        start_date: u64,
        created_at: u64,
    ) -> Medication {
        let seconds_per_day = 86400;
        let end_date = start_date + (duration_days * seconds_per_day);

        Medication {
            id,
            medical_record_id,
            pet_id,
            name,
            dosage,
            frequency,
            duration_days,
            instructions,
            prescribed_date,
            start_date,
            end_date,
            is_completed: false,
            is_active: true,
            created_at,
        }
    }
    fn medications() -> Array<Medication> {
        let med_1 = new_medication(
            0, // id
            0, // medical_record_id
            0, // pet_id
            "Med A", // name
            "10mg", // dosage
            "Once daily", // frequency
            10, // duration_days
            "Take after meals", // instructions
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let med_2 = new_medication(
            2,
            102,
            1002,
            "Med B",
            "5mg",
            "Twice daily",
            7,
            "Take before sleep",
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let med_3 = new_medication(
            0,
            0,
            0,
            "Med C",
            "20mg",
            "Once daily",
            14,
            "Avoid alcohol",
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let med_4 = new_medication(
            0,
            0,
            0,
            "Med D",
            "15mg",
            "Three times daily",
            5,
            "Do not skip doses",
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let med_5 = new_medication(
            0,
            0,
            0,
            "Med E",
            "50mg",
            "Once weekly",
            30,
            "Store in cool place",
            calculate_seconds_in_day(0), // prescribed_date
            calculate_seconds_in_day(0), // start_date
            calculate_seconds_in_day(0) // created_at
        );
        let medications: Array<Medication> = array![med_1, med_2, med_3, med_4, med_5];

        medications
    }
}
