# New Contract Issues for PetChain

## Issue 1: Pet Insurance Integration System
**Priority**: P1 (High)  
**Difficulty**: Hard  
**Labels**: `rust`, `enhancement`, `insurance`, `integration`

### Description
Create a system to integrate pet insurance information and claims directly into the smart contract.

### What needs to be done
- Add insurance policy storage (policy_id, provider, coverage_type, expiry_date)
- Implement `add_insurance_policy()` function
- Implement `get_pet_insurance()` function
- Implement `update_insurance_status()` function
- Add insurance claim tracking
- Emit events for insurance updates

### Why this matters
Pet owners need to track insurance information alongside medical records for seamless claim processing.

### Files to modify
- `stellar-contracts/src/lib.rs` - Add insurance structs and functions

### Code example
```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsurancePolicy {
    pub policy_id: String,
    pub provider: String,
    pub coverage_type: String,
    pub premium: u64,
    pub coverage_limit: u64,
    pub start_date: u64,
    pub expiry_date: u64,
    pub active: bool,
}

pub fn add_insurance_policy(
    env: Env,
    pet_id: u64,
    policy_id: String,
    provider: String,
    coverage_type: String,
    premium: u64,
    coverage_limit: u64,
    expiry_date: u64,
) -> bool {
    // Implementation
}
```

### Acceptance criteria
- [ ] Insurance policy can be added to a pet
- [ ] Insurance information can be retrieved
- [ ] Policy status can be updated
- [ ] Events are emitted for insurance changes
- [ ] Tests cover all insurance functions

---

## Issue 2: Pet Breeding Records System
**Priority**: P2 (Medium)  
**Difficulty**: Medium  
**Labels**: `rust`, `enhancement`, `breeding`, `good-first-issue`

### Description
Implement a breeding records system to track lineage, breeding history, and offspring information.

### What needs to be done
- Add breeding record struct (sire_id, dam_id, breeding_date, offspring_ids)
- Implement `add_breeding_record()` function
- Implement `get_breeding_history()` function
- Implement `get_offspring()` function
- Add pedigree tracking

### Why this matters
Breeders need to maintain accurate breeding records for pedigree documentation and health tracking.

### Files to modify
- `stellar-contracts/src/lib.rs` - Add breeding structs and functions

### Code example
```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreedingRecord {
    pub id: u64,
    pub sire_id: u64,
    pub dam_id: u64,
    pub breeding_date: u64,
    pub offspring_ids: Vec<u64>,
    pub breeder: Address,
    pub notes: String,
}

pub fn add_breeding_record(
    env: Env,
    sire_id: u64,
    dam_id: u64,
    breeding_date: u64,
) -> u64 {
    // Implementation
}
```

### Acceptance criteria
- [ ] Breeding records can be created
- [ ] Offspring can be linked to parents
- [ ] Breeding history can be retrieved
- [ ] Tests cover breeding functions

---

## Issue 3: Pet Nutrition & Diet Tracking
**Priority**: P2 (Medium)  
**Difficulty**: Easy  
**Labels**: `rust`, `enhancement`, `nutrition`, `good-first-issue`

### Description
Add functionality to track pet nutrition plans, dietary restrictions, and feeding schedules.

### What needs to be done
- Add diet plan struct (food_type, portion_size, frequency, restrictions)
- Implement `set_diet_plan()` function
- Implement `get_diet_plan()` function
- Add allergy and restriction tracking
- Track weight changes over time

### Why this matters
Proper nutrition tracking helps vets and owners maintain pet health and identify dietary issues.

### Files to modify
- `stellar-contracts/src/lib.rs` - Add nutrition structs and functions

### Code example
```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DietPlan {
    pub pet_id: u64,
    pub food_type: String,
    pub portion_size: String,
    pub feeding_frequency: String,
    pub dietary_restrictions: Vec<String>,
    pub allergies: Vec<String>,
    pub created_by: Address,
    pub created_at: u64,
}

pub fn set_diet_plan(
    env: Env,
    pet_id: u64,
    food_type: String,
    portion_size: String,
    frequency: String,
    restrictions: Vec<String>,
) -> bool {
    // Implementation
}
```

### Acceptance criteria
- [ ] Diet plans can be created and updated
- [ ] Dietary restrictions can be tracked
- [ ] Diet history can be retrieved
- [ ] Tests cover nutrition functions

---

## Issue 4: Pet Behavioral Records System
**Priority**: P2 (Medium)  
**Difficulty**: Medium  
**Labels**: `rust`, `enhancement`, `behavior`

### Description
Implement a system to track pet behavioral patterns, training progress, and behavioral issues.

### What needs to be done
- Add behavior record struct (behavior_type, severity, date, notes)
- Implement `add_behavior_record()` function
- Implement `get_behavior_history()` function
- Add training milestone tracking
- Track behavioral improvements

### Why this matters
Behavioral records help trainers and vets identify patterns and provide better care recommendations.

### Files to modify
- `stellar-contracts/src/lib.rs` - Add behavior structs and functions

### Code example
```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BehaviorType {
    Aggression,
    Anxiety,
    Training,
    Socialization,
    Other,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BehaviorRecord {
    pub id: u64,
    pub pet_id: u64,
    pub behavior_type: BehaviorType,
    pub severity: u32,
    pub description: String,
    pub recorded_by: Address,
    pub recorded_at: u64,
}

pub fn add_behavior_record(
    env: Env,
    pet_id: u64,
    behavior_type: BehaviorType,
    severity: u32,
    description: String,
) -> u64 {
    // Implementation
}
```

### Acceptance criteria
- [ ] Behavior records can be added
- [ ] Behavior history can be retrieved
- [ ] Training milestones can be tracked
- [ ] Tests cover behavior functions

---

## Issue 5: Pet Grooming & Maintenance Records
**Priority**: P3 (Low)  
**Difficulty**: Easy  
**Labels**: `rust`, `enhancement`, `grooming`, `good-first-issue`

### Description
Add functionality to track grooming appointments, maintenance schedules, and grooming history.

### What needs to be done
- Add grooming record struct (service_type, groomer, date, notes)
- Implement `add_grooming_record()` function
- Implement `get_grooming_history()` function
- Add grooming schedule reminders
- Track grooming expenses

### Why this matters
Regular grooming is essential for pet health, and tracking helps maintain schedules and identify issues.

### Files to modify
- `stellar-contracts/src/lib.rs` - Add grooming structs and functions

### Code example
```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroomingRecord {
    pub id: u64,
    pub pet_id: u64,
    pub service_type: String,
    pub groomer: String,
    pub date: u64,
    pub next_due: u64,
    pub cost: u64,
    pub notes: String,
}

pub fn add_grooming_record(
    env: Env,
    pet_id: u64,
    service_type: String,
    groomer: String,
    cost: u64,
    notes: String,
) -> u64 {
    // Implementation
}
```

### Acceptance criteria
- [ ] Grooming records can be added
- [ ] Grooming history can be retrieved
- [ ] Next grooming date can be calculated
- [ ] Tests cover grooming functions

---

## Issue 6: Multi-Signature Wallet for Pet Ownership
**Priority**: P1 (High)  
**Difficulty**: Hard  
**Labels**: `rust`, `enhancement`, `security`, `multisig`

### Description
Enhance the existing multisig system to support multi-signature requirements for critical pet operations.

### What needs to be done
- Extend current multisig to cover pet transfers
- Add signature threshold configuration
- Implement `require_multisig_for_transfer()` function
- Add signature collection and verification
- Support time-locked operations

### Why this matters
For high-value pets or shared custody situations, multi-signature approval adds security and accountability.

### Files to modify
- `stellar-contracts/src/lib.rs` - Extend multisig functionality

### Code example
```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultisigConfig {
    pub pet_id: u64,
    pub signers: Vec<Address>,
    pub threshold: u32,
    pub enabled: bool,
}

pub fn configure_multisig(
    env: Env,
    pet_id: u64,
    signers: Vec<Address>,
    threshold: u32,
) -> bool {
    // Implementation
}

pub fn multisig_transfer_pet(
    env: Env,
    pet_id: u64,
    to: Address,
    signatures: Vec<Address>,
) -> bool {
    // Verify threshold met before transfer
}
```

### Acceptance criteria
- [ ] Multisig can be configured per pet
- [ ] Transfers require threshold signatures
- [ ] Signature verification works correctly
- [ ] Tests cover multisig scenarios

---

## Issue 7: Pet Activity & Exercise Tracking
**Priority**: P2 (Medium)  
**Difficulty**: Easy  
**Labels**: `rust`, `enhancement`, `health`, `good-first-issue`

### Description
Implement activity tracking to monitor pet exercise, activity levels, and health metrics.

### What needs to be done
- Add activity record struct (activity_type, duration, intensity, date)
- Implement `add_activity_record()` function
- Implement `get_activity_history()` function
- Calculate activity statistics
- Track exercise goals

### Why this matters
Activity tracking helps owners and vets ensure pets get adequate exercise and identify health changes.

### Files to modify
- `stellar-contracts/src/lib.rs` - Add activity structs and functions

### Code example
```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivityType {
    Walk,
    Run,
    Play,
    Training,
    Other,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityRecord {
    pub id: u64,
    pub pet_id: u64,
    pub activity_type: ActivityType,
    pub duration_minutes: u32,
    pub intensity: u32,
    pub distance_meters: u32,
    pub recorded_at: u64,
    pub notes: String,
}

pub fn add_activity_record(
    env: Env,
    pet_id: u64,
    activity_type: ActivityType,
    duration: u32,
    intensity: u32,
) -> u64 {
    // Implementation
}

pub fn get_activity_stats(
    env: Env,
    pet_id: u64,
    days: u32,
) -> (u32, u32) {
    // Return (total_duration, total_distance)
}
```

### Acceptance criteria
- [ ] Activity records can be added
- [ ] Activity history can be retrieved
- [ ] Activity statistics can be calculated
- [ ] Tests cover activity functions

---

## Summary

These 7 issues cover:
1. **Insurance Integration** (P1, Hard) - Critical for real-world adoption
2. **Breeding Records** (P2, Medium) - Important for breeders
3. **Nutrition Tracking** (P2, Easy) - Health management
4. **Behavioral Records** (P2, Medium) - Training and behavior
5. **Grooming Records** (P3, Easy) - Maintenance tracking
6. **Enhanced Multisig** (P1, Hard) - Security improvement
7. **Activity Tracking** (P2, Easy) - Health monitoring

**Priority Distribution:**
- P1 (High): 2 issues
- P2 (Medium): 4 issues
- P3 (Low): 1 issue

**Difficulty Distribution:**
- Easy: 3 issues (good-first-issue)
- Medium: 2 issues
- Hard: 2 issues
