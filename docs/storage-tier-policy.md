# Storage tier policy (`stellar-contracts`)

This document inventories how `stellar-contracts/src/lib.rs` uses Soroban's
storage tiers and why, so future keys are placed consistently rather than
by habit.

Soroban exposes three tiers, reached via `env.storage().instance()`,
`.persistent()`, and `.temporary()`:

- **Instance** — lives and expires alongside the contract instance itself;
  bumping the instance's TTL (which every invocation already does for
  active contracts) extends every instance-tier entry at once, for free.
  Cheapest to read/write per entry, but all instance data shares one TTL.
- **Persistent** — has its own per-key TTL, must be bumped independently,
  and is the right place for data that either (a) grows without bound and
  would otherwise inflate every instance-tier read/write, or (b) needs a
  lifetime independent of the contract's own instance bump cadence.
- **Temporary** — cheapest, expires quickly, and is only appropriate for
  data with no correctness requirement to survive past a short window
  (e.g. request-scoped nonces or replay-protection windows shorter than a
  ledger's practical TTL). **Not currently used** by this contract — see
  "Candidates for temporary storage" below.

## Current inventory

| Data class | Examples | Tier | Rationale |
|---|---|---|---|
| Core records | `Pet`, `Vet`, `InsurancePolicy`, `MultisigConfig` | Instance | Read on almost every invocation touching that entity; small, bounded size; benefits from the free TTL bump every active call already performs. |
| Counts & sequence cursors | `PetCount`, `PetCountByOwner`, `SpeciesPetCount`, `AccessGrantCount` | Instance | Single scalar per key, always read/written alongside the record they count; no reason to diverge from the record's tier. |
| Indexes | `OwnerPetIndex`, `SpeciesPetIndex`, `AccessGrantIndex`, `PetTreatmentIndex` (and the other `*Index` families) | Instance | Small per-entry (a single id), read in the same call path as the record/count they index; keeping them on the same tier as the count avoids a TTL-consistency gap where the count says N entries exist but an individual index slot has already expired. |
| Accumulating logs | Access logs (`access_logs`), `EmergencyAccessLogs`, moderation/audit trails | Persistent | Unbounded-ish, append-heavy collections that are read far less often than the core record; putting them on the instance tier would mean every instance TTL bump (and every instance read cost calculation) scales with log size instead of staying flat. |
| Long-lived derived state | `ActivityStreak` | Persistent | Explicitly documented in-line (see `record_activity`) as needing to survive independently of the instance's own bump cadence, since streaks must persist across gaps in activity without being tied to unrelated instance writes. |

## Candidates for temporary storage

No current key class is a good fit for the temporary tier: nonces here are
consumed via `consume_caller_nonce` and stored on the instance tier because
they must remain valid for as long as the associated pet/owner record does
(there is no short, fixed window after which a stale nonce becomes provably
safe to forget). If a future feature introduces genuinely short-lived,
disposable state — e.g. a rate-limit window scoped to a single ledger close
— it belongs on the temporary tier rather than instance or persistent.

## Enforcement

`stellar-contracts/src/test_storage_tier_policy.rs` pins the tier used by
one representative key from each class above, so a future change that
moves a key to a different tier without updating this document will fail
that test and prompt a deliberate policy decision instead of a silent
drift.
