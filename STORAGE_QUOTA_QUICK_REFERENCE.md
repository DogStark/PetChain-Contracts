# Storage Quota System - Quick Reference

## Issue #676: Soroban Contract Storage Quota

### Problem
No per-pet storage quota existed. A single pet could consume unbounded storage.

### Solution
Implemented configurable storage quotas enforced at write time with per-pet tracking.

---

## Quick API Reference

### Query Functions

#### Get Storage Usage
```rust
pub fn get_storage_usage(env: Env, pet_id: u64) -> StorageUsage
```
**Returns:**
```rust
StorageUsage {
    pet_id: u64,
    current_count: u64,  // Current number of entries
    quota: u64,          // Maximum allowed entries
}
```
**Example:**
```rust
let usage = client.get_storage_usage(&pet_id);
println!("Used: {}/{}", usage.current_count, usage.quota);
```

---

### Admin Functions

#### Set Global Default Quota
```rust
pub fn set_global_storage_quota(env: Env, admin: Address, quota: u64)
```
**Purpose:** Sets default quota for all pets without custom quotas  
**Auth:** Admin only  
**Event:** `GlobalStorageQuotaSet`

**Example:**
```rust
// Set global quota to 500 entries
client.set_global_storage_quota(&admin, &500);
```

#### Set Per-Pet Custom Quota
```rust
pub fn set_pet_storage_quota(env: Env, admin: Address, pet_id: u64, quota: u64)
```
**Purpose:** Sets custom quota for specific pet (overrides global)  
**Auth:** Admin only  
**Event:** `PetStorageQuotaSet`

**Example:**
```rust
// Give VIP pet 2000 entry quota
client.set_pet_storage_quota(&admin, &pet_id, &2000);
```

---

## Default Values

| Setting | Value | Description |
|---------|-------|-------------|
| `DEFAULT_STORAGE_QUOTA` | 1000 | Hardcoded default if no global quota set |
| Initial pet usage | 0 | New pets start with zero entries |
| Global quota (unset) | 1000 | Falls back to constant |

---

## Error Codes

| Error | Code | When It Occurs |
|-------|------|----------------|
| `StorageQuotaExceeded` | 160 | Write operation would exceed pet's quota |
| `PetNotFound` | 3 | Querying/setting quota for non-existent pet |
| `Unauthorized` | 1 | Non-admin trying to set quotas |

---

## Operations That Count Toward Quota

### Medical (6 operations)
- ✅ `add_medical_record()` - Diagnoses and treatments
- ✅ `add_vaccination()` - Vaccine records
- ✅ `add_lab_result()` - Lab test results
- ✅ `add_medication()` - Prescriptions
- ✅ `add_treatment()` - Treatment history
- ✅ `add_weight_entry()` - Weight measurements

### Behavioral (2 operations)
- ✅ `add_behavior_record()` - Behavior observations
- ✅ `add_training_milestone()` - Training achievements

### Activity (1 operation)
- ✅ `add_activity_record()` - Exercise logs

### Breeding (1 operation)
- ✅ `add_breeding_record()` - Breeding events (counts for BOTH sire and dam)

### Grooming (1 operation)
- ✅ `add_grooming_record()` - Grooming services

### Insurance (1 operation)
- ✅ `add_insurance_policy()` - Insurance policies

**Total: 13 write operations tracked**

---

## Quota Hierarchy

```
1. Per-Pet Custom Quota (if set)
   ↓ (if not set)
2. Global Default Quota (if set)
   ↓ (if not set)
3. DEFAULT_STORAGE_QUOTA constant (1000)
```

---

## Common Scenarios

### Scenario 1: New Pet
```rust
let pet_id = client.register_pet(...);
let usage = client.get_storage_usage(&pet_id);
// usage.current_count = 0
// usage.quota = 1000 (default)
```

### Scenario 2: Set Global Quota
```rust
client.set_global_storage_quota(&admin, &500);
let pet_id = client.register_pet(...);
let usage = client.get_storage_usage(&pet_id);
// usage.quota = 500 (global default)
```

### Scenario 3: VIP Pet with Custom Quota
```rust
let pet_id = client.register_pet(...);
client.set_pet_storage_quota(&admin, &pet_id, &5000);
let usage = client.get_storage_usage(&pet_id);
// usage.quota = 5000 (custom override)
```

### Scenario 4: Quota Exceeded
```rust
// Pet has quota of 2, already has 2 entries
client.add_medical_record(&pet_id, ...);
// ❌ Panics with ContractError::StorageQuotaExceeded
```

### Scenario 5: Check Before Write
```rust
let usage = client.get_storage_usage(&pet_id);
if usage.current_count < usage.quota {
    // Safe to write
    client.add_medical_record(&pet_id, ...);
} else {
    // Handle quota full
    println!("Storage quota full!");
}
```

---

## Testing Checklist

- [x] Storage usage starts at 0 for new pets
- [x] Usage increments on each write
- [x] Write rejected when quota exceeded
- [x] Global quota applies to all pets
- [x] Per-pet quota overrides global
- [x] Admin auth required for quota changes
- [x] Query returns correct count and quota
- [x] Quotas isolated per pet
- [x] Breeding increments both sire and dam

---

## Implementation Notes

### Storage Keys Used
```rust
DataKey::PetStorageUsage(pet_id)     // Current count
DataKey::PetStorageQuota(pet_id)     // Custom quota (optional)
DataKey::GlobalStorageQuota          // Global default (optional)
```

### Events Emitted
```rust
("GlobalStorageQuotaSet", quota)
("PetStorageQuotaSet", pet_id, quota)
```

### Performance
- **Reads per write:** 2 (count + quota)
- **Writes per write:** 1 (increment count)
- **Complexity:** O(1) for all operations
- **Storage per pet:** 8-16 bytes (count + optional custom quota)

---

## Migration Guide

### For Existing Deployments

1. **Deploy Updated Contract**
   - All existing pets automatically have 0 usage count
   - Default quota of 1000 applies

2. **Set Global Quota (Optional)**
   ```rust
   client.set_global_storage_quota(&admin, &desired_quota);
   ```

3. **Set Custom Quotas for Special Pets (Optional)**
   ```rust
   client.set_pet_storage_quota(&admin, &vip_pet_id, &higher_quota);
   ```

4. **Monitor Usage**
   ```rust
   let usage = client.get_storage_usage(&pet_id);
   if usage.current_count > usage.quota * 80 / 100 {
       // Warn user: 80% quota used
   }
   ```

### Backward Compatibility
- ✅ No breaking changes to existing functions
- ✅ Existing pets work normally
- ✅ No data migration required
- ✅ Gradual rollout possible

---

## Troubleshooting

### "StorageQuotaExceeded" Error
**Cause:** Pet has reached its storage quota  
**Solutions:**
1. Admin increases pet's quota: `set_pet_storage_quota()`
2. Admin increases global quota: `set_global_storage_quota()`
3. User archives/deletes old records (if implemented)

### "PetNotFound" Error
**Cause:** Querying or setting quota for non-existent pet  
**Solution:** Verify pet_id is correct and pet is registered

### "Unauthorized" Error
**Cause:** Non-admin trying to set quotas  
**Solution:** Use admin address for quota management

---

## Best Practices

1. **Set Reasonable Defaults**
   - Start with generous quotas (e.g., 1000)
   - Monitor actual usage patterns
   - Adjust based on data

2. **Monitor Usage Proactively**
   - Query usage regularly
   - Warn users at 80% capacity
   - Prevent surprise quota errors

3. **Custom Quotas for Special Cases**
   - Service animals: higher quotas
   - Show animals: higher quotas
   - Test pets: lower quotas

4. **Document Quota Policies**
   - Communicate limits to users
   - Explain what counts toward quota
   - Provide upgrade paths

---

## Support

For issues or questions:
- See full implementation: `STORAGE_QUOTA_IMPLEMENTATION.md`
- Review tests: `stellar-contracts/src/test_storage_quota.rs`
- Check code: `stellar-contracts/src/lib.rs` (search for "Issue #676")
