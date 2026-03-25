$f = 'c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\test.rs'
$lines = [System.IO.File]::ReadAllLines($f)
$newLines = [System.Collections.Generic.List[string]]::new()

for ($i = 0; $i -lt $lines.Length; $i++) {
    $ln = $i + 1
    $line = $lines[$i]

    # After line 1378 (assert!(revoked);), add closing brace for test_revoke_consent
    if ($ln -eq 1379 -and $line.Trim() -eq '') {
        $newLines.Add('}')
        $newLines.Add('')
        continue
    }

    # After line 1433 (assert_eq!(history.len(), 1);), add closing brace for test_consent_history
    if ($ln -eq 1434 -and $line.Trim() -eq '') {
        $newLines.Add('}')
        $newLines.Add('')
        continue
    }

    $newLines.Add($line)
}

[System.IO.File]::WriteAllLines($f, $newLines)
Write-Host ('Done. Lines: ' + $newLines.Count)
