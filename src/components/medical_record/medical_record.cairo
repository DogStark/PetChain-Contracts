#[starknet::component]
pub mod MedicalRecordsComponent {
    use petchain::components::medical_record::interface::IMedicalRecords;
    use petchain::components::medical_record::types::{
        MedicalRecord, Vaccination, LabResult, Medication, MedicalRecordType, VaccineType,
        VaccineToFelt, FeltToVaccine, MedicalRecordToFelt, FelttoRecordType,
    };
    use starknet::{ContractAddress, get_block_timestamp, get_caller_address};
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess, Map};

    #[storage]
    pub struct Storage {
        // Medical Records
        medical_records: Map<u256, MedicalRecord>,
        pet_medical_records: Map<(u256, u256), u256>, // (pet_id, index) -> record_id
        pet_record_count: Map<u256, u256>,
        medical_record_count: u256,
        // Medications Storage
        medications: Map<u256, Medication>, // medication_id -> Medication
        medical_record_medications: Map<
            (u256, u256), u256,
        >, // (medical_record_id, index) -> medication_id
        medical_record_medication_count: Map<u256, u256>, // medical_record_id -> medication_count
        medication_count: u256, // Global medication counter
        // Vaccinations
        vaccinations: Map<u256, Vaccination>,
        pet_vaccinations: Map<(u256, u256), u256>, // (pet_id, index) -> vaccination_id
        pet_vaccination_count: Map<u256, u256>,
        vaccination_count: u256,
        // Lab Results
        lab_results: Map<u256, LabResult>,
        pet_lab_results: Map<(u256, u256), u256>, // (pet_id, index) -> lab_result_id
        pet_lab_count: Map<u256, u256>,
        lab_result_count: u256,
        // Vet Records Tracking
        vet_records: Map<(ContractAddress, u256), u256>, // (vet_address, index) -> record_id
        vet_record_count: Map<ContractAddress, u256>,
        // Emergency and Follow-up Tracking
        emergency_count: u256,
        emergency_records: Map<(u256, u256), u256>, // (pet_id, index) -> record_id
        emergency_record_count: Map<u256, u256>,
        follow_up_required: Map<u256, bool>, // record_id -> needs_follow_up
        pet_medications: Map<(u256, u256), u256>, // (pet_id, index) -> medication_id 
        pet_medication_count: Map<u256, u256>, // pet_id -> total_medication_count
        active_medications: Map<u256, bool> // medication_id -> is_active
    }

    #[embeddable_as(MedicalRecordsImpl)]
    impl MedicalRecordsComponentImpl<
        TContractState, +HasComponent<TContractState>,
    > of IMedicalRecords<ComponentState<TContractState>> {
        //TODO
        fn create_medical_record(
            ref self: ComponentState<TContractState>,
            pet_id: u256,
            record_type: MedicalRecordType,
            title: ByteArray,
            description: ByteArray,
            diagnosis: ByteArray,
            treatment: ByteArray,
            medications: Array<Medication>,
            next_visit_date: u64,
            is_emergency: bool,
            is_follow_up_required: bool,
            visit_cost: u256,
        ) -> u256 {
            // Increment global medical record counter
            let record_id = self.medical_record_count.read() + 1_u256;

            // Get current timestamp and caller address
            let timestamp = get_block_timestamp();
            let vet_address = get_caller_address();

            // Convert enum to felt252
            let record_type_felt: felt252 = MedicalRecordToFelt::into(record_type);

            // Build and store medical record
            let record = MedicalRecord {
                id: record_id,
                pet_id,
                vet_address,
                record_type: record_type_felt,
                title: title.clone(),
                description: description.clone(),
                diagnosis: diagnosis.clone(),
                treatment: treatment.clone(),
                next_visit_date,
                is_emergency,
                is_follow_up_required,
                visit_cost,
                created_at: timestamp,
                updated_at: timestamp,
            };

            if is_emergency {
                let new_count = self.emergency_count.read() + 1;
                self.emergency_count.write(new_count);

                let pet_emergencies_count = self.emergency_record_count.read(pet_id) + 1;
                self.emergency_record_count.write(pet_id, pet_emergencies_count);
            }

            if is_follow_up_required {
                self.follow_up_required.write(record_id, true);
            }

            self.medical_records.write(record_id, record);

            // Store medications
            let total_meds = medications.len();

            let mut i = 0;
            loop {
                if i == total_meds {
                    break;
                }

                let medi = medications.at(i);
                let medication_id = self.medication_count.read() + 1;
                let cur_pet_med_count = self.pet_medication_count.read(pet_id) + 1;

                let medication = Medication {
                    id: medication_id,
                    medical_record_id: record_id,
                    pet_id: pet_id,
                    name: medi.name.clone(),
                    dosage: medi.dosage.clone(),
                    frequency: medi.frequency.clone(),
                    duration_days: *medi.duration_days,
                    instructions: medi.instructions.clone(),
                    prescribed_date: *medi.prescribed_date,
                    start_date: *medi.start_date,
                    end_date: *medi.start_date + (*medi.duration_days * 86400),
                    is_completed: *medi.is_completed,
                    is_active: *medi.is_active,
                    created_at: *medi.created_at,
                };

                self.medication_count.write(medication_id);
                self.medications.write(medication_id, medication);

                self.pet_medications.write((pet_id, cur_pet_med_count), medication_id);
                self.pet_medication_count.write(pet_id, cur_pet_med_count);
                self.medical_record_medications.write((record_id, i.into()), medication_id);

                i += 1;
            };

            // Record how many medications are linked to this medical record
            self.medical_record_medication_count.write(record_id, total_meds.into());

            // Store the medical record index for the pet
            let pet_record_index = self.pet_record_count.read(pet_id);
            self.pet_medical_records.write((pet_id, pet_record_index.into()), record_id);

            // Increment pet's total medical records
            let pet_new_count = pet_record_index + 1;
            self.pet_record_count.write(pet_id, pet_new_count);

            // Update global medical record count
            self.medical_record_count.write(record_id);

            record_id
        }


        // TODO
        fn record_vaccination(
            ref self: ComponentState<TContractState>,
            pet_id: u256,
            vaccine_name: ByteArray,
            vaccine_type: VaccineType,
            batch_number: ByteArray,
            manufacturer: ByteArray,
            administered_date: u64,
            next_due_date: u64,
            side_effects: ByteArray,
            is_booster: bool,
        ) -> u256 {
            0
        }

        fn get_medical_record(
            self: @ComponentState<TContractState>, record_id: u256,
        ) -> MedicalRecord {
            self.medical_records.read(record_id)
        }

        // TODO
        fn get_pet_medical_history(
            self: @ComponentState<TContractState>, pet_id: u256,
        ) -> Span<MedicalRecord> {
            let total_records = self.pet_record_count.read(pet_id);

            if total_records == 0 {
                return array![].span();
            }

            let mut records: Array<MedicalRecord> = array![];

            let mut i = 0;
            loop {
                if i == total_records {
                    break;
                };

                let record_id = self.pet_medical_records.read((pet_id, i.into()));
                let record = self.medical_records.read(record_id);

                records.append(record);

                i += 1;
            };

            records.span()
        }

        // TODO
        fn get_pet_vaccinations(
            self: @ComponentState<TContractState>, pet_id: u256,
        ) -> Span<Vaccination> {
            array![].span()
        }

        // TODO
        fn get_upcoming_vaccinations(
            self: @ComponentState<TContractState>, pet_id: u256,
        ) -> Span<Vaccination> {
            array![].span()
        }

        // TODO
        fn get_emergency_records(
            self: @ComponentState<TContractState>, pet_id: u256,
        ) -> Span<MedicalRecord> {
            array![].span()
        }

        fn get_emergency_count(self: @ComponentState<TContractState>) -> u256 {
            self.emergency_count.read()
        }

        fn get_is_follow_up_required(
            self: @ComponentState<TContractState>, record_id: u256,
        ) -> bool {
            let require_follow_up = self.follow_up_required.read(record_id);
            require_follow_up
        }

        // TODO
        fn is_authorized_to_view(
            self: @ComponentState<TContractState>, pet_id: u256, viewer: ContractAddress,
        ) -> bool {
            // TODO: Implement proper access control
            // Pet owners can view their pet's records
            // Vets can view records they created or for emergency access
            // Emergency vets can view in emergencies
            true
        }

        //TODO
        fn is_authorized_to_edit(
            self: @ComponentState<TContractState>, pet_id: u256, editor: ContractAddress,
        ) -> bool {
            // TODO: Implement proper access control
            // Only the vet who created the record can edit it
            // Or authorized emergency vets in critical situations
            true
        }

        // TODO
        fn update_medical_record(
            ref self: ComponentState<TContractState>,
            record_id: u256,
            title: ByteArray,
            description: ByteArray,
            diagnosis: ByteArray,
            treatment: ByteArray,
            medications: Span<Medication>,
            next_visit_date: u64,
            is_follow_up_required: bool,
        ) -> bool {
            true
        }

        // TODO
        fn add_follow_up_note(
            ref self: ComponentState<TContractState>, record_id: u256, follow_up_notes: ByteArray,
        ) -> bool {
            true
        }

        // TODO
        fn update_vaccination_record(
            ref self: ComponentState<TContractState>,
            vaccination_id: u256,
            side_effects: ByteArray,
            next_due_date: u64,
        ) -> bool {
            true
        }

        // TODO
        fn add_lab_result(
            ref self: ComponentState<TContractState>,
            pet_id: u256,
            test_type: ByteArray,
            test_name: ByteArray,
            results: ByteArray,
            reference_range: ByteArray,
            is_abnormal: bool,
            lab_notes: ByteArray,
            test_date: u64,
        ) -> u256 {
            0
        }

        // TODO
        fn get_pet_lab_results(
            self: @ComponentState<TContractState>, pet_id: u256,
        ) -> Span<LabResult> {
            array![].span()
        }

        // TODO
        fn get_medical_records_by_vet(
            self: @ComponentState<TContractState>, vet_address: ContractAddress,
        ) -> Span<MedicalRecord> {
            array![].span()
        }

        // TODO
        fn mark_as_emergency(ref self: ComponentState<TContractState>, record_id: u256) -> bool {
            true
        }


        // TODO
        fn get_critical_alerts(
            self: @ComponentState<TContractState>, pet_id: u256,
        ) -> Span<ByteArray> {
            array![].span()
        }

        // TODO
        fn get_medical_record_medications(
            self: @ComponentState<TContractState>, record_id: u256,
        ) -> Span<Medication> {
            let total_meds = self.medical_record_medication_count.read(record_id);

            if total_meds == 0 {
                return array![].span();
            }

            let mut medications: Array<Medication> = array![];

            let mut i = 0;
            loop {
                if i == total_meds {
                    break;
                }

                let medication_id = self.medical_record_medications.read((record_id, i.into()));
                let mut medication = self.medications.read(medication_id);
                let timestamp = get_block_timestamp();

                if (timestamp > medication.end_date) {
                    medication.is_active = false;
                    medication.is_completed = true;
                    // self.active_medications.write(medication_id, false);
                }

                // Ensure medication belongs to the correct medical record
                if medication.medical_record_id == record_id {
                    medications.append(medication);
                }

                i += 1;
            };

            medications.span()
        }


        // TODO
        fn get_pet_active_medications(
            ref self: ComponentState<TContractState>, pet_id: u256,
        ) -> Span<Medication> {
            let total_meds = self.pet_medication_count.read(pet_id);

            if total_meds == 0 {
                return array![].span();
            }

            let mut active_meds: Array<Medication> = array![];

            let mut i = 0;
            loop {
                if i == total_meds {
                    break;
                }

                let medication_id = self.pet_medications.read((pet_id, i.into() + 1));

                let mut medication = self.get_medication(medication_id);

                if (medication.is_active) {
                    active_meds.append(medication);
                }

                i += 1;
            };

            active_meds.span()
        }


        fn mark_medication_completed(
            ref self: ComponentState<TContractState>, medication_id: u256,
        ) -> bool {
            let timestamp = get_block_timestamp();

            // Read medication from storage
            let mut medication = self.get_medication(medication_id);

            // Validate that medication exists
            assert(medication.id == medication_id, 'medication not found');

            // Validate medication is not already completed
            assert(!medication.is_completed, 'medication already completed');

            // Mark as completed only if current time is past end_date
            if timestamp > medication.end_date {
                medication.is_active = false;
                medication.is_completed = true;
                medication.end_date = timestamp; // update to actual completion time
                self.active_medications.write(medication_id, false);
            } else {
                // Optionally: decide if you want to allow early completion
                medication.is_active = false;
                medication.is_completed = true;
                medication.end_date = timestamp; // mark as early completion
                self.active_medications.write(medication_id, false);
            }

            // Write updated medication back to storage
            self.medications.write(medication_id, medication);

            return true;
        }


        fn update_medication(
            ref self: ComponentState<TContractState>,
            medication_id: u256,
            name: ByteArray,
            dosage: ByteArray,
            frequency: ByteArray,
            duration_days: u64,
            instructions: ByteArray,
            start_date: u64,
            is_active: bool,
        ) -> bool {
            assert(duration_days > 0, 'duration must be greater than 0');

            let timestamp = get_block_timestamp();

            // Read medication from storage
            let mut medication = self.medications.read(medication_id);
            assert(medication.id == medication_id, 'medication not found');

            // Validate: medication must not be completed
            assert(!medication.is_completed, 'medication already completed');

            // Update mutable fields
            medication.name = name;
            medication.dosage = dosage;
            medication.frequency = frequency;
            medication.duration_days = duration_days;
            medication.instructions = instructions;
            medication.start_date = start_date;
            medication.end_date = start_date + (duration_days * 86400);

            // Check if new schedule already ended
            if timestamp >= medication.end_date {
                medication.is_completed = true;
                medication.is_active = false;
                self.active_medications.write(medication_id, false);
            } else {
                // If explicitly marked as inactive, assume it was manually completed
                if !is_active {
                    medication.is_completed = true;
                    medication.is_active = false;
                    medication.end_date = timestamp; // actual completion time
                    self.active_medications.write(medication_id, false);
                } else {
                    // Active medication update
                    if medication.is_active != is_active {
                        medication.is_active = is_active;
                        self.active_medications.write(medication_id, is_active);
                    }
                }
            }

            // Write back to storage
            self.medications.write(medication_id, medication);

            return true;
        }

        // FOR TEST PURPOSE
        fn get_total_medication_count(self: @ComponentState<TContractState>) -> u256 {
            let total = self.medication_count.read();

            total
        }
        fn get_medication(
            ref self: ComponentState<TContractState>, medication_id: u256,
        ) -> Medication {
            let mut medication = self.medications.read(medication_id);
            let timestamp = get_block_timestamp();

            if !medication.is_completed && timestamp > medication.end_date {
                medication.is_completed = true;
                medication.is_active = false;
                self.active_medications.write(medication_id, false);
                self.medications.write(medication_id, medication.clone());
            }

            medication
        }
    }
}
