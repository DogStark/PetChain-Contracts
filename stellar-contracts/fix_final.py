with open("src/lib.rs", "r") as f:
    text = f.read()

text = text.replace(
"""            specialization,
            specializations: Vec::new(&env),
            certifications: Vec::new(&env),
            verified: false,""",
"""            specialization,
            verified: false,""")

text = text.replace(
"""            .get::<DataKey, BytesN<32>>(&TagKey::PetTagId(pet_id))""",
"""            .get::<TagKey, BytesN<32>>(&TagKey::PetTagId(pet_id))""")

text = text.replace(
"""pub struct MedicalRecordInput {
    pub pet_id: u64,
    pub diagnosis: String,
    pub treatment: String,
    pub medications: String,
    pub notes: String,
}""",
"""pub struct MedicalRecordInput {
    pub pet_id: u64,
    pub diagnosis: String,
    pub treatment: String,
    pub medications: Vec<Medication>,
    pub notes: String,
}""")

insurance_key = """
#[contracttype]
pub enum InsuranceKey {
    Policy(u64),
}
"""

if "enum InsuranceKey" not in text:
    text = text.replace("#![no_std]", "#![no_std]\n" + insurance_key)

with open("src/lib.rs", "w") as f:
    f.write(text)

