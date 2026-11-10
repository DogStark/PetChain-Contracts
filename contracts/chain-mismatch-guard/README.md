# Deterministic Chain-Mismatch Guard (`#1508`)

Validates deployment network names, network passphrases, Celo/EVM chain IDs, and contract address metadata on cross-network calls and deployment manifests, failing with deterministic error codes (`NetworkNameMismatch`, `NetworkPassphraseMismatch`, `ChainIdMismatch`, `ContractAddressMismatch`) without mutating contract storage.
