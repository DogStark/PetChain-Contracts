$f = 'c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\test.rs'
$lines = [System.IO.File]::ReadAllLines($f)
$newLines = [System.Collections.Generic.List[string]]::new()

# Keep lines 1..1312 unchanged (0-indexed: 0..1311)
for ($i = 0; $i -lt 1312; $i++) {
    $newLines.Add($lines[$i])
}

# Close test_book_slot fn and mod test
$newLines.Add('    }')
$newLines.Add('}')
$newLines.Add('')

# New properly structured module for the orphaned consent/upgrade tests
$newLines.Add('#[cfg(test)]')
$newLines.Add('mod test_consent_upgrade {')
$newLines.Add('    use crate::*;')
$newLines.Add('    use soroban_sdk::{testutils::Address as _, Address, Env, String};')
$newLines.Add('')

# test_grant_consent (was lines 1315-1361, 0-indexed 1314-1360)
# but lines 1316-1326 (0-indexed 1315-1325) were the nested test_get_version — skip those
# test_grant_consent body: lines 1315 fn open, then 1327-1361 body (after skipping nested fn)
# Let's just write it cleanly
$newLines.Add('    #[test]')
$newLines.Add('    fn test_grant_consent() {')
# lines 1327-1361 (0-indexed 1326-1360) are the real body of test_grant_consent
for ($i = 1326; $i -le 1360; $i++) {
    $newLines.Add('    ' + $lines[$i])
}
$newLines.Add('')

# test_propose_upgrade (was lines 1328-1361 but now we need the original)
# Actually test_propose_upgrade starts at original line 1329 (0-indexed 1328)
# Let's re-read: after our fix script ran, the lines shifted. Let me use the
# content we know from the original read:
# test_propose_upgrade: lines 1329-1361 in original (0-indexed 1328-1360)
# But we already used 1326-1360 for test_grant_consent body above.
# The original structure was:
#   1314: #[test]
#   1315: fn test_grant_consent {
#   1316:   #[test]           <- nested, skip
#   1317:   fn test_get_version { <- nested, skip
#   ...1326: }               <- end of nested fn, skip
#   1327: (blank)
#   1328: #[test]
#   1329: fn test_propose_upgrade {
#   ...1361: }
#   1362: (blank)
#   1363: #[test]
#   1364: fn test_revoke_consent {
#   ...1381: }
#   1382: (blank)
#   1383: #[test]
#   1384: fn test_approve_upgrade {
#   ...1417: }
#   1418: (blank)
#   1419: #[test]
#   1420: fn test_consent_history {
#   ...1437: }
#   1438: (blank)
#   1439: #[test]
#   1440: fn test_migrate_version {
#   ...1477: }
#   1478: }  <- old mod test close
#   1479: (blank)
#   1480: #[test]
#   1481: #[should_panic]
#   1482: fn test_upgrade_requires_admin {
#   ...1491: }
#   1492: }  <- extra brace

# Wait - the lines above are from the ORIGINAL file before our previous fix ran.
# After fix_tests.ps1 ran, lines shifted. Let me just read current file state.
Write-Host 'Script needs current file state - aborting and using direct approach'
