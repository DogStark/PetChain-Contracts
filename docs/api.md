# API Overview

## Smart Contracts

### Main contract

The primary contract lives in `stellar-contracts/src/lib.rs` and exposes functionality around:

- pet registration and profile management
- ownership and access grants
- vet registration and verification
- medical records, lab results, and attachments
- insurance, grooming, nutrition, activity, and behavior records
- emergency contacts and emergency access logs
- multisig administration and upgrade proposals

### Transfer and adoption contract

The transfer-focused contract lives in `stellar-contracts/contracts/pet-transfer-adoption/src/lib.rs` and handles:

- pet creation
- transfer initiation and acceptance
- transfer cancellation and reclaim flows
- ownership history tracking

## Backend 2FA

The backend crate provides:

- 2FA enrollment
- token verification and activation
- login-time token checks
- disable and recovery flows
- request tracing middleware
- in-memory and Redis-backed rate limiting

For implementation details, read the crate sources in `backend-2fa/src/`.
