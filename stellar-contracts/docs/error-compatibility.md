# Contract Error Compatibility Policy

Clients match on the **numeric** code of a contract error, so a code must
keep one meaning for the lifetime of the contract. This policy covers every
`#[contracterror]` enum in `stellar-contracts` (Issue #1255).

## Source of truth

[`error-codes.json`](../error-codes.json) is the client mapping fixture. For
each enum it lists every error's stable `code`, `name`, and `retry` guidance,
plus `retired` codes that must never be reused.

| `retry` | Client behaviour |
|---|---|
| `never` | Permanent for this request: fix the input, caller, or target. |
| `after_state_change` | A precondition is not met yet (time window, approval, verification, nonce). Re-read state and retry once it changes. |
| `backoff` | Transient throttling. Retry with exponential backoff. |
| `treat_as_success` | Idempotent replay: the effect already exists. Do not retry. |

## Rules

1. **Append only.** A new error takes an unused code. Never renumber an
   existing variant or give a code a new meaning.
2. **Never delete a published code.** To retire an error, remove the variant
   and move its entry from `errors` to `retired` in `error-codes.json`; the
   code stays reserved forever.
3. **Every new error needs** an entry in `error-codes.json` (with `retry`
   guidance) and a `CHANGELOG.md` entry that names it.
4. **New error enums** must be registered in `error-codes.json`.

## CI enforcement

The `error-compat` job runs:

```bash
python3 stellar-contracts/scripts/check_error_compat.py --base <base-branch error-codes.json>
```

It fails on removed, reused, or renumbered discriminants, errors missing
from the fixture or without valid retry guidance, published codes dropped or
reassigned in the fixture, and new errors without a `CHANGELOG.md` entry.
Run it locally without `--base` to check sources against the fixture.

Representative errors for each public module are pinned by tests in
`src/test_error_registry.rs` and
`contracts/pet-transfer-adoption/src/test_error_codes.rs`.

## Insurance enrollment and coverage snapshots

The insurance enrollment workflow (Issue #1470) and coverage schedule
snapshots (Issue #1472) add errors to the insurance contract's
`#[contracterror]` enum. They follow the same append-only rules: each code
below is new, unused, and reserved forever once published.

| Code | Name | `retry` | Meaning |
|---|---|---|---|
| `TBD` | `EnrollmentNotFound` | `never` | No enrollment exists for the given id. |
| `TBD` | `EnrollmentStateInvalid` | `never` | The requested transition is not allowed from the enrollment's current state (draft, submitted, under-review, accepted, declined). |
| `TBD` | `EnrollmentUnauthorized` | `never` | Caller is not the enrollment owner or an authorized reviewer. |
| `TBD` | `EnrollmentAlreadyDecided` | `treat_as_success` | The enrollment was already accepted or declined; replaying the decision is idempotent. |
| `TBD` | `EnrollmentExpired` | `after_state_change` | The submission window for the enrollment has closed. |
| `TBD` | `PolicyVersionMismatch` | `after_state_change` | The referenced policy version is not the current one; re-read the policy and retry. |
| `TBD` | `CoverageSnapshotMissing` | `never` | No immutable coverage/exclusion snapshot is bound to the enrollment. |
| `TBD` | `CoverageSnapshotExists` | `treat_as_success` | A snapshot is already bound to the enrollment; the write is an idempotent replay. |
| `TBD` | `CoverageSnapshotTooLarge` | `never` | The coverage or exclusion list exceeds the bounded maximum size. |

The concrete numeric codes are assigned in the implementation and recorded in
`error-codes.json`; this table documents intent and retry guidance only. When
the codes land, replace each `TBD` with the assigned number and add the
matching `CHANGELOG.md` entry naming the error, as required by rule 3.

### Events and storage

- Enrollment state transitions emit an event carrying the enrollment id, the
  previous and new state, and the bound policy version.
- Accept/decline decisions record the deciding policy version so a later
  policy change cannot silently alter an existing contract.
- Coverage and exclusion snapshots are written once at enrollment and are
  immutable thereafter; attempts to overwrite them fail with
  `CoverageSnapshotExists` (idempotent replay) or `CoverageSnapshotMissing`
  (no snapshot to read).
- Snapshot input sizes are bounded; oversized lists fail with
  `CoverageSnapshotTooLarge` and leave state unchanged.

### Compatibility fixtures

Because these errors extend a public enum, `error-codes.json` must list every
new code with its `retry` guidance, and any ABI/storage compatibility fixture
covering the insurance contract must be updated in the same change.
