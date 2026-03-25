
# ── Fix 1: test.rs ────────────────────────────────────────────────────────────
# Lines 1314-1491 are orphaned #[test] fns inside mod test at wrong nesting.
# The structure is:
#   line 1313: closing } of test_book_slot (inside mod test)
#   line 1314: #[test]
#   line 1315: fn test_grant_consent {   <- this fn contains #[test] fn test_get_version nested inside it
#   ...
#   line 1478: }  <- closes mod test
#   line 1479: (blank)
#   line 1480: #[test] / fn test_upgrade_requires_admin  <- outside mod test
#   line 1491: }
#   line 1492: }  <- extra brace
#
# Fix: replace lines 1313-1492 with properly structured code

$f = 'c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\test.rs'
$lines = [System.IO.File]::ReadAllLines($f)
$newLines = [System.Collections.Generic.List[string]]::new()

for ($i = 0; $i -lt $lines.Length; $i++) {
    $ln = $i + 1

    # At line 1313 (closing brace of test_book_slot inside mod test),
    # close mod test and start a new mod for the orphaned functions
    if ($ln -eq 1313) {
        $newLines.Add('    }')  # close test_book_slot
        $newLines.Add('}')      # close mod test
        $newLines.Add('')
        $newLines.Add('#[cfg(test)]')
        $newLines.Add('mod test_consent_upgrade {')
        $newLines.Add('    use crate::*;')
        $newLines.Add('    use soroban_sdk::{testutils::Address as _, Address, Env, String};')
        $newLines.Add('')
        continue
    }

    # Skip lines 1314-1492 (the old orphaned section + old mod test closing braces)
    if ($ln -ge 1314 -and $ln -le 1492) {
        # But we need to keep the content of the functions, just re-emit them
        # with proper indentation inside the new mod
        $line = $lines[$i]

        # Skip the nested #[test] inside test_grant_consent (line 1316) and
        # the fn test_get_version that was nested inside it (lines 1317-1326)
        if ($ln -ge 1316 -and $ln -le 1326) { continue }

        # Skip the old mod test closing braces (lines 1478, 1492)
        if ($ln -eq 1478 -or $ln -eq 1492) { continue }

        # Add proper indentation (these fns were at module level, add 4 spaces)
        if ($line.Length -gt 0 -and -not $line.StartsWith('#')) {
            $newLines.Add('    ' + $line)
        } else {
            $newLines.Add($line)
        }
        continue
    }

    # Close the new mod after line 1492
    if ($ln -eq 1493) {
        $newLines.Add('}')
        $newLines.Add('')
    }

    $newLines.Add($lines[$i])
}

[System.IO.File]::WriteAllLines($f, $newLines)
Write-Host ('test.rs done. Lines: ' + $newLines.Count)

# ── Fix 2: test_admin_initialization.rs — remove unused Ledger import ─────────
$f2 = 'c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\test_admin_initialization.rs'
$lines2 = [System.IO.File]::ReadAllLines($f2)
$new2 = [System.Collections.Generic.List[string]]::new()
for ($i = 0; $i -lt $lines2.Length; $i++) {
    $line = $lines2[$i]
    # Replace the import line to remove Ledger
    if ($line -match 'testutils.*Ledger') {
        $new2.Add('    testutils::Address as _,')
        continue
    }
    $new2.Add($line)
}
[System.IO.File]::WriteAllLines($f2, $new2)
Write-Host ('test_admin_initialization.rs done.')

# ── Fix 3: test_search_medical_records.rs ─────────────────────────────────────
$f3 = 'c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\test_search_medical_records.rs'
$lines3 = [System.IO.File]::ReadAllLines($f3)
$new3 = [System.Collections.Generic.List[string]]::new()
for ($i = 0; $i -lt $lines3.Length; $i++) {
    $ln = $i + 1
    $line = $lines3[$i]
    # Remove unused Medication import
    if ($line -match 'use crate.*Medication') {
        $new3.Add('    use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};')
        continue
    }
    # Fix useless >= 0 comparison (line 137)
    if ($line -match 'assert!\(results\.len\(\) >= 0\)') {
        $new3.Add('        let _ = results.len(); // boundary check — no panic')
        continue
    }
    $new3.Add($line)
}
[System.IO.File]::WriteAllLines($f3, $new3)
Write-Host ('test_search_medical_records.rs done.')

# ── Fix 4: test_get_pet_decryption.rs ─────────────────────────────────────────
$f4 = 'c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\test_get_pet_decryption.rs'
$lines4 = [System.IO.File]::ReadAllLines($f4)
$new4 = [System.Collections.Generic.List[string]]::new()
for ($i = 0; $i -lt $lines4.Length; $i++) {
    $ln = $i + 1
    $line = $lines4[$i]
    # Remove unused Vec import
    if ($line -match 'testutils::Address as _, Address, Bytes, Env, String, Vec') {
        $new4.Add('        testutils::Address as _, Address, Bytes, Env, String,')
        continue
    }
    # Fix unused env variable (line 222)
    if ($ln -eq 222 -and $line -match 'let \(env, client\) = setup\(\)') {
        $new4.Add('        let (_env, client) = setup();')
        continue
    }
    $new4.Add($line)
}
[System.IO.File]::WriteAllLines($f4, $new4)
Write-Host ('test_get_pet_decryption.rs done.')

# ── Fix 5: test_input_limits.rs — add lifetime annotation ─────────────────────
$f5 = 'c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\test_input_limits.rs'
$lines5 = [System.IO.File]::ReadAllLines($f5)
$new5 = [System.Collections.Generic.List[string]]::new()
for ($i = 0; $i -lt $lines5.Length; $i++) {
    $line = $lines5[$i]
    # Fix setup return type
    if ($line -match '^fn setup\(env: &Env\) -> \(PetChainContractClient,') {
        $new5.Add("fn setup(env: &Env) -> (PetChainContractClient<'_>, Address, Address, u64) {")
        continue
    }
    # Fix setup_with_vet return type
    if ($line -match '^fn setup_with_vet\(env: &Env\) -> \(PetChainContractClient,') {
        $new5.Add("fn setup_with_vet(env: &Env) -> (PetChainContractClient<'_>, Address, Address, Address, u64) {")
        continue
    }
    $new5.Add($line)
}
[System.IO.File]::WriteAllLines($f5, $new5)
Write-Host ('test_input_limits.rs done.')
