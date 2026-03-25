$t = [System.IO.File]::ReadAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs')

$t = $t.Replace(
    'Self::get_pet(env.clone(), pid, owner.clone())',
    'Self::get_pet(env.clone(), pid)'
)
$t = $t.Replace(
    'Self::get_pet(env.clone(), pid, raw.owner.clone())',
    'Self::get_pet(env.clone(), pid)'
)
$t = $t.Replace(
    'Self::get_pet(env.clone(), id, pet.owner.clone())',
    'Self::get_pet(env.clone(), id)'
)

[System.IO.File]::WriteAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs', $t)
Write-Host "Done. New length: $($t.Length)"
