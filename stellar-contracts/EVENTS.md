# PetChain Contract Events

All major contract actions emit events via `env.events().publish()` so frontends and backends can react to on-chain changes in real-time. Events are indexed by topic and optional key (e.g. `pet_id`) for efficient filtering.

## Event list

| Topic | When | Data structure | Index key |
|-------|------|----------------|-----------|
| **PetRegistered** | `register_pet()` | `PetRegisteredEvent`: pet_id, owner, name, species, timestamp | pet_id |
| **PetUpdated** | `update_pet_profile()` | `PetUpdatedEvent`: pet_id, updated_by, timestamp | pet_id |
| **RecordAdded** | `add_medical_record()` | `RecordAddedEvent`: pet_id, record_id, vet_address, timestamp | pet_id |
| **VaccinationAdded** | `add_vaccination()` | `VaccinationAddedEvent`: vaccine_id, pet_id, veterinarian, vaccine_type, next_due_date, timestamp | pet_id |
| **AccessGranted** | `grant_access()` | `AccessGrantedEvent`: pet_id, granter, grantee, access_level, expires_at, timestamp | pet_id |
| **AccessRevoked** | `revoke_access()` | `AccessRevokedEvent`: pet_id, granter, grantee, timestamp | pet_id |
| **PetOwnershipTransferred** | `transfer_ownership()` / multisig transfer | `PetOwnershipTransferredEvent`: pet_id, old_owner, new_owner, timestamp | (topic only or proposal id) |
| **TagLinked** | `link_tag()` | `TagLinkedEvent`: tag_id, pet_id, owner, timestamp | pet_id |
| **LostPetReported** | `report_lost()` | `LostPetReportedEvent`: alert_id, pet_id, reported_by, timestamp | pet_id |

Additional events (same pattern):

- **TAG_DEACTIVATED** / **TAG_REACTIVATED** – tag lifecycle
- **MedicalRecordAdded** – legacy alias; prefer **RecordAdded** for new indexing
- **TreatmentAdded**, **InsuranceAdded**, **InsuranceUpdated**, **InsuranceClaimSubmitted**, **InsuranceClaimStatusUpdated** – medical/insurance flows

## Indexing

- **By topic**: Subscribe to e.g. `PetRegistered` to see all new registrations.
- **By key**: When the first topic element is `(topic, pet_id)`, indexers can filter by `pet_id` to get events for a single pet.
- **By contract**: All events are emitted by the contract instance; filter by contract ID to get only PetChain events.

## Usage

Off-chain services should listen for these topics and persist or forward events for:

- Real-time UI updates (e.g. “New vaccination recorded”)
- Analytics and dashboards
- Access and audit logs
- Lost-pet alert feeds
