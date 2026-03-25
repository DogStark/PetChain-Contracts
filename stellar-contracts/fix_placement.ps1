$t = [System.IO.File]::ReadAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs')

# ── 1. Remove the misplaced safe_increment (between #[contractimpl] and impl) ─
$misplaced = "#[contractimpl]`r`n// --- OVERFLOW-SAFE COUNTER HELPER ---`r`npub(crate) fn safe_increment(count: u64) -> u64 {`r`n    count.checked_add(1).unwrap_or(u64::MAX)`r`n}`r`n`r`nimpl PetChainContract {"
$t = $t.Replace($misplaced, "#[contractimpl]`r`nimpl PetChainContract {")

# Also try LF version
$misplacedLF = "#[contractimpl]`n// --- OVERFLOW-SAFE COUNTER HELPER ---`npub(crate) fn safe_increment(count: u64) -> u64 {`n    count.checked_add(1).unwrap_or(u64::MAX)`n}`n`nimpl PetChainContract {"
$t = $t.Replace($misplacedLF, "#[contractimpl]`nimpl PetChainContract {")

# ── 2. Insert safe_increment BEFORE #[contractimpl] ───────────────────────────
$contractImplAttr = "#[contractimpl]`r`nimpl PetChainContract {"
$safeIncWithImpl = "// --- OVERFLOW-SAFE COUNTER HELPER ---`r`npub(crate) fn safe_increment(count: u64) -> u64 {`r`n    count.checked_add(1).unwrap_or(u64::MAX)`r`n}`r`n`r`n#[contractimpl]`r`nimpl PetChainContract {"
$t = $t.Replace($contractImplAttr, $safeIncWithImpl)

# Also try LF version
$contractImplAttrLF = "#[contractimpl]`nimpl PetChainContract {"
$safeIncWithImplLF = "// --- OVERFLOW-SAFE COUNTER HELPER ---`npub(crate) fn safe_increment(count: u64) -> u64 {`n    count.checked_add(1).unwrap_or(u64::MAX)`n}`n`n#[contractimpl]`nimpl PetChainContract {"
$t = $t.Replace($contractImplAttrLF, $safeIncWithImplLF)

[System.IO.File]::WriteAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs', $t)
Write-Host "Done. New length: $($t.Length)"

# Verify placement
$t2 = [System.IO.File]::ReadAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs')
$idx = $t2.IndexOf('pub(crate) fn safe_increment')
Write-Host "safe_increment now at: $idx"
Write-Host $t2.Substring($idx - 50, 200)
