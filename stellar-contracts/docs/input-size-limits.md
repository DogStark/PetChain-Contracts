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
| `submit_enrollment_application` | `notes` | `MAX_ENROLLMENT_NOTES_LEN = 500` bytes |
| `submit_enrollment_application` | `applicant_name` | `MAX_ENROLLMENT_APPLICANT_NAME_LEN = 100` bytes |
| `submit_enrollment_application` | `policy_version` | `MAX_POLICY_VERSION_LEN = 50` bytes |

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
| `submit_enrollment_application` | `coverage_items` (snapshot) | `MAX_COVERAGE_ITEMS = 50` |
| `submit_enrollment_application` | `exclusions` (snapshot) | `MAX_EXCLUSIONS = 50` |
| `submit_enrollment_application` | `documents` | `MAX_ENROLLMENT_DOCUMENTS = 20` |
| `set_condition_coverage` / `submit_condition_claim` | policy id | 64 bytes |
| `set_condition_coverage` / `submit_condition_claim` | condition | 64 bytes |
| `submit_condition_claim` | description | 500 bytes |

### Enrollment coverage & exclusion snapshots

`submit_enrollment_application` stores an immutable snapshot of the policy
version's coverage and exclusion lists on the enrollment record at
submission time. Later policy edits never mutate an existing enrollment's
snapshot. Each snapshot list is capped (`MAX_COVERAGE_ITEMS = 50`,
`MAX_EXCLUSIONS = 50`); an over-cap list is rejected with
`ContractError::TooManyItems` (`27`) before any storage write, so a failed
submission leaves no partial enrollment record behind. Individual snapshot
entries are `String`s bounded by `MAX_COVERAGE_ITEM_LEN = 100` bytes and
`MAX_EXCLUSION_LEN = 100` bytes respectively, rejected with
`ContractError::InputStringTooLong` (`8`).

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

Enrollment-specific cases additionally assert that a rejected submission
(over-long string, over-cap coverage/exclusion list, or over-long snapshot
entry) leaves the enrollment counter and storage unchanged, and that a
successful submission's snapshot is byte-identical after a subsequent
policy update.

Condition benefit terms and claim state have additional cardinality limits:
32 configured conditions per pet and 256 benefit claims per pet and policy.
Waiting-period,
coverage-limit, and settlement behavior is documented in
[`insurance-benefits.md`](insurance-benefits.md).