#[cfg(test)]
mod tests {
    use crate::{
        Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species,
    };
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    fn setup() -> (Env, PetChainContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &id);
        (env, client)
    }

    fn register_pet(client: &PetChainContractClient, env: &Env, owner: &Address) -> u64 {
        client.register_pet(
            owner,
            &String::from_str(env, "Buddy"),
            &String::from_str(env, "1609459200"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(env, "Labrador"),
            &String::from_str(env, "Brown"),
            &25u32,
            &None,
            &PrivacyLevel::Public,
        )
    }

    /// Ciphertext stored on-chain must differ from the plaintext name.
    /// With the SHA-256 CTR cipher the stored bytes are tag||XOR(plaintext),
    /// which is never equal to the raw XDR of the name string.
    #[test]
    fn test_ciphertext_differs_from_plaintext() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);

        // Read the raw Pet from storage and compare ciphertext to the
        // XDR-encoded plaintext name.
        let pet: crate::Pet = env
            .storage()
            .instance()
            .get(&crate::DataKey::Pet(pet_id))
            .unwrap();

        use soroban_sdk::xdr::ToXdr;
        let name_xdr = String::from_str(&env, "Buddy").to_xdr(&env);

        // The ciphertext must NOT equal the raw plaintext bytes.
        assert_ne!(
            pet.encrypted_name.ciphertext,
            name_xdr,
            "ciphertext must not equal plaintext"
        );
    }

    /// Two separate registrations of pets with the same name must produce
    /// different ciphertexts because each call uses a unique nonce
    /// (timestamp || monotonic counter).
    #[test]
    fn test_same_plaintext_different_ciphertexts() {
        let (env, client) = setup();
        let owner1 = Address::generate(&env);
        let owner2 = Address::generate(&env);

        let pet1 = register_pet(&client, &env, &owner1);
        let pet2 = register_pet(&client, &env, &owner2);

        let p1: crate::Pet = env
            .storage()
            .instance()
            .get(&crate::DataKey::Pet(pet1))
            .unwrap();
        let p2: crate::Pet = env
            .storage()
            .instance()
            .get(&crate::DataKey::Pet(pet2))
            .unwrap();

        // Same plaintext ("Buddy") but different nonces → different ciphertexts.
        assert_ne!(
            p1.encrypted_name.ciphertext,
            p2.encrypted_name.ciphertext,
            "same plaintext must produce different ciphertexts due to unique nonces"
        );
        assert_ne!(
            p1.encrypted_name.nonce,
            p2.encrypted_name.nonce,
            "nonces must be unique across calls"
        );
    }

    /// Nonce must be exactly 12 bytes (8-byte timestamp || 4-byte counter).
    #[test]
    fn test_nonce_is_12_bytes() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);

        let pet: crate::Pet = env
            .storage()
            .instance()
            .get(&crate::DataKey::Pet(pet_id))
            .unwrap();

        assert_eq!(
            pet.encrypted_name.nonce.len(),
            12,
            "nonce must be exactly 12 bytes"
        );
    }

    /// Ciphertext must be at least 33 bytes: 32-byte auth tag + ≥1 byte payload.
    #[test]
    fn test_ciphertext_has_auth_tag_prefix() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);

        let pet: crate::Pet = env
            .storage()
            .instance()
            .get(&crate::DataKey::Pet(pet_id))
            .unwrap();

        assert!(
            pet.encrypted_name.ciphertext.len() > 32,
            "ciphertext must contain 32-byte auth tag plus encrypted payload"
        );
    }

    /// Decryption round-trip: get_pet must return the original plaintext name.
    #[test]
    fn test_decryption_roundtrip() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);

        let profile = client.get_pet(&pet_id).unwrap();
        assert_eq!(
            profile.name,
            String::from_str(&env, "Buddy"),
            "decrypted name must match original plaintext"
        );
        assert_eq!(
            profile.breed,
            String::from_str(&env, "Labrador"),
            "decrypted breed must match original plaintext"
        );
    }

    /// Corrupt ciphertext (wrong auth tag) must cause get_pet to return None,
    /// not a partial profile — proving the auth tag is actually verified.
    #[test]
    fn test_tampered_ciphertext_rejected() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);

        // Flip the first byte of the auth tag to simulate tampering.
        let mut pet: crate::Pet = env
            .storage()
            .instance()
            .get(&crate::DataKey::Pet(pet_id))
            .unwrap();

        let first_byte = pet.encrypted_name.ciphertext.get(0).unwrap();
        let mut tampered = soroban_sdk::Bytes::new(&env);
        tampered.push_back(first_byte ^ 0xFF); // flip all bits of first byte
        for i in 1..pet.encrypted_name.ciphertext.len() {
            tampered.push_back(pet.encrypted_name.ciphertext.get(i).unwrap());
        }
        pet.encrypted_name.ciphertext = tampered;
        env.storage()
            .instance()
            .set(&crate::DataKey::Pet(pet_id), &pet);

        let result = client.get_pet(&pet_id);
        assert!(
            result.is_none(),
            "tampered ciphertext must be rejected — auth tag verification must fail"
        );
    }

    /// Counter increments: the nonce counter portion (last 4 bytes) must
    /// increase with each successive encryption call.
    #[test]
    fn test_nonce_counter_increments() {
        let (env, client) = setup();
        let owner1 = Address::generate(&env);
        let owner2 = Address::generate(&env);
        let owner3 = Address::generate(&env);

        let p1 = register_pet(&client, &env, &owner1);
        let p2 = register_pet(&client, &env, &owner2);
        let p3 = register_pet(&client, &env, &owner3);

        let get_counter = |pet_id: u64| -> u32 {
            let pet: crate::Pet = env
                .storage()
                .instance()
                .get(&crate::DataKey::Pet(pet_id))
                .unwrap();
            // Counter is in the last 4 bytes of the 12-byte nonce of the
            // *first* encrypted field (encrypted_name).
            // Each register_pet call encrypts 6 fields, so counters advance by 6.
            let n = pet.encrypted_name.nonce;
            let b0 = n.get(8).unwrap() as u32;
            let b1 = n.get(9).unwrap() as u32;
            let b2 = n.get(10).unwrap() as u32;
            let b3 = n.get(11).unwrap() as u32;
            (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
        };

        let c1 = get_counter(p1);
        let c2 = get_counter(p2);
        let c3 = get_counter(p3);

        assert!(c2 > c1, "counter must increase between registrations");
        assert!(c3 > c2, "counter must increase between registrations");
    }
}
