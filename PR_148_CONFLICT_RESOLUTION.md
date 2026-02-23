# PR #148 Merge Conflict Resolution Guide

## Summary
PR #148 (Co-ownership Support) has merge conflicts with main branch due to significant changes merged after the PR was created.

## Conflicts to Resolve

### 1. Test Module Declarations (Lines 46-78)
**Issue**: PR only has `test_export` and `test_coownership`, but main has 14 test modules.

**Resolution**: Keep ALL test modules in alphabetical order:
```rust
#[cfg(test)]
mod test;
#[cfg(test)]
mod test_access_control;
#[cfg(test)]
mod test_activity;
#[cfg(test)]
mod test_attachments;
#[cfg(test)]
mod test_behavior;
#[cfg(test)]
mod test_coownership;  // ADD THIS
#[cfg(test)]
mod test_emergency_contacts;
#[cfg(test)]
mod test_emergency_override;
#[cfg(test)]
mod test_export;
#[cfg(test)]
mod test_grooming;
#[cfg(test)]
mod test_insurance;
#[cfg(test)]
mod test_insurance_claims;
#[cfg(test)]
mod test_insurance_comprehensive;
#[cfg(test)]
mod test_multisig_transfer;
#[cfg(test)]
mod test_nutrition;
#[cfg(test)]
mod test_pet_age;
#[cfg(test)]
mod test_statistics;
```

### 2. Pet Struct Fields (Line 373)
**Issue**: PR adds `archived` and `notes`, main adds `allergies`.

**Resolution**: Keep ALL fields:
```rust
pub struct Pet {
    // ... existing fields ...
    pub microchip_id: Option<String>,
    pub photo_hashes: Vec<String>,
    pub archived: bool,           // FROM PR
    pub notes: String,            // FROM PR
    pub allergies: Vec<Allergy>,  // FROM MAIN
}
```

### 3. PetProfile Struct
**Issue**: Similar to Pet struct - needs both PR and main additions.

**Resolution**: Add `archived`, `notes` from PR AND `allergies` from main.

### 4. Function Conflicts
**Multiple locations**: PR changes authorization from `owner.require_auth()` to `require_any_owner_auth()`.

**Resolution**: 
- Keep PR's co-ownership authorization logic
- Ensure all new functions from main (activity, behavior, nutrition, grooming, insurance) are preserved
- Update new functions to use `require_any_owner_auth()` where appropriate

### 5. DataKey Enum
**Issue**: Both PR and main may have added new variants.

**Resolution**: Merge all DataKey variants from both branches.

## Steps to Resolve

1. **Rebase on latest main**:
   ```bash
   git checkout feat/co-ownership-support
   git fetch origin
   git rebase origin/main
   ```

2. **For each conflict**:
   - Open `stellar-contracts/src/lib.rs`
   - Find conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`)
   - Manually merge keeping:
     - All test modules from main + test_coownership
     - All struct fields from both branches
     - All functions from both branches
     - PR's authorization changes applied to existing functions

3. **Test after resolution**:
   ```bash
   cd stellar-contracts
   cargo build --target wasm32-unknown-unknown --release
   cargo test
   ```

4. **Commit and push**:
   ```bash
   git add stellar-contracts/src/lib.rs
   git rebase --continue
   git push --force-with-lease
   ```

## Key Changes from Main to Preserve

- Activity tracking functions (add_activity_record, get_activity_history, get_activity_stats)
- Behavior tracking functions (add_behavior_record, get_behavior_history)
- Nutrition/diet functions (set_diet_plan, get_diet_plan, get_diet_history)
- Pet age calculation (get_pet_age)
- Insurance functions (add_insurance_policy, submit_insurance_claim, etc.)
- Grooming struct (GroomingRecord) - note: functions not yet implemented
- Multisig transfer functions
- All new test files and their declarations

## Key Changes from PR to Preserve

- Co-ownership fields: `primary_owner`, `owners: Vec<Address>`
- Co-ownership functions: `add_co_owner`, `remove_co_owner`, `get_co_owners`
- Authorization refactor: `require_any_owner_auth()` helper
- `archived` and `notes` fields in Pet/PetProfile
- test_coownership.rs file

## Contact
If you need help resolving these conflicts, reach out in the PR comments or Telegram group.
