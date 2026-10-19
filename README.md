# PetChain

PetChain is a decentralized pet health record platform built on Stellar Soroban smart contracts.

## Soroban Resource-Budget Regression Checks

Large workflows can stay logically correct while silently exceeding Soroban instruction,
memory, or storage budgets after a feature addition. To catch this early, the contract
test suite measures representative worst-case calls and compares them against documented
budget snapshots.

### Covered workflows

| Workflow | Representative worst-case call | Budgeted resources |
| --- | --- | --- |
| Registration | Register a pet with the maximum allowed metadata fields | instructions, memory, storage |
| Record | Append a health record to a pet at the maximum record count | instructions, memory, storage |
| Consent | Grant consent to the maximum number of authorized parties | instructions, memory, storage |
| Claim | Submit and settle a claim with the largest allowed payload | instructions, memory, storage |
| Pagination | Page through the largest supported result set | instructions, memory |

### Budget snapshots

Each workflow has a committed budget snapshot under `contracts/test/budgets/`. A snapshot
records the measured `instructions`, `memory`, and `storage` values for that workflow.

### Running the checks in CI

CI runs the budget tests on every pull request. When a workflow exceeds its snapshot, the
job fails and reports the specific resource that regressed, for example:

```
[budget] record: instructions regressed (measured 1_240_000 > budget 1_100_000)
```

The failure names the workflow and the resource (`instructions`, `memory`, or `storage`)
so the regression can be attributed without re-running the suite locally.

### Updating the baseline

Budgets are intentional baselines, not auto-generated values. To update a snapshot after a
reviewed change:

1. Run the budget tests with the baseline-update flag enabled.
2. Review the diff in `contracts/test/budgets/` to confirm the increase is expected.
3. Commit the updated snapshot together with the change that caused it.

Snapshots must only be raised deliberately; an unexplained increase should be treated as a
regression and investigated before merging.

### Fixtures

Budget tests use bounded, realistic fixtures: the largest inputs the contracts are expected
to accept (maximum metadata fields, record counts, consent parties, claim payloads, and
page sizes). Fixtures are capped so the tests stay deterministic and fast while still
exercising the worst case for each workflow.

## Soroban Error Registry

Public contract errors are documented in a deterministic registry so clients can
map a numeric discriminant to its meaning and recovery semantics without parsing
error text. The registry is generated from the contract source (the `#[contracterror]`
enums) rather than hand-maintained, so it stays in sync with the contracts.

Each entry records:

- **code** — the stable numeric discriminant returned on-chain.
- **meaning** — a human-readable description of the failure.
- **retryability** — whether a client may safely retry, and under what conditions.

See [Error Codes](docs/error-codes.md) for the full registry and recovery
guidance. Discriminants are treated as a stable public interface: existing codes
must not change silently, duplicate codes fail CI, and any intentional change
requires updating the generated snapshot and the accompanying intentional-change
test.

## Batch Operation Atomicity Policy

Soroban batch operations in this repository follow a single, documented
atomicity model: **all-or-nothing**. A batch either applies every item or
applies none of them. There is no partial-success mode.

- **Atomicity model.** Each batch is executed as one atomic unit. If any item
  fails validation or execution, the entire batch is rolled back and no state
  change from that batch is committed. Ownership, consent, and custody records
  therefore never expose a partially applied batch.
- **Rollback / error behavior.** On the first failing item the batch aborts and
  returns an error identifying the failing item index and reason. Because the
  whole batch is rolled back, no partial ownership or consent state is
  observable after a failed batch.
- **Per-item results.** Per-item results are only exposed when they are safe:
  on full success (each item's committed result) or as a non-mutating preview
  that performs no writes. Failed batches do not return per-item success
  results, since none of the items were committed.
- **Item limits.** Batch size is bounded by a documented maximum. Batches that
  exceed the limit are rejected before any state is touched, so oversized
  batches cannot partially apply.

## Pagination Policy

All Stellar paginated read endpoints (pet, record, vet, consent, custody, and
activity reads) share a single bounded page-size policy. The policy is
centralized in the contracts crate so every endpoint validates page sizes the
same way and returns identical cursor semantics.

| Parameter | Value |
| --- | --- |
| Minimum page size | `1` |
| Default page size | `20` |
| Maximum page size | `100` |

Rules:

- A page size of `0` (or any value below the minimum) is rejected with a
  deterministic error.
- A page size above the maximum is rejected with a deterministic error.
- Omitting the page size uses the documented default of `20`.
- Every paginated endpoint returns the same cursor semantics: the cursor is an
  opaque token identifying the next page, and an empty/absent cursor starts at
  the first page.
- Existing callers that pass a page size within `[1, 100]` remain compatible.

## Adoption Consent Evidence Commitments

Adoption consent is recorded on-chain as a **commitment**, never as the
underlying applicant document. The contract stores only a canonical digest of
the required consent artifacts, so applicant personal documents are never
persisted in plaintext.

- **Canonical digest.** Each consent artifact is hashed with the contract's
  canonical hashing routine (SHA-256 over the artifact bytes) and the resulting
  digests are combined into a single commitment. The commitment is the only
  value written to persistent storage.
- **Required artifacts.** A consent submission must bind every artifact required
  by the adoption policy. A submission missing a required artifact is rejected
  with a deterministic error and leaves state unchanged.
- **No plaintext storage.** Artifact bytes are consumed for hashing only and are
  never written to contract storage, events, or return values. Only the
  commitment digest is observable on-chain.
- **Authorization.** Submitting or revoking a consent commitment requires the
  same authorization as the existing consent flow; unauthorized callers are
  rejected before any state is touched.
- **Replay and expiry.** A commitment is bound to the adoption and carries an
  expiry. Re-submitting an identical commitment for the same adoption is
  rejected as a replay, and an expired commitment is rejected; both failures
  leave state unchanged.
- **Bounds.** The number of artifacts per submission and the size of each
  artifact are bounded by documented maxima. Oversized submissions are rejected
  before any state is touched.

New error codes, events, and any ABI/storage changes introduced by this feature
are recorded in the [Error Codes](docs/error-codes.md) registry and the
compatibility fixtures, following the existing storage-version conventions.

## Medication Administration Attestation

Scheduled medication doses can be attested by an authorized caregiver as
`administered`, `skipped`, or `refused`. Attestation is recorded on-chain so the
care plan reflects what actually happened to each scheduled dose.

### Authorization

Only a caregiver already authorized for the pet (via the existing consent /
caregiver authorization conventions) may attest a dose. Unauthorized callers are
rejected with a deterministic error and no state is written.

### Inputs

| Field | Bound |
| --- | --- |
| `status` | one of `administered`, `skipped`, `refused` |
| `reason_code` | bounded numeric code; required for `skipped` and `refused` |
| `idempotency_key` | bounded byte length; unique per attestation |

Oversized or invalid inputs are rejected before any state is touched.

### Idempotency and replay

Each attestation carries an idempotency key. Replaying the same key returns the
previously recorded attestation without writing new state, so retries are safe.
A key that has expired is rejected. Failed calls leave storage unchanged.

### Storage and ABI

Attestations are stored under the existing storage-version conventions; the
storage version is unchanged and no existing keys are renumbered. New error
codes and the attestation event are documented in
[Error Codes](docs/error-codes.md).

## Preventive-Care Schedule Records

Routine exams and preventive care (vaccinations, dental checks, wellness visits,
and similar) can be recorded as owner-approved schedules. Each schedule is
versioned so updates are explicit and auditable, and can be cancelled by the
owner.

### Authorization

Only the pet owner may create, update, or cancel a preventive-care schedule.
Unauthorized callers are rejected with a deterministic error and no state is
written.

### Inputs

| Field | Bound |
| --- | --- |
| `care_type` | bounded enum (e.g. `exam`, `vaccination`, `dental`, `wellness`) |
| `interval_days` | bounded positive integer |
| `next_due_at` | bounded ledger timestamp |
| `notes` | bounded byte length |

Oversized or invalid inputs are rejected before any state is touched.

### Versioning

Each schedule carries an explicit version. Creating a schedule starts at version
`1`; every owner-approved update increments the version and records the new
values. Reads return the current version so clients can detect stale copies.

### Cancellation

An owner may cancel a schedule. Cancellation is terminal for that schedule
version: the schedule is marked cancelled and no further updates are accepted.
Cancelling an already-cancelled schedule is rejected with a deterministic error
and leaves state unchanged.

### Replay and expiry

Schedule mutations carry an idempotency key. Replaying the same key returns the
previously recorded result without writing new state. A key that has expired is
rejected. Failed calls leave storage unchanged.

### Storage and ABI

Schedules are stored under the existing storage-version conventions; the storage
version is unchanged and no existing keys are renumbered. New error codes and
the schedule event are documented in [Error Codes](docs/error-codes.md).

## Documentation

- [Architecture](docs/architecture.md)
- [Development](docs/development.md)
- [API Overview](docs/api.md)
- [Error Codes](docs/error-codes.md)
- [Dependency Policy](docs/dependency-policy.md)
- [Security Policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)

## License

MIT

## Handsoff notes

<!-- handsoff-issue-1305 -->
- #1305: [Contracts] Add Stellar storage-key namespace collision tests

<!-- handsoff-issue-1307 -->
- #1307: [Contracts] Add cursor invalidation tests aft

## Documentation

- [Architecture](docs/architecture.md)
- [Development](docs/development.md)
- [API Overview](docs/api.md)
- [Error Codes](docs/error-codes.md)
- [Dependency Policy](docs/dependency-policy.md)
- [Security Policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)

## License

MIT

## Handsoff notes

<!-- handsoff-issue-1305 -->
- #1305: [Contracts] Add Stellar storage-key namespace collision tests

<!-- handsoff-issue-1307 -->
- #1307: [Contracts] Add cursor invalidation tests after deletion and state transitions

<!-- handsoff-issue-1311 -->
- #1311: [Contracts] Add emergency-contact priority and duplicate invariant tests

<!-- handsoff-issue-1312 -->
- #1312: [Contracts] Add insurance claim evidence lifecycle and expiry tests

<!-- handsoff-issue-1313 -->
- #1313: [Contracts] Add Soroban persistent-storage TTL extension failure handling

<!-- handsoff-issue-1188 -->
- #1188: [pet-transfer-adoption] Prevent adoption of nonexistent or inactive pets

<!-- handsoff-issue-1197 -->
- #1197: [stellar-contracts] Add emergency-over

<!-- handsoff-issue-1200 -->
- #1200: [stellar-contracts] Minimize public emergency-profile data

<!-- handsoff-issue-1212 -->
- #1212: [stellar-contracts] Add governance timelock and cancellation policy

<!-- handsoff-issue-1213 -->
- #1213: [stellar-contracts] Secure contract-upgrade authorization and hash binding

<!-- handsoff-issue-1223 -->
- #1223: [backend-2fa] Version encrypted TOTP secret envelopes

<!-- handsoff-issue-1224 -->
- #1224: [backend-2fa] Add online key rotation for encrypted TOTP secrets

<!-- handsoff-issue-1227 -->
- #1227: [backend-2fa] Enforce TOTP replay protection atomically

<!-- handsoff-issue-1261 -->
- #1261: [backend-2fa] Add webhook signature key versioning

<!-- handsoff-issue-1304 -->
- #1304: [Contracts] Add Celo invariant tests for registry pause and recovery behavior

<!-- handsoff-issue-1320 -->
- #1320: [Contracts] Add Celo event indexing compatibility fixtures

<!-- handsoff-issue-1331 -->
- #1331: [Contracts] Add Soroban event privacy regression tests

<!-- handsoff-issue-1332 -->
- #1332: [Contracts] Add Soroban upgrade proposal expiry semantics

<!-- handsoff-issue-1340 -->
- #1340: [Contracts] Add Soroban insurance reserve accounting audit view

<!-- handsoff-issue-1405 -->
- #1405: [Contracts] Add travel certificate lifecycle

<!-- handsoff-issue-1406 -->
- #1406: [Contracts] Add travel readiness evaluation for a pet

<!-- handsoff-issue-1407 -->
- #1407: [Contracts] Add destination-specific travel requirement profiles

<!-- handsoff-issue-1408 -->
- #1408: [Contracts] Add time-limited passport sharing grants

<!-- handsoff-issue-1425 -->
- #1425: [Contracts] Add care-plan access purpose checks

<!-- handsoff-issue-1426 -->
- #1426: [Contracts] Add care-plan deletion and retention rules

<!-- handsoff-issue-1427 -->
- #1427: [Contracts] Add canonical laboratory analyte identifiers

<!-- handsoff-issue-1428 -->
- #1428: [Contracts] Add species-specific laboratory reference ranges

<!-- handsoff-issue-1429 -->
- #1429: [Contracts] Add explicit laboratory unit conversion rules

<!-- handsoff-issue-1430 -->
- #1430: [Contracts] Add laboratory panel completeness metadata

<!-- handsoff-issue-1432 -->
- #1432: [Contracts] Add laboratory result verification attestations

<!-- handsoff-issue-1433 -->
- #1433: [Contracts] Add sample collection and processing timestamps

<!-- handsoff-issue-1434 -->
- #1434: [Contracts] Add laboratory result provenance for imported data

<!-- handsoff-issue-1435 -->
- #1435: [Contracts] Add laboratory panel trend comparison API

<!-- handsoff-issue-1437 -->
- #1437: [Contracts] Add laboratory result privacy tiers

<!-- handsoff-issue-1438 -->
- #1438: [Contracts] Add lab data validation fixtures across supported species

<!-- handsoff-issue-1439 -->
- #1439: [Contracts] Add vaccine batch and manufacturer provenance

<!-- handsoff-issue-1440 -->
- #1440: [Contracts] Add multi-dose vaccination series tracking

<!-- handsoff-issue-1441 -->
- #1441: [Contracts] Add vaccine schedule policy versions

<!-- handsoff-issue-1442 -->
- #1442: [Contracts] Add vaccine adverse-event reports

<!-- handsoff-issue-1443 -->
- #1443: [Contracts] Add certificate replacement with supersession links

<!-- handsoff-issue-1444 -->
- #1444: [Contracts] Add vaccination administration-site metadata

<!-- handsoff-issue-1445 -->
- #1445: [Contracts] Add vaccination certificate offline verification payload

<!-- handsoff-issue-1446 -->
- #1446: [Contracts] Add vaccination certificate batch verification

<!-- handsoff-issue-1447 -->
- #1447: [Contracts] Add vaccine product registry governance

<!-- handsoff-issue-1448 -->
- #1448: [Contracts] Add vaccination schedule reconciliation preview

<!-- handsoff-issue-1449 -->
- #1449: [Contracts] Add foster custody agreements with automatic expiry

<!-- handsoff-issue-1451 -->
- #1451: [Contracts] Add shelter intake attestation

<!-- handsoff-issue-1454 -->
- #1454: [Contracts] Add adoption trial period policy

<!-- handsoff-issue-1455 -->
- #1455: [Contracts] Add adoption fee accounting receipt

<!-- handsoff-issue-1456 -->
- #1456: [Contracts] Add adopter eligibility attestations

<!-- handsoff-issue-1457 -->
- #1457: [Contracts] Add shelter authority delegation

<!-- handsoff-issue-1458 -->
- #1458: [Contracts] Add custody dispute evidence bundle references

<!-- handsoff-issue-1459 -->
- #1459: [Contracts] Add ownership recovery after failed adoption

<!-- handsoff-issue-1463 -->
- #1463: [Contracts] Add verified sighting reporter attestations

<!-- handsoff-issue-1464 -->
- #1464: [Contracts] Add private sighting location reveal authorization

<!-- handsoff-issue-1465 -->
- #1465: [Contracts] Add lost-pet alert escalation rules

<!-- handsoff-issue-1466 -->
- #1466: [Contracts] Add lost-pet alert radius and scope validation

<!-- handsoff-issue-1467 -->
- #1467: [Contracts] Add found-pet intake and reunification workflow

<!-- handsoff-issue-1468 -->
- #1468: [Contracts] Add responder organization credentials
