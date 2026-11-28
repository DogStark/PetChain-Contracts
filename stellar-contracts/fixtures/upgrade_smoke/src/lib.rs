#![no_std]

// Upgrade smoke fixture (v1 schema). Seeds one representative persistent
// value per domain (pet, record, consent, custody, vet). Build with
// `--features v2` for the upgraded build: the storage keys and value types
// must stay identical, so `read_all` must still return true after upgrade.

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

#[contracttype]
pub enum DataKey {
    Pet(u64),
    Record(u64),
    Consent(u64),
    Custody(u64),
    Vet(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pet {
    pub id: u64,
    pub name: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    pub pet_id: u64,
    pub kind: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Consent {
    pub granted: bool,
    pub version: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Custody {
    pub pet_id: u64,
    pub seq: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vet {
    pub license: String,
    pub active: bool,
}

fn pet(env: &Env) -> Pet {
    Pet {
        id: 1,
        name: String::from_str(env, "Rex"),
    }
}

fn record() -> Record {
    Record { pet_id: 1, kind: 1 }
}

fn consent() -> Consent {
    Consent {
        granted: true,
        version: 1,
    }
}

fn custody() -> Custody {
    Custody { pet_id: 1, seq: 1 }
}

fn vet(env: &Env) -> Vet {
    Vet {
        license: String::from_str(env, "VET-0001"),
        active: true,
    }
}

#[contract]
pub struct UpgradeSmoke;

#[contractimpl]
impl UpgradeSmoke {
    pub fn schema_version(_env: Env) -> u32 {
        if cfg!(feature = "v2") {
            2
        } else {
            1
        }
    }

    pub fn seed(env: Env, vet_addr: Address) {
        let s = env.storage().persistent();
        s.set(&DataKey::Pet(1), &pet(&env));
        s.set(&DataKey::Record(1), &record());
        s.set(&DataKey::Consent(1), &consent());
        s.set(&DataKey::Custody(1), &custody());
        s.set(&DataKey::Vet(vet_addr), &vet(&env));
    }

    /// Returns true only if every seeded key still deserializes to its seeded value.
    pub fn read_all(env: Env, vet_addr: Address) -> bool {
        let s = env.storage().persistent();
        s.get::<_, Pet>(&DataKey::Pet(1)) == Some(pet(&env))
            && s.get::<_, Record>(&DataKey::Record(1)) == Some(record())
            && s.get::<_, Consent>(&DataKey::Consent(1)) == Some(consent())
            && s.get::<_, Custody>(&DataKey::Custody(1)) == Some(custody())
            && s.get::<_, Vet>(&DataKey::Vet(vet_addr)) == Some(vet(&env))
    }
}
