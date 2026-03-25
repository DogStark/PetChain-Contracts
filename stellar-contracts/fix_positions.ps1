$t = [System.IO.File]::ReadAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs')

# ── 1. Remove duplicate const block (positions 59615..59939) ──────────────────
# The duplicate block starts just before "const MAX_STR_SHORT" (second occurrence)
# and ends just before "pub fn register_vet"
$dupStart = 59615   # includes the leading newline
$dupEnd   = 59939   # up to but not including "    pub fn register_vet"
$t = $t.Substring(0, $dupStart) + $t.Substring($dupEnd)

# Recalculate positions after removal (removed 324 chars)
$offset1 = 324

# ── 2. Fix get_vet_average_rating (was at 150681, now 150681-324=150357) ───────
$vatStart = 150681 - $offset1
$vatEnd   = 151158 - $offset1

$fixedRating = "pub fn get_vet_average_rating(env: Env, vet: Address) -> u32 {
        let reviews = Self::get_vet_reviews(env.clone(), vet);
        if reviews.is_empty() {
            return 0;
        }
        let mut total = 0u32;
        for review in reviews.iter() {
            total = total.saturating_add(review.rating);
        }
        total / reviews.len()
    }

    // --- MEDICATION TRACKING ---

    "

$t = $t.Substring(0, $vatStart) + $fixedRating + $t.Substring($vatEnd)

# Recalculate offset (original was 477 chars, new is ~280 chars)
$origVatLen = $vatEnd - $vatStart
$newVatLen  = $fixedRating.Length
$offset2    = $offset1 + ($origVatLen - $newVatLen)

# ── 3. Fix broken get_grooming_history (was at 201455, now adjusted) ──────────
$ghStart = 201455 - $offset2
$ghEnd   = 202773 - $offset2

$fixedGrooming = "pub fn get_grooming_history(env: Env, pet_id: u64) -> Vec<GroomingRecord> {
        let count: u64 = env.storage().instance()
            .get(&(Symbol::new(&env, ""pet_grooming""), pet_id)).unwrap_or(0);
        let mut history = Vec::new(&env);
        for i in 1..=count {
            if let Some(record_id) = env.storage().instance()
                .get::<_, u64>(&(Symbol::new(&env, ""pet_grooming_idx""), pet_id, i))
            {
                if let Some(record) = env.storage().instance()
                    .get::<_, GroomingRecord>(&(Symbol::new(&env, ""grooming""), record_id))
                {
                    history.push_back(record);
                }
            }
        }
        history
    }

    "

$t = $t.Substring(0, $ghStart) + $fixedGrooming + $t.Substring($ghEnd)

[System.IO.File]::WriteAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs', $t)
Write-Host "Done. New length: $($t.Length)"
