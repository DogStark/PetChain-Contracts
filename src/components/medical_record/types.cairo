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
    #[default]
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
    #[default]
    Other,
    Rabies,
    DHPP, // Distemper, Hepatitis, Parvovirus, Parainfluenza
    Bordetella,
    Lyme,
    FeLV, // Feline Leukemia (for cats)
    FVRCP // Feline Viral Rhinotracheitis, Calicivirus, Panleukopenia
}

#[derive(Drop, Serde, Clone, starknet::Store)]
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

pub impl VaccineToFelt of Into<VaccineType, felt252> {
    fn into(self: VaccineType) -> felt252 {
        match self {
            VaccineType::Rabies => 'RABIES',
            VaccineType::DHPP => 'DHPP',
            VaccineType::Bordetella => 'BORDETELLA',
            VaccineType::Lyme => 'LYME',
            VaccineType::FeLV => 'FELV',
            VaccineType::FVRCP => 'FVRCP',
            VaccineType::Other => 'OTHER',
        }
    }
}

pub impl FeltToVaccine of TryInto<felt252, VaccineType> {
    fn try_into(self: felt252) -> Option<VaccineType> {
        if self == 'RABIES' {
            Option::Some(VaccineType::Rabies)
        } else if self == 'DHPP' {
            Option::Some(VaccineType::DHPP)
        } else if self == 'BORDETELLA' {
            Option::Some(VaccineType::Bordetella)
        } else if self == 'LYME' {
            Option::Some(VaccineType::Lyme)
        } else if self == 'FELV' {
            Option::Some(VaccineType::FeLV)
        } else if self == 'FVRCP' {
            Option::Some(VaccineType::FVRCP)
        } else if self == 'OTHER' {
            Option::Some(VaccineType::Other)
        } else {
            Option::None
        }
    }
}

pub impl MedicalRecordToFelt of Into<MedicalRecordType, felt252> {
    fn into(self: MedicalRecordType) -> felt252 {
        match self {
            MedicalRecordType::Checkup => 'CHECKUP',
            MedicalRecordType::Vaccination => 'VACCINATION',
            MedicalRecordType::Surgery => 'SURGERY',
            MedicalRecordType::Emergency => 'EMERGENCY',
            MedicalRecordType::Dental => 'DENTAL',
            MedicalRecordType::Laboratory => 'LABORATORY',
            MedicalRecordType::Prescription => 'PRESCRIPTION',
            MedicalRecordType::FollowUp => 'FOLLOWUP',
        }
    }
}
pub impl FelttoRecordType of TryInto<felt252, MedicalRecordType> {
    fn try_into(self: felt252) -> Option<MedicalRecordType> {
        if self == 'CHECKUP' {
            Option::Some(MedicalRecordType::Checkup)
        } else if self == 'VACCINATION' {
            Option::Some(MedicalRecordType::Vaccination)
        } else if self == 'SURGERY' {
            Option::Some(MedicalRecordType::Surgery)
        } else if self == 'EMERGENCY' {
            Option::Some(MedicalRecordType::Emergency)
        } else if self == 'DENTAL' {
            Option::Some(MedicalRecordType::Dental)
        } else if self == 'LABORATORY' {
            Option::Some(MedicalRecordType::Laboratory)
        } else if self == 'PRESCRIPTION' {
            Option::Some(MedicalRecordType::Prescription)
        } else if self == 'FOLLOWUP' {
            Option::Some(MedicalRecordType::FollowUp)
        } else {
            Option::None
        }
    }
}
