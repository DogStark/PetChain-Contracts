// Domain separation for every stored hash (Issue #1168).
//
// `PetChainContract::compute_domain_hash` is the canonical way to hash
// content bound for storage as a `BytesN<32>` (evidence, attachments,
// claim documents, certificates, medical records, tag IDs). Each domain
// gets its own versioned ASCII prefix -- `petchain:hash:<domain>:v1` --
// so the same raw content hashes to a different value in each domain,
// and can never be replayed across domains.
//
// The vectors below were computed independently with Python's hashlib
// (`sha256(tag + content)`) and are pinned here so any accidental change
// to the tag strings, the prefixing order, or the hash algorithm is
// caught immediately.
use crate::{HashDomain, PetChainContract};
use soroban_sdk::{Bytes, Env};

fn vector(env: &Env, domain: HashDomain, expected: [u8; 32]) {
    let content = Bytes::from_slice(env, b"hello-petchain");
    let got = PetChainContract::compute_domain_hash(env.clone(), domain, content);
    assert_eq!(got.to_array(), expected);
}

#[test]
fn test_vector_evidence() {
    let env = Env::default();
    vector(
        &env,
        HashDomain::Evidence,
        [
            0x80, 0x7d, 0xbe, 0x3e, 0x10, 0x46, 0x01, 0x4c, 0xa3, 0xad, 0xbd, 0x9e, 0x9e, 0x50,
            0xd0, 0x13, 0x0e, 0x00, 0x8f, 0xf3, 0x83, 0x06, 0x7f, 0x8a, 0x3f, 0x42, 0xdf, 0xee,
            0xc8, 0xf0, 0x21, 0xde,
        ],
    );
}

#[test]
fn test_vector_attachment() {
    let env = Env::default();
    vector(
        &env,
        HashDomain::Attachment,
        [
            0xe4, 0x4c, 0xae, 0xad, 0x73, 0x38, 0x45, 0x5d, 0x5c, 0xb2, 0x6e, 0x7e, 0x8a, 0x0f,
            0x96, 0xd0, 0x2a, 0x07, 0x68, 0x98, 0x5f, 0x61, 0x65, 0xf2, 0xe7, 0x48, 0xff, 0x09,
            0xfe, 0xe8, 0x45, 0x2b,
        ],
    );
}

#[test]
fn test_vector_claim_document() {
    let env = Env::default();
    vector(
        &env,
        HashDomain::ClaimDocument,
        [
            0xa5, 0x75, 0x67, 0x6b, 0xe4, 0xb3, 0x3a, 0x5e, 0x6d, 0x89, 0xd0, 0x37, 0x23, 0x22,
            0x61, 0x8e, 0x93, 0x07, 0x41, 0x21, 0xf2, 0xf9, 0xd8, 0x83, 0x9a, 0xd0, 0x68, 0x78,
            0x65, 0x53, 0x4f, 0x1c,
        ],
    );
}

#[test]
fn test_vector_certificate() {
    let env = Env::default();
    vector(
        &env,
        HashDomain::Certificate,
        [
            0x8d, 0x59, 0xcb, 0xb5, 0x52, 0xdc, 0xb0, 0xa8, 0xeb, 0xa9, 0x0a, 0xa1, 0x6d, 0x57,
            0x7e, 0x63, 0x4b, 0x32, 0xab, 0x38, 0x34, 0x06, 0x3a, 0x5d, 0x42, 0xbf, 0xf8, 0x6e,
            0xb5, 0x3f, 0x2e, 0x28,
        ],
    );
}

#[test]
fn test_vector_medical_record() {
    let env = Env::default();
    vector(
        &env,
        HashDomain::MedicalRecord,
        [
            0x46, 0xa0, 0x3f, 0x5e, 0x07, 0x81, 0x5b, 0x48, 0xa3, 0xea, 0x07, 0xbd, 0x98, 0x9f,
            0x9a, 0xb0, 0xa7, 0x7e, 0xfd, 0x9f, 0x5c, 0x0e, 0x7c, 0xde, 0xef, 0xc8, 0x7f, 0x17,
            0x92, 0x10, 0x86, 0x27,
        ],
    );
}

#[test]
fn test_vector_tag_id() {
    let env = Env::default();
    vector(
        &env,
        HashDomain::TagId,
        [
            0x8e, 0xa0, 0x6e, 0x04, 0xe5, 0x2c, 0xb6, 0xd5, 0x78, 0xce, 0x04, 0x74, 0x57, 0xf5,
            0x92, 0x0f, 0x81, 0xaf, 0x43, 0x56, 0x00, 0xf9, 0x31, 0x78, 0xb9, 0xf6, 0x7b, 0xf2,
            0x91, 0xf8, 0xa9, 0xe0,
        ],
    );
}

/// The same content must hash differently across every domain -- no two
/// domains may ever collide for identical input.
#[test]
fn test_domains_are_pairwise_distinct_for_same_content() {
    let env = Env::default();
    let content = Bytes::from_slice(&env, b"same-bytes-everywhere");
    let domains = [
        HashDomain::Evidence,
        HashDomain::Attachment,
        HashDomain::ClaimDocument,
        HashDomain::Certificate,
        HashDomain::MedicalRecord,
        HashDomain::TagId,
    ];

    let mut hashes = soroban_sdk::Vec::new(&env);
    for domain in domains.iter() {
        let h = PetChainContract::compute_domain_hash(env.clone(), domain.clone(), content.clone());
        for existing in hashes.iter() {
            assert_ne!(h, existing, "two distinct domains must never collide");
        }
        hashes.push_back(h);
    }
}

/// Changing the content changes the hash within a fixed domain (sanity
/// check that the function isn't a constant per-domain value).
#[test]
fn test_same_domain_different_content_differs() {
    let env = Env::default();
    let a = PetChainContract::compute_domain_hash(
        env.clone(),
        HashDomain::Evidence,
        Bytes::from_slice(&env, b"content-a"),
    );
    let b = PetChainContract::compute_domain_hash(
        env.clone(),
        HashDomain::Evidence,
        Bytes::from_slice(&env, b"content-b"),
    );
    assert_ne!(a, b);
}

/// Empty content is a valid, well-defined edge case (domain tag alone).
#[test]
fn test_empty_content_is_deterministic() {
    let env = Env::default();
    let a = PetChainContract::compute_domain_hash(
        env.clone(),
        HashDomain::Attachment,
        Bytes::new(&env),
    );
    let b = PetChainContract::compute_domain_hash(
        env.clone(),
        HashDomain::Attachment,
        Bytes::new(&env),
    );
    assert_eq!(a, b);
}
