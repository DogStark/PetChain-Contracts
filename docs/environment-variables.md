# Environment Variables Reference — backend-2fa

Single authoritative list of every environment variable that affects the
`petchain-2fa` service at runtime. Variables are grouped by subsystem.

## Database

| Variable | Required | Default | Description |
|---|---|---|---|
| `DATABASE_URL` | **yes** | — | PostgreSQL connection string (`postgresql://user:pass@host:5432/db`). |
| `DB_POOL_MIN` | no | `1` | Minimum number of connections kept in the pool. |
| `DB_POOL_MAX` | no | `10` | Maximum number of connections the pool will open. |
| `DB_POOL_ACQUIRE_TIMEOUT_SECS` | no | `30` | Seconds to wait for a connection before timing out. |
| `DB_POOL_CONNECT_TIMEOUT_SECS` | no | `30` | Seconds allowed for opening a new connection. The pool applies the tighter of this and `DB_POOL_ACQUIRE_TIMEOUT_SECS`. |
| `POOL_STATS_ENABLED` | no | `0` | Set to `1` to expose pool utilisation in the metrics/health endpoint. |

## Secret Provider

| Variable | Required | Default | Description |
|---|---|---|---|
| `SECRET_PROVIDER` | no | `env` | How secrets are resolved: `env` reads from environment variables, `aws` delegates to the AWS Secrets Manager provider. |
| `AWS_SECRETS_JSON` | only when `SECRET_PROVIDER=aws` | — | JSON object mapping secret names to values (e.g. `{"DATABASE_URL":"postgresql://..."}"`). Used by the AWS provider for local/test environments. |
| `TOTP_ENCRYPTION_KEY` | **yes in production** | — | 32-byte AES-256-GCM key as 64 hex characters, used to encrypt TOTP secrets at rest. Resolved through the secret provider, so it is read from the environment when `SECRET_PROVIDER=env` and from AWS Secrets Manager when `SECRET_PROVIDER=aws`. **If unset or malformed, the service silently stores TOTP secrets in plaintext.** Required before running `migrations/005_encrypt_existing_secrets.sql`'s re-encryption step. |
| `AWS_REGION` | only when `SECRET_PROVIDER=aws` | — | AWS region for Secrets Manager. Read by the AWS SDK default configuration chain, not by this crate directly. |
| `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` | only when `SECRET_PROVIDER=aws` and not using an instance/task role | — | AWS credentials. Read by the AWS SDK default credential chain, not by this crate directly. |

## Two-Factor

| Variable | Required | Default | Description |
|---|---|---|---|
| `TOTP_ISSUER` | no | `PetChain` | Issuer name shown in authenticator apps. Empty or whitespace-only values fall back to `PetChain`. |

## Leaderboard

| Variable | Required | Default | Description |
|---|---|---|---|
| `LEADERBOARD_DECAY_LAMBDA` | no | `0.1` | Exponential decay lambda used for time-weighted leaderboard scoring. Higher values discount older activity more aggressively. |
| `LEADERBOARD_MAX_CONN_PER_IP` | no | `5` | Maximum concurrent WebSocket connections allowed from a single client IP. |
| `LEADERBOARD_BROADCAST_CAPACITY` | no | `256` | Capacity of the in-memory broadcast channel that fans score updates out to WebSocket clients. |
| `LEADERBOARD_WS_JWT_SECRET` | only when clients send JWT bearer tokens | — | HS256 secret used to verify JWT tokens on the leaderboard WebSocket. Tokens are rejected if this is unset. |
| `LEADERBOARD_WS_AUTH_TOKEN` | no | — | Shared legacy token for the leaderboard WebSocket. **If unset, any non-empty opaque token is accepted** (development mode). Always set this in production. |

## Webhooks

| Variable | Required | Default | Description |
|---|---|---|---|
| `WEBHOOK_LOG_MAX_ENTRIES` | no | `1000` | Maximum number of delivery-log entries retained in memory per webhook subscription. |

## Redis

| Variable | Required | Default | Description |
|---|---|---|---|
| `REDIS_URL` | no | — | Connection URL for a Redis instance. Required only when using the Redis-backed rate limiter; the service starts without Redis if unset. |
| `RATE_LIMITER_URL` | no | — | Read by the two-factor handlers at startup, but **currently has no effect**: the value is discarded and the in-memory rate limiter is always used. Do not rely on it to configure rate limiting. |

## Observability

| Variable | Required | Default | Description |
|---|---|---|---|
| `RUST_LOG` | no | — | Standard `tracing-subscriber` / `env_filter` directive (e.g. `info`, `petchain_2fa=debug`). Controls log verbosity. |
| `TRACE_CONTEXT_AUTOGENERATE` | no | enabled | Set to `0` to stop the service from generating a trace context when an incoming request has none. Any other value, or unset, keeps generation enabled. |

## Test-only

Read only by the test suite. Not needed to run the service.

| Variable | Required | Default | Description |
|---|---|---|---|
| `REDIS_URL` | no | — | Redis instance used by Redis-backed tests. Tests that need it are skipped when unset. |
| `AWS_SECRETS_INTEGRATION_TEST` | no | — | Set to any value to enable the live AWS Secrets Manager integration test. |
| `AWS_TEST_SECRET_NAME` | only with `AWS_SECRETS_INTEGRATION_TEST` | — | Name of the existing secret the integration test reads. |

## Build-time

These are injected by `build.rs` and read with `env!()` / `option_env!()` at
compile time. They do not need to be set by operators at runtime.

| Variable | Set by | Description |
|---|---|---|
| `GIT_SHA` | `build.rs` | Short git commit hash baked into the binary for the `/build-info` metrics label. Falls back to `"unknown"` when git is unavailable. |
| `CARGO_PKG_VERSION` | Cargo | Crate version from `Cargo.toml`, used alongside `GIT_SHA` in build-info metrics. |
