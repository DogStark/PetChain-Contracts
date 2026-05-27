# Development Notes

## Soroban Gas Profiling

Budget measurements are taken in unit tests with `env.cost_estimate().budget()` after fixture setup, so setup cost is excluded and only the target call is tracked.

### Top 10 Profiled Functions

| Function | Baseline Instructions | Optimized Instructions | Memory Bound | Notes |
|---|---:|---:|---:|---|
| `get_behavior_by_type` | 17,500,000 | 14,000,000 | 2,500,000 | Avoids building an intermediate full history vector before filtering. |
| `get_activity_stats` | 15,000,000 | 12,000,000 | 2,500,000 | Iterates indexed storage directly instead of materializing full history. |
| `get_consent_history_page` | 12,000,000 | 9,000,000 | 2,000,000 | Reads only requested page indices instead of all consent history. |
| `get_activity_summary` | 15,000,000 | 12,000,000 | 2,500,000 | Shares the direct indexed iteration pattern. |
| `get_behavior_history` | 11,000,000 | 11,000,000 | 2,500,000 | Baseline retained. |
| `get_activity_history` | 10,500,000 | 10,500,000 | 2,500,000 | Baseline retained. |
| `get_pet_insurance_claims` | 10,000,000 | 10,000,000 | 2,500,000 | Baseline retained. |
| `get_training_milestones` | 9,500,000 | 9,500,000 | 2,500,000 | Baseline retained. |
| `get_active_consents` | 9,000,000 | 9,000,000 | 2,000,000 | Baseline retained. |
| `get_vet_reviews` | 8,500,000 | 8,500,000 | 2,000,000 | Baseline retained. |

### Regression Bounds

`gas_profile_tests` in `stellar-contracts/src/lib.rs` asserts that optimized instruction and memory costs remain under the documented optimized bounds for `get_behavior_by_type`, `get_activity_stats`, and `get_consent_history_page`.
# Development

## Prerequisites

- Rust toolchain
- Stellar CLI for contract work
- PostgreSQL if you want to exercise the backend database store
- Redis only for the optional Redis rate-limiter tests

## Common Commands

### Stellar contracts

```bash
cd stellar-contracts
cargo fmt
cargo test
```

### Backend 2FA crate

```bash
cd backend-2fa
cargo fmt
cargo test
```

## Notes

- The backend test suite skips Redis integration tests unless `REDIS_URL` is set.
- The repo contains two independent Rust crates, so build and test them separately.
- Use `.env.example` as the starting point for local environment variables.
