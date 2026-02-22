import re

with open("src/test.rs", "r") as f:
    text = f.read()

# Fix register_vet missing `name` argument
text = text.replace(
"""    client.register_vet(
        &unverified_vet,
        &String::from_str(&env, "FAKE-001"),
        &String::from_str(&env, "General"),
    );""",
"""    client.register_vet(
        &unverified_vet,
        &String::from_str(&env, "Dr. Fake"),
        &String::from_str(&env, "FAKE-001"),
        &String::from_str(&env, "General"),
    );""")

# Fix register_pet_owner missing args 
text = text.replace(
"""    client.register_pet_owner(&owner, &String::from_str(&env, "Alice"));""",
"""    client.register_pet_owner(&owner, &String::from_str(&env, "Alice"), &String::from_str(&env, "alice@alice.com"), &String::from_str(&env, "Bob 555-1234"));""")
text = text.replace(
"""    client.register_pet_owner(&owner, &String::from_str(&env, "Bob"));""",
"""    client.register_pet_owner(&owner, &String::from_str(&env, "Bob"), &String::from_str(&env, "bob@bob.com"), &String::from_str(&env, "Alice 555-1234"));""")

# Fix register_pet missing args
# Original register_pet call from test:
#     let pet_id = client.register_pet(
#         &owner,
#         &String::from_str(&env, "Nala"),
#         &Species::Dog,
#         &Gender::Female,
#         &0u64,
#         &String::from_str(&env, "Poodle"),
#     );
# Let's replace with regex since there are multiple of these
def pet_repl(m):
    return f"""let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "{m.group(1)}"),
        &String::from_str(&env, "2020-01-01"),
        &Species::{m.group(2)},
        &Gender::{m.group(3)},
        &0u64,
        &String::from_str(&env, "{m.group(4)}"),
        &String::from_str(&env, "Brown"),
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

