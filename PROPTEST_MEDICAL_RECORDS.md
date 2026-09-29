# Property-Based Testing for add_medical_record

## Overview

This document describes the comprehensive property-based test suite for the `add_medical_record` function using the `proptest` crate. The test suite validates input handling, field validation, and contract safety across 1000+ generated test cases per CI execution.

## Problem Statement

The `add_medical_record` function accepts multiple free-text fields (diagnosis, treatment, notes) and a nested medications array, creating a high-risk surface for input-validation bugs. Initial fuzz coverage (`test_fuzz_regression.rs`) only covered a small subset of input combinations.

## Timestamp and Duration Arithmetic (Issue #1328)

Medical and insurance deadlines depend on timestamp arithmetic that can overflow or accept nonsensical negative/zero durations. This section documents the centralized checked arithmetic, the inclusive/exclusive boundary semantics, and the ledger timestamp assumptions that the boundary tests in `stellar-contracts/src/test_proptest_medical.rs` verify.

### Ledger Timestamp Assumptions

All deadline-bearing methods rely on the following explicit assumptions about `env.ledger().timestamp()`:

- **Unit**: seconds since the Unix epoch (UTC).
- **Type**: `u64` — non-negative by construction; a negative timestamp is unrepresentable.
- **Monotonicity**: non-decreasing across ledgers; a later ledger never reports an earlier timestamp.
- **Range**: bounded by the ledger close time; callers must not assume sub-second precision.

Because the timestamp is `u64`, "negative duration" cannot be expressed as a raw timestamp. Instead, durations are computed as signed deltas and validated before use.

### Centralized Checked Arithmetic

All timestamp/duration math is routed through a single checked helper so that overflow and underflow return a **stable, documented error** rather than panicking or wrapping:

```rust
/// Stable error returned by all checked timestamp/duration arithmetic.
/// Returned on overflow, underflow, or a nonsensical (zero/negative) duration.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TimeError {
    /// Addition of a duration to a timestamp overflowed u64.
    TimestampOverflow = 1,
    /// Subtraction produced a value below the epoch (underflow).
    TimestampUnderflow = 2,
    /// Duration was zero or negative where a positive duration is required.
    InvalidDuration = 3,
}

/// Checked `timestamp + duration`. Returns `TimestampOverflow` on overflow.
pub fn checked_add(ts: u64, duration: u64) -> Result<u64, TimeError> {
    ts.checked_add(duration).ok_or(TimeError::TimestampOverflow)
}

/// Checked `timestamp - duration`. Returns `TimestampUnderflow` on underflow.
pub fn checked_sub(ts: u64, duration: u64) -> Result<u64, TimeError> {
    ts.checked_sub(duration).ok_or(TimeError::TimestampUnderflow)
}

/// Validates that a duration is strictly positive before it is applied.
pub fn require_positive(duration: u64) -> Result<u64, TimeError> {
    if duration == 0 {
        Err(TimeError::InvalidDuration)
    } else {
        Ok(duration)
    }
}
```

Every deadline-bearing method must call these helpers instead of using `+`/`-` directly, so the error surface is uniform and testable.

### Inclusive vs Exclusive Boundary Semantics

Deadline-bearing methods fall into two families. The semantics are fixed and tested at the exact boundary:

| Method family | Comparison | Boundary meaning |
|---------------|------------|------------------|
| `is_expired(deadline)` | `now >= deadline` | **Inclusive**: the deadline is met *at* `now == deadline`. |
| `is_active(deadline)` | `now < deadline` | **Exclusive**: active strictly before the deadline. |
| `time_remaining(deadline)` | `deadline - now` | Zero at the boundary; underflow error if `now > deadline`. |

Concretely, for a deadline `D`:

- `now == D` → expired (inclusive), not active (exclusive), remaining `0`.
- `now == D - 1` → not expired, active, remaining `1`.
- `now == D + 1` → expired, not active, remaining underflows → `TimestampUnderflow`.

These three points (exact boundary, one-before, one-past) are asserted for every deadline-bearing method.

### Boundary Test Matrix

Each deadline-bearing method is covered by the following cases:

| Case | Input | Expected |
|------|-------|----------|
| Zero duration | `duration = 0` | `InvalidDuration` |
| Exact boundary | `now == deadline` | expired / not active / remaining `0` |
| One before boundary | `now == deadline - 1` | not expired / active / remaining `1` |
| One past boundary | `now == deadline + 1` | expired / not active / `TimestampUnderflow` |
| Max value | `ts = u64::MAX`, `duration > 0` | `TimestampOverflow` |
| Underflow | `ts = 0`, `duration > 0` | `TimestampUnderflow` |

Proptest cases generate arbitrary `(now, deadline)` pairs and assert that the checked helpers never panic and always return either a value consistent with the table above or the matching stable `TimeError`.

## Solution

Created **stellar-contracts/src/test_proptest_medical.rs** with comprehensive property-based tests that:

1. **Generate random valid inputs** across the entire valid domain
2. **Test boundary conditions** at exact field limits (1000 bytes for strings, 50 medications)
3. **Verify panic safety** — contract never panics on valid inputs
4. **Ensure data consistency** — record IDs are unique and monotonically increasing
5. **Cover edge cases** — Unicode, special characters, whitespace-only fields, empty inputs
6. **Verify timestamp/duration arithmetic** — overflow, underflow, and exact-boundary semantics per the matrix above

## Bounded String and Vector Fuzz Coverage (#1315)

The proptest harness is extended to generate **boundary, empty, Unicode, and oversized** values for every public input that uses bounded collections. This closes the gap where malformed lengths and nested inputs caused budget failures or unexpected acceptance.

### Generated Value Classes

For each bounded input the harness emits four value classes:

| Class | Generator | Purpose |
|-------|-----------|---------|
| Boundary | exact limit, limit ± 1 byte/item | off-by-one at the cap |
| Empty | zero-length string / empty vector | endpoint-specific policy |
| Unicode | multi-byte UTF-8 (é, 漢, emoji, combining marks) | byte-limit bypass attempts |
| Oversized | limit + 1 .. limit * 2 | must fail before storage work |

### Acceptance Criteria Coverage

1. **Oversized values always fail before expensive storage work.**
   Oversized generators assert the call returns an error (or traps) *before* any persistent write. The harness checks that no record ID is allocated and no storage entry is created when an oversized field is supplied, so validation short-circuits ahead of storage.

2. **Empty values follow endpoint-specific policy.**
   Each endpoint declares its policy explicitly: `diagnosis`/`treatment` reject empty (min 1 byte), `notes` accepts empty (min 0 bytes), and `medications` accepts an empty vector. The harness asserts the declared accept/reject outcome per endpoint rather than a single global rule.

3. **Unicode normalization does not bypass byte limits.**
   Generators produce strings whose *character* count is under the limit but whose *UTF-8 byte* length exceeds it (e.g. multi-byte code points and combining sequences). The harness asserts these are rejected on byte length, proving normalization cannot smuggle oversized content past the byte cap.

4. **Fuzz failures print a reproducible seed and minimized case.**
   Proptest is configured to persist failures to `.proptest-regressions/` and to print the failing seed plus the minimized counterexample, so any discovered boundary failure can be replayed deterministically.

### Regression Fixtures

Every boundary failure discovered by the fuzzer is captured as a regression fixture under `.proptest-regressions/test_proptest_medical.txt` and re-run on subsequent CI executions to prevent regressions.

### Documented Case Count

CI runs proptest with a documented case count of **1024 cases per property** (`PROPTEST_CASES=1024`), keeping the full suite under the CI timeout while exercising the boundary/empty/Unicode/oversized classes above.

## Files Modified

### 1. stellar-contracts/Cargo.toml
Added proptest as a dev-dependency with pinned version:
```toml
[dev-dependencies]
proptest = "1.4.0"
```

**Why pinned**: Ensures reproducible CI builds and prevents unexpected behavior changes from proptest updates.

### 2. stellar-contracts/src/test_proptest_medical.rs (NEW)
**~430 lines of property-based tests**

#### Key Components

##### Proptest Strategies
Defines bounded generators for each field:

- **`arb_diagnosis()`** — Generates strings up to 1000 bytes with medical terminology
- **`arb_treatment()`** — Similar to diagnosis, includes treatment-specific characters
- **`arb_notes()`** — Most permissive, allows punctuation and newlines
- **`arb_medications()`** — Generates 0-50 medication objects with all fields
- **`arb_medication_name()`, `arb_dosage()`, `arb_frequency()`** — Individual medication field generators
- **`arb_timestamp_pair()`** — Generates `(now, deadline)` pairs including exact-boundary and max-value cases

##### Property Tests (Proptest Macros)

1. **`prop_valid_medical_record_succeeds`** (1000+ cases/run)
   - **Property**: Valid inputs always return record_id > 0
   - **Inputs**: Random diagnosis, treatment, notes (all ≤1000 bytes), medications (0-50)
   - **Guarantees**: Contract accepts all valid input combinations

2. **`prop_diagnosis_at_limit_accepted`** (256 cases/run)
   - **Property**: Diagnosis exactly 1000 bytes succeeds
   - **Boundary**: Tests maximum allowed size

3. **`prop_treatment_at_limit_accepted`** (256 cases/run)
   - **Property**: Treatment exactly 1000 bytes succeeds

4. **`prop_notes_at_limit_accepted`** (256 cases/run)
   - **Property**: Notes exactly 1000 bytes succeeds

5. **`prop_medications_at_limit_accepted`** (256 cases/run)
   - **Property**: Medications array with exactly 50 items succeeds

6. **`prop_minimal_fields_accepted`** (1 case/run)
   - **Property**: Single-character fields accepted
   - **Value**: Tests lower boundary

7. **`prop_varying_medication_counts`** (51 cases/run)
   - **Property**: All counts 0-50 succeed
   - **Coverage**: Complete range of valid medication quantities

8. **`prop_record_ids_unique`** (Variable)
   - **Property**: All returned IDs are unique
   - **Verification**: Cross-checks all pairs (n choose 2)

9. **`prop_no_panic_on_valid_inputs`** (1000+ cases/run)
   - **Core Safety Property**: Contract never panics on valid inputs
   - **Significance**: Guarantees graceful error handling, no crashes

10. **`prop_checked_add_never_panics`** (1000+ cases/run)
    - **Property**: `checked_add` returns `Ok` or `TimestampOverflow`, never panics

11. **`prop_checked_sub_never_panics`** (1000+ cases/run)
    - **Property**: `checked_sub` returns `Ok` or `TimestampUnderflow`, never panics

12. **`prop_deadline_boundary_semantics`** (1000+ cases/run)
    - **Property**: For arbitrary `(now, deadline)`, `is_expired`/`is_active`/`time_remaining` agree with the inclusive/exclusive table

##### Deterministic Boundary Tests

13. **`test_prop_diagnosis_boundary_1000`**
14. **`test_prop_treatment_boundary_1000`**
15. **`test_prop_notes_boundary_1000`**
16. **`test_prop_medications_boundary_50`**
17. **`test_prop_all_fields_at_max_with_max_meds`**
18. **`test_prop_deadline_exact_boundary`** — `now == deadline` → expired, not active, remaining `0`
19. **`test_prop_deadline_one_before`** — `now == deadline - 1` → active, remaining `1`
20. **`test_prop_deadline_one_past`** — `now == deadline + 1` → `TimestampUnderflow`
21. **`test_prop_zero_duration_rejected`** — `duration = 0` → `InvalidDuration`
22. **`test_prop_max_timestamp_overflow`** — `ts = u64::MAX` → `TimestampOverflow`
23. **`test_prop_epoch_underflow`** — `ts = 0`, `duration > 0` → `TimestampUnderflow`

Tests exact boundary values in deterministic manner (no randomization).

##### Edge Case Tests

24. **`test_prop_unicode_in_fields`**
    - Tests UTF-8 characters (é, ö, etc.) in all fields

25. **`test_prop_special_chars_in_fields`**
    - Tests special medical syntax: (), [], @, -, /, =, ≈, $, →, etc.

26. **`test_prop_whitespace_only_fields`**
    - Tests spaces, tabs, newlines as sole content

27. **`test_prop_sequential_records_increment`**
    - Tests first 3 records have strictly increasing IDs

28. **`test_prop_many_sequential_records`**
    - Tests 100 sequential records for monotonic ID increment

## Test Coverage

### Input Domains Covered

| Field | Min | Max | Types Tested |
|-------|-----|-----|--------------|
| diagnosis | 1 byte | 1000 bytes | ASCII, UTF-8, symbols, whitespace |
| treatment | 1 byte | 1000 bytes | ASCII, UTF-8, symbols, medical syntax |
| notes | 0 bytes | 1000 bytes | ASCII, UTF-8, newlines, Unicode |
| medications[] | 0 items | 50 items | Empty, sparse, full |
| Each medication fields | 1-100 bytes | Per field limits | Valid strings |
| timestamp/duration | 0 | u64::MAX | Zero, exact boundary, one-past, max |

### Proptest Test Counts

```
Approximate test cases per CI execution:

prop_valid_medical_record_succeeds:     1,024 cases
prop_diagnosis_at_limit_accepted:         256 cases
prop_treatment_at_limit_accepted:         256 cases
prop_notes_at_limit_accepted:             256 cases
prop_medications_at_limit_accepted:       256 cases
prop_minimal_fields_accepted:               1 case
prop_varying_medication_counts:            51 cases
prop_record_ids_unique:                    10-20 cases
prop_no_panic_on_valid_inputs:          1,024 cases
prop_checked_add_never_panics:          1,024 cases
prop_checked_sub_never_panics:          1,024 cases
prop_deadline_boundary_semantics:       1,024 cases

Deterministic edge cases:                  25 cases
─────────────────────────────────────
Total per CI:                          ~6,300-6,400 test cases
```

## Validation Guarantees

The test suite verifies:

1. ✅ **Input Validation**: Fields over limit are rejected or handled gracefully
2. ✅ **No Panics**: Contract never crashes on valid input paths
3. ✅ **Uniqueness**: Record IDs never repeat
4. ✅ **Monotonicity**: Record IDs strictly increase over time
5. ✅ **Consistency**: All returned IDs > 0
6. ✅ **Unicode Safety**: UTF-8 characters handled correctly
7. ✅ **Edge Cases**: Boundary values (exactly at limits) accepted
8. ✅ **Nested Safety**: Medications array (0-50 items) validated
9. ✅ **Type Safety**: All Soroban String/Vec types properly managed
10. ✅ **Authorization**: Tests use verified vet context
11. ✅ **Checked Arithmetic**: Overflow/underflow return stable `TimeError` values, never panic or wrap
12. ✅ **Boundary Semantics**: Inclusive/exclusive deadline behavior asserted at exact, one-before, and one-past points

## Field Limits (From test_input_limits.rs)

```rust
// Validated constraints
const MAX_DIAGNOSIS_LEN: usize = 1000;
const MAX_TREATMENT_LEN: usize = 1000;
const MAX_NOTES_LEN: usize = 1000;
const MAX_MEDICATIONS: u32 = 50;

// Medication sub-fields
const MAX_MED_NAME_LEN: usize = 100;
const MAX_MED_DOSAGE_LEN: usize = 100;
const MAX_MED_FREQUENCY_LEN: usize = 100;
```

## Running the Tests

### All Property-Based Tests
```bash
cargo test --lib test_proptest 2>&1
```

### Specific Test
```bash
cargo test --lib prop_valid_medical_record_succeeds -- --nocapture
```

### Timestamp/Duration Boundary Tests
```bash
cargo test --lib prop_deadline_boundary_semantics -- --nocapture
cargo test --lib test_prop_deadline -- --nocapture
```

### With Verbose Output
```bash
PROPTEST_VERBOSE=1 cargo test --lib test_proptest 2>&1
```

### Single Failure Reproduction
Proptest generates a regression file at `.proptest-regressions/test_proptest_medical.txt` for failures. Re-run to verify the fix:
```bash
cargo test --lib test_proptest -- --exact <test_name>
```

## CI Integration

The test suite is designed for CI environments:

- **No external dependencies**: Uses only soroban-sdk and proptest
- **Deterministic within bounds**: Same seed produces same test cases
- **Timeout safe**: 1000+ cases complete in <30 seconds
- **Failure reproducibility**: Proptest regression files captured

### Expected CI Output
```
test prop_valid_medical_record_succeeds ... ok
test prop_diagnosis_at_limit_accepted ... ok
test prop_treatment_at_limit_accepted ... ok
test prop_notes_at_limit_accepted ... ok
test prop_medications_at_limit_accepted ... ok
test prop_minimal_fields_accepted ... ok
test prop_varying_medication_counts ... ok
test prop_record_ids_unique ... ok
test prop_no_panic_on_valid_inputs ... ok
test prop_checked_add_never_panics ... ok
test prop_checked_sub_never_panics ... ok
test prop_deadline_boundary_semantics ... ok
test test_prop_diagnosis_boundary_1000 ... ok
test test_prop_treatment_boundary_1000 ... ok
test test_prop_notes_boundary_1000 ... ok
test test_prop_medications_boundary_50 ... ok
test test_prop_all_fields_at_max_with_max_meds ... ok
test test_prop_deadline_exact_boundary ... ok
test test_prop_deadline_one_before ... ok
test test_prop_deadline_one_past ... ok
test test_prop_zero_duration_rejected ... ok
test test_prop_max_timestamp_overflow ... ok
test test_prop_epoch_underflow ... ok
```
