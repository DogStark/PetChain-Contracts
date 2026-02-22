import re

with open("src/lib.rs", "r") as f:
    text = f.read()

# Comment out ClinicInfo definition
text = text.replace(
"""#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClinicInfo {
    pub clinic_name: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub operating_hours: String,
    pub emergency_available: bool,
}""",
"""/*
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClinicInfo {
    pub clinic_name: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub operating_hours: String,
    pub emergency_available: bool,
}
*/""")


# Comment out clinic_info from Vet
text = text.replace(
"""    pub verified: bool,
    pub clinic_info: Option<ClinicInfo>,
}""",
"""    pub verified: bool,
    // pub clinic_info: Option<ClinicInfo>,
}""")

# Comment out update_clinic_info function
text = re.sub(r'pub fn update_clinic_info.*?\} \else \{.*?false.*?\n\s{4}\}', '/* ClinicInfo bypassed\npub fn update_clinic_info */', text, flags=re.DOTALL)
text = re.sub(r'pub fn update_clinic_info.*?\n\s{4}\}', '/* ClinicInfo bypassed\npub fn update_clinic_info */', text, flags=re.DOTALL)

with open("src/lib.rs", "w") as f:
    f.write(text)

