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

### Error codes

| Code | Meaning |
| --- | --- |
| `CellOutOfRange` | Provided cell id is outside the valid range |
| `NotOwner` | Caller is not the pet owner |
| `ConsentRequired` | Precise coordinates requested without consent |
| `AlreadySet` | Cell already set for this pet (replay protection) |
| `Expired` | Consent or alert has expired |

### Events

- `cell_set` — emitted when a coarse cell is written.
- `consent_granted` / `consent_revoked` — emitted on consent changes.

### ABI / storage changes

- Added `set_cell`, `get_cell`, `grant_consent`, `revoke_consent`, and
  `get_precise` entry points.
- Added the `Cell`, `Precise`, and `Consent` storage keys described above.
- Compatibility fixtures must be updated when these public interfaces change.

### Tests

Focused tests cover successful use, unauthorized access, boundary values for
cell ids, and replay/expiry behavior. Failed calls are asserted to leave state
unchanged.
