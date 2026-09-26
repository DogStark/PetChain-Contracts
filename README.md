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
