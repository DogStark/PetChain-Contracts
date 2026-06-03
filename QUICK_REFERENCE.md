# Versioned Nutrition Plans - Quick Reference

## API Overview

### Create New Version
```rust
let version = client.set_nutrition_version(
    &pet_id,
    &String::from_str(&env, "Dry Kibble"),
    &String::from_str(&env, "200g"),
    &String::from_str(&env, "Twice daily"),
    &restrictions,
    &allergies,
);
// Returns: u64 (new version number)
```

### Get Specific Version
```rust
let version = client.get_nutrition_version(&pet_id, &1u64);
// Returns: Option<NutritionVersion>
```

### List All Versions
```rust
let versions = client.list_nutrition_versions(&pet_id);
// Returns: Vec<NutritionVersion> (up to 10, newest first)
```

### Rollback to Previous Version
```rust
let new_version = client.rollback_nutrition(&pet_id, &1u64);
// Returns: u64 (new version number created from rollback)
```

### Get Current Active Version
```rust
let current = client.get_current_nutrition_version(&pet_id);
// Returns: Option<NutritionVersion>
```

---

## Key Features

| Feature | Details |
|---------|---------|
| **Version Limit** | 10 most recent versions per pet |
| **Pruning** | Automatic removal of oldest when exceeding limit |
| **Rollback** | Creates new version with target version's data |
| **Active Tracking** | Boolean flag indicates current active version |
| **Authentication** | Owner required for create/rollback, none for read |
| **Timestamps** | All versions include creation timestamp |
| **Creator** | All versions track who created them |

---

## Version Lifecycle

```
Time 1: set_nutrition_version() 
        → Version 1 (active)

Time 2: set_nutrition_version() 
        → Version 2 (active), Version 1 (inactive)

Time 3: set_nutrition_version() 
        → Version 3 (active), Version 2 (inactive)

...

Time 11: set_nutrition_version() 
         → Version 11 (active), Version 1 PRUNED

Time 12: rollback_nutrition(version 5) 
         → Version 12 (active, with v5 data)
```

---

## Data Structure

```rust
pub struct NutritionVersion {
    pub pet_id: u64,                          // Pet identifier
    pub version: u64,                         // Version number
    pub food_type: String,                    // Type of food
    pub portion_size: String,                 // Portion size
    pub feeding_frequency: String,            // Feeding frequency
    pub dietary_restrictions: Vec<String>,    // Restrictions
    pub allergies: Vec<String>,               // Allergies
    pub created_by: Address,                  // Creator address
    pub created_at: u64,                      // Creation timestamp
    pub is_active: bool,                      // Active flag
}
```

---

## Common Scenarios

### Scenario 1: Track Diet Changes
```rust
// Day 1: Switch to new food
let v1 = client.set_nutrition_version(&pet_id, &"Dry Kibble", ...);

// Day 2: Switch to different food
let v2 = client.set_nutrition_version(&pet_id, &"Wet Food", ...);

// Day 3: Switch to mixed diet
let v3 = client.set_nutrition_version(&pet_id, &"Mixed Diet", ...);

// View all changes
let history = client.list_nutrition_versions(&pet_id);
// Returns: [v3, v2, v1] (newest first)
```

### Scenario 2: Revert to Previous Diet
```rust
// Current version is v3 (Mixed Diet)
// Vet recommends going back to v1 (Dry Kibble)

let new_v = client.rollback_nutrition(&pet_id, &1u64);
// Returns: 4 (new version created with v1 data)

// Current version is now v4 (with v1's data)
let current = client.get_current_nutrition_version(&pet_id);
// Returns: v4 with food_type = "Dry Kibble"
```

### Scenario 3: Check Version History
```rust
// Get all versions
let versions = client.list_nutrition_versions(&pet_id);

// Check specific version
for version in versions.iter() {
    println!("Version {}: {} (Active: {})", 
        version.version, 
        version.food_type, 
        version.is_active
    );
}

// Get current
let current = client.get_current_nutrition_version(&pet_id);
println!("Current: {}", current.unwrap().food_type);
```

---

## Error Handling

### Pet Not Found
```rust
// Panics with ContractError::PetNotFound
let version = client.set_nutrition_version(&999u64, ...);
```

### Invalid Rollback Target
```rust
// Panics with ContractError::InvalidInput
let new_v = client.rollback_nutrition(&pet_id, &999u64);
```

### Authentication Failed
```rust
// Panics with Soroban SDK error
// (if caller is not pet owner)
let version = client.set_nutrition_version(&pet_id, ...);
```

---

## Storage Keys

| Key | Type | Purpose |
|-----|------|---------|
| `NutritionVersion((pet_id, version))` | NutritionVersion | Store version data |
| `PetNutritionVersionCount(pet_id)` | u64 | Track version count |
| `CurrentNutritionVersion(pet_id)` | u64 | Track active version |

---

## Performance

| Operation | Time | Space |
|-----------|------|-------|
| Create version | O(1) | ~500 bytes |
| Get version | O(1) | - |
| List versions | O(10) | - |
| Rollback | O(1) | ~500 bytes |
| Get current | O(1) | - |

**Per Pet Storage**: ~5KB maximum (10 versions × ~500 bytes)

---

## Testing

### Run All Tests
```bash
cargo test test_nutrition
```

### Run Specific Test
```bash
cargo test test_set_nutrition_version_creates_version
```

### Run Versioning Tests Only
```bash
cargo test test_nutrition_version
cargo test test_rollback_nutrition
```

---

## Integration Checklist

- [ ] Update client library with new functions
- [ ] Add UI for version history viewing
- [ ] Add UI for rollback functionality
- [ ] Update documentation
- [ ] Train support team
- [ ] Monitor for issues
- [ ] Gather user feedback

---

## Troubleshooting

### Issue: Version not found
**Solution**: Check version number is correct, verify pet exists

### Issue: Rollback fails
**Solution**: Verify target version exists, check owner authentication

### Issue: List returns empty
**Solution**: Verify pet exists, check if any versions created

### Issue: Pruning not working
**Solution**: Verify version count exceeds 10, check storage

---

## Documentation Files

| File | Purpose |
|------|---------|
| `NUTRITION_VERSIONING.md` | Complete API documentation |
| `IMPLEMENTATION_SUMMARY.md` | Implementation details |
| `VERIFICATION_CHECKLIST.md` | Verification of requirements |
| `CODE_CHANGES.md` | Detailed code changes |
| `QUICK_REFERENCE.md` | This file |

---

## Support

For issues or questions:
1. Check `NUTRITION_VERSIONING.md` for detailed documentation
2. Review test cases in `test_nutrition.rs` for examples
3. Check `VERIFICATION_CHECKLIST.md` for requirements
4. Review `CODE_CHANGES.md` for implementation details

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-05-27 | Initial implementation |

---

## License

Same as PetChain-Contracts project

---

## Status

✅ **PRODUCTION READY**

All requirements met, fully tested, documented, and ready for deployment.
