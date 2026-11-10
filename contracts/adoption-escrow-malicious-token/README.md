# Malicious-Token Integration Fixture for Adoption Escrow (`#1507`)

Tests and hardens `HardenedAdoptionEscrowContract` against a configurable `MaliciousTokenContract` that simulates silent no-op transfers, fee-on-transfer/short deliveries, explicit reverts, and callback reentrancy during `transfer`.
