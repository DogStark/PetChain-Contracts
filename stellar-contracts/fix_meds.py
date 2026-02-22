import re

with open("src/lib.rs", "r") as f:
    text = f.read()

# Fix MedicalRecord struct
text = text.replace(
"""pub struct MedicalRecord {
    pub id: u64,
    pub pet_id: u64,
    pub vet_address: Address,
    pub diagnosis: String,
    pub treatment: String,
    pub medications: String,
    pub date: u64,
    pub notes: String,
}""",
"""pub struct MedicalRecord {
    pub id: u64,
    pub pet_id: u64,
    pub vet_address: Address,
    pub diagnosis: String,
    pub treatment: String,
    pub medications: Vec<Medication>,
    pub date: u64,
    pub updated_at: u64,
    pub notes: String,
}""")

# Fix add_medical_record signature
text = text.replace(
"""        treatment: String,
        medications: String,
        notes: String,
    ) -> u64 {""",
"""        treatment: String,
        medications: Vec<Medication>,
        notes: String,
    ) -> u64 {""")

text = text.replace(
"""            medications,
            date: now,
            notes,""",
"""            medications,
            date: now,
            updated_at: now,
            notes,""")

# Fix update_medical_record signature
text = text.replace(
"""    pub fn update_medical_record(
        env: Env,
        record_id: u64,
        diagnosis: String,
        treatment: String,
        medications: String,
        notes: String,
    ) -> bool {""",
"""    pub fn update_medical_record(
        env: Env,
        record_id: u64,
        diagnosis: String,
        treatment: String,
        medications: Vec<Medication>,
        notes: String,
    ) -> bool {""")

# Fix veterinarian -> vet_address in update_medical_record log
text = text.replace(
"""            Self::log_access(
                &env,
                record.pet_id,
                record.veterinarian,
                AccessAction::Write,
                String::from_str(&env, "Medical record updated"),
            );""",
"""            Self::log_access(
                &env,
                record.pet_id,
                record.vet_address.clone(),
                AccessAction::Write,
                String::from_str(&env, "Medical record updated"),
            );""")

# Fix veterinarian -> vet_address in add_medical_record log
text = text.replace(
"""            MedicalRecordAddedEvent {
                pet_id,
                updated_by: veterinarian.clone(),
                timestamp: now,
            },""",
"""            MedicalRecordAddedEvent {
                pet_id,
                updated_by: vet_address.clone(),
                timestamp: now,
            },""")

text = text.replace(
"""        Self::log_access(
            &env,
            pet_id,
            veterinarian,
            AccessAction::Write,
            String::from_str(&env, "Medical record added"),
        );""",
"""        Self::log_access(
            &env,
            pet_id,
            vet_address.clone(),
            AccessAction::Write,
            String::from_str(&env, "Medical record added"),
        );""")

# Fix mark_record_med_completed
text = text.replace(
"""            record.veterinarian.require_auth();""",
"""            record.vet_address.require_auth();""")

with open("src/lib.rs", "w") as f:
    f.write(text)

