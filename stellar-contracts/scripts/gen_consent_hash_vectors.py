#!/usr/bin/env python3
"""
Reference implementation for the PetChain canonical consent hash (Issue #1337).

Reproduces, byte-for-byte, `canonical_consent_hash` in
stellar-contracts/src/consent_canon.rs and prints the vectors asserted in
stellar-contracts/src/test_consent_canonicalization.rs.

Canonical format (domain "petchain:consent:v1")
-----------------------------------------------

    hash = sha256(domain
                  || u64be(pet_id)
                  || xdr(subject)
                  || u32be(purpose_code)
                  || u32be(scope_mask)
                  || u32be(version))

  * xdr(subject) is the soroban Address ScVal for a G... account:
        u32be(18) || u32be(0) || u32be(0) || 32-byte ed25519 key
  * purpose_code: Insurance=1, Research=2, PublicHealth=3, Other=4
  * scope_mask:   ReadMedical=1, WriteMedical=2, ReadLab=4, EmergencyOnly=8
"""

import base64
import hashlib
import struct

DOMAIN = b"petchain:consent:v1"


def crc16_xmodem(data: bytes) -> int:
    crc = 0
    for byte in data:
        crc ^= byte << 8
        for _ in range(8):
            if crc & 0x8000:
                crc = ((crc << 1) ^ 0x1021) & 0xFFFF
            else:
                crc = (crc << 1) & 0xFFFF
    return crc


def account_strkey(i: int) -> str:
    """Deterministic, valid G... account key with payload == i (32 bytes BE)."""
    data = bytes([6 << 3]) + i.to_bytes(32, "big")
    return base64.b32encode(data + struct.pack("<H", crc16_xmodem(data))).decode("ascii")


def address_xdr(i: int) -> bytes:
    return struct.pack(">III", 18, 0, 0) + i.to_bytes(32, "big")


def consent_hash(pet_id: int, subject: int, purpose: int, mask: int, version: int) -> str:
    preimage = (
        DOMAIN
        + struct.pack(">Q", pet_id)
        + address_xdr(subject)
        + struct.pack(">III", purpose, mask, version)
    )
    return hashlib.sha256(preimage).hexdigest()


VECTORS = [
    # (name, pet_id, subject payload, purpose, scope_mask, version)
    ("RESEARCH_READ_MEDICAL_LAB_V1", 1, 1, 2, 0b0101, 1),
    ("RESEARCH_READ_MEDICAL_LAB_V2", 1, 1, 2, 0b0101, 2),
    ("INSURANCE_ALL_SCOPES_V1", 7, 2, 1, 0b1111, 1),
]

if __name__ == "__main__":
    print(f'const SUBJECT_1: &str = "{account_strkey(1)}";')
    print(f'const SUBJECT_2: &str = "{account_strkey(2)}";')
    for name, pet, subject, purpose, mask, version in VECTORS:
        print(f'const {name}: &str = "{consent_hash(pet, subject, purpose, mask, version)}";')
