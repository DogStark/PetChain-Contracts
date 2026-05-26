# Changelog

All notable changes to the PetChain-Contracts project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Pull request template (`.github/PULL_REQUEST_TEMPLATE.md`) to standardize contributor submissions

## [0.1.0] - 2024-XX-XX

### Added

#### Core Pet Management

- Pet registration with unique IDs on the Stellar blockchain
- Pet profile storage including name, species, and ownership
- Pet status updates (active/inactive) with owner authentication
- Pet information retrieval by ID

#### Medical Records System

- Store and retrieve pet medical records on-chain
- Paginated medical record queries (`get_pet_medical_records`)
- Search medical records functionality
- Complete medical history tracking per pet

#### Vaccination Tracking

- Record and track pet vaccinations
- Vaccination history retrieval
- Upcoming vaccination reminders
- Overdue vaccination detection
- Vaccination status validation (`is_vaccination_current`)

#### Grooming Tracking System

- Add detailed grooming records with service type, groomer, cost, and notes
- Automatic calculation of next grooming due date (default: 60 days)
- Complete grooming history retrieval per pet
- Grooming expense tracking across all sessions
- Next grooming date estimation

#### Insurance System

- Add insurance policies to pets with provider, coverage type, premium, and expiry
- Retrieve pet insurance information
- Update insurance policy active status
- `InsuranceAddedEvent` and `InsuranceUpdatedEvent` events

#### Access Control & Privacy

- Owner-based access control for all state-changing operations
- Privacy levels for pet data: Public, Restricted, and Private
- Access grant system for authorized users
- `get_authorized_users` and access validation
- Consent pagination for managing data access permissions

#### Security & Encryption

- Encryption for sensitive pet and owner fields
- Deterministic key derivation using domain separator and contract address
- **Nonce uniqueness fix**: replaced fixed `[0u8; 12]` nonce with timestamp + counter based unique nonces to prevent replay attacks
- Decryption system with privacy level enforcement
- Input validation for all public functions
- Overflow protection on arithmetic operations

#### Emergency & Dispute Systems

- Emergency contacts management per pet
- Emergency override functionality for critical situations
- Dispute system for handling ownership or data conflicts

#### Transfer & Ownership

- Pet transfer functionality with ownership updates
- Multisig transfer support for enhanced security
- Pet adoption workflow support

#### Vet Registry

- Vet registration and verification system
- Vet information storage and retrieval

#### Behavior & Nutrition Tracking

- Pet behavior logging and history
- Nutrition tracking and records management

#### Data & Storage

- IPFS integration for off-chain data storage
- Attachment support for linking external files
- Efficient on-chain storage patterns with optimized data structures
- Statistics aggregation for pet data insights

#### Governance & Upgrades

- Upgrade proposal system for contract improvements
- Admin initialization and management

#### Backend Services

- Configurable TOTP-based Two-Factor Authentication (2FA)
- Cryptographic agility (SHA1/SHA256/SHA512, 6/8 digits, custom periods)
- QR code generation for authenticator apps
- 8 backup codes generation with recovery mechanism
- Rate limiting support
- Database schema for 2FA data persistence

#### Testing

- Comprehensive test suite with 43+ tests covering all 13 public contract functions
- 100% function coverage
- Decryption test suite with 21 tests for privacy enforcement and error handling
- Encryption nonce tests with 8 test cases for uniqueness and format validation
- Gas optimization benchmarks
- Insurance comprehensive tests
- Edge case and error condition testing

#### Documentation

- API reference documentation for all contract functions
- Grooming API reference
- Insurance API reference
- Development setup guide
- Contributing guidelines
- Security policy with vulnerability reporting process
- Gas optimization report with benchmark results
- Test documentation and coverage summaries

#### CI/CD & Tooling

- `.gitignore` configured for Rust/Stellar project
- Development and configuration guides

### Changed

#### Gas Optimization

- Storage pattern optimization: single storage instance reused across operations, batched storage writes
- Loop optimization for vaccination history functions (40-60% gas reduction)
- Access control function inlining (25-30% reduction)
- Vector pre-allocation for known sizes
- Timestamp and storage instance caching
- **Overall average gas savings: 30-40%** across all operations

### Fixed

- **Critical cryptographic vulnerability**: Fixed fixed-nonce encryption in `encrypt_sensitive_data` that enabled replay attacks, dictionary attacks, and information leakage. Now uses ledger timestamp + persistent counter for unique 12-byte nonces.
- Input limit validations for public functions
- Proper error handling for invalid pet IDs and missing records

### Security

- Implemented deterministic encryption key derivation (`petchain:encryption-key:v1` + contract address + admin context)
- Replaced static all-zero encryption key with runtime-derived key material
- Added authorization checks for all state-changing functions
- Rate limiting for 2FA endpoints
- Secure backup code generation and invalidation after use
- HTTPS enforcement recommendations for backend services

[unreleased]: https://github.com/DogStark/PetChain-Contracts/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/DogStark/PetChain-Contracts/releases/tag/v0.1.0
