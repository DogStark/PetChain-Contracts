# Grooming Feature - Quick Reference

## Function Signatures

```rust
// Add a grooming record
pub fn add_grooming_record(
    env: Env,
    pet_id: u64,
    service_type: String,
    groomer: String,
    cost: u64,
    notes: String,
) -> u64

// Get all grooming records for a pet
pub fn get_grooming_history(env: Env, pet_id: u64) -> Vec<GroomingRecord>

// Get next grooming date
pub fn get_next_grooming_date(env: Env, pet_id: u64) -> Option<u64>

// Get total grooming expenses
pub fn get_grooming_expenses(env: Env, pet_id: u64) -> u64
```

## Quick Usage Examples

### Add Grooming Record
```rust
let grooming_id = client.add_grooming_record(
    &pet_id,
    &String::from_str(&env, "Full Grooming"),
    &String::from_str(&env, "Pet Spa"),
    &5000,
    &String::from_str(&env, "Haircut and bath"),
);
```

### Get History
```rust
let history = client.get_grooming_history(&pet_id);
```

### Check Next Date
```rust
let next_date = client.get_next_grooming_date(&pet_id);
```

### Get Total Expenses
```rust
let total = client.get_grooming_expenses(&pet_id);
```

## Test Commands

```bash
# Run all grooming tests
cargo test test_grooming

# Run specific test
cargo test test_add_grooming_record

# Build contract
cargo build --target wasm32-unknown-unknown --release
```

## Key Features
✅ Owner-only access
✅ Automatic next_due calculation (60 days)
✅ Expense tracking
✅ Complete history
✅ Safe arithmetic
✅ Comprehensive tests

## Files Changed
- `src/lib.rs` - Core implementation
- `src/test_grooming.rs` - Tests
- `GROOMING_API.md` - Full documentation
- `GROOMING_IMPLEMENTATION.md` - Implementation details
