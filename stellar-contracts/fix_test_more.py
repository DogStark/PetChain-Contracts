import re
with open("src/test.rs", "r") as f:
    text = f.read()

# Remove client.initialize(&admin); Since it is non-existent
text = re.sub(r'\s*client\.initialize\(&admin\);', '', text)

# Fix register_vet missing `name` argument
text = text.replace(
"""    client.register_vet(&vet, &String::from_str(&env, "Dr. Kim"), &String::from_str(&env, "LIC-005"));""",
"""    client.register_vet(&vet, &String::from_str(&env, "Dr. Kim"), &String::from_str(&env, "LIC-005"), &String::from_str(&env, "General"));""")

# Fix old register_pet calls
def pet_repl(m):
    return f"""let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "{m.group(1)}"),
        &String::from_str(&env, "2020-01-01"),
        &Species::{m.group(3)},
        &Gender::{m.group(2)},
        &String::from_str(&env, "Brown"),
        &String::from_str(&env, "{m.group(4)}"),
        &0u32,
        &None,
        &PrivacyLevel::Public,
    );"""

text = re.sub(
r'''let pet_id = client\.register_pet\(\s*&owner,\s*&String::from_str\(&env,\s*"([^"]+)"\),\s*&Species::([^,]+),\s*&Gender::([^,]+),\s*&0u64,\s*&String::from_str\(&env,\s*"([^"]+)"\),\s*\);''',
pet_repl, text
)

with open("src/test.rs", "w") as f:
    f.write(text)

