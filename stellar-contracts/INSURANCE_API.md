# Insurance System - Quick Reference Guide

## API Reference

### Functions

#### `add_insurance_policy`
Adds an insurance policy to a pet.

**Signature:**
```rust
pub fn add_insurance_policy(
    env: Env,
    pet_id: u64,
    policy_id: String,
    provider: String,
    coverage_type: String,
    premium: u64,
    coverage_limit: u64,
    expiry_date: u64,
) -> bool
```

**Parameters:**
- `pet_id` - The ID of the pet to insure
- `policy_id` - Unique identifier for the insurance policy
- `provider` - Name of the insurance provider
- `coverage_type` - Type of coverage (e.g., "Comprehensive", "Basic", "Accident Only")
- `premium` - Annual premium amount in stroops
- `coverage_limit` - Maximum coverage amount in stroops
- `expiry_date` - Unix timestamp when policy expires

**Returns:** `true` if successful, `false` if pet doesn't exist

**Events Emitted:** `InsuranceAddedEvent`

---

#### `get_pet_insurance`
Retrieves the insurance policy for a pet.

**Signature:**
```rust
pub fn get_pet_insurance(env: Env, pet_id: u64) -> Option<InsurancePolicy>
```

**Parameters:**
- `pet_id` - The ID of the pet

**Returns:** `Some(InsurancePolicy)` if policy exists, `None` otherwise

---

#### `update_insurance_status`
Updates the active status of an insurance policy.

**Signature:**
```rust
pub fn update_insurance_status(env: Env, pet_id: u64, active: bool) -> bool
```

**Parameters:**
- `pet_id` - The ID of the pet
- `active` - New status (true = active, false = inactive)

**Returns:** `true` if successful, `false` if policy doesn't exist

**Events Emitted:** `InsuranceUpdatedEvent`

---

## Data Structures

### InsurancePolicy
```rust
pub struct InsurancePolicy {
    pub policy_id: String,        // Unique policy identifier
    pub provider: String,          // Insurance provider name
    pub coverage_type: String,     // Type of coverage
    pub premium: u64,              // Annual premium amount
    pub coverage_limit: u64,       // Maximum coverage
    pub start_date: u64,           // Policy start timestamp
    pub expiry_date: u64,          // Policy expiry timestamp
    pub active: bool,              // Current status
}
```

---

## Events

### InsuranceAddedEvent
Emitted when a new insurance policy is added.

```rust
pub struct InsuranceAddedEvent {
    pub pet_id: u64,
    pub policy_id: String,
    pub provider: String,
    pub timestamp: u64,
}
```

### InsuranceUpdatedEvent
Emitted when an insurance policy status is updated.

```rust
pub struct InsuranceUpdatedEvent {
    pub pet_id: u64,
    pub policy_id: String,
    pub active: bool,
    pub timestamp: u64,
}
```

---

## Usage Examples

### Example 1: Add Insurance Policy
```rust
use soroban_sdk::{Env, String};

let env = Env::default();
let pet_id = 1;
let expiry = env.ledger().timestamp() + 31536000; // 1 year from now

let success = contract.add_insurance_policy(
    env.clone(),
    pet_id,
    String::from_str(&env, "POL-2024-001"),
    String::from_str(&env, "PetGuard Insurance"),
    String::from_str(&env, "Comprehensive"),
    2500,      // $25.00 premium (in stroops)
    100000,    // $1000.00 coverage limit
    expiry,
);

if success {
    println!("Insurance policy added successfully!");
}
```

### Example 2: Retrieve Insurance Information
```rust
let policy = contract.get_pet_insurance(env.clone(), pet_id);

match policy {
    Some(p) => {
        println!("Provider: {}", p.provider);
        println!("Coverage: {} stroops", p.coverage_limit);
        println!("Premium: {} stroops", p.premium);
        println!("Active: {}", p.active);
    },
    None => println!("No insurance policy found"),
}
```

### Example 3: Deactivate Policy
```rust
let success = contract.update_insurance_status(
    env.clone(),
    pet_id,
    false  // Deactivate
);

if success {
    println!("Policy deactivated");
}
```

### Example 4: Reactivate Policy
```rust
let success = contract.update_insurance_status(
    env.clone(),
    pet_id,
    true  // Reactivate
);

if success {
    println!("Policy reactivated");
}
```

---

## Error Handling

### Common Scenarios

1. **Pet doesn't exist**
   - `add_insurance_policy` returns `false`
   - `get_pet_insurance` returns `None`

2. **No insurance policy exists**
   - `get_pet_insurance` returns `None`
   - `update_insurance_status` returns `false`

3. **Policy already exists**
   - Calling `add_insurance_policy` again will overwrite the existing policy

---

## Integration with Frontend

### Listening to Events

```javascript
// Listen for insurance added events
contract.on('InsuranceAdded', (event) => {
  console.log(`Insurance added for pet ${event.pet_id}`);
  console.log(`Policy ID: ${event.policy_id}`);
  console.log(`Provider: ${event.provider}`);
});

// Listen for insurance updated events
contract.on('InsuranceUpdated', (event) => {
  console.log(`Insurance ${event.active ? 'activated' : 'deactivated'}`);
  console.log(`Pet ID: ${event.pet_id}`);
});
```

### Display Insurance Information

```javascript
async function displayInsurance(petId) {
  const policy = await contract.get_pet_insurance(petId);
  
  if (policy) {
    return {
      policyId: policy.policy_id,
      provider: policy.provider,
      coverageType: policy.coverage_type,
      premium: policy.premium / 10000000, // Convert stroops to XLM
      coverageLimit: policy.coverage_limit / 10000000,
      startDate: new Date(policy.start_date * 1000),
      expiryDate: new Date(policy.expiry_date * 1000),
      active: policy.active,
      daysUntilExpiry: Math.floor((policy.expiry_date - Date.now()/1000) / 86400)
    };
  }
  return null;
}
```

---

## Testing

Run insurance tests:
```bash
cd stellar-contracts
cargo test test_insurance --lib
```

Run comprehensive insurance tests:
```bash
cargo test test_insurance_comprehensive --lib
```

Run all tests:
```bash
cargo test --lib
```

---

## Deployment

Build the contract:
```bash
cd stellar-contracts
cargo build --target wasm32-unknown-unknown --release
```

Deploy to testnet:
```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/petchain_stellar.wasm \
  --network testnet
```

---

## Future Enhancements

Potential features to add:
- Multiple policies per pet
- Claim submission and tracking
- Automatic policy renewal
- Premium payment tracking
- Coverage utilization tracking
- Policy document storage (IPFS hashes)
- Beneficiary management
- Co-insurance support

---

## Support

For issues or questions:
- GitHub Issues: [PetChain-Contracts](https://github.com/DogStark/PetMedTracka-Contracts/issues)
- Telegram: [@PetChain Telegram Group](https://t.me/+Jw8HkvUhinw2YjE0)
- Contact: [@llins_x](https://t.me/llins_x)
