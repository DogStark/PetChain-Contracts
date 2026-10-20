# Lost-Pet Contract

Soroban contract module for public lost-pet alerts.

## Privacy-preserving location cells

Public lost-pet alerts store only a **coarse geospatial cell** so that a pet's
approximate area can be shared without exposing the owner's precise location.
Precise coordinates are kept **owner-only** and are only readable when the owner
has explicitly granted consent.

### Storage layout

| Key | Value | Visibility |
| --- | --- | --- |
| `Cell(pet_id)` | coarse cell id (bounded integer) | public |
| `Precise(pet_id)` | precise coordinates | owner-only, consent-gated |
| `Consent(pet_id, viewer)` | consent flag | owner-managed |

Storage-version conventions are preserved: new entries are written under the
existing versioned key namespace and existing keys are not migrated in place.

### Authorization

- Writing a cell or precise coordinates requires the pet owner's authorization.
- Reading precise coordinates requires the owner's authorization **and** an
explicit consent entry for the caller.
- Reading a coarse cell is public and requires no authorization.

### Bounds and limits

- Cell ids are bounded to the valid cell range; out-of-range values are rejected.
- Iteration over cells is bounded to a fixed maximum per call.
- Persistent state growth is bounded by one cell and at most one precise record
  per pet, plus one consent entry per granted viewer.

## Active-alert index and pagination

Active alerts are listed through a stable, cursor-based index keyed by region
and status. The index is append-only per alert and never rewrites existing
entries, so cursors remain valid across concurrent writes.

### `list_active_alerts`

`list_active_alerts(region, status, cursor, page_size)` returns a bounded page
of active alerts for the given region and status.

- `page_size` is bounded to a fixed maximum; larger values are rejected with
  `PageSizeOutOfRange` and zero is rejected as well.
- The returned cursor is **opaque**: callers must pass it back unmodified and
  must not parse or construct cursors themselves.
- A cursor is bound to the index **generation** it was issued under. Any
  operation that changes the index generation (for example a region/status
  reindex) invalidates outstanding cursors; a stale cursor is rejected with
  `CursorInvalid` rather than silently returning shifted results.
- Iteration per call is bounded to `page_size` entries, so a single call can
  never scan the whole index.
- A `None` cursor starts from the beginning of the index for that
  region/status pair.

### Duplicate-alert detection

An `ActiveAlertByPet(pet_id)` index tracks the currently active alert for a
pet. While an earlier alert is active, a repeated alert for the same pet is
rejected with `DuplicateActiveAlert`; the existing alert id is returned in the
error payload so callers can link to it.

Changed circumstances have a documented resolution path:

1. The owner resolves or cancels the earlier alert (`resolve_alert`), which
   clears `ActiveAlertByPet(pet_id)` and bumps the index generation.
2. The owner may then submit a new alert for the same pet.

Linking instead of rejecting is not supported implicitly: a caller that wants
to reference the earlier alert must resolve it first, keeping the index
consistent and bounded to at most one active alert per pet.

### Storage layout (alerts)

| Key | Value | Visibility |
| --- | --- | --- |
| `AlertIndex(region, status, seq)` | alert id | public |
| `AlertIndexGeneration(region, status)` | generation counter | public |
| `ActiveAlertByPet(pet_id)` | active alert id | public |

Storage-version conventions are preserved: new entries are written under the
existing versioned key namespace and existing keys are not migrated in place.

### Bounds and limits

- `page_size` is bounded to a fixed maximum per call.
- At most one active alert per pet is tracked in `ActiveAlertByPet`.
- Index growth is bounded by the number of active alerts; resolved alerts are
  removed from the active index.

### Error codes

| Code | Meaning |
| --- | --- |
| `CellOutOfRange` | Provided cell id is outside the valid range |
| `NotOwner` | Caller is not the pet owner |
| `ConsentRequired` | Precise coordinates requested without consent |
| `AlreadySet` | Cell already set for this pet (replay protection) |
| `Expired` | Consent or alert has expired |
| `PageSizeOutOfRange` | Requested page size is zero or above the maximum |
| `CursorInvalid` | Cursor is malformed or bound to a stale generation |
| `DuplicateActiveAlert` | An active alert already exists for this pet |

### Events

- `cell_set` — emitted when a coarse cell is written.
- `consent_granted` / `consent_revoked` — emitted on consent changes.
- `alert_indexed` — emitted when an alert is added to the active index.
- `alert_resolved` — emitted when an active alert is resolved or cancelled.
- `alert_index_reindexed` — emitted when the index generation is bumped.

### ABI / storage changes

- Added `set_cell`, `get_cell`, `grant_consent`, `revoke_consent`, and
  `get_precise` entry points.
- Added `list_active_alerts` entry point with cursor-based pagination.
- Added the `Cell`, `Precise`, and `Consent` storage keys described above.
- Added the `AlertIndex`, `AlertIndexGeneration`, and `ActiveAlertByPet`
  storage keys described above.
- Compatibility fixtures must be updated when these public interfaces change.

### Tests

Focused tests cover successful use, unauthorized access, boundary values for
cell ids, and replay/expiry behavior. Failed calls are asserted to leave state
unchanged.

Additional focused tests cover:

- Pagination: successful multi-page listing, `page_size` boundary values
  (zero, maximum, maximum + 1), and stale-cursor rejection after a generation
  bump.
- Duplicate detection: rejection of a repeated alert while an earlier alert is
  active, the resolve-then-resubmit resolution path, and unchanged state after
  a rejected duplicate submission.
