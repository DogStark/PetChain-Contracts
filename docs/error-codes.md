# PetChain Contract Error Codes

## Overview

This document provides a comprehensive reference for all error codes in the PetChain Soroban smart contract, including multi-language support for error messages.

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

<!-- BEGIN GENERATED ERROR CODES: stellar-contracts/scripts/generate_error_docs.py -->
## Error Code Reference

Generated from [`stellar-contracts/error-codes.json`](../stellar-contracts/error-codes.json). Do not edit this section by hand: change the registry, then run `python3 stellar-contracts/scripts/generate_error_docs.py`.

### Retry Policies

| Policy | Meaning |
|--------|---------|
| `never` | Permanent for this request: fix the input, caller, or target. Resubmitting unchanged always fails. |
| `after_state_change` | A precondition is not met yet (time window, approval, verification, nonce). Re-read state and retry once it changes; do not loop blindly. |
| `backoff` | Transient throttling. Retry the same request with exponential backoff. |
| `treat_as_success` | The requested effect already exists (idempotent replay). Clients should treat it as success and not retry. |

### `ContractError` (petchain)

Source: `stellar-contracts/src/lib.rs`

| Code | Name | Retry |
|------|------|-------|
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

### `PetChainError` (petchain)

Source: `stellar-contracts/src/lib.rs`

| Code | Name | Retry |
|------|------|-------|
| 1 | `NonceReused` | `after_state_change` |
| 2 | `SelfLineage` | `never` |
| 3 | `CircularLineage` | `never` |
| 4 | `KeywordTooLong` | `never` |
| 5 | `TooManySearchTokens` | `never` |

### `ContractError` (pet-transfer-adoption)

Source: `stellar-contracts/contracts/pet-transfer-adoption/src/lib.rs`

| Code | Name | Retry |
|------|------|-------|
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

### `EscrowError` (pet-transfer-adoption (escrow))

Source: `stellar-contracts/contracts/pet-transfer-adoption/src/escrow.rs`

| Code | Name | Retry |
|------|------|-------|
| 1 | `FeeBpsTooHigh` | `never` |
| 2 | `InvalidAmount` | `never` |
| 3 | `EscrowAlreadyExists` | `never` |
| 4 | `EscrowNotFound` | `never` |
| 5 | `InvalidEscrowState` | `never` |
| 6 | `Unauthorized` | `never` |
| 7 | `EscrowNotExpired` | `after_state_change` |
| 8 | `InvalidTokenAddress` | `never` |

### `ContractError` (vet-registry)

Source: `stellar-contracts/contracts/pet-transfer-adoption/src/vet_registry.rs`

| Code | Name | Retry |
|------|------|-------|
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
let message = client.get_error_message(&24, &String::from_str(&env, "en"));
// Returns: Some("Pet not found")

// Get error message in Spanish
let message = client.get_error_message(&24, &String::from_str(&env, "es"));
// Returns: Some("Mascota no encontrada")

// Get error message for unsupported language
let message = client.get_error_message(&24, &String::from_str(&env, "fr"));
// Returns: None
```

### Example 2: Set Custom Error Message

```rust
// Admin sets a custom error message
client.set_error_message(
    &admin,
    &31,
    &String::from_str(&env, "en"),
    &String::from_str(&env, "Veterinarian record not found in database")
);
```

### Example 3: Add New Language

```rust
// Admin adds French translations
client.set_error_message(
    &admin,
    &24,
    &String::from_str(&env, "fr"),
    &String::from_str(&env, "Animal de compagnie non trouvé")
);

client.set_error_message(
    &admin,
    &25,
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
**Status:** ✅ Complete
