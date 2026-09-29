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

## Documentation

- [Architecture](docs/architecture.md)
- [Development](docs/development.md)
- [API Overview](docs/api.md)
- [Error Codes](docs/error-codes.md)
- [Security Policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)

## License

MIT

## Handsoff notes

<!-- handsoff-issue-1188 -->
- #1188: [pet-transfer-adoption] Prevent adoption of nonexistent or inactive pets

<!-- handsoff-issue-1197 -->
- #1197: [stellar-contracts] Add emergency-override least-privilege scopes

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
