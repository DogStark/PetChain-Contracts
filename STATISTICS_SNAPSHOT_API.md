# Statistics Snapshot API Reference

## Overview
The Statistics Snapshot system provides governance reporting capabilities by capturing point-in-time snapshots of key platform metrics.

## Functions

### 1. `take_statistics_snapshot(admin: Address) -> u64`

**Description**: Captures a complete snapshot of all key statistics for governance reporting.

**Authorization**: ⚠️ Requires multisig admin authorization

**Parameters**:
- `admin: Address` - The admin address authorized to take the snapshot

**Returns**: `u64` - The unique snapshot ID

**Captured Metrics**:
- Total pets registered
- Active pets count
- Species distribution (Dog, Cat, Bird, Rabbit, Other)
- Total veterinarians
- Total medical records
- Total vaccinations
- Total insurance claims
- Ledger timestamp

**Storage Limit**: Maximum 100 snapshots. When the 101st is taken, the oldest is automatically purged.

**Example**:
```rust
let admin = Address::from(...);
let snapshot_id = contract.take_statistics_snapshot(&admin);
// Returns: 1 (first snapshot)
```

**Error Conditions**:
- Panics with `NotAnAdmin` if caller is not an authorized admin
- Panics with `CounterOverflow` if snapshot count exceeds u64::MAX

---

### 2. `get_snapshot(snapshot_id: u64) -> Option<StatisticsSnapshot>`

**Description**: Retrieves a previously captured snapshot by its ID.

**Authorization**: ✅ Public (no authorization required)

**Parameters**:
- `snapshot_id: u64` - The ID of the snapshot to retrieve

**Returns**: `Option<StatisticsSnapshot>`
- `Some(snapshot)` if the snapshot exists
- `None` if the snapshot doesn't exist or was purged

**Example**:
```rust
let snapshot = contract.get_snapshot(&1);
match snapshot {
    Some(s) => {
        println!("Total Pets: {}", s.total_pets);
        println!("Active Pets: {}", s.active_pets);
    },
    None => println!("Snapshot not found"),
}
```

---

### 3. `get_snapshot_count() -> u64`

**Description**: Returns the total number of snapshots that have been taken (including purged ones).

**Authorization**: ✅ Public (no authorization required)

**Returns**: `u64` - The total snapshot counter (never decreases)

**Example**:
```rust
let count = contract.get_snapshot_count();
println!("Total snapshots taken: {}", count);
// If 105 snapshots have been taken, returns 105
// (even though only the last 100 are stored)
```

---

### 4. `get_available_snapshot_ids() -> Vec<u64>`

**Description**: Returns a list of all currently stored snapshot IDs.

**Authorization**: ✅ Public (no authorization required)

**Returns**: `Vec<u64>` - List of snapshot IDs that can be retrieved (max 100)

**Example**:
```rust
let ids = contract.get_available_snapshot_ids();
println!("Available snapshots: {:?}", ids);
// Example output: [6, 7, 8, ..., 105]
// (IDs 1-5 were purged when snapshots 101-105 were taken)

// Iterate and retrieve all available snapshots
for id in ids.iter() {
    if let Some(snapshot) = contract.get_snapshot(&id) {
        // Process snapshot
    }
}
```

---

## Data Structures

### `StatisticsSnapshot`

```rust
pub struct StatisticsSnapshot {
    pub snapshot_id: u64,
    pub timestamp: u64,
    pub total_pets: u64,
    pub active_pets: u64,
    pub species_distribution: Map<String, u64>,
    pub total_vets: u64,
    pub total_medical_records: u64,
    pub total_vaccinations: u64,
    pub total_insurance_claims: u64,
}
```

**Fields**:
- `snapshot_id` - Unique identifier for this snapshot
- `timestamp` - Unix timestamp when the snapshot was taken (from ledger)
- `total_pets` - Total number of pets ever registered
- `active_pets` - Number of currently active pets
- `species_distribution` - Map of species name to count
  - Keys: "Dog", "Cat", "Bird", "Rabbit", "Other"
  - Values: Count of pets for that species
- `total_vets` - Total number of registered veterinarians
- `total_medical_records` - Total number of medical records
- `total_vaccinations` - Total number of vaccinations administered
- `total_insurance_claims` - Total number of insurance claims submitted

---

## Usage Scenarios

### Scenario 1: Monthly Governance Report

```rust
// At the end of each month, admin takes a snapshot
let snapshot_id = contract.take_statistics_snapshot(&admin);

// Later, anyone can retrieve and analyze
let snapshot = contract.get_snapshot(&snapshot_id).unwrap();

// Generate report
println!("Monthly Report:");
println!("Total Pets: {}", snapshot.total_pets);
println!("Active Pets: {}", snapshot.active_pets);
println!("Total Vets: {}", snapshot.total_vets);
```

### Scenario 2: Historical Comparison

```rust
// Get list of all available snapshots
let ids = contract.get_available_snapshot_ids();

// Compare first and last snapshot
let first = contract.get_snapshot(&ids.first().unwrap()).unwrap();
let last = contract.get_snapshot(&ids.last().unwrap()).unwrap();

let growth = last.total_pets - first.total_pets;
println!("Pet registrations grew by {}", growth);
```

### Scenario 3: Species Distribution Analysis

```rust
let snapshot = contract.get_snapshot(&snapshot_id).unwrap();

println!("Species Distribution:");
for (species, count) in snapshot.species_distribution.iter() {
    let percentage = (count * 100) / snapshot.total_pets;
    println!("{}: {} ({}%)", species, count, percentage);
}
```

### Scenario 4: Platform Health Dashboard

```rust
// Get latest snapshot
let count = contract.get_snapshot_count();
let latest = contract.get_snapshot(&count).unwrap();

// Calculate metrics
let active_rate = (latest.active_pets * 100) / latest.total_pets;
let claims_per_pet = latest.total_insurance_claims / latest.total_pets;

println!("Platform Health:");
println!("Active Pet Rate: {}%", active_rate);
println!("Average Claims per Pet: {}", claims_per_pet);
```

---

## Storage Mechanics

### Circular Buffer (Maximum 100 Snapshots)

The system maintains exactly 100 snapshots using a circular buffer:

1. **Snapshots 1-100**: All stored at their respective index positions (0-99)
2. **Snapshot 101**: Stored at index 0, replacing snapshot 1
3. **Snapshot 102**: Stored at index 1, replacing snapshot 2
4. And so on...

**Index Calculation**: `index = (snapshot_id - 1) % 100`

**Example**:
```
Snapshot ID: 1   → Index: 0
Snapshot ID: 2   → Index: 1
...
Snapshot ID: 100 → Index: 99
Snapshot ID: 101 → Index: 0  (replaces snapshot 1)
Snapshot ID: 102 → Index: 1  (replaces snapshot 2)
Snapshot ID: 205 → Index: 4  (replaces snapshot 105)
```

### Storage Keys Used

- `SystemKey::StatisticsSnapshot(snapshot_id)` - Stores the snapshot data
- `SystemKey::SnapshotCount` - Stores the total count (never decreases)
- `SystemKey::SnapshotIndex(index)` - Stores which snapshot_id is at each index

---

## Best Practices

### 1. Regular Snapshot Schedule
```rust
// Take snapshots at regular intervals (e.g., monthly)
// This provides consistent historical data points
```

### 2. Document Snapshot Purpose
```rust
// Consider maintaining off-chain metadata about why each snapshot was taken
// Example: "Q1 2024 End", "Pre-Migration", "Post-Upgrade"
```

### 3. Verify Before Purge
```rust
// Before taking the 101st snapshot, you may want to:
let ids = contract.get_available_snapshot_ids();
let oldest = ids.first().unwrap();
let snapshot = contract.get_snapshot(&oldest).unwrap();
// Export or back up the oldest snapshot if needed
```

### 4. Handle Missing Snapshots
```rust
// Always check if snapshot exists before using
match contract.get_snapshot(&id) {
    Some(snapshot) => {
        // Process snapshot
    },
    None => {
        // Handle missing snapshot (may have been purged)
    }
}
```

---

## Integration with Existing Systems

The snapshot system integrates with existing platform components:

- **Pet Registration**: Captures `DataKey::PetCount`
- **Pet Activation**: Captures `StatsKey::ActivePetsCount`
- **Species Tracking**: Captures `DataKey::SpeciesPetCount(species)`
- **Vet Registry**: Captures `DataKey::VetCount`
- **Medical Records**: Captures `MedicalKey::MedicalRecordCount`
- **Vaccinations**: Captures `MedicalKey::VaccinationCount`
- **Insurance**: Captures `InsuranceKey::ClaimCount`

All counts are read-only and captured atomically at the moment of snapshot creation.

---

## Security & Authorization

### Admin Functions (Require Authorization)
- ✅ `take_statistics_snapshot` - Only admins can create snapshots

### Public Functions (No Authorization)
- ✅ `get_snapshot` - Transparent access for governance
- ✅ `get_snapshot_count` - Public metric
- ✅ `get_available_snapshot_ids` - Public discovery

**Rationale**: Snapshot creation is protected to prevent spam and ensure data integrity. Snapshot retrieval is public to enable transparent governance and community verification.

---

## Performance Considerations

### Gas Costs
- **Taking Snapshot**: Reads ~8 storage keys + writes 3 storage keys
- **Getting Snapshot**: Single storage read
- **Listing IDs**: Reads up to 100 storage keys (for checking existence)

### Optimization Tips
1. Take snapshots during off-peak times
2. Cache snapshot IDs off-chain if doing frequent historical analysis
3. Use `get_available_snapshot_ids()` sparingly (check existence of many snapshots)

---

## Error Handling

### Common Issues

**Issue**: Snapshot returns `None`
```rust
// Solution: Check if snapshot was purged
let count = contract.get_snapshot_count();
if snapshot_id < count - 100 {
    println!("Snapshot {} was purged", snapshot_id);
}
```

**Issue**: Admin authorization fails
```rust
// Solution: Verify admin list and authorization
let admins = contract.get_admins();
if !admins.contains(&caller) {
    println!("Not authorized - must be admin");
}
```

---

## Testing

See `test_statistics_snapshot.rs` for comprehensive test coverage including:
- Basic snapshot creation and retrieval
- Multiple snapshot independence
- Purge behavior (101st snapshot)
- Authorization checks
- Species distribution accuracy
- Point-in-time immutability
- Public access verification
