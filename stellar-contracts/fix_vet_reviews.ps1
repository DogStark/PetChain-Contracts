$f = 'c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\test.rs'
$lines = [System.IO.File]::ReadAllLines($f)
$newLines = [System.Collections.Generic.List[string]]::new()

for ($i = 0; $i -lt $lines.Length; $i++) {
    $ln = $i + 1
    $line = $lines[$i]

    # After line 1016 (env.mock_all_auths(); inside test_vet_reviews), close the function
    if ($ln -eq 1016) {
        $newLines.Add($line)
        $newLines.Add('    }')
        $newLines.Add('')
        continue
    }

    $newLines.Add($line)
}

[System.IO.File]::WriteAllLines($f, $newLines)
Write-Host ('Done. Lines: ' + $newLines.Count)
