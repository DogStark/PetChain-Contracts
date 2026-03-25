$f = 'c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\test.rs'
$lines = [System.IO.File]::ReadAllLines($f)
$newLines = [System.Collections.Generic.List[string]]::new()
for ($i = 0; $i -lt $lines.Length; $i++) {
    $ln = $i + 1
    if ($ln -eq 3890) {
        $newLines.Add('    #[allow(dead_code)]')
    }
    $newLines.Add($lines[$i])
}
[System.IO.File]::WriteAllLines($f, $newLines)
Write-Host ('Done. Lines: ' + $newLines.Count)
