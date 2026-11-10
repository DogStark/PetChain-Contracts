# Contract State Export and Restore Rehearsal (`#1505`)

Implements a rehearsal-gated export/import harness (`export_state_bundle` and `restore_state_bundle`) for PetChain state records, proving that restored state preserves owner/vet/admin authorization rules, nonces, and schema version metadata.
