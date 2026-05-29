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

### Alert System Errors (50-51)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 50 | `AlertNotFound` | Alert not found | Alerta no encontrada |
| 51 | `AlertNotActive` | Alert not active | Alerta no activa |

**Description:**
- Errors related to lost pet alert system
- Occur when trying to access or modify non-existent or inactive alerts

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

### Proposal & Governance Errors (80-87)

| Code | Name | English Message | Spanish Message |
|------|------|-----------------|-----------------|
| 80 | `ProposalNotFound` | Proposal not found | Propuesta no encontrada |
| 81 | `ProposalAlreadyExecuted` | Proposal already executed | Propuesta ya ejecutada |
| 82 | `ProposalExpired` | Proposal expired | Propuesta expirada |
| 83 | `ThresholdNotMet` | Threshold not met | Umbral no alcanzado |
| 84 | `AdminAlreadyApproved` | Admin already approved | Administrador ya aprobó |
| 85 | `TimelockNotExpired` | Timelock not expired | Bloqueo de tiempo no expirado |
| 86 | `ProposalVetoed` | Proposal vetoed | Propuesta vetada |
| 87 | `ProposalNotExecutable` | Proposal not executable | Propuesta no ejecutable |

**Description:**
- Errors related to multisig governance and upgrade proposals
- Occur during proposal lifecycle management

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

---

**Last Updated:** 2024  
**Issue:** #684  
**Status:** ✅ Complete
