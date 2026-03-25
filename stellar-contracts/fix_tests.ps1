$f = 'c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\test.rs'
$lines = [System.IO.File]::ReadAllLines($f)
Write-Host ('Total lines: ' + $lines.Length)

$newLines = [System.Collections.Generic.List[string]]::new()

for ($i = 0; $i -lt $lines.Length; $i++) {
    $lineNum = $i + 1
    $line = $lines[$i]

    # Fix: record.vet_address -> record.veterinarian
    if ($lineNum -in @(556, 1591, 2434, 3131)) {
        $line = $line -replace 'record\.vet_address', 'record.veterinarian'
    }

    # Fix: get_emergency_info missing caller arg
    if ($lineNum -in @(812, 1847, 2690, 3387)) {
        $line = $line -replace 'client\.get_emergency_info\(&pet_id\)', 'client.get_emergency_info(&pet_id, &owner)'
    }

    # Fix: book_slot missing booker arg (line 1301) - use vet as booker placeholder
    if ($lineNum -eq 1301) {
        $line = $line -replace 'client\.book_slot\(&vet, &slot_index\)', 'client.book_slot(&vet, &vet, &slot_index)'
    }

    # Fix: Medication missing id and pet_id fields
    if ($lineNum -in @(1915, 1947, 1956, 2763, 2795, 2804)) {
        $line = $line -replace 'Medication \{', 'Medication { id: 0, pet_id: 0,'
    }

    # Fix: end_date: 200 -> end_date: Some(200)
    if ($lineNum -in @(1920, 1952, 2768, 2800)) {
        $line = $line -replace 'end_date: 200,', 'end_date: Some(200),'
    }

    # Fix: end_date: update_time + 100 -> end_date: Some(update_time + 100)
    if ($lineNum -in @(1961, 2809)) {
        $line = $line -replace 'end_date: update_time \+ 100,', 'end_date: Some(update_time + 100),'
    }

    # Fix: add_lab_result missing reference_ranges and medical_record_id args
    # Line 1880: &None, -> &String::from_str(&env, ""), &None, &None,
    if ($lineNum -eq 1880) {
        $line = '            &String::from_str(&env, ""),  // reference_ranges'
        $newLines.Add($line)
        $newLines.Add('            &None,  // attachment_hash')
        $newLines.Add('            &None,  // medical_record_id')
        continue
    }
    if ($lineNum -eq 2723) {
        $line = '            &String::from_str(&env, ""),  // reference_ranges'
        $newLines.Add($line)
        $newLines.Add('            &None,  // attachment_hash')
        $newLines.Add('            &None,  // medical_record_id')
        continue
    }

    # Fix: register_pet missing color/weight/microchip_id
    # Line 476: &PrivacyLevel::Public, -> add missing args before it
    # Pattern: line has &PrivacyLevel::Public and previous line has &breed
    if ($lineNum -eq 476) {
        $newLines.Add('            &String::from_str(&env, "Unknown"),  // color')
        $newLines.Add('            &0u32,  // weight')
        $newLines.Add('            &None,  // microchip_id')
    }
    if ($lineNum -eq 2354) {
        $newLines.Add('            &String::from_str(&env, "Unknown"),  // color')
        $newLines.Add('            &0u32,  // weight')
        $newLines.Add('            &None,  // microchip_id')
    }

    # Fix: Duplicate mod test -> rename
    if ($lineNum -eq 1480) { $line = 'mod test_b {' }
    if ($lineNum -eq 2326) { $line = 'mod test_c {' }
    if ($lineNum -eq 3020) { $line = 'mod test_d {' }

    # Fix: Duplicate mod test_vet -> rename
    if ($lineNum -eq 3896) { $line = 'mod test_vet_b {' }

    # Fix: orphaned test_revoke_consent (lines 1358-1368) - replace body with proper setup
    if ($lineNum -eq 1359) {
        $newLines.Add('fn test_revoke_consent() {')
        $newLines.Add('    let env = Env::default();')
        $newLines.Add('    env.mock_all_auths();')
        $newLines.Add('    let contract_id = env.register_contract(None, PetChainContract);')
        $newLines.Add('    let client = PetChainContractClient::new(&env, &contract_id);')
        $newLines.Add('    let admin = Address::generate(&env);')
        $newLines.Add('    client.init_admin(&admin);')
        $newLines.Add('    let owner = Address::generate(&env);')
        $newLines.Add('    let pet_id = client.register_pet(')
        $newLines.Add('        &owner, &String::from_str(&env, "Buddy"), &String::from_str(&env, "2020"),')
        $newLines.Add('        &Gender::Male, &Species::Dog, &String::from_str(&env, "Lab"),')
        $newLines.Add('        &String::from_str(&env, "Brown"), &5u32, &None, &PrivacyLevel::Public,')
        $newLines.Add('    );')
        $newLines.Add('    let grantee = Address::generate(&env);')
        $newLines.Add('    let consent_id = client.grant_consent(&pet_id, &owner, &ConsentType::Insurance, &grantee);')
        $newLines.Add('    let revoked = client.revoke_consent(&consent_id, &owner);')
        $newLines.Add('    assert!(revoked);')
        continue
    }
    # Skip the old wrong body lines 1360-1368
    if ($lineNum -in @(1360, 1361, 1362, 1363, 1364, 1365, 1366, 1367, 1368)) {
        continue
    }

    # Fix: orphaned test_consent_history (lines 1407-1416) - replace body with proper setup
    if ($lineNum -eq 1407) {
        $newLines.Add('fn test_consent_history() {')
        $newLines.Add('    let env = Env::default();')
        $newLines.Add('    env.mock_all_auths();')
        $newLines.Add('    let contract_id = env.register_contract(None, PetChainContract);')
        $newLines.Add('    let client = PetChainContractClient::new(&env, &contract_id);')
        $newLines.Add('    let admin = Address::generate(&env);')
        $newLines.Add('    client.init_admin(&admin);')
        $newLines.Add('    let owner = Address::generate(&env);')
        $newLines.Add('    let pet_id = client.register_pet(')
        $newLines.Add('        &owner, &String::from_str(&env, "Buddy"), &String::from_str(&env, "2020"),')
        $newLines.Add('        &Gender::Male, &Species::Dog, &String::from_str(&env, "Lab"),')
        $newLines.Add('        &String::from_str(&env, "Brown"), &5u32, &None, &PrivacyLevel::Public,')
        $newLines.Add('    );')
        $newLines.Add('    let grantee = Address::generate(&env);')
        $newLines.Add('    client.grant_consent(&pet_id, &owner, &ConsentType::Insurance, &grantee);')
        $newLines.Add('    let history = client.get_consent_history(&pet_id);')
        $newLines.Add('    assert_eq!(history.len(), 1);')
        continue
    }
    # Skip the old wrong body lines 1408-1416
    if ($lineNum -in @(1408, 1409, 1410, 1411, 1412, 1413, 1414, 1415, 1416)) {
        continue
    }

    # Fix: orphaned code at lines 1458-1464 (outside any module after closing })
    # These lines are between } at 1457 and #[cfg(test)] at 1479
    # They reference client/env that don't exist - just remove them
    if ($lineNum -in @(1458, 1459, 1460, 1461, 1462, 1463, 1464)) {
        continue
    }

    $newLines.Add($line)
}

[System.IO.File]::WriteAllLines($f, $newLines)
Write-Host ('Done. New line count: ' + $newLines.Count)
