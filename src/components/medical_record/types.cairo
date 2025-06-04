use starknet::ContractAddress;

#[derive(Drop, Serde, starknet::Store)]
pub struct MedicalRecord {
    pub id: u256,
    pub pet_id: u256,
    pub vet_address: ContractAddress,
    pub record_type: felt252, // MedicalRecordType
    pub title: ByteArray,
    pub description: ByteArray,
    pub diagnosis: ByteArray,
    pub treatment: ByteArray,
    pub next_visit_date: u64, // timestamp
    pub is_emergency: bool,
    pub is_follow_up_required: bool,
    pub visit_cost: u256,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Drop, Serde, starknet::Store)]
pub enum MedicalRecordType {
    Checkup,
    Vaccination,
    Surgery,
    Emergency,
    Dental,
    Laboratory,
    Prescription,
    FollowUp,
}

#[derive(Drop, Serde, starknet::Store)]
pub struct Vaccination {
    pub id: u256,
    pub pet_id: u256,
    pub vet_address: ContractAddress,
    pub vaccine_name: ByteArray,
    pub vaccine_type: VaccineType,
    pub batch_number: ByteArray,
    pub manufacturer: ByteArray,
    pub administered_date: u64,
    pub next_due_date: u64,
    pub side_effects: ByteArray,
    pub is_booster: bool,
    pub created_at: u64,
}

#[derive(Drop, Serde, starknet::Store)]
pub enum VaccineType {
    Rabies,
    DHPP, // Distemper, Hepatitis, Parvovirus, Parainfluenza
    Bordetella,
    Lyme,
    FeLV, // Feline Leukemia (for cats)
    FVRCP, // Feline Viral Rhinotracheitis, Calicivirus, Panleukopenia
    Other,
}

#[derive(Drop, Serde, starknet::Store)]
pub struct Medication {
    pub id: u256,
    pub medical_record_id: u256, // NEW: Link back to the medical record
    pub pet_id: u256, // NEW: Direct link to pet for easier querying
    pub name: ByteArray,
    pub dosage: ByteArray,
    pub frequency: ByteArray,
    pub duration_days: u32,
    pub instructions: ByteArray,
    pub prescribed_date: u64,
    pub start_date: u64,
    pub end_date: u64, // (start_date + duration)
    pub is_completed: bool,
    pub is_active: bool, // Is currently being taken
    pub created_at: u64,
}

#[derive(Drop, Serde, starknet::Store)]
pub struct LabResult {
    pub id: u256,
    pub pet_id: u256,
    pub vet_address: ContractAddress,
    pub medical_record_id: u256,
    pub test_type: ByteArray,
    pub test_name: ByteArray,
    pub results: ByteArray,
    pub reference_range: ByteArray,
    pub is_abnormal: bool,
    pub lab_notes: ByteArray,
    pub test_date: u64,
    pub created_at: u64,
}

#[derive(Drop, Serde, starknet::Store)]
pub struct EmergencyContact {
    pub name: ByteArray,
    pub phone: ByteArray,
    pub relationship: ByteArray, // e.g., "Owner", "Emergency Vet", "Family"
    pub is_primary: bool,
}
