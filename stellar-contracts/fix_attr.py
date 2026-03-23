with open("src/lib.rs", "r") as f:
    text = f.read()

text = text.replace(
"""#![no_std]

#[contracttype]
pub enum InsuranceKey {
    Policy(u64),
}

#![allow(clippy::too_many_arguments)]""",
"""#![no_std]
#![allow(clippy::too_many_arguments)]

#[contracttype]
pub enum InsuranceKey {
    Policy(u64),
}
""")

with open("src/lib.rs", "w") as f:
    f.write(text)

