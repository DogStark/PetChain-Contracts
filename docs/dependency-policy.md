# Dependency Policy

> Tracks issue [#1349](../../issues/1349) — Add dependency source and license policy for Rust and npm artifacts.

This document describes the approved licenses, approved registry sources, and the exception/review process for all production dependencies in this repository.

---

## Scope

The policy covers two artifact types:

| Artifact type | Manifest | Lock file | Policy tool |
| --- | --- | --- | --- |
| Rust (workspace) | `Cargo.toml`, `*/Cargo.toml` | `Cargo.lock` | `cargo-deny` (`deny.toml`) |
| npm (celo-contracts) | `celo-contracts/package.json` | `celo-contracts/package-lock.json` | `check-licenses.js` (`allowed-licenses.json`) |

---

## Approved licenses

### Rust

The following SPDX identifiers are approved for all workspace dependencies. The authoritative list is `deny.toml` under `[licenses].allow`.

| License | Notes |
| --- | --- |
| MIT | |
| Apache-2.0 | |
| Apache-2.0 WITH LLVM-exception | Rust standard library exception |
| BSD-2-Clause | |
| BSD-3-Clause | |
| ISC | |
| MPL-2.0 | |
| Unicode-3.0 | Unicode data crates |
| Unicode-DFS-2016 | Unicode data crates |
| Zlib | |

### npm

The following SPDX identifiers are approved for all `celo-contracts` packages. The authoritative list is `celo-contracts/allowed-licenses.json` under `allowedLicenses`.

| License | Notes |
| --- | --- |
| MIT | |
| Apache-2.0 | |
| BSD-2-Clause | |
| BSD-3-Clause | |
| ISC | |
| Unlicense | |
| CC0-1.0 | |
| 0BSD | |
| BlueOak-1.0.0 | |
| Python-2.0 | |

---

## Disallowed licenses

The following licenses are **explicitly disallowed** for both Rust and npm production artifacts:

| License | Reason |
| --- | --- |
| GPL-2.0, GPL-3.0, GPL-3.0-only | Copyleft — incompatible with the MIT production license |
| LGPL-2.0, LGPL-2.1, LGPL-3.0 | Weak copyleft — may impose distribution obligations |
| AGPL-3.0 | Network copyleft — incompatible with SaaS deployment |
| SSPL-1.0 | Service copyleft — incompatible with commercial deployment |
| Commons-Clause | Source-available restriction — not OSI-approved |
| BUSL-1.1 | Time-limited restriction — changes license without review |

---

## Approved sources

### Rust

All crates must be fetched from the official crates.io index:

```
https://github.com/rust-lang/crates.io-index
```

Any other registry or git source requires an explicit entry in `deny.toml` under `[sources].allow-git` or `allow-registry` with a pinned revision comment and a PR review. See the [exception process](#exception-process) below.

### npm

All packages must be resolved from the public npm registry:

```
https://registry.npmjs.org/
```

This is enforced by `celo-contracts/.npmrc`. Private or alternative registries are not approved.

---

## CI enforcement

Two workflows enforce this policy:

### `dep-policy.yml` (primary)

Runs on every PR and push that touches a manifest, lock file, or policy file. Also runs weekly.

| Job | What it checks |
| --- | --- |
| `rust-dep-policy` | Runs `cargo deny check` — advisories, licenses, bans, sources |
| `npm-license-policy` | Runs `npm run license:check` — SPDX license per package |
| `dep-policy-fixtures` | Validates the intentionally-disallowed test fixtures |
| `lockfile-reproducibility` | Asserts that `npm ci` and `cargo fetch --locked` do not modify the lock files |

### `cargo-deny.yml` (fast feedback)

Runs on every push and PR to `main`, plus weekly. Provides fast Rust-specific feedback for PRs that do not touch npm files.

### Violation output format

Both tools are configured to include the **dependency name, version, source URL, and violation type** in every error line so failures are immediately attributable without re-running locally.

**cargo-deny example:**
```
error[L002]: license 'GPL-3.0-only' is not in the allow list
  --> evil-gpl-crate v1.0.0 (registry+https://github.com/rust-lang/crates.io-index)
```

**check-licenses.js example:**
```
[license-check] FAIL — 1 license violation(s):
  evil-gpl-package@2.0.0  license=GPL-3.0-only  source=https://registry.npmjs.org/evil-gpl-package  violation=disallowed-license
```

---

## Exception process

Exceptions are versioned entries in the policy files. They are not informal overrides — each one must go through the PR review cycle below.

### How to add an exception

1. **Identify the violation.** Run the policy check locally:
   - Rust: `cargo deny check`
   - npm: `npm run license:check` (from `celo-contracts/`)

2. **Evaluate the risk.** Confirm the dependency, version, license, and source are acceptable for production use.

3. **Add a versioned exception** in the appropriate file:

   **Rust (deny.toml)**
   ```toml
   [[licenses.exceptions]]
   allow = ["LGPL-2.1-only"]
   name = "some-crate"
   version = "=1.2.3"
   # reason: #NNNN — reviewed YYYY-MM-DD, expires YYYY-MM-DD
   ```

   **npm (celo-contracts/allowed-licenses.json)**
   ```json
   {
     "name": "some-package",
     "version": "^1.0.0",
     "license": "LGPL-2.1",
     "reason": "Transitive dependency of hardhat-toolbox; LGPL limited to tooling, not production artifacts.",
     "reviewedDate": "YYYY-MM-DD",
     "expires": "YYYY-MM-DD",
     "issue": "#NNNN"
   }
   ```

4. **Open a PR.** The PR description must include:
   - The crate/package name and version
   - The license or source being excepted
   - The reason the dependency is required
   - The expiry date (maximum one year from the review date)

5. **Get approval** from a repository maintainer.

### Expiry policy

Every exception carries an `expires` date. The CI job fails on expired exceptions so they cannot persist indefinitely without re-review. The default expiry window is **one year** from the review date. Exceptions for actively-maintained crates on major dependency paths may be renewed for another year upon re-review.

---

## Lockfile reproducibility

Production builds are lockfile-reproducible:

- **Rust:** `cargo build --locked` and `cargo test --locked` are used in all CI jobs. This prevents Cargo from resolving newer patch versions during CI.
- **npm:** `npm ci` is used exclusively in CI. This installs exactly the versions recorded in `package-lock.json` and fails if the lock file is out of date.
- **Verification:** The `lockfile-reproducibility` job in `dep-policy.yml` asserts that neither lock file is modified by a clean install, failing the build if any drift is detected.

To update a lock file intentionally, run `cargo update` or `npm update` locally, review the diff, and commit the updated lock file in its own PR with a clear explanation of what changed and why.

---

## Policy test fixtures

The `fixtures/dep-policy/` directory contains intentionally-disallowed examples that document what a violation looks like and are validated by the `dep-policy-fixtures` CI job:

| Fixture | Purpose |
| --- | --- |
| `disallowed-rust-license.toml` | Documents a cargo-deny license violation (`GPL-3.0-only`) and source violation |
| `disallowed-npm-license.json` | Documents a check-licenses.js violation (`GPL-3.0-only`) with expected output |

These fixtures are integrity-tested in CI by `celo-contracts/scripts/test-dep-policy-fixtures.js` to ensure they stay consistent with the live policy files.

---

## Banned crates

The following Rust crates are explicitly banned from the dependency graph (see `deny.toml`):

| Crate | Reason |
| --- | --- |
| `openssl`, `openssl-sys` | Prefer `rustls`; avoids system-TLS coupling and duplicate native deps |
| `native-tls` | Prefer `rustls` for a consistent, auditable, memory-safe TLS stack |
| `time < 0.3.36` | RUSTSEC-2024-0370 / CVE-2024-42304 |
| `atty` | RUSTSEC-2021-0145: soundness issue on Windows; use `is-terminal` instead |
| `ansi_term` | Unmaintained (RUSTSEC-2021-0139); use `owo-colors` or similar |

To ban a new crate, add a `[[bans.deny]]` entry with a `reason` field and open a PR.
