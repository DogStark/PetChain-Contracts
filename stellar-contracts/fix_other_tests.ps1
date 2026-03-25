
# Fix test_admin_initialization.rs
$f1 = 'c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\test_admin_initialization.rs'
$lines1 = [System.IO.File]::ReadAllLines($f1)
$new1 = [System.Collections.Generic.List[string]]::new()
for ($i = 0; $i -lt $lines1.Length; $i++) {
    $ln = $i + 1
    $line = $lines1[$i]
    if ($ln -eq 123) {
        $new1.Add('    let mut new_admins_a = soroban_sdk::Vec::new(&env);')
        $new1.Add('    new_admins_a.push_back(proposer.clone());')
        $new1.Add('    let action = ProposalAction::ChangeAdmin((new_admins_a, 1u32));')
        continue
    }
    if ($ln -eq 175) {
        $new1.Add('    let mut new_admins_b = soroban_sdk::Vec::new(&env);')
        $new1.Add('    new_admins_b.push_back(admin.clone());')
        $new1.Add('    new_admins_b.push_back(admin2.clone());')
        $new1.Add('    let action = ProposalAction::ChangeAdmin((new_admins_b, 2u32));')
        continue
    }
    $new1.Add($line)
}
[System.IO.File]::WriteAllLines($f1, $new1)
Write-Host ('test_admin_initialization.rs done. Lines: ' + $new1.Count)

# Fix test_nutrition.rs - remove extra &owner arg from get_pet call
$f2 = 'c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\test_nutrition.rs'
$lines2 = [System.IO.File]::ReadAllLines($f2)
$new2 = [System.Collections.Generic.List[string]]::new()
for ($i = 0; $i -lt $lines2.Length; $i++) {
    $ln = $i + 1
    $line = $lines2[$i]
    if ($ln -eq 80) {
        $new2.Add('    let profile = client.get_pet(&pet_id).unwrap();')
        continue
    }
    $new2.Add($line)
}
[System.IO.File]::WriteAllLines($f2, $new2)
Write-Host ('test_nutrition.rs done. Lines: ' + $new2.Count)
