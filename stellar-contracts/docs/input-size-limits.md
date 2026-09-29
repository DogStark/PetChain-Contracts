# Input Size Limits (Issue #1252)

This document is the contract surface contract for **client implementers**.
Every public method that accepts a `String`, `Bytes`, or `Vec` enforces an
explicit maximum at the **transaction boundary** — before any storage
write, index update, or hashing runs — and fails with a **typed** contract
error. Reading this table up front lets a client reject oversized inputs
client-side (and show a friendly message) instead of paying on-chain fees
for a call that is guaranteed to fail.

## Why limits exist

Unbounded strings/vecs inflate ledger entries and increase read/write fees
linearly; a malicious caller could submit multi-KiB inputs to exhaust
budget or push a record past the ledger's ~64 KiB XDR limit, bricking the
record for everyone who later reads it. The caps below keep every stored
entry well within bounds without restricting realistic usage.

## Error semantics

| Violation | Typed error | Discriminant |
|---|---|---|
| `String`/`Bytes` over `MAX_*_LEN` | `ContractError::InputStringTooLong` | `8` |
| `Vec` over an element cap | `ContractError::TooManyItems` | `27` |
| Invalid vet/pet string (empty or bad) via `validate_len` | `ContractError::InvalidInput` | `12` |

Over-limit inputs are rejected *before* expensive work: the boundary check
is the first instruction in each method, so a rejected call writes nothing
and advances no counter.

## String & Bytes limits

| Method | Field | Limit |
|---|---|---|
| `register_pet` | `color` | `MAX_COLOR_LEN = 50` bytes |
| `register_pet` | `name` | validated by `validate_pet_name` (non-empty, non-huge) |
| `register_vet` | `name` | `PetChainContract::MAX_VET_NAME_LEN = 100` bytes |
| `register_vet` | `license_number` | `PetChainContract::MAX_VET_LICENSE_LEN = 50` bytes |
| `register_vet` | `specialization` | `PetChainContract::MAX_VET_SPEC_LEN = 100` bytes |
| `add_behavior_record` | `description` | `MAX_BEHAVIOR_DESC_LEN = 500` bytes |
| `add_activity_record` | `notes` | `MAX_ACTIVITY_NOTES_LEN = 500` bytes |
| `add_medical_record` | `diagnosis` | `MAX_MEDICAL_DIAGNOSIS_LEN = 500` bytes |
| `add_medical_record` | `treatment` | `MAX_MEDICAL_TREATMENT_LEN = 500` bytes |
| `add_medical_record` | `notes` | `MAX_MEDICAL_NOTES_LEN = 1000` bytes |
| `add_lab_result` | `test_type` | `MAX_LAB_TEST_TYPE_LEN = 100` bytes |
| `add_lab_result` | `results` | `MAX_LAB_RESULTS_LEN = 1000` bytes |
| `add_lab_result` | `reference_ranges` | `MAX_LAB_REF_RANGES_LEN = 500` bytes |
| `add_breeding_record` | `notes` | `MAX_BREEDING_NOTES_LEN = 500` bytes |

Limits are byte lengths (`String::len()`), i.e. UTF-8 encoded size, not
character counts. Multi-byte characters consume more than one "letter".

## Vec element caps

| Method | Field | Cap |
|---|---|---|
| `add_medical_record` | `medications` | `MAX_VEC_MEDS = 20` |
| `add_medical_record` / `add_attachment` | attachments per record | `MAX_ATTACHMENTS_PER_RECORD = 20` |
| `add_pet_photo` | `photo_hashes` on `Pet` | `MAX_PHOTO_HASHES = 20` |
| `add_nutrition_plan` | `ingredients` | `MAX_INGREDIENTS = 50` |
| `add_training_milestone` | `prerequisites` | `MAX_PREREQUISITES = 20` |
| `setup_pet_multisig` | `signers` | `MAX_MULTISIG_SIGNERS = 20` |

## Notes for client implementers

- **Validate before sending.** Each of the above becomes a guaranteed
  on-chain rejection; catching it client-side avoids the fee. ASCII byte
  length is the correct unit for all `String` checks.
- **Stable discriminants.** `InputStringTooLong = 8`, `TooManyItems = 27`,
  `InvalidInput = 12` (see `src/test_error_registry.rs` for the full,
  stability-pinned list — do not treat these numbers as reorderable).
- **Exposed limits.** Module-level `MAX_*` constants are re-usable from the
  crate root; vet limits are exposed as associated constants
  (`PetChainContract::MAX_VET_NAME_LEN`, `MAX_VET_LICENSE_LEN`,
  `MAX_VET_SPEC_LEN`).

## Test coverage

`src/test_max_input_sizes.rs` (wired into `cargo test`) proves, for each
public input above, both halves of the boundary:
- exactly-at-limit accepted, and
- one-unit-over-limit rejected with its typed error (`#8` strings / `#27`
  vecs), run on the **default finite env budget** to demonstrate rejection
  is cheap and fires before storage.