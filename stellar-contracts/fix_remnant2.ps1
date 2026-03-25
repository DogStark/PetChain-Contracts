$t = [System.IO.File]::ReadAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs')

# Remove the orphaned remnant with exact CRLF endings
$orphan = "            total = total.checked_add(1).unwrap_or_else(|| panic_with_error!(&env, ContractError::CounterOverflow));`r`n        }`r`n        total`r`n    }`r`n}"
$replacement = "}"
$before = $t.Length
$t = $t.Replace($orphan, $replacement)
$after = $t.Length
[Console]::WriteLine("Removed: " + ($before - $after) + " chars")

[System.IO.File]::WriteAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs', $t)
Write-Host "Done. New length: $($t.Length)"
