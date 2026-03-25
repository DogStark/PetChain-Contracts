$t = [System.IO.File]::ReadAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs')

# Find and remove the orphaned remnant lines
$remnantStart = $t.IndexOf('            total = total.checked_add(1).unwrap_or_else(|| panic_with_error!(&env, ContractError::CounterOverflow));')
[Console]::WriteLine("Remnant at: $remnantStart")

# The remnant is: "            total = total.checked_add(1)...\n        }\n        total\n    }\n"
# Find the closing "}" of the impl block that follows
$remnantEnd = $t.IndexOf('// --- ENC', $remnantStart)
[Console]::WriteLine("Remnant end at: $remnantEnd")
[Console]::WriteLine($t.Substring($remnantStart - 20, $remnantEnd - $remnantStart + 30))

# Remove the remnant (keep the "}\n\n\n\n" before "// --- ENC")
# The remnant block is: "            total = ...\n        }\n        total\n    }\n}\n\n\n\n"
# We want to keep just "}\n\n\n\n" (the impl closing brace)
$implClose = $t.IndexOf('}', $remnantStart)
[Console]::WriteLine("impl close at: $implClose")
[Console]::WriteLine($t.Substring($implClose - 5, 30))

# Remove from remnantStart to just before the impl closing brace
# Actually the structure is:
#   ...get_grooming_expenses body...
#   }          <- closes get_grooming_expenses
#   total      <- orphan
#   }          <- orphan  
#   total = total.checked_add... <- orphan
#   }          <- orphan
#   total      <- orphan
#   }          <- closes impl block
# }            <- extra orphan brace

# Let's just remove the specific orphan text
$orphan = "            total = total.checked_add(1).unwrap_or_else(|| panic_with_error!(&env, ContractError::CounterOverflow));`n        }`n        total`n    }`n}"
$t = $t.Replace($orphan, "}")

[System.IO.File]::WriteAllText('c:\Users\DELL\Documents\GitHub\PetChain-Contracts\stellar-contracts\src\lib.rs', $t)
Write-Host "Done. New length: $($t.Length)"
