# Grooming API Quick Reference

## Functions

### add_grooming_record
Add a new grooming record for a pet.

**Parameters:**
- `pet_id: u64` - The pet's ID
- `service_type: String` - Type of grooming service (e.g., "Full Grooming", "Nail Trim")
- `groomer: String` - Name of groomer or grooming facility
- `cost: u64` - Cost of the service
- `notes: String` - Additional notes about the grooming session

**Returns:** `u64` - The grooming record ID

**Auth:** Requires pet owner authentication

**Panics:** If pet not found

---

### get_grooming_history
Retrieve all grooming records for a pet.

**Parameters:**
- `pet_id: u64` - The pet's ID

**Returns:** `Vec<GroomingRecord>` - List of all grooming records (empty if none)

**Auth:** None required (read-only)

---

### get_next_grooming_date
Get the next scheduled grooming date for a pet.

**Parameters:**
- `pet_id: u64` - The pet's ID

**Returns:** `Option<u64>` - Next grooming date timestamp, or None if no history

**Auth:** None required (read-only)

**Note:** Returns the next_due date from the most recent grooming record

---

### get_grooming_expenses
Calculate total grooming expenses for a pet.

**Parameters:**
- `pet_id: u64` - The pet's ID

**Returns:** `u64` - Total cost of all grooming services

**Auth:** None required (read-only)

---

## GroomingRecord Structure

```rust
{
    id: u64,           // Unique record ID
    pet_id: u64,       // Pet this record belongs to
    service_type: String,  // Type of service
    groomer: String,   // Groomer name/facility
    date: u64,         // When grooming occurred (timestamp)
    next_due: u64,     // Next recommended grooming (timestamp)
    cost: u64,         // Cost of service
    notes: String,     // Additional notes
}
```

## Example Usage

```rust
// Add grooming record
let id = contract.add_grooming_record(
    1,  // pet_id
    String::from_str(&env, "Full Grooming"),
    String::from_str(&env, "Happy Paws Spa"),
    5000,  // cost
    String::from_str(&env, "Haircut, bath, nail trim")
);

// Get history
let history = contract.get_grooming_history(1);
for record in history.iter() {
    // Process each record
}

// Check next grooming date
if let Some(next_date) = contract.get_next_grooming_date(1) {
    // Schedule reminder
}

// Get total expenses
let total = contract.get_grooming_expenses(1);
```
