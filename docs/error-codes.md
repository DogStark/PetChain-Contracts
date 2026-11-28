# PetChain Contract Error Codes

## Overview

This document provides a reference for error codes in the PetChain Soroban smart contracts, including multi-language support for error messages. The [Registered Error Enums](#registered-error-enums-error-codesjson) section lists every error enum in `stellar-contracts/error-codes.json`, including `pet-transfer-adoption` and `vet-registry`. The legacy tables below cover only part of the `petchain` enum.

## Multi-Language Error Registry (Issue #684)

The contract supports human-readable error messages in multiple languages through an error registry system. Error messages can be queried programmatically and managed by multisig admins.

### Supported Languages

- **English** (`en`)
- **Spanish** (`es`)

Additional languages can be added by admins using the `set_error_message` function.

### API Functions

#### Query Functions

**`get_error_message(error_code: u32, language: String) -> Option<String>`**
- Returns the error message for a specific error code and language
- Returns `None` if no message is found for that code/language combination

**`get_supported_languages() -> Vec<String>`**
- Returns a list of all supported languages in the error registry

#### Admin Functions (Multisig Admin Only)

**`set_error_message(admin: Address, error_code: u32, language: String, message: String)`**
- Sets an error message for a specific error code and language
- Automatically adds the language to supported languages list
- Validates: language length (1-10 chars), message length (1-500 chars)

**`batch_set_error_messages(admin: Address, messages: Vec<ErrorMessage>)`**
- Sets multiple error messages at once
- More efficient for bulk operations
- Each message must pass validation

**`initialize_error_messages(admin: Address)`**
- Initializes default error messages in English and Spanish
- Covers the most common error codes
- Should be called once after contract deployment

**`remove_error_message(admin: Address, error_code: u32, language: String)`**
- Removes an error message for a specific error code and language
- Used for cleanup or corrections

---

## Error Code Reference

### Authorization & Admin Errors (1-20)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 1 | `Unauthorized` | Unauthorized access | Acceso no autorizado |
| 2 | `AdminNotInitialized` | Admin not initialized | Administrador no inicializado |
| 16 | `AdminAlreadySet` | Admin already set | Administrador ya establecido |
| 17 | `AdminsNotSet` | Admins not set | Administradores no establecidos |
| 18 | `NoAdminsConfigured` | No admins configured | No hay administradores configurados |
| 19 | `NotAnAdmin` | Not an admin | No es un administrador |
| 20 | `InvokerNotInAdminList` | Invoker not in admin list | Invocador no está en la lista de administradores |
| 21 | `InvalidThreshold` | Invalid threshold | Umbral inválido |

**Description:**
- These errors occur when there are authorization failures or admin configuration issues
- Most operations require proper authentication and admin privileges

---

### Pet Management Errors (3, 22, 60)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 3 | `PetNotFound` | Pet not found | Mascota no encontrada |
| 22 | `SireNotFound` | Sire not found | Padre no encontrado |
| 60 | `NotPetOwner` | Not pet owner | No es el dueño de la mascota |

**Description:**
- Errors related to pet records and ownership
- Occur when trying to access or modify non-existent pets or without proper ownership

---

### Veterinarian Errors (4-7, 23)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 4 | `VetNotFound` | Veterinarian not found | Veterinario no encontrado |
| 5 | `VeterinarianNotVerified` | Veterinarian not verified | Veterinario no verificado |
| 6 | `VetAlreadyRegistered` | Veterinarian already registered | Veterinario ya registrado |
| 7 | `LicenseAlreadyRegistered` | License already registered | Licencia ya registrada |
| 23 | `VetNotVerified` | Vet not verified | Veterinario no verificado |

**Description:**
- Errors related to veterinarian registration and verification
- Only verified vets can perform certain operations (add medical records, vaccinations, etc.)

---

### Input Validation Errors (8, 11-15, 25-27)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 8 | `InputStringTooLong` | Input string too long | Cadena de entrada demasiado larga |
| 11 | `CounterOverflow` | Counter overflow | Desbordamiento de contador |
| 12 | `TooManyItems` | Too many items | Demasiados elementos |
| 13 | `InvalidState` | Invalid state | Estado inválido |
| 14 | `InvalidInput` | Invalid input | Entrada inválida |
| 15 | `CommentTooLong` | Comment too long | Comentario demasiado largo |
| 25 | `FilenameEmpty` | Filename empty | Nombre de archivo vacío |
| 26 | `FileTypeEmpty` | File type empty | Tipo de archivo vacío |
| 27 | `FileSizeZero` | File size zero | Tamaño de archivo cero |

**Description:**
- Validation errors for input data
- Occur when inputs exceed limits or are malformed

---

### Tag & IPFS Errors (9-10, 24)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 9 | `PetAlreadyHasLinkedTag` | Pet already has linked tag | Mascota ya tiene etiqueta vinculada |
| 10 | `InvalidIpfsHash` | Invalid IPFS hash | Hash IPFS inválido |
| 24 | `TagAlreadyLinked` | Tag already linked | Etiqueta ya vinculada |

**Description:**
- Errors related to NFC tag linking and IPFS content
- Each pet can only have one active tag at a time

---

### Attachment Errors (43)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 43 | `InvalidAttachmentIndex` | Invalid attachment index | Índice de adjunto inválido |

**Description:**
- Errors when accessing attachments with invalid indices

---

### Alert System Errors (50-51, 52-55)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 50 | `AlertNotFound` | Alert not found | Alerta no encontrada |
| 51 | `AlertNotActive` | Alert not active | Alerta no activa |
| 52 | `AlertAlreadyActive` | An active alert already exists for this pet | Ya existe una alerta activa para esta mascota |
| 53 | `InvalidPageSize` | Invalid page size | Tamaño de página inválido |
| 54 | `InvalidAlertCursor` | Invalid or expired alert cursor | Cursor de alerta inválido o expirado |
| 55 | `AlertRegionTooLong` | Alert region too long | Región de alerta demasiado larga |

**Description:**
- Errors related to lost pet alert system
- Occur when trying to access or modify non-existent or inactive alerts

#### Duplicate Alert Detection (Code 52)

`create_alert` rejects a new alert when the same pet already has an active alert
(`status == Active`). The failed call leaves all state unchanged: no alert record,
no index entry, and no event is written.

Resolution path for changed circumstances:
1. The pet owner (or an authorized admin) calls `resolve_alert` / `cancel_alert`
   on the existing alert, which moves it out of `Active`.
2. Once no active alert remains for the pet, `create_alert` succeeds and the new
   alert is indexed normally.

Retry guidance: do **not** blindly retry `create_alert` after code 52. Query the
pet's active alert first (`get_active_alert_for_pet`), then either resolve/cancel
it or surface the existing alert to the user.

#### Alert Index Pagination (Codes 53-55)

`list_alerts_by_region` / `list_alerts_by_status` return a bounded page of active
alerts using an opaque cursor.

- **Page size bounds:** `page_size` must be in `1..=MAX_ALERT_PAGE_SIZE` (50).
  `0` or values above the maximum fail with `InvalidPageSize` (53).
- **Cursor format:** the cursor is an opaque, contract-issued token. Clients must
  treat it as a black box and pass back exactly what the previous page returned.
- **Cursor invalidation:** cursors are bound to the region/status filter they were
  issued for and to the storage version. A cursor becomes invalid (54) when:
  - it was issued for a different region or status filter,
  - it is malformed or truncated,
  - the contract storage version has changed (upgrade),
  - the underlying alert it points at has been resolved or cancelled.
- **Replay/expiry:** cursors are single-use per listing session; replaying a stale
  cursor after the index changed returns `InvalidAlertCursor` (54) rather than
  silently skipping or duplicating entries. On 54, restart pagination from the
  first page (no cursor) instead of retrying the same cursor.
- **Region bounds:** `region` must be non-empty and at most `MAX_REGION_LEN` (64)
  bytes; otherwise `AlertRegionTooLong` (55).

Retry guidance: codes 53 and 55 are deterministic input errors — fix the input,
do not retry. Code 54 is recoverable by restarting from the first page.

---

### Consent Errors (61-62)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 61 | `NotConsentOwner` | Not consent owner | No es el propietario del consentimiento |
| 62 | `ConsentAlreadyRevoked` | Consent already revoked | Consentimiento ya revocado |

**Description:**
- Errors related to data sharing consent management
- Occur when trying to modify consent without proper authorization

---

### Booking Errors (70-71)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 70 | `SlotAlreadyBooked` | Slot already booked | Espacio ya reservado |
| 71 | `SlotNotBooked` | Slot not booked | Espacio no reservado |

**Description:**
- Errors related to veterinary appointment booking system
- Occur when trying to book already-booked slots or cancel non-existent bookings

---

### Proposal & Governance Errors

The numeric codes below are the ones emitted by the current Stellar contract.
Codes 80–85 and 87 appeared in an older documentation table and are not emitted
by the current contract; code 86 is restored for the veto path (issue #1649).

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 1 | `AdminAlreadyApproved` | Admin already approved / already vetoed | Administrador ya aprobó / ya vetó |
| 16 | `InvalidState` | Proposal is not in the required state | La propuesta no está en el estado requerido |
| 26 | `ThresholdNotMet` | Approval threshold not met | Umbral de aprobación no alcanzado |
| 38 | `ProposalAlreadyExecuted` | Proposal already executed | Propuesta ya ejecutada |
| 43 | `ProposalExpired` | Proposal expired | Propuesta expirada |
| 47 | `ProposalNotFound` | Proposal not found | Propuesta no encontrada |
| 86 | `ProposalVetoed` | Proposal vetoed by an administrator | Propuesta vetada por un administrador |
| 228 | `ProposalCancelled` | Proposal already cancelled | Propuesta ya cancelada |
| 229 | `NotProposalProposer` | Caller is not the original proposer | El llamador no es el proponente original |
| 230 | `CancellationWindowClosed` | Cancellation window is closed | La ventana de cancelación está cerrada |

**Description:**
- Errors related to multisig governance and upgrade proposals
- Occur during proposal lifecycle management
- `NotProposalProposer` and `CancellationWindowClosed` are distinct from veto
  errors so proposer withdrawal and administrative veto stay separate

---

### Review Errors (90-91)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 90 | `InvalidRating` | Invalid rating | Calificación inválida |
| 91 | `DuplicateReview` | Duplicate review | Reseña duplicada |

**Description:**
- Errors related to veterinarian review system
- Occur when ratings are out of range or users try to review the same vet twice

---

### Medication Errors (100)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 100 | `MedicationNotFound` | Medication not found | Medicamento no encontrado |

**Description:**
- Errors when accessing non-existent medication records

---

### Multisig Errors (110-113)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 110 | `MultisigNotConfigured` | Multisig not configured | Multifirma no configurada |
| 111 | `MultisigNotEnabled` | Multisig not enabled | Multifirma no habilitada |
| 112 | `NotAuthorizedSigner` | Not authorized signer | Firmante no autorizado |
| 113 | `AlreadySigned` | Already signed | Ya firmado |

**Description:**
- Errors related to multisig pet transfer functionality
- Occur when multisig is not properly configured or signers are unauthorized

---

### Range Validation Errors (120-121)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 120 | `SeverityOutOfRange` | Severity out of range | Severidad fuera de rango |
| 121 | `IntensityOutOfRange` | Intensity out of range | Intensidad fuera de rango |

**Description:**
- Errors when numeric values are outside acceptable ranges
- Used in behavioral tracking and activity recording

---

### Custody Errors (130)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 130 | `CustodyNotFound` | Custody not found | Custodia no encontrada |

**Description:**
- Errors related to temporary custody records
- Occur when trying to access non-existent custody arrangements

---

### Grooming Errors (140)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 140 | `UnregisteredGroomer` | Unregistered groomer | Peluquero no registrado |

**Description:**
- Errors when unregistered groomers try to perform operations

---

### Training & Breeding Errors (141-143, 150)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 141 | `PrerequisiteIncomplete` | Prerequisite incomplete | Prerrequisito incompleto |
| 142 | `CircularDependency` | Circular dependency | Dependencia circular |
| 143 | `CrossPetComparison` | Cross pet comparison | Comparación entre mascotas |
| 150 | `BreedingRecordNotFound` | Breeding record not found | Registro de cría no encontrado |

**Description:**
- Errors related to training milestones and breeding records
- Occur when prerequisites aren't met or records don't exist

---

### Storage Quota Errors (160)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 160 | `StorageQuotaExceeded` | Storage quota exceeded | Cuota de almacenamiento excedida |

**Description:**
- Error when a pet's storage quota is exceeded
- Prevents unbounded storage consumption
- Admin can increase quota via `set_pet_storage_quota` or `set_global_storage_quota`

---

### Error Registry Errors (170)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 170 | `ErrorMessageNotFound` | Error message not found | Mensaje de error no encontrado |

**Description:**
- Error when querying an error message that doesn't exist in the registry
- Indicates the error code or language combination is not registered

---

## Registered Error Enums (`error-codes.json`)

`stellar-contracts/error-codes.json` is the authoritative registry of every `#[contracterror]` enum in `stellar-contracts`. The tables below list all five registered enums. Codes are scoped per enum, so the same number in two enums means two different errors. Always read the contract and enum name together.

The retry column follows the policy in [`stellar-contracts/docs/error-compatibility.md`](../stellar-contracts/docs/error-compatibility.md):

| `retry` | Client behaviour |
|---|---|
| `never` | Permanent for this request: fix the input, caller, or target. |
| `after_state_change` | A precondition is not met yet. Re-read state and retry once it changes. |
| `backoff` | Transient throttling. Retry with exponential backoff. |
| `treat_as_success` | Idempotent replay: the effect already exists. Do not retry. |

### `petchain::ContractError` (114 variants)

Source: `stellar-contracts/src/lib.rs`

| Code | Name | Retry |
|---|---|---|
| 1 | `AdminAlreadyApproved` | `treat_as_success` |
| 2 | `AdminAlreadySet` | `never` |
| 3 | `AdminNotInitialized` | `after_state_change` |
| 4 | `AdminsNotSet` | `after_state_change` |
| 5 | `BatchTooLarge` | `never` |
| 6 | `CertificateAlreadyAnchored` | `treat_as_success` |
| 7 | `CounterOverflow` | `never` |
| 8 | `InputStringTooLong` | `never` |
| 9 | `InvalidBreed` | `never` |
| 10 | `InvalidCallerNonce` | `after_state_change` |
| 11 | `InvalidCertificateHash` | `never` |
| 12 | `InvalidInput` | `never` |
| 13 | `InvalidIpfsHash` | `never` |
| 14 | `InvalidPetName` | `never` |
| 15 | `InvalidRating` | `never` |
| 16 | `InvalidState` | `after_state_change` |
| 17 | `InvalidThreshold` | `never` |
| 18 | `InvokerNotInAdminList` | `never` |
| 19 | `LicenseAlreadyRegistered` | `never` |
| 20 | `NoAdminsConfigured` | `after_state_change` |
| 21 | `NotAnAdmin` | `never` |
| 22 | `NotPetOwner` | `never` |
| 23 | `PetAlreadyHasLinkedTag` | `never` |
| 24 | `PetNotFound` | `never` |
| 25 | `StorageQuotaExceeded` | `after_state_change` |
| 26 | `ThresholdNotMet` | `after_state_change` |
| 27 | `TooManyItems` | `never` |
| 28 | `Unauthorized` | `never` |
| 29 | `VaccinationNotFound` | `never` |
| 30 | `VetAlreadyRegistered` | `never` |
| 31 | `VetNotFound` | `never` |
| 32 | `VetNotVerified` | `after_state_change` |
| 33 | `VeterinarianNotVerified` | `after_state_change` |
| 34 | `SlotAlreadyBooked` | `never` |
| 35 | `DuplicateActivity` | `treat_as_success` |
| 36 | `InbreedingThresholdExceeded` | `never` |
| 37 | `SelfBreeding` | `never` |
| 38 | `ProposalAlreadyExecuted` | `treat_as_success` |
| 39 | `InvalidNonce` | `after_state_change` |
| 40 | `RollbackWindowExpired` | `never` |
| 41 | `NoPreviousUpgrade` | `never` |
| 43 | `ProposalExpired` | `never` |
| 44 | `ProposalNotApproved` | `after_state_change` |
| 45 | `QuorumNotMet` | `after_state_change` |
| 46 | `RateLimitExceeded` | `backoff` |
| 47 | `ProposalNotFound` | `never` |
| 160 | `AlreadyDeleted` | `treat_as_success` |
| 161 | `RecordAlreadyDeleted` | `treat_as_success` |
| 162 | `RetentionPeriodNotMet` | `after_state_change` |
| 163 | `RecordNotFound` | `never` |
| 164 | `DisputeNotFound` | `never` |
| 165 | `DisputeNotOpenForVoting` | `after_state_change` |
| 166 | `NotDisputeStakeholder` | `never` |
| 167 | `NotInEvidencePhase` | `never` |
| 168 | `NotDisputeParty` | `never` |
| 169 | `InvalidTimestamp` | `never` |
| 170 | `StaleMigration` | `treat_as_success` |
| 171 | `VetCredentialsExpired` | `after_state_change` |
| 172 | `CertificateNotFound` | `never` |
| 173 | `CertificateRevoked` | `never` |
| 174 | `CertificateExpired` | `never` |
| 175 | `CertificateHashConflict` | `never` |
| 176 | `NonceReused` | `after_state_change` |
| 177 | `NotificationExpired` | `never` |
| 178 | `NotificationAlreadySent` | `treat_as_success` |
| 179 | `UnknownEmergencyRecipient` | `never` |
| 180 | `ConsentNotFound` | `never` |
| 181 | `ConsentVersionMismatch` | `after_state_change` |
| 182 | `IssuerAlreadyRegistered` | `never` |
| 183 | `IssuerNotFound` | `never` |
| 184 | `IssuerRevoked` | `never` |
| 185 | `IssuerKeyVersionNotFound` | `never` |
| 186 | `IssuerKeyVersionInactive` | `never` |
| 187 | `IssuerKeyReused` | `never` |
| 188 | `RevokedConsentReplay` | `never` |
| 189 | `EnrollmentNotFound` | `never` |
| 190 | `InvalidEnrollmentState` | `after_state_change` |
| 191 | `EnrollmentAlreadyExists` | `treat_as_success` |
| 192 | `EnrollmentNotExpired` | `after_state_change` |
| 193 | `PolicyVersionNotFound` | `never` |
| 194 | `PolicyVersionMismatch` | `after_state_change` |
| 195 | `CoverageSnapshotNotFound` | `never` |
| 196 | `CoverageSnapshotMismatch` | `never` |
| 197 | `TooManyCoverageItems` | `never` |
| 198 | `TooManyExclusionItems` | `never` |
| 199 | `InvalidCoverageAmount` | `never` |
| 200 | `ClaimNotFound` | `never` |
| 201 | `ClaimInvalidStatus` | `after_state_change` |
| 202 | `ClaimNotRejected` | `never` |
| 203 | `AppealWindowExpired` | `never` |
| 204 | `ClaimAlreadyAppealed` | `never` |
| 205 | `ClaimNotUnderAppeal` | `never` |
| 206 | `ReviewerCannotBeOriginal` | `never` |
| 207 | `ClaimDocumentLimitReached` | `never` |
| 208 | `PolicyPetNotFound` | `never` |
| 209 | `BenefitConditionNotFound` | `never` |
| 210 | `BenefitConditionLimitReached` | `never` |
| 211 | `BenefitWaitingPeriod` | `after_state_change` |
| 212 | `BenefitInvalidServiceDate` | `never` |
| 213 | `BenefitCoverageLimitExceeded` | `never` |
| 214 | `BenefitClaimNotPending` | `never` |
| 215 | `BenefitClaimAlreadyFinalized` | `never` |
| 216 | `BenefitInvalidTerms` | `never` |
| 217 | `BenefitClaimLimitReached` | `never` |
| 218 | `BenefitClaimMustUseSettlement` | `never` |
| 219 | `BenefitTermsAlreadyConfigured` | `never` |
| 220 | `EnrollmentNotFound` | `never` |
| 221 | `EnrollmentInvalidTransition` | `after_state_change` |
| 222 | `EnrollmentAlreadyDecided` | `never` |
| 223 | `EnrollmentLimitReached` | `never` |
| 224 | `EnrollmentSnapshotTooLarge` | `never` |
| 225 | `InsurancePolicyInactive` | `after_state_change` |
| 226 | `InsurancePolicyNotFound` | `never` |
| 227 | `BenefitClaimLimitPerPetReached` | `never` |

Note: `EnrollmentNotFound` appears twice in this enum (codes 189 and 220) with the same name. Clients should match on the code, not the name.

### `petchain::PetChainError` (5 variants)

Source: `stellar-contracts/src/lib.rs`

| Code | Name | Retry |
|---|---|---|
| 1 | `NonceReused` | `after_state_change` |
| 2 | `SelfLineage` | `never` |
| 3 | `CircularLineage` | `never` |
| 4 | `KeywordTooLong` | `never` |
| 5 | `TooManySearchTokens` | `never` |

### `pet-transfer-adoption::ContractError` (34 variants)

Source: `stellar-contracts/contracts/pet-transfer-adoption/src/lib.rs`

| Code | Name | Retry |
|---|---|---|
| 1 | `PetNotFound` | `never` |
| 2 | `Unauthorized` | `never` |
| 3 | `TransferAlreadyPending` | `after_state_change` |
| 4 | `NoPendingTransfer` | `never` |
| 5 | `InvalidRecipient` | `never` |
| 6 | `EmptyOwnershipHistory` | `never` |
| 7 | `MissingOwnershipRecord` | `never` |
| 8 | `TransferNotExpired` | `after_state_change` |
| 9 | `StaleCancellation` | `never` |
| 10 | `EmptyBatch` | `never` |
| 11 | `BatchOwnerMismatch` | `never` |
| 12 | `NoEscrowedTransfer` | `never` |
| 13 | `DisputeWindowNotElapsed` | `after_state_change` |
| 14 | `TransferAlreadyDisputed` | `never` |
| 15 | `AlreadyInitialized` | `treat_as_success` |
| 16 | `InvalidThreshold` | `never` |
| 17 | `NotMultisigAdmin` | `never` |
| 18 | `ThresholdNotMet` | `after_state_change` |
| 19 | `UntrustedContract` | `never` |
| 20 | `NoPendingAdoption` | `never` |
| 21 | `WaitingPeriodNotElapsed` | `after_state_change` |
| 22 | `AdoptionAlreadyCompleted` | `treat_as_success` |
| 23 | `AdoptionNotConfigurable` | `never` |
| 24 | `InvalidWaitingPeriod` | `never` |
| 25 | `AdoptionConfigNotFound` | `never` |
| 26 | `BatchTooLarge` | `never` |
| 27 | `InvalidBatch` | `never` |
| 28 | `OrganizationApprovalRequired` | `after_state_change` |
| 29 | `AdoptionRejected` | `never` |
| 30 | `InvalidApprover` | `never` |
| 31 | `InvalidTimeoutDays` | `never` |
| 32 | `AdopterApprovalRequired` | `after_state_change` |
| 33 | `InputStringTooLong` | `never` |
| 34 | `AdoptionNotExpired` | `after_state_change` |

**Not yet registered:** the source enum also defines `CustodyNotConfirmed` (35), `CustodyAlreadyConfirmed` (36), `CustodyConfirmationExpired` (37) and `TransferNotDisputed` (38), which are documented in [`contracts/pet-transfer-adoption/README.md`](../stellar-contracts/contracts/pet-transfer-adoption/README.md#error-codes). They are missing from `error-codes.json`, so they have no retry guidance in the registry yet.

### `pet-transfer-adoption::EscrowError` (8 variants)

Source: `stellar-contracts/contracts/pet-transfer-adoption/src/escrow.rs`

| Code | Name | Retry |
|---|---|---|
| 1 | `FeeBpsTooHigh` | `never` |
| 2 | `InvalidAmount` | `never` |
| 3 | `EscrowAlreadyExists` | `never` |
| 4 | `EscrowNotFound` | `never` |
| 5 | `InvalidEscrowState` | `never` |
| 6 | `Unauthorized` | `never` |
| 7 | `EscrowNotExpired` | `after_state_change` |
| 8 | `InvalidTokenAddress` | `never` |

### `vet-registry::ContractError` (12 variants)

Source: `stellar-contracts/contracts/pet-transfer-adoption/src/vet_registry.rs`

| Code | Name | Retry |
|---|---|---|
| 0 | `AlreadyInitialized` | `treat_as_success` |
| 1 | `Unauthorized` | `never` |
| 2 | `VetAlreadyRegistered` | `never` |
| 3 | `VetNotFound` | `never` |
| 4 | `LicenseAlreadyUsed` | `never` |
| 5 | `VetNotVerified` | `after_state_change` |
| 6 | `InputTooLong` | `never` |
| 7 | `VetAlreadyVerified` | `treat_as_success` |
| 8 | `StaleMigration` | `treat_as_success` |
| 9 | `InvalidMigrationTarget` | `never` |
| 10 | `InvalidPageLimit` | `never` |
| 11 | `InvalidCursor` | `never` |

---

## Usage Examples

### Example 1: Query Error Message

```rust
// Get error message in English
let message = client.get_error_message(&3, &String::from_str(&env, "en"));
// Returns: Some("Pet not found")

// Get error message in Spanish
let message = client.get_error_message(&3, &String::from_str(&env, "es"));
// Returns: Some("Mascota no encontrada")

// Get error message for unsupported language
let message = client.get_error_message(&3, &String::from_str(&env, "fr"));
// Returns: None
```

### Example 2: Set Custom Error Message

```rust
// Admin sets a custom error message
client.set_error_message(
    &admin,
    &100,
    &String::from_str(&env, "en"),
    &String::from_str(&env, "Medication record not found in database")
);
```

### Example 3: Add New Language

```rust
// Admin adds French translations
client.set_error_message(
    &admin,
    &3,
    &String::from_str(&env, "fr"),
    &String::from_str(&env, "Animal de compagnie non trouvé")
);

client.set_error_message(
    &admin,
    &160,
    &String::from_str(&env, "fr"),
    &String::from_str(&env, "Quota de stockage dépassé")
);
```

### Example 4: Batch Initialize Messages

```rust
// Initialize default messages in English and Spanish
client.initialize_error_messages(&admin);

// Check supported languages
let languages = client.get_supported_languages();
// Returns: ["en", "es"]
```

### Example 5: Handle Errors with Messages

```rust
// Catch error and display localized message
match client.try_add_medical_record(...) {
    Ok(record_id) => println!("Record added: {}", record_id),
    Err(error) => {
        let error_code = error as u32;
        let user_language = get_user_language(); // e.g., "es"
        
        if let Some(message) = client.get_error_message(&error_code, &user_language) {
            println!("Error: {}", message);
        } else {
            println!("Error code: {}", error_code);
        }
    }
}
```

---

## Best Practices

### For Developers

1. **Always Initialize Error Messages**
   - Call `initialize_error_messages()` after contract deployment
   - Ensures basic error messages are available

2. **Provide Fallback Logic**
   - Check if message exists before displaying
   - Fall back to error code if message not found
   - Consider default language (English) as fallback

3. **Use Batch Operations**
   - Use `batch_set_error_messages()` for multiple messages
   - More efficient than individual calls

4. **Keep Messages Concise**
   - Maximum 500 characters per message
   - Focus on clarity and actionability

### For Administrators

1. **Maintain Consistency**
   - Keep messages consistent across languages
   - Use similar tone and terminology

2. **Regular Updates**
   - Add messages for new error codes
   - Update messages when functionality changes

3. **Language Coverage**
   - Prioritize languages based on user base
   - Ensure critical errors are translated first

4. **Test Translations**
   - Verify translations are accurate
   - Consider cultural context

---

## Error Message Guidelines

### Writing Good Error Messages

1. **Be Specific**
   - ❌ "Error occurred"
   - ✅ "Pet not found"

2. **Be Actionable**
   - ❌ "Invalid input"
   - ✅ "Input string too long (max 500 characters)"

3. **Be User-Friendly**
   - ❌ "Unauthorized"
   - ✅ "You don't have permission to perform this action"

4. **Be Consistent**
   - Use consistent terminology across messages
   - Follow the same structure

### Translation Guidelines

1. **Maintain Meaning**
   - Preserve the original intent
   - Don't add or remove information

2. **Cultural Adaptation**
   - Consider cultural context
   - Use appropriate formality level

3. **Technical Accuracy**
   - Keep technical terms consistent
   - Use standard translations for technical concepts

4. **Length Considerations**
   - Some languages are more verbose
   - Ensure translations fit within 500 character limit

---

## API Reference Summary

### Query Functions (Public)

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `get_error_message` | `error_code: u32, language: String` | `Option<String>` | Get error message for code and language |
| `get_supported_languages` | None | `Vec<String>` | Get list of supported languages |

### Admin Functions (Multisig Admin Only)

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `set_error_message` | `admin: Address, error_code: u32, language: String, message: String` | None | Set single error message |
| `batch_set_error_messages` | `admin: Address, messages: Vec<ErrorMessage>` | None | Set multiple error messages |
| `initialize_error_messages` | `admin: Address` | None | Initialize default English and Spanish messages |
| `remove_error_message` | `admin: Address, error_code: u32, language: String` | None | Remove error message |

---

## Events

### ErrorMessageSet
**Emitted when:** A single error message is set  
**Topics:** `("ErrorMessageSet", error_code)`  
**Data:** `(language, message)`

### ErrorMessagesBatchSet
**Emitted when:** Multiple error messages are set  
**Topics:** `("ErrorMessagesBatchSet")`  
**Data:** `count` (number of messages set)

### ErrorMessageRemoved
**Emitted when:** An error message is removed  
**Topics:** `("ErrorMessageRemoved", error_code)`  
**Data:** `language`

### AlertCreated
**Emitted when:** A new lost-pet alert is created and indexed  
**Topics:** `("AlertCreated", pet_id)`  
**Data:** `(alert_id, region, status)`

### AlertResolved
**Emitted when:** An active alert is resolved or cancelled, invalidating any cursors pointing at it  
**Topics:** `("AlertResolved", alert_id)`  
**Data:** `(pet_id, status)`

### TrialPeriodExpired
**Emitted when:** The first interaction with a trial-period adoption after `trial_expires_at` settles the trial. Emitted at most once per adoption.  
**Topics:** `("TrialPeriodExpired", pet_id, adoption_id)`  
**Data:** `TrialPeriodExpiredEvent { version, pet_id, adoption_id, outcome, new_owner, previous_owner, timestamp }`

`outcome` is `Converted` (the adoption becomes permanent and `new_owner` is the
adopter) or `Reversed` (the pet returns to `previous_owner`). `new_owner` always
carries the resulting ownership state, and `previous_owner` is always present so
integrators can reconcile both sides of the transition. The event contains no
personal documents or private contact details. Before expiry, `settle_adoption_trial`
and `get_adoption_trial` are no-ops with respect to this event.

---

## Storage Structure

### ErrorRegistryKey Enum

```rust
pub enum ErrorRegistryKey {
    ErrorMessage((u32, String)),  // (error_code, language) -> message
    SupportedLanguages,            // Vec<String> of supported languages
}
```

### ErrorMessage Struct

```rust
pub struct ErrorMessage {
    pub code: u32,
    pub language: String,
    pub message: String,
}
```

---

## Migration Guide

### Adding Error Messages to Existing Contract

1. **Deploy Updated Contract**
   - Contract includes error registry system

2. **Initialize Default Messages**
   ```rust
   client.initialize_error_messages(&admin);
   ```

3. **Add Custom Messages (Optional)**
   ```rust
   client.set_error_message(&admin, &custom_code, &lang, &message);
   ```

4. **Verify Messages**
   ```rust
   let languages = client.get_supported_languages();
   let message = client.get_error_message(&code, &lang);
   ```

---

## Security Considerations

1. **Admin-Only Modification**
   - Only multisig admins can set/remove messages
   - Prevents unauthorized message manipulation

2. **Input Validation**
   - Language code: 1-10 characters
   - Message: 1-500 characters
   - Prevents storage abuse

3. **No Sensitive Information**
   - Error messages should not contain sensitive data
   - Keep messages generic and safe for public display

4. **Immutable Error Codes**
   - Error codes themselves cannot be changed
   - Only messages can be updated

---

## Future Enhancements

### Potential Improvements (Not in Current Scope)

1. **Additional Languages**
   - French, German, Portuguese, Chinese, etc.
   - Community-contributed translations

2. **Message Templates**
   - Support for parameter substitution
   - Dynamic error messages with context

3. **Versioning**
   - Track message versions
   - Support for message history

4. **Bulk Export/Import**
   - Export all messages to JSON
   - Import translations from external sources

5. **Message Categories**
   - Group messages by category
   - Easier management and organization

---

## Support

For questions or issues related to error messages:
- Review this documentation
- Check the contract implementation in `stellar-contracts/src/lib.rs`
- Search for "Issue #684" in code comments

---

## Changelog

### Version 1.0.0 - Initial Implementation (Issue #684)
- Added multi-language error registry system
- Implemented English and Spanish translations
- Added admin functions for message management
- Added query functions for message retrieval
- Documented all error codes

### Version 1.1.0 - Alert Index Pagination & Duplicate Detection (Issues #1469, #1471)
- Added error codes 52-55 (`AlertAlreadyActive`, `InvalidPageSize`, `InvalidAlertCursor`, `AlertRegionTooLong`)
- Documented cursor-based pagination for active alerts by region/status, including page-size bounds and cursor invalidation/replay behavior
- Documented duplicate active-alert rejection and its resolution path
- Added `AlertCreated` and `AlertResolved` events
- No changes to existing error codes; storage version bumped for the new alert index keys

---

**Last Updated:** 2024  
**Issues:** #684, #1469, #1471  
**Status:** Registered enums documented from `error-codes.json` (see [Registered Error Enums](#registered-error-enums-error-codesjson)). The legacy flat-numbered tables above are not yet reconciled with the registry; where they disagree, `error-codes.json` is authoritative.
