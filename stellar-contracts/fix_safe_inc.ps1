$t = [System.IO.File]::ReadAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs')

# ── 1. Revert crate:: prefix back to bare safe_increment ──────────────────────
$t = $t.Replace("crate::safe_increment(", "safe_increment(")

# ── 2. Remove safe_increment definition from its current location ─────────────
$safeDefBlock = "// --- OVERFLOW-SAFE COUNTER HELPER ---`r`npub(crate) fn safe_increment(count: u64) -> u64 {`r`n    count.checked_add(1).unwrap_or(u64::MAX)`r`n}"
$t = $t.Replace($safeDefBlock, "")

# Also try LF-only version
$safeDefBlockLF = "// --- OVERFLOW-SAFE COUNTER HELPER ---`npub(crate) fn safe_increment(count: u64) -> u64 {`n    count.checked_add(1).unwrap_or(u64::MAX)`n}"
$t = $t.Replace($safeDefBlockLF, "")

# ── 3. Insert safe_increment BEFORE the impl block ────────────────────────────
$implMarker = "impl PetChainContract {"
$safeIncDef = "// --- OVERFLOW-SAFE COUNTER HELPER ---`r`npub(crate) fn safe_increment(count: u64) -> u64 {`r`n    count.checked_add(1).unwrap_or(u64::MAX)`r`n}`r`n`r`n"
$t = $t.Replace($implMarker, $safeIncDef + $implMarker)

# ── 4. Fix duplicate const blocks ─────────────────────────────────────────────
# Read lines and remove the second duplicate block
$lines = $t -split "`n"
$dupLines = @(
    "    const MAX_STR_SHORT: u32 = 100;      // names, types, test_type, outcome",
    "    const MAX_STR_LONG: u32 = 1000;      // description, notes, results, reference_ranges",
    "    const MAX_VEC_MEDS: u32 = 50;        // medications vec in a medical record",
    "    const MAX_VEC_ATTACHMENTS: u32 = 20; // attachment_hashes vec"
)
$dupCount = 0
$newLines = @()
foreach ($line in $lines) {
    $stripped = $line.TrimEnd("`r")
    if ($dupLines -contains $stripped) {
        $dupCount++
        if ($dupCount -le 4) {
            # Keep first occurrence
            $newLines += $line
        }
        # Skip second occurrence (dupCount 5-8)
    } else {
        $newLines += $line
    }
}
$t = $newLines -join "`n"

# ── 5. Fix grooming functions (outside impl, use PetChainContract::) ──────────
$t = $t.Replace(
    "        let history = Self::get_grooming_history(env, pet_id);",
    "        let history = PetChainContract::get_grooming_history(env, pet_id);"
)

# ── 6. Fix broken get_grooming_history / get_vet_average_rating ───────────────
# These functions have mangled bodies from the original file. Read them and fix.
# get_grooming_history: the for loop body is broken (mixed with other functions)
# We'll find and replace the broken section

[System.IO.File]::WriteAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs', $t)
Write-Host "Done. New length: $($t.Length)"
