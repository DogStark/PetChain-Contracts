# Celo Contracts

Soroban/Celo contract modules for the PetChain platform.

## Modules

- `pet_medical_history` — stores pet medical records with cursor-based pagination.

## Pet Medical History Pagination

Medical records are returned using **cursor-based pagination** with deterministic
ordering and bounded page sizes.

### Ordering

Records are ordered deterministically by `(created_at, record_id)` ascending.
This guarantees a stable, reproducible order across pages and nodes.

### Page size

- `DEFAULT_PAGE_SIZE = 20`
- `MAX_PAGE_SIZE = 100`

Requests with `limit == 0` fall back to `DEFAULT_PAGE_SIZE`; requests above
`MAX_PAGE_SIZE` are clamped to `MAX_PAGE_SIZE`. This bounds iteration and
persistent state growth per call.

### Cursor

A cursor is an opaque `u64` encoding the last returned `record_id`. Passing
`None` starts from the beginning. The cursor is validated against the pet's
record set; an unknown cursor returns `Error::InvalidCursor`.

### Authorization

All read calls require the caller to be the pet owner or an authorized
veterinarian, preserving the existing authorization conventions. Unauthorized
callers receive `Error::Unauthorized` and no state is mutated.

### Error codes

| Code | Name             | Meaning                                   |
|------|------------------|-------------------------------------------|
| 1    | `Unauthorized`   | Caller lacks access to the pet's records  |
| 2    | `InvalidCursor`  | Cursor does not belong to the pet's set   |
| 3    | `InvalidLimit`   | Limit out of accepted bounds              |

### Events

- `MedicalHistoryPage { pet_id, cursor, limit, returned }` — emitted on each
  successful paginated read.

### Storage version

The storage layout is unchanged; the existing storage-version key is preserved
and no migration is required for this feature.

### Compatibility

Public ABI adds the paginated read entry point and the `MedicalHistoryPage`
event. Update compatibility fixtures when the ABI snapshot changes.
