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
| `POOL_STATS_ENABLED` | no | `0` | Set to `1` to expose pool utilisation in the metrics/health endpoint. |

## Secret Provider

| Variable | Required | Default | Description |
|---|---|---|---|
| `SECRET_PROVIDER` | no | `env` | How secrets are resolved: `env` reads from environment variables, `aws` delegates to the AWS Secrets Manager provider. |
| `AWS_SECRETS_JSON` | only when `SECRET_PROVIDER=aws` | — | JSON object mapping secret names to values (e.g. `{"DATABASE_URL":"postgresql://..."}"`). Used by the AWS provider for local/test environments. |

## Leaderboard

| Variable | Required | Default | Description |
|---|---|---|---|
| `LEADERBOARD_DECAY_LAMBDA` | no | `0.1` | Exponential decay lambda used for time-weighted leaderboard scoring. Higher values discount older activity more aggressively. |

## Webhooks

| Variable | Required | Default | Description |
|---|---|---|---|
| `WEBHOOK_LOG_MAX_ENTRIES` | no | `1000` | Maximum number of delivery-log entries retained in memory per webhook subscription. |

## Redis

| Variable | Required | Default | Description |
|---|---|---|---|
| `REDIS_URL` | no | — | Connection URL for a Redis instance. Required only when using the Redis-backed rate limiter; the service starts without Redis if unset. |

## Observability

| Variable | Required | Default | Description |
|---|---|---|---|
| `RUST_LOG` | no | — | Standard `tracing-subscriber` / `env_filter` directive (e.g. `info`, `petchain_2fa=debug`). Controls log verbosity. |

## Build-time

These are injected by `build.rs` and read with `env!()` / `option_env!()` at
compile time. They do not need to be set by operators at runtime.

| Variable | Set by | Description |
|---|---|---|
| `GIT_SHA` | `build.rs` | Short git commit hash baked into the binary for the `/build-info` metrics label. Falls back to `"unknown"` when git is unavailable. |
| `CARGO_PKG_VERSION` | Cargo | Crate version from `Cargo.toml`, used alongside `GIT_SHA` in build-info metrics. |
