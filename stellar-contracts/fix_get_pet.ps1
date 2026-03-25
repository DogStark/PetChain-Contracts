$t = [System.IO.File]::ReadAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs')

$startIdx = 44126
$endIdx   = 44126 + 2214  # covers the full broken get_pet body up to the blank line before get_pet_age

$newGetPet = @'
pub fn get_pet(env: Env, id: u64) -> Option<PetProfile> {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))?;

        let key = Self::get_encryption_key(&env);

        // Propagate decryption failures as None rather than masking them
        // with a sentinel "Error" string. Any corrupt ciphertext or nonce
        // mismatch causes the whole read to return None deterministically.
        let decrypted_name = decrypt_sensitive_data(
            &env,
            &pet.encrypted_name.ciphertext,
            &pet.encrypted_name.nonce,
            &key,
        )
        .ok()?;
        let name = String::from_xdr(&env, &decrypted_name).ok()?;

        let decrypted_birthday = decrypt_sensitive_data(
            &env,
            &pet.encrypted_birthday.ciphertext,
            &pet.encrypted_birthday.nonce,
            &key,
        )
        .ok()?;
        let birthday = String::from_xdr(&env, &decrypted_birthday).ok()?;

        let decrypted_breed = decrypt_sensitive_data(
            &env,
            &pet.encrypted_breed.ciphertext,
            &pet.encrypted_breed.nonce,
            &key,
        )
        .ok()?;
        let breed = String::from_xdr(&env, &decrypted_breed).ok()?;

        let a_bytes = decrypt_sensitive_data(
            &env,
            &pet.encrypted_allergies.ciphertext,
            &pet.encrypted_allergies.nonce,
            &key,
        )
        .ok()?;
        let allergies = Vec::<Allergy>::from_xdr(&env, &a_bytes).ok()?;

        let profile = PetProfile {
            id: pet.id,
            owner: pet.owner.clone(),
            privacy_level: pet.privacy_level,
            name,
            birthday,
            active: pet.active,
            created_at: pet.created_at,
            updated_at: pet.updated_at,
            new_owner: pet.new_owner,
            species: pet.species,
            gender: pet.gender,
            breed,
            color: pet.color,
            weight: pet.weight,
            microchip_id: pet.microchip_id,
            allergies,
        };

        Self::log_access(
            &env,
            id,
            pet.owner,
            AccessAction::Read,
            String::from_str(&env, "Pet profile accessed"),
        );
        Some(profile)
    }

    
'@

$before = $t.Substring(0, $startIdx)
$after  = $t.Substring($endIdx)
$result = $before + $newGetPet + $after

[System.IO.File]::WriteAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs', $result)
Write-Host "Done. New length: $($result.Length)"
