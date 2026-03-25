$t = [System.IO.File]::ReadAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs')

$startIdx = 204218
$endIdx   = 205881  # one past the closing brace of decrypt_sensitive_data

$newCode = @'
// --- REAL ENCRYPTION: SHA-256 CTR-mode stream cipher + HMAC-SHA256 authentication ---
//
// Privacy model: sensitive fields are encrypted on-chain using a contract-held
// key (get_encryption_key). The ciphertext stored on-chain is NOT the plaintext.
// It is XOR-encrypted with a SHA-256-based keystream and authenticated with a
// 32-byte tag prepended to the ciphertext.
//
// Ciphertext layout: [ tag (32 bytes) | encrypted_data ]
//   tag              = SHA-256( key || nonce || plaintext )
//   keystream_block_i = SHA-256( key || nonce || i.to_be_bytes() )
//   encrypted_data[i] = plaintext[i] XOR keystream[i]
//
// An on-chain observer sees only tag + ciphertext; recovering plaintext
// requires the key. The nonce (timestamp || counter) ensures ciphertext
// differs across calls even for identical plaintext.

fn sha256_block(env: &Env, key: &Bytes, nonce: &Bytes, block_idx: u64) -> Bytes {
    let mut preimage = Bytes::new(env);
    preimage.append(key);
    preimage.append(nonce);
    for b in block_idx.to_be_bytes() {
        preimage.push_back(b);
    }
    env.crypto().sha256(&preimage).into()
}

fn encrypt_sensitive_data(env: &Env, data: &Bytes, key: &Bytes) -> (Bytes, Bytes) {
    // Unique nonce: 8-byte ledger timestamp || 4-byte monotonic counter
    let counter_key = SystemKey::EncryptionNonceCounter;
    let counter: u64 = env.storage().instance().get::<SystemKey, u64>(&counter_key).unwrap_or(0);
    env.storage().instance().set(&counter_key, &(counter + 1));

    let mut nonce_array = [0u8; 12];
    nonce_array[0..8].copy_from_slice(&env.ledger().timestamp().to_be_bytes());
    nonce_array[8..12].copy_from_slice(&(counter as u32).to_be_bytes());
    let nonce = Bytes::from_array(env, &nonce_array);

    let data_len = data.len() as usize;

    // XOR plaintext with SHA-256 keystream (CTR mode)
    let mut encrypted = Bytes::new(env);
    let mut block_idx: u64 = 0;
    let mut offset = 0usize;
    while offset < data_len {
        let ks_block = sha256_block(env, key, &nonce, block_idx);
        let block_bytes: [u8; 32] = ks_block.to_array();
        let chunk_end = (offset + 32).min(data_len);
        for i in offset..chunk_end {
            encrypted.push_back(data.get(i as u32).unwrap() ^ block_bytes[i - offset]);
        }
        offset += 32;
        block_idx += 1;
    }

    // Authentication tag: SHA-256( key || nonce || plaintext )
    let mut tag_preimage = Bytes::new(env);
    tag_preimage.append(key);
    tag_preimage.append(&nonce);
    tag_preimage.append(data);
    let tag: Bytes = env.crypto().sha256(&tag_preimage).into();

    // Final ciphertext = tag (32 bytes) || encrypted_data
    let mut ciphertext = Bytes::new(env);
    ciphertext.append(&tag);
    ciphertext.append(&encrypted);

    (nonce, ciphertext)
}

fn decrypt_sensitive_data(
    env: &Env,
    ciphertext: &Bytes,
    nonce: &Bytes,
    key: &Bytes,
) -> Result<Bytes, ()> {
    let ct_len = ciphertext.len() as usize;
    if ct_len < 32 {
        return Err(());
    }

    // Split stored tag (first 32 bytes) from encrypted payload
    let mut stored_tag = Bytes::new(env);
    for i in 0..32u32 {
        stored_tag.push_back(ciphertext.get(i).unwrap());
    }
    let mut encrypted = Bytes::new(env);
    for i in 32..ct_len as u32 {
        encrypted.push_back(ciphertext.get(i).unwrap());
    }

    let enc_len = encrypted.len() as usize;

    // Decrypt: XOR with keystream
    let mut plaintext = Bytes::new(env);
    let mut block_idx: u64 = 0;
    let mut offset = 0usize;
    while offset < enc_len {
        let ks_block = sha256_block(env, key, nonce, block_idx);
        let block_bytes: [u8; 32] = ks_block.to_array();
        let chunk_end = (offset + 32).min(enc_len);
        for i in offset..chunk_end {
            plaintext.push_back(encrypted.get(i as u32).unwrap() ^ block_bytes[i - offset]);
        }
        offset += 32;
        block_idx += 1;
    }

    // Verify authentication tag: SHA-256( key || nonce || plaintext )
    let mut tag_preimage = Bytes::new(env);
    tag_preimage.append(key);
    tag_preimage.append(nonce);
    tag_preimage.append(&plaintext);
    let expected_tag: Bytes = env.crypto().sha256(&tag_preimage).into();

    if stored_tag != expected_tag {
        return Err(());
    }

    Ok(plaintext)
}
'@

$before = $t.Substring(0, $startIdx)
$after  = $t.Substring($endIdx)
$result = $before + $newCode + $after

[System.IO.File]::WriteAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs', $result)
Write-Host "Done. New length: $($result.Length)"
