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
