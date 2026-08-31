# Event Schema Versioning (Issue #1251)

## Overview

Every public event emitted by the PetChain contract includes a `version: u32` field
set to the value of `EVENT_SCHEMA_VERSION` (currently **1**). This lets off-chain
indexers detect schema evolution and apply migration logic without silently breaking.

## Migration Path

| Version | Description |
|---------|-------------|
| **v0** (pre-versioning) | Events had no `version` field. Indexers should treat events without a `version` field as version 0 and apply default values for any new fields. |
| **v1** (current) | `version: u32` field added to every event struct. Set to `EVENT_SCHEMA_VERSION` at emit time. |

## Policies for Schema Changes

### Additive Changes (Non-Breaking)

Additive changes include:
- Adding a new optional field (`Option<T>`) to an existing event struct
- Adding a new event struct
- Adding new variants to existing enums used in events

**Procedure:**
1. Add the new field/struct.
2. Bump `EVENT_SCHEMA_VERSION` only if the new field changes the XDR layout of
   existing events (adding a new struct does not require a bump).
3. Update the `EventSchema` enum with a new variant if the version is bumped.
4. Add a migration note in this document.

### Breaking Changes

Breaking changes include:
- Removing a field from an event struct
- Renaming a field
- Changing a field type
- Reordering fields (changes XDR serialization)

**Procedure:**
1. Create a new variant in the `EventSchema` enum.
2. Bump `EVENT_SCHEMA_VERSION`.
3. Update the header comment in `lib.rs`.
4. Add a migration note below.
5. Update all event publishers.
6. Ensure CI snapshot tests pass.

## Currently Emitted Events (v1)

All events below include `version: u32` set to `EVENT_SCHEMA_VERSION`.

| Event Struct | Topic | Description |
|---|---|---|
| `PetRegisteredEvent` | `PetRegistered` | A new pet was registered |
| `PetOwnershipTransferredEvent` | `PetOwnershipTransferred` | Pet ownership changed |
| `PetProfileUpdatedEvent` | `PetProfileUpdated` | Pet profile fields updated |
| `VaccinationAddedEvent` | `VaccinationAdded` | A vaccination record was added |
| `VaccinationRevokedEvent` | `VaccinationRevoked` | A vaccination was revoked |
| `VaccinationExpiringSoonEvent` | `VaccinationExpiringSoon` | A vaccination is about to expire |
| `CertificateAnchoredEvent` | `CertificateAnchored` | A vaccination certificate was anchored |
| `MedicalRecordAddedEvent` | `MedicalRecordAdded` | A medical record was added |
| `MedicalRecordDeletedEvent` | `MedicalRecordDeleted` | A medical record was soft-deleted |
| `MedicalRecordPurgedEvent` | `MedicalRecordPurged` | Soft-deleted records were purged |
| `AccessGrantedEvent` | `AccessGranted` | Access was granted to a user |
| `AccessRevokedEvent` | `AccessRevoked` | Access was revoked from a user |
| `AccessExtendedEvent` | `AccessExtended` | Access expiration was extended |
| `AccessExpiredEvent` | `AccessExpired` | A timed access grant expired |
| `InsuranceAddedEvent` | (insurance topic) | A new insurance policy was added |
| `InsuranceUpdatedEvent` | (insurance topic) | An insurance policy was updated |
| `InsuranceClaimSubmittedEvent` | (insurance topic) | An insurance claim was submitted |
| `InsuranceClaimStatusUpdatedEvent` | (insurance topic) | Claim status changed |
| `InsuranceClaimFlaggedEvent` | (insurance topic) | A claim was flagged for fraud |
| `PolicyExpiringSoonEvent` | (insurance topic) | A policy is about to expire |
| `PolicyRenewedEvent` | (insurance topic) | A policy was renewed |
| `FlaggedClaimApprovedEvent` | (insurance topic) | Admin approved a flagged claim |
| `ClaimAppealedEvent` | (insurance topic) | A rejected claim was appealed |
| `AppealDecisionEvent` | (insurance topic) | Appeal received a final decision |
| `ClaimDocumentIntegrityEvent` | `ClaimDocIntegrity` | Document integrity verified |
| `VetLicenseVerifiedEvent` | (vet topic) | A vet's license was verified |
| `VetLicenseRevokedEvent` | (vet topic) | A vet's license was revoked |
| `TempVetGrantExpiredEvent` | (vet topic) | Temp vet access expired |
| `GroomingRecordCreatedEvent` | (grooming topic) | A grooming record was created |
| `StreakMilestoneEvent` | `streak_milestone` | Pet reached a streak milestone |
| `TagLinkedEvent` | `TAG_LINKED` | NFC/QR tag linked to pet |
| `TagDeactivatedEvent` | `TAG_DEACTIVATED` | NFC/QR tag deactivated |
| `TagReactivatedEvent` | `TAG_REACTIVATED` | NFC/QR tag reactivated |
| `AccessEvent` | (access log) | Access-control event in export |
| `TreatmentAddedEvent` | (treatment topic) | Vet treatment record added |
| `BiomarkerTrendAlert` | (biomarker topic) | Deteriorating biomarker trend |
| `LabResultAnomaly` | `LAB_RESULT_ANOMALY` | Lab result statistical anomaly |
| `ConsentRevoked` | (consent topic) | Consent revoked in cascade |
| `CrossChainIdentityRegistered` | (cross-chain topic) | External chain identity linked |

## Field Semantics Convention

All event structs follow this convention:
1. The first field is `pub version: u32` — the schema version.
2. Domain-specific identifying fields follow (e.g. `pet_id`, `claim_id`).
3. Contextual fields (addresses, types, amounts) follow.
4. The last field is `pub timestamp: u64` (when applicable).

## Fixture for v1 Events

```json
{
  "version": 1,
  "pet_id": 12345,
  "owner": "G...",
  "name": "PROTECTED",
  "species": "Dog",
  "timestamp": 1700000000,
  "subscription_ids": []
}
```

## Detecting Accidental Changes

CI snapshots the contract ABI (function signatures) via `generate_abi_snapshot.sh`.
The snapshot is stored in `abi-snapshot.txt` and checked on every PR. Any change to
public function signatures must be reviewed and the snapshot regenerated.

For event schema stability, the unit test `test_event_schema_version_fields` in
`test_event_subscriptions.rs` verifies that:
1. Every event struct variant in `EventSchema` has a known version.
2. The `EVENT_SCHEMA_VERSION` constant matches the latest `EventSchema` variant.
