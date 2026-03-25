$f = 'c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs'
$lines = [System.IO.File]::ReadAllLines($f)
Write-Host ('Total lines: ' + $lines.Length)

$newLines = [System.Collections.Generic.List[string]]::new()

# Keep lines 1..6494 (0-indexed 0..6493)
for ($i = 0; $i -lt 6494; $i++) {
    $newLines.Add($lines[$i])
}

# Insert calculate_age inside the impl, then close impl
$newLines.Add('')
$newLines.Add('    // --- AGE CALCULATION ---')
$newLines.Add('    /// Calculates a pet''s approximate age from a Unix timestamp birthday.')
$newLines.Add('    ///')
$newLines.Add('    /// # Approximation')
$newLines.Add('    /// Uses 365 days/year and 30 days/month. This is intentionally approximate')
$newLines.Add('    /// and may deviate by +/-1 month from calendar-accurate results due to leap')
$newLines.Add('    /// years and variable month lengths. Sufficient for display purposes.')
$newLines.Add('    pub fn calculate_age(env: Env, birthday_timestamp: u64) -> PetAge {')
$newLines.Add('        let now = env.ledger().timestamp();')
$newLines.Add('        let elapsed_secs = if now > birthday_timestamp { now - birthday_timestamp } else { 0 };')
$newLines.Add('        let elapsed_days = elapsed_secs / 86400;')
$newLines.Add('        let years = elapsed_days / 365;')
$newLines.Add('        let remaining_days = elapsed_days % 365;')
$newLines.Add('        let months = remaining_days / 30;')
$newLines.Add('        PetAge { years, months }')
$newLines.Add('    }')
$newLines.Add('}')
$newLines.Add('')

# Lines from REAL ENCRYPTION onwards (0-indexed 6519..)
for ($i = 6519; $i -lt $lines.Length; $i++) {
    $newLines.Add($lines[$i])
}

[System.IO.File]::WriteAllLines($f, $newLines)
Write-Host ('Done. New line count: ' + $newLines.Count)
