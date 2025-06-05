use starknet::ContractAddress;
use petchain::components::medical_record::types::{
    MedicalRecord, Vaccination, LabResult, Medication, MedicalRecordType, VaccineType,
};

#[starknet::interface]
pub trait IMedicalRecords<TContractState> {
    // Medical Records Management
    fn create_medical_record(
        ref self: TContractState,
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
    ) -> u256;

    fn update_medical_record(
        ref self: TContractState,
        record_id: u256,
        title: ByteArray,
        description: ByteArray,
        diagnosis: ByteArray,
        treatment: ByteArray,
        medications: Span<Medication>,
        next_visit_date: u64,
        is_follow_up_required: bool,
    ) -> bool;

    fn add_follow_up_note(
        ref self: TContractState, record_id: u256, follow_up_notes: ByteArray,
    ) -> bool;

    // Vaccination Management
    fn record_vaccination(
        ref self: TContractState,
        pet_id: u256,
        vaccine_name: ByteArray,
        vaccine_type: VaccineType,
        batch_number: ByteArray,
        manufacturer: ByteArray,
        administered_date: u64,
        next_due_date: u64,
        side_effects: ByteArray,
        is_booster: bool,
    ) -> u256;

    fn update_vaccination_record(
        ref self: TContractState, vaccination_id: u256, side_effects: ByteArray, next_due_date: u64,
    ) -> bool;

    // Lab Results Management
    fn add_lab_result(
        ref self: TContractState,
        pet_id: u256,
        test_type: ByteArray,
        test_name: ByteArray,
        results: ByteArray,
        reference_range: ByteArray,
        is_abnormal: bool,
        lab_notes: ByteArray,
        test_date: u64,
    ) -> u256;

    // Query Functions
    fn get_medical_record(self: @TContractState, record_id: u256) -> MedicalRecord;
    fn get_pet_medical_history(self: @TContractState, pet_id: u256) -> Span<MedicalRecord>;
    fn get_pet_vaccinations(self: @TContractState, pet_id: u256) -> Span<Vaccination>;
    fn get_pet_lab_results(self: @TContractState, pet_id: u256) -> Span<LabResult>;
    fn get_upcoming_vaccinations(self: @TContractState, pet_id: u256) -> Span<Vaccination>;
    fn get_medical_records_by_vet(
        self: @TContractState, vet_address: ContractAddress,
    ) -> Span<MedicalRecord>;
    fn get_medical_record_medications(self: @TContractState, record_id: u256) -> Span<Medication>;
    fn get_pet_active_medications(self: @TContractState, pet_id: u256) -> Span<Medication>;

    // Emergency and Critical Functions
    fn mark_as_emergency(ref self: TContractState, record_id: u256) -> bool;
    fn get_emergency_records(self: @TContractState, pet_id: u256) -> Span<MedicalRecord>;
    fn get_emergency_count(self: @TContractState) -> u256;
    fn get_critical_alerts(self: @TContractState, pet_id: u256) -> Span<ByteArray>;

    // Access Control Helpers
    fn is_authorized_to_view(self: @TContractState, pet_id: u256, viewer: ContractAddress) -> bool;
    fn is_authorized_to_edit(self: @TContractState, pet_id: u256, editor: ContractAddress) -> bool;

    // Follow up
    fn get_is_follow_up_required(self: @TContractState, record_id: u256) -> bool;
    fn mark_medication_completed(ref self: TContractState, medication_id: u256) -> bool;
    fn update_medication(
        ref self: TContractState,
        medication_id: u256,
        name: ByteArray,
        dosage: ByteArray,
        frequency: ByteArray,
        duration_days: u32,
        instructions: ByteArray,
        start_date: u64,
        is_active: bool,
    ) -> bool;
}
