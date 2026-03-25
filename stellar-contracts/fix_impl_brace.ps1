$t = [System.IO.File]::ReadAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs')

# Find the gap between calculate_age closing and the encryption section
$encIdx = $t.IndexOf('// --- REAL ENCRYPTION')
[Console]::WriteLine('encryption section at: ' + $encIdx)

# Check what's right before the encryption section
[Console]::WriteLine('Before encryption: ' + $t.Substring($encIdx - 30, 30))

# Insert closing brace for impl block right before the encryption section
$before = $t.Substring(0, $encIdx)
$after  = $t.Substring($encIdx)

# The impl block needs a closing brace
$result = $before + "}`n`n" + $after

[System.IO.File]::WriteAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs', $result)
Write-Host "Done. New length: $($result.Length)"
