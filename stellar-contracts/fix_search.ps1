$f = 'c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\test_search_medical_records.rs'
$lines = [System.IO.File]::ReadAllLines($f)
$newLines = [System.Collections.Generic.List[string]]::new()
for ($i = 0; $i -lt $lines.Length; $i++) {
    $ln = $i + 1
    $line = $lines[$i]
    if ($ln -eq 121) {
        $newLines.Add('        let start_val = u64::MAX - 100;')
        $newLines.Add('        let end_val = u64::MAX;')
        $newLines.Add('        let results = client.search_records_by_date_range(&pet_id, &start_val, &end_val);')
        continue
    }
    $newLines.Add($line)
}
[System.IO.File]::WriteAllLines($f, $newLines)
Write-Host ('Done. Lines: ' + $newLines.Count)
