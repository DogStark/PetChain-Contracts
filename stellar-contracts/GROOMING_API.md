# Grooming Tracking System

## Overview
The grooming tracking system allows pet owners to maintain comprehensive records of their pet's grooming appointments, track expenses, and schedule future grooming sessions.

## Features

### 1. Grooming Record Management
- Add detailed grooming records with service type, groomer, cost, and notes
- Automatic calculation of next grooming due date (default: 60 days)
- Complete grooming history retrieval
- Expense tracking across all grooming sessions

### 2. Data Structure

```rust
pub struct GroomingRecord {
    pub id: u64,              // Unique record identifier
    pub pet_id: u64,          // Associated pet ID
    pub service_type: String, // Type of grooming service
    pub groomer: String,      // Name of groomer/facility
    pub date: u64,            // Timestamp of service
    pub next_due: u64,        // Next recommended grooming date
    pub cost: u64,            // Cost in base currency units
    pub notes: String,        // Additional notes
}
```

## API Functions

### add_grooming_record
Adds a new grooming record for a pet.

**Parameters:**
- `pet_id: u64` - The pet's unique identifier
- `service_type: String` - Type of grooming (e.g., "Full Grooming", "Nail Trim")
- `groomer: String` - Name of groomer or facility
- `cost: u64` - Cost of the service
- `notes: String` - Additional notes about the service

**Returns:** `u64` - The unique ID of the created grooming record

**Authorization:** Requires pet owner authentication

**Example:**
```rust
let grooming_id = client.add_grooming_record(
    &pet_id,
    &String::from_str(&env, "Full Grooming"),
    &String::from_str(&env, "Pet Spa"),
    &5000,
    &String::from_str(&env, "Haircut and bath"),
);
```

### get_grooming_history
Retrieves all grooming records for a specific pet.

**Parameters:**
- `pet_id: u64` - The pet's unique identifier

**Returns:** `Vec<GroomingRecord>` - Vector of all grooming records

**Example:**
```rust
let history = client.get_grooming_history(&pet_id);
for record in history.iter() {
    // Process each grooming record
}
```

### get_next_grooming_date
Calculates the next recommended grooming date based on the most recent grooming record.

**Parameters:**
- `pet_id: u64` - The pet's unique identifier

**Returns:** `Option<u64>` - Next grooming date timestamp, or None if no records exist

**Example:**
```rust
let next_date = client.get_next_grooming_date(&pet_id);
if let Some(date) = next_date {
    // Schedule reminder for this date
}
```

### get_grooming_expenses
Calculates total grooming expenses for a pet across all records.

**Parameters:**
- `pet_id: u64` - The pet's unique identifier

**Returns:** `u64` - Total expenses in base currency units

**Example:**
```rust
let total_expenses = client.get_grooming_expenses(&pet_id);
```

## Storage Keys

The grooming system uses the following storage keys:

```rust
pub enum GroomingKey {
    GroomingRecord(u64),           // Individual record by ID
    GroomingRecordCount,           // Global count of records
    PetGroomingCount(u64),         // Count per pet
    PetGroomingIndex((u64, u64)),  // Pet-specific index
}
```

## Use Cases

### 1. Regular Grooming Maintenance
Pet owners can track regular grooming appointments to maintain their pet's hygiene and appearance.

### 2. Expense Management
Track grooming costs over time to budget for pet care expenses.

### 3. Schedule Reminders
Use next_due dates to set up automated reminders for upcoming grooming appointments.

### 4. Service History
Maintain a complete history of grooming services for veterinary reference or when changing groomers.

## Testing

Comprehensive tests are provided in `test_grooming.rs`:

- ✅ Add grooming records
- ✅ Retrieve grooming history
- ✅ Calculate next grooming date
- ✅ Track grooming expenses
- ✅ Handle invalid pet IDs
- ✅ Handle empty grooming history

## Integration

The grooming system integrates seamlessly with the existing PetChain contract:

1. **Pet Ownership Verification**: Only pet owners can add grooming records
2. **Timestamp Management**: Uses Stellar ledger timestamps for consistency
3. **Storage Efficiency**: Indexed storage for fast retrieval
4. **Type Safety**: Strongly typed with Soroban SDK

## Future Enhancements

Potential improvements for future versions:

1. **Groomer Ratings**: Allow owners to rate grooming services
2. **Service Templates**: Pre-defined grooming service types
3. **Photo Attachments**: Link grooming photos via IPFS
4. **Recurring Schedules**: Automated scheduling for regular grooming
5. **Multi-Pet Discounts**: Track discounts for multiple pets
6. **Groomer Verification**: Verified groomer registry similar to vet system

## Security Considerations

- Only pet owners can add grooming records (enforced via `require_auth()`)
- All records are immutable once created
- Pet existence is verified before adding records
- Overflow protection on cost calculations

## Performance

- O(1) record creation
- O(n) history retrieval where n = number of grooming records per pet
- Efficient indexing for pet-specific queries
- Minimal storage overhead per record
