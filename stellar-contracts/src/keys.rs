//! Canonical storage-key namespace table for the Soroban contracts.
//!
//! Every persistent key family is constructed here so that the namespace
//! (the leading discriminant byte) is defined in exactly one place. Adding a
//! new key family requires adding a variant to [`Namespace`] and a constructor
//! below; the `namespace_table_is_unique` test then forces an explicit fixture
//! for the new family.

use soroban_sdk::{contracttype, Address, Bytes, Env, Symbol};

/// Canonical namespace discriminants for every persistent key family.
///
/// The numeric value is the first byte of every key produced by the matching
/// constructor, which guarantees domains cannot collide even if their payloads
/// are byte-identical.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Namespace {
    Pet = 0x01,
    Record = 0x02,
    Consent = 0x03,
    Insurance = 0x04,
    Dispute = 0x05,
    Index = 0x06,
}

impl Namespace {
    /// The discriminant byte written as the leading byte of a key.
    pub fn discriminant(self) -> u8 {
        self as u8
    }

    /// Human-readable label used in test diagnostics.
    pub fn label(self) -> &'static str {
        match self {
            Namespace::Pet => "pet",
            Namespace::Record => "record",
            Namespace::Consent => "consent",
            Namespace::Insurance => "insurance",
            Namespace::Dispute => "dispute",
            Namespace::Index => "index",
        }
    }
}

/// Build a namespaced key: `[discriminant, payload...]`.
fn namespaced(env: &Env, ns: Namespace, payload: &Bytes) -> Bytes {
    let mut key = Bytes::new(env);
    key.push_back(ns.discriminant());
    key.append(payload);
    key
}

fn address_payload(env: &Env, address: &Address) -> Bytes {
    let mut payload = Bytes::new(env);
    payload.append(&address.clone().to_string().into_bytes());
    payload
}

fn symbol_payload(env: &Env, symbol: &Symbol) -> Bytes {
    let mut payload = Bytes::new(env);
    payload.append(&symbol.clone().to_string().into_bytes());
    payload
}

/// Pet domain key.
pub fn pet_key(env: &Env, pet_id: &Symbol) -> Bytes {
    namespaced(env, Namespace::Pet, &symbol_payload(env, pet_id))
}

/// Medical record domain key.
pub fn record_key(env: &Env, pet_id: &Symbol) -> Bytes {
    namespaced(env, Namespace::Record, &symbol_payload(env, pet_id))
}

/// Consent domain key.
pub fn consent_key(env: &Env, owner: &Address) -> Bytes {
    namespaced(env, Namespace::Consent, &address_payload(env, owner))
}

/// Insurance domain key.
pub fn insurance_key(env: &Env, pet_id: &Symbol) -> Bytes {
    namespaced(env, Namespace::Insurance, &symbol_payload(env, pet_id))
}

/// Dispute domain key.
pub fn dispute_key(env: &Env, dispute_id: &Symbol) -> Bytes {
    namespaced(env, Namespace::Dispute, &symbol_payload(env, dispute_id))
}

/// Index domain key.
pub fn index_key(env: &Env, name: &Symbol) -> Bytes {
    namespaced(env, Namespace::Index, &symbol_payload(env, name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, Symbol};

    /// Every key family paired with a representative constructor call.
    /// New families must be added here, which forces an explicit fixture.
    fn namespace_table(env: &Env) -> [(Namespace, Bytes); 6] {
        let pet = Symbol::new(env, "pet-1");
        let owner = Address::generate(env);
        [
            (Namespace::Pet, pet_key(env, &pet)),
            (Namespace::Record, record_key(env, &pet)),
            (Namespace::Consent, consent_key(env, &owner)),
            (Namespace::Insurance, insurance_key(env, &pet)),
            (Namespace::Dispute, dispute_key(env, &pet)),
            (Namespace::Index, index_key(env, &pet)),
        ]
    }

    #[test]
    fn namespace_table_is_unique() {
        let env = Env::default();
        let table = namespace_table(&env);
        for (i, (ns_a, key_a)) in table.iter().enumerate() {
            for (ns_b, key_b) in table.iter().skip(i + 1) {
                assert_ne!(
                    ns_a.discriminant(),
                    ns_b.discriminant(),
                    "namespace collision between {} and {}",
                    ns_a.label(),
                    ns_b.label()
                );
                assert_ne!(
                    key_a,
                    key_b,
                    "key collision between {} and {}",
                    ns_a.label(),
                    ns_b.label()
                );
            }
        }
    }

    #[test]
    fn domains_do_not_overwrite_each_other() {
        let env = Env::default();
        let pet = Symbol::new(&env, "pet-1");
        let owner = Address::generate(&env);

        // Write a representative value in every domain using the same payload.
        let pet_k = pet_key(&env, &pet);
        let record_k = record_key(&env, &pet);
        let consent_k = consent_key(&env, &owner);
        let insurance_k = insurance_key(&env, &pet);
        let dispute_k = dispute_key(&env, &pet);
        let index_k = index_key(&env, &pet);

        env.storage().persistent().set(&pet_k, &1u32);
        env.storage().persistent().set(&record_k, &2u32);
        env.storage().persistent().set(&consent_k, &3u32);
        env.storage().persistent().set(&insurance_k, &4u32);
        env.storage().persistent().set(&dispute_k, &5u32);
        env.storage().persistent().set(&index_k, &6u32);

        // Read each back and confirm no domain overwrote another.
        assert_eq!(env.storage().persistent().get::<Bytes, u32>(&pet_k), Some(1));
        assert_eq!(env.storage().persistent().get::<Bytes, u32>(&record_k), Some(2));
        assert_eq!(env.storage().persistent().get::<Bytes, u32>(&consent_k), Some(3));
        assert_eq!(env.storage().persistent().get::<Bytes, u32>(&insurance_k), Some(4));
        assert_eq!(env.storage().persistent().get::<Bytes, u32>(&dispute_k), Some(5));
        assert_eq!(env.storage().persistent().get::<Bytes, u32>(&index_k), Some(6));
    }
}
