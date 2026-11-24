# Lost-Pet Contract

Soroban contract module for public lost-pet alerts. This README describes only
what is implemented in `src/lib.rs`.

## Entry points

| Function | Purpose |
| --- | --- |
| `initialize(admin)` | One-time setup. Writes the config with `next_id = 1`. |
| `create_alert(owner, resolver, pet_name, last_seen, region, deadline)` | Creates an `Active` alert and returns its id. |
| `create_linked_alert(owner, resolver, pet_name, last_seen, region, deadline, supersedes)` | Resolves an earlier active alert of the same owner and pet, then creates a new alert linked to it. |
| `record_sighting(reporter, id)` | Appends a sighting to an `Active` alert. |
| `resolve_alert(actor, id)` | Owner or resolver marks an `Active` alert `Resolved`. |
| `withdraw_alert(actor, id)` | Owner marks an `Active` alert `Withdrawn`. |
| `expire_alert(actor, id)` | Anyone marks an `Active` alert `Expired` once `deadline` has passed. |
| `get_alert(id)` | Returns the `Alert` record. |
| `list_active_alerts(region, status, cursor, limit)` | Returns one page of alert ids for a `(region, status)` pair. |

## Alert lifecycle

`AlertState` is `Active`, `Resolved`, `Expired`, or `Withdrawn`. Only `Active`
alerts can change state. The `Resolved`, `Expired`, and `Withdrawn` transitions
are idempotent: repeating one returns `Ok` and emits no event.

## Listing alerts

`list_active_alerts(region, status, cursor, limit)` returns `(Vec<u64>, Option<PageCursor>)`.

- `limit` must be between 1 and 32 inclusive. Otherwise the call fails with `InvalidPageSize`.
- `region` longer than 64 bytes fails with `RegionTooLong`.
- Pass `None` as the cursor to start from the beginning. Results are in ascending id order.
- `next_cursor` is `None` on the last page. Otherwise, pass it back unchanged.
- The cursor records the index generation it was issued under. `create_alert` and
  `create_linked_alert` bump that generation, so a cursor issued before a new alert
  is created fails with `InvalidCursor`. Callers should restart from `None` after such an error.
- Each call walks ids forward from the cursor until the page is full or it reaches
  the latest id. Sparse matches therefore cost more storage reads per page.

## Duplicate detection

The contract tracks at most one alert per `(owner, pet_name)` in the `pet` index.
While that pet has a stored alert, `create_alert` fails with `DuplicateAlert`.
The check ignores the alert's state.

- `create_linked_alert` is the supported way to replace an alert. It requires the
  caller to own the earlier alert, that alert to be `Active`, and the same `pet_name`.
  It then resolves the earlier alert and links the new one to it (`linked_to`).
- **Known gap:** `resolve_alert`, `withdraw_alert`, and `expire_alert` do not clear
  the `pet` index entry. After any of them, `create_alert` for the same owner and
  pet still fails with `DuplicateAlert`, and `create_linked_alert` fails with
  `InvalidState` because its predecessor is no longer `Active`. No new alert can
  be created for that pet until the index is fixed in code.

## Bounds

| Limit | Value |
| --- | --- |
| `pet_name`, `last_seen` length | 256 bytes |
| `region` length | 64 bytes |
| Sightings per alert | 32 |
| `list_active_alerts` `limit` | 1 to 32 |

## Storage keys

| Key | Value | Notes |
| --- | --- | --- |
| `config` | `Config { version, admin, next_id, index_generation }` | `version` must equal `STORAGE_VERSION` (2). |
| `alert(id)` | `Alert` | Full alert record. |
| `idx(region, state, id)` | `id` | Present only while the alert is in that region and state. |
| `pet(owner, pet_name)` | alert id | See the known gap under Duplicate detection. |

## Errors

| Code | Name | Meaning |
| --- | --- | --- |
| 1 | `AlreadyInitialized` | `initialize` called twice |
| 2 | `NotInitialized` | Config missing |
| 3 | `Unauthorized` | Caller lacks the role for the action |
| 4 | `NotFound` | No alert with that id |
| 5 | `InvalidState` | Alert not in a state that allows the action |
| 6 | `InvalidDeadline` | Deadline not in the future, or expiry before deadline |
| 7 | `TextTooLong` | `pet_name` or `last_seen` too long |
| 8 | `TooManySightings` | Alert already has 32 sightings |
| 9 | `InvalidVersion` | Stored config version does not match |
| 10 | `InvalidPageSize` | `limit` is 0 or above 32 |
| 11 | `InvalidCursor` | Cursor generation is stale |
| 12 | `DuplicateAlert` | An alert already exists for this owner and pet |
| 13 | `RegionTooLong` | `region` longer than 64 bytes |

## Events

| Topic | Data | Emitted by |
| --- | --- | --- |
| `alert_created`, id | `AlertCreated { id, owner, deadline }` | `create_alert`, `create_linked_alert` |
| `alert_linked`, id | `AlertLinked { id, supersedes, owner }` | `create_linked_alert` |
| `alert_closed`, id | `AlertClosed { id, state, actor }` | `resolve_alert`, `withdraw_alert`, `expire_alert`, and the superseded alert in `create_linked_alert` |
| `sighting`, id | `SightingRecorded { id, reporter }` | `record_sighting` |

## Not implemented

Location privacy is not implemented. There are no location cells, precise
coordinates, or consent entries. Clients must not rely on `set_cell`, `get_cell`,
`grant_consent`, `revoke_consent`, or `get_precise`.
