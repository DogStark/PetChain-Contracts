# Role-Change Impact Preview Contract (`#1512`)

Provides a read-only preview (`preview_role_change`) of affected pet access grants, active authority delegations, and administrator council quorum impact before executing a pet owner transfer, veterinarian role revocation, or administrator removal.

## Storage & Event Versioning
- `STORAGE_SCHEMA_VERSION = 1`
- `EVENT_SCHEMA_VERSION = 1` (`RoleChangeExecutedEvent`)
- Bounded collections: `MAX_GRANTS_PER_PET = 32`, `MAX_DELEGATIONS_PER_ACTOR = 32`, `MAX_ADMINS = 16`, `MAX_SCOPE_LEN = 64`.
