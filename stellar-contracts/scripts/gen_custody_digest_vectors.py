#!/usr/bin/env python3
"""
Reference implementation for the PetChain custody-chain canonical digest.

This script reproduces, byte-for-byte, the canonical serialisation used by
`PetChainContract::get_custody_chain_digest` (see stellar-contracts/src/lib.rs)
and prints the published vectors asserted in
stellar-contracts/src/test_custody_digest.rs.

Canonical format (CUSTODY_DIGEST_DOMAIN = "petchain.custody", version = 1)
--------------------------------------------------------------------------
Each CustodyEntry is canonicalised as:

    entry_bytes = u32be(len(from_xdr)) || from_xdr
                  || u32be(len(to_xdr)) || to_xdr
                  || u64be(timestamp)
                  || u32be(transfer_type)

where:
  * from_xdr / to_xdr are the XDR encodings of the soroban Address ScVal
    (SCV_ADDRESS = 18, followed by the SCAddress union):
        account:  u32be(18) || u32be(0) || u32be(0) || 32-byte ed25519 key
        contract: u32be(18) || u32be(1) || 32-byte contract id
  * timestamp is the ledger timestamp (u64, big-endian)
  * transfer_type is the enum discriminant: Direct=0, Adoption=1, Multisig=2

The digest is a SHA-256 hash chain over the entries in canonical order:

    state = sha256(domain_bytes || u32be(version) || u64be(pet_id))
    for each entry in chain order:
        state = sha256(state || u32be(len(entry_bytes)) || entry_bytes)
    digest = sha256(state || u32be(sequence))

sequence is the number of entries hashed, so appending an entry changes the
digest and reordering entries cannot reproduce it.

Offline verification (issue #1445)
----------------------------------
The same preimage format is used for offline verification of vaccination
certificates. Certificate payloads use the domain tag "PC-VACC-CERT" with
version 1, and are encoded as:

    cert_payload = u32be(len(domain)) || domain
                   || u32be(1)                  (version)
                   || u64be(pet_id)
                   || u32be(len(vaccine)) || vaccine
                   || u64be(administered_at)
                   || u64be(valid_until)
                   || u32be(len(administrator_xdr)) || administrator_xdr

All integers are big-endian; strings are UTF-8 with explicit length prefixes;
there are no separators beyond length prefixes. The verifier signs/hashes
sha256(cert_payload).
"""

import hashlib
import struct
import base64


# ---------------------------------------------------------------------------
# Stellar strkey helpers (account keys start with 'G')
# ---------------------------------------------------------------------------

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


def encode_strkey(version_byte: int, payload: bytes) -> str:
    data = bytes([version_byte]) + payload
    # Stellar strkey checksums are appended little-endian.
    check = struct.pack("<H", crc16_xmodem(data))
    return base64.b32encode(data + check).decode("ascii")


def account_strkey(i: int) -> str:
    """Deterministic, valid G... account key with payload == i (32 bytes BE)."""
    return encode_strkey(6 << 3, i.to_bytes(32, "big"))


# ---------------------------------------------------------------------------
# Canonical serialisation
# ---------------------------------------------------------------------------

DOMAIN = b"petchain.custody"
VERSION = 1

SCV_ADDRESS = 18
SC_ADDRESS_TYPE_ACCOUNT = 0
SC_ADDRESS_TYPE_CONTRACT = 1
CRYPTO_KEY_TYPE_ED25519 = 0


def address_xdr(strkey: str) -> bytes:
    """XDR of the soroban Address ScVal for a G... (account) strkey."""
    payload = base64.b32decode(strkey + "=" * (-len(strkey) % 8))[1:-2]
    assert len(payload) == 32, payload.hex()
    return (
        struct.pack(">I", SCV_ADDRESS)
        + struct.pack(">I", SC_ADDRESS_TYPE_ACCOUNT)
        + struct.pack(">I", CRYPTO_KEY_TYPE_ED25519)
        + payload
    )


def entry_bytes(from_key: str, to_key: str, timestamp: int, transfer_type: int) -> bytes:
    from_xdr = address_xdr(from_key)
    to_xdr = address_xdr(to_key)
    return (
        struct.pack(">I", len(from_xdr)) + from_xdr
        + struct.pack(">I", len(to_xdr)) + to_xdr
        + struct.pack(">Q", timestamp)
        + struct.pack(">I", transfer_type)
    )


def digest(pet_id: int, entries: list[bytes]) -> str:
    state = hashlib.sha256(DOMAIN + struct.pack(">I", VERSION) + struct.pack(">Q", pet_id)).digest()
    for eb in entries:
        state = hashlib.sha256(state + struct.pack(">I", len(eb)) + eb).digest()
    state = hashlib.sha256(state + struct.pack(">I", len(entries))).digest()
    return state.hex()


# ---------------------------------------------------------------------------
# Vaccination certificate offline payload (issue #1445)
# ---------------------------------------------------------------------------

CERT_DOMAIN = b"PC-VACC-CERT"
CERT_VERSION = 1


def _lp(data: bytes) -> bytes:
    """Length-prefixed bytes: u32be(len) || data."""
    return struct.pack(">I", len(data)) + data


def cert_payload(
    pet_id: int,
    vaccine: str,
    administered_at: int,
    valid_until: int,
    administrator_key: str,
) -> bytes:
    """Canonical vaccination certificate payload for offline verification."""
    return (
        _lp(CERT_DOMAIN)
        + struct.pack(">I", CERT_VERSION)
        + struct.pack(">Q", pet_id)
        + _lp(vaccine.encode("utf-8"))
        + struct.pack(">Q", administered_at)
        + struct.pack(">Q", valid_until)
        + _lp(address_xdr(administrator_key))
    )


def cert_digest(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


# ---------------------------------------------------------------------------
# Published vectors
# ---------------------------------------------------------------------------

A0 = account_strkey(0)
A1 = account_strkey(1)
A2 = account_strkey(2)
A3 = account_strkey(3)

print("== addresses ==")
for name, key in (("A0", A0), ("A1", A1), ("A2", A2), ("A3", A3)):
    print(f"{name} = {key}")

e_1 = entry_bytes(A0, A1, 1_000_000_000, 0)  # Direct
e_2 = entry_bytes(A1, A2, 2_000_000_000, 1)  # Adoption
e_3 = entry_bytes(A2, A3, 3_000_000_000, 2)  # Multisig
e_2_tampered = entry_bytes(A1, A3, 2_000_000_000, 1)

print("== entry sizes ==")
print("entry size (account addresses):", len(e_1))

print("== vectors ==")
print(f"empty (pet 1):              {digest(1, [])}")
print(f"single entry (pet 1):       {digest(1, [e_1])}")
print(f"two entries (pet 1):        {digest(1, [e_1, e_2])}")
print(f"three entries (pet 1):      {digest(1, [e_1, e_2, e_3])}")
print(f"reordered 2 entries (pet 1):{digest(1, [e_2, e_1])}")
print(f"tampered 'to' (pet 1):      {digest(1, [e_1, e_2_tampered])}")
print(f"two entries, pet 7:         {digest(7, [e_1, e_2])}")

print("== vaccination certificate vectors (issue #1445) ==")
cert_1 = cert_payload(1, "Rabies", 1_700_000_000, 1_731_536_000, A0)
cert_2 = cert_payload(7, "Parvovirus", 1_710_000_000, 1_741_536_000, A1)
cert_3 = cert_payload(42, "FVRCP", 1_720_000_000, 1_751_536_000, A2)
print(f"cert payload 1 hex:  {cert_1.hex()}")
print(f"cert digest 1:       {cert_digest(cert_1)}")
print(f"cert payload 2 hex:  {cert_2.hex()}")
print(f"cert digest 2:       {cert_digest(cert_2)}")
print(f"cert payload 3 hex:  {cert_3.hex()}")
print(f"cert digest 3:       {cert_digest(cert_3)}")
