# Changelog

All notable changes to the PetChain-Contracts project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Contract error compatibility policy (`stellar-contracts/docs/error-compatibility.md`): stable codes, names and retry guidance for every contract error in `stellar-contracts/error-codes.json`, enforced in CI by `scripts/check_error_compat.py` (removed, reused or renumbered codes; new errors need a fixture and changelog entry)
- Storage-rent and cleanup observability: read-only `get_storage_metrics` reports per-pet quota usage, configured cap, remaining headroom and the soft-deleted medical-record cleanup backlog (counts only, no medical content), scanned in bounded, resumable pages
- Cross-contract interface compatibility tests: CI deploys the main, ownership/adoption/escrow and vet-registry contracts into one local Soroban environment and checks shared authorization, errors, events and state transitions, with a fixture that fails on signature or enum drift (`stellar-contracts/docs/cross-contract-compat.md`)
- Pull request template (`.github/PULL_REQUEST_TEMPLATE.md`) to standardize contributor submissions
- Claim-document status/version semantics (#1341): documents are `Active`, `Superseded` or `Revoked`; revoked documents stay auditable but their digest is burned and can never back a new approval; settled claims are immutable and `settlement_has_revoked_documents` flags later revocations
- Breeding eligibility and cooldown invariants (#1342): `record_mating` enforces active status, sex/species compatibility, minimum age, per-parent cooldowns (inclusive at the exact boundary), COI threshold and both owners' consent through one evaluator (`check_breeding_eligibility`); replays of the same mating are idempotent
- Health-score input provenance (#1343): `record_health_score` binds each score to a canonical input digest, algorithm version, scorer and timestamp; identical inputs are deterministic and no algorithm version can overwrite another's history
- Confirmed, audited data-retention purge (#1344): `purge_records_confirmed` requires owner/admin role, a per-account nonce and a scope-binding confirmation digest, honors admin purge holds, and emits an audit event with only a count and records digest
- Canonical chain-of-custody digest (`get_custody_chain_digest`) so consumers can prove a returned custody history is complete and ordered: SHA-256 hash chain over domain, version, pet ID, sequence, and every entry in canonical order, with published test vectors and tamper cases
- Repaired pre-existing `stellar-contracts` build breakage (the crate did not compile at `main`): removed a duplicated `MAX_PREREQUISITES` const and duplicated `ContractError` variants, restored `ProposalNotFound = 47` and `StaleMigration = 169`, added the missing `TrainingMilestone::prerequisites` field, fixed ~20 `safe_increment` call sites, and refreshed the stale ABI snapshot
- Soroban storage schema migration fixtures: old-version snapshot fixtures for core persistent-storage domains (keys and value shapes) plus fixture-based upgrade/read tests. Current code reads every supported fixture; unsupported schema versions fail with an explicit `UnsupportedSchemaVersion` error instead of a silent misread; migrations are asserted not to widen unbounded values (no unbounded `Vec`/`String` growth). A schema-version CI check guards the fixtures against drift.
- Soroban batch-operation atomicity policy: batch writes are **all-or-nothing**. A batch is validated and applied as a single unit; if any item fails, the entire batch is rolled back so no partial ownership or consent state is ever observable. Per-item results are returned only on full success (or via a non-mutating preview), and batch size is bounded by a documented maximum (`MAX_BATCH_ITEMS`).
- Laboratory result amendment workflow (#1431): the originating verified clinic can issue a corrected result via `amend_lab_result`, which stores a new `LabResultAmendment` that links to the original result by `(pet_id, result_id)` and never overwrites it. The original record is marked `Amended` (status only) and remains readable; amendments are append-only, bounded (`MAX_AMENDMENT_REASON_LEN`, `MAX_AMENDMENTS_PER_RESULT`), and replay-safe via a per-result amendment nonce. New errors `LabResultNotFound` (172), `LabResultNotVerified` (173), `AmendmentNotAuthorized` (174), `AmendmentReasonTooLong` (175), `AmendmentLimitReached` (176) and `AmendmentReplay` (177); new event `LabResultAmendedEvent`; storage is additive (no schema bump required).
- Abnormal-result acknowledgement workflow (#1436): a verified vet can acknowledge an abnormal lab result and record a follow-up disposition (`Pending`, `Scheduled`, `Referred`, `Resolved`) without mutating the measured value. Acknowledgements are keyed by `(pet_id, result_id)`, store the acknowledging vet, timestamp, bounded disposition note and an optional follow-up time, and are idempotent per vet; the measured value, units and reference range are never rewritten. New errors: `ResultNotFound` (172), `ResultNotAbnormal` (173), `AcknowledgementAlreadyExists` (174), `InvalidDispositionNote` (175). New event: `AbnormalResultAcknowledgedEvent`. Storage is additive (new `AbnormalResultAck` key), so no schema bump is required.

### Changed / Migration notes (certificates, vet credentials, duration safety, idempotency)

- `CertificateAnchor` is now paired with a separate `CertificateLifecycle` record
  (`cert_id`, `issue_time`, `expiry`, `revoked`, `revoked_at`,
  `revocation_reason`). The change is storage-additive — existing anchors are
  still readable. Operators should re-anchor certificates whose cert_id / expiry
  / revocation tracking is required. `verify_certificate` now returns `false`
  for expired or revoked certificates. Run `migrate_storage` to v3.0.0 to record
  the schema bump.

- Vet credentials now support an optional expiry (`VetCredentialsExpiry`
  storage key). `verify_vet` continues to produce a perpetual (no-expiry) vet;
  use `verify_vet_with_expiry` to bind an expiry. `is_verified_vet` now rejects
  vets whose credentials have expired. All privileged medical operations that
  previously gated on vet verification (`add_vaccination`, `add_medical_record`,
  `anchor_certificate`) now enforce credential validity.

- `get_upcoming_vaccinations` and groomer-slot overlap checks use saturating
  duration arithmetic to prevent ledger-timestamp panics near `u64::MAX`.

- `anchor_certificate_idempotent` provides replay-safe certificate anchoring:
  re-submitting the same `(pet_id, vaccination_id, cert_hash)` returns the
  existing `cert_id`; a conflicting hash raises `CertificateHashConflict`.

- New errors: `CertificateNotFound` (47), `CertificateRevoked` (48),
  `CertificateExpired` (49), `VetCredentialsExpired` (50),
  `CertificateHashConflict` (51).
  Correction: `CertificateNotFound` is **171**, because 47 is
  `ProposalNotFound`. These variants were lost in a merge and have been
  restored (see `stellar-contracts/docs/abi-migrations.md`).

## [0.1.0] - 2024-XX-XX

### Added

#### Core Pet Management

- Pet registration with unique IDs on the Stellar blockchain
- Pet profile storage including name, species, and ownership
- Pet status updates (active/inactive) with owner authentication
- Pet information retrieval by ID

#### Medical Records System

- Store and retrieve pet medical records on-chain
- Paginated medical record queries (`get_pet_medical_records`)
- Search medical records functionality
- Complete medical history tracking per pet

#### Vaccination Tracking

- Record and track pet vaccinations
- Vaccination history retrieval
- Upcoming vaccination reminders
- Overdue vaccination detection
- Vaccination status validation (`is_vaccination_current`)

#### Grooming Tracking System

- Add detailed grooming records with service type, groomer, cost, and notes
- Automatic calculation of next grooming due date (default: 60 days)
- Complete grooming history retrieval per pet
- Grooming expense tracking across all sessions
- Next grooming date estimation

#### Insurance System

- Add insurance policies to pets with provider, coverage type, premium, and expiry
- Retrieve pet insurance information
- Update insurance policy active status
- `InsuranceAddedEvent` and `InsuranceUpdatedEvent` events

#### Access Control & Privacy

- Owner-based access control for all state-changing operations
- Privacy levels for pet data: Public, Restricted, and Private
- Access grant system for authorized users
- `get_authorized_users` and access validation
- Consent pagination for managing data access permissions

#### Security & Encryption

- Encryption for sensitive pet and owner fields
- Deterministic key derivation using domain separator and contract address
- **Nonce uniqueness fix**: replaced fixed `[0u8; 12]` nonce with timestamp + counter based unique nonces to prevent replay attacks
- Decryption system with privacy level enforcement
- Input validation for all public functions
- Overflow protection on arithmetic operations

#### Emergency & Dispute Systems

- Emergency contacts management per pet
- Emergency override functionality for critical situations
- Dispute system for handling ownership or data conflicts

#### Transfer & Ownership

- Pet transfer functionality w

/* … truncated 2629 chars — edit only what you need near the top … */
