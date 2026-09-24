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
