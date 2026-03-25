$t = [System.IO.File]::ReadAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs')

# ── 1. Remove duplicate const block (second occurrence with comments) ──────────
$dupBlock = "    const MAX_STR_SHORT: u32 = 100;      // names, types, test_type, outcome`n    const MAX_STR_LONG: u32 = 1000;      // description, notes, results, reference_ranges`n    const MAX_VEC_MEDS: u32 = 50;        // medications vec in a medical record`n    const MAX_VEC_ATTACHMENTS: u32 = 20; // attachment_hashes vec"
$t = $t.Replace($dupBlock, "")

# ── 2. Fix broken get_grooming_history ────────────────────────────────────────
$brokenGrooming = @'
pub fn get_grooming_history(env: Env, pet_id: u64) -> Vec<GroomingRecord> {
        let count: u64 = env.storage().instance().get(&(Symbol::new(&env, "pet_grooming"), pet_id)).unwrap_or(0);
        let mut history = Vec::new(&env);
        for i in 1..=count {
            if let Some(record_id) = env.storage().instance().get::<_, u64>(&(Symbol::new(&env, "pet_grooming_idx"), pet_id, i)) {
                if let Some(record) = env.storage().instance().get::<_, GroomingRecord>(&(Symbol::new(&env, "grooming"), record_id)) {
        let count: u64 = env
            .storage()
            .instance()
            .get(&(Symbol::new(&env, "pet_grooming"), pet_id))
            .unwrap_or(0);
        let mut history = Vec::new(&env);
        for i in 1..=count {
            if let Some(record_id) = env.storage().instance().get::<_, u64>(&(
                Symbol::new(&env, "pet_grooming_idx"),
                pet_id,
                i,
            )) {
                if let Some(record) = env
                    .storage()
                    .instance()
                    .get::<_, GroomingRecord>(&(Symbol::new(&env, "grooming"), record_id))
                {
                    history.push_back(record);
                }
            }
        }
        history
    }

    
'@

$fixedGrooming = @'
pub fn get_grooming_history(env: Env, pet_id: u64) -> Vec<GroomingRecord> {
        let count: u64 = env.storage().instance()
            .get(&(Symbol::new(&env, "pet_grooming"), pet_id)).unwrap_or(0);
        let mut history = Vec::new(&env);
        for i in 1..=count {
            if let Some(record_id) = env.storage().instance()
                .get::<_, u64>(&(Symbol::new(&env, "pet_grooming_idx"), pet_id, i))
            {
                if let Some(record) = env.storage().instance()
                    .get::<_, GroomingRecord>(&(Symbol::new(&env, "grooming"), record_id))
                {
                    history.push_back(record);
                }
            }
        }
        history
    }

    
'@

$t = $t.Replace($brokenGrooming, $fixedGrooming)

# ── 3. Fix get_vet_average_rating (total is u32 but closure captures env) ─────
# The issue: `total = total.checked_add(1)` — `total` is u32 but the closure
# captures `&env` which is moved. Fix: use rating instead of 1.
$brokenRating = "        let mut total = 0u32;`n        for review in reviews.iter() {`n            total = total.checked_add(1).unwrap_or_else(|| panic_with_error!(&env, ContractError::CounterOverflow));`n        }`n        total / reviews.len()"
$fixedRating   = "        let mut total = 0u32;`n        for review in reviews.iter() {`n            total = total.saturating_add(review.rating);`n        }`n        total / reviews.len()"
$t = $t.Replace($brokenRating, $fixedRating)

[System.IO.File]::WriteAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs', $t)
Write-Host "Done. New length: $($t.Length)"
