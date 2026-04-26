# Development Guide

## Architecture Overview

```
PetChain-Contracts/
├── stellar-contracts/                         # Soroban/Stellar contract workspace
│   ├── Cargo.toml                            # Main PetChain contract package
│   ├── src/
│   │   ├── lib.rs                            # Main PetChain contract implementation
│   │   └── test_*.rs                         # Unit and workflow test modules
│   └── contracts/
│       └── pet_transfer_adoption/            # Pet transfer/adoption contract package
│           ├── Cargo.toml
│           └── src/
│               ├── lib.rs                    # Ownership transfer and history contract
│               ├── test.rs                   # Transfer/adoption tests
│               └── vet_registry.rs           # Vet registry contract helpers
├── backend-2fa-implementation/               # Rust 2FA backend support service
│   ├── Cargo.toml
│   ├── schema.sql
│   └── src/
│       ├── db.rs
│       ├── handlers.rs
│       ├── rate_limiter.rs
│       ├── tests.rs
│       └── two_factor.rs
├── .github/                                  # CI/CD workflows and templates
├── API.md                                    # Contract documentation
└── README.md                                 # Project overview
```

## Environment Setup

### Prerequisites
- Rust toolchain (see [rustup.rs](https://rustup.rs/))
- Stellar CLI (`cargo install stellar-cli`)
- PostgreSQL database

### Configuration
1. Copy the environment template:
   ```bash
   cp .env.example .env
   ```
2. Fill in the required values in `.env`:
   - **STELLAR_NETWORK**: Set to `testnet` for development
   - **STELLAR_RPC_URL**: Use `https://soroban-testnet.stellar.org`
   - **CONTRACT_ADDRESS**: Obtain from deployed contract
   - **JWT_SECRET**: Generate a secure random string
   - **TOTP_ISSUER**: Set to `PetChain`
   - **DATABASE_URL**: PostgreSQL connection string

### Stellar Testnet Faucet
For development, get test XLM from the Stellar testnet faucet:
1. Visit the [Stellar Testnet Faucet](https://laboratory.stellar.org/#account-creator?network=testnet)
2. Generate a test account
3. Fund it with test XLM

## Smart Contract Structure

### Core Components
- **Main PetChain contract**: Pet registration, owner access control, encrypted pet data, medical records, vaccinations, grooming, insurance, activity, behavior, nutrition, emergency contacts, and multisig transfer flows in `stellar-contracts/src/lib.rs`.
- **Transfer/adoption contract**: Dedicated ownership transfer, pending transfer, ownership history, and vet registry support under `stellar-contracts/contracts/pet_transfer_adoption`.
- **2FA backend**: Rust backend support for TOTP, handlers, database access, and rate limiting under `backend-2fa-implementation`.
- **Testing**: Multiple focused Rust test modules in `stellar-contracts/src/test_*.rs`, plus `pet_transfer_adoption/src/test.rs` and backend tests.

### Future Components
- Deployment and release automation for contract artifacts.
- Production API integration around the existing contracts and 2FA backend.
- Expanded audit, performance, and monitoring tooling.

## Development Workflow

### 1. Issue Selection
- Browse the repository's GitHub issues
- Start with `good-first-issue` labels
- Comment to claim an issue

### 2. Implementation
```bash
# Setup
git checkout -b feature/issue-X-description
cd stellar-contracts

# Development cycle
cargo build --target wasm32-unknown-unknown --release
cargo test
cargo fmt
```

### 3. Testing Requirements
- Unit tests for all functions
- Integration tests for workflows
- Error case testing
- >90% code coverage

### 4. Code Review
- Automated CI checks
- Security review
- Performance assessment
- Documentation review

## Contract Patterns

### Function Structure
```rust
pub fn function_name(env: Env, param: Type) -> ReturnType {
    // 1. Authentication
    caller.require_auth();
    
    // 2. Input validation
    assert!(param.is_valid(), "Invalid input");
    
    // 3. Business logic
    let result = process_logic(param);
    
    // 4. Storage update
    env.storage().instance().set(&key, &value);
    
    // 5. Return result
    result
}
```

### Error Handling
```rust
// Use assertions for invalid states
assert!(condition, "Error message");

// Return Option for not-found cases
pub fn get_item(id: u64) -> Option<Item> {
    env.storage().instance().get(&DataKey::Item(id))
}
```

### Storage Patterns
```rust
#[contracttype]
pub enum DataKey {
    Item(u64),           // Individual items
    ItemCount,           // Counters
    UserItems(Address),  // User mappings
}
```

## Testing Patterns

### Basic Test Structure
```rust
#[test]
fn test_function_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Contract);
    let client = ContractClient::new(&env, &contract_id);
    
    let result = client.function(&param);
    assert_eq!(result, expected);
}
```

### Authentication Testing
```rust
#[test]
fn test_requires_auth() {
    // Test that functions require proper authentication
    let owner = Address::generate(&env);
    let result = client.function(&owner, &params);
    // Should succeed with proper auth
}
```

## Performance Considerations

### Gas Optimization
- Minimize storage operations
- Use efficient data structures
- Batch operations when possible
- Profile gas usage regularly

### Storage Efficiency
- Pack data structures
- Use appropriate key types
- Minimize redundant data
- Consider data access patterns

## Security Guidelines

### Input Validation
```rust
// Always validate inputs
assert!(!name.is_empty(), "Name cannot be empty");
assert!(amount > 0, "Amount must be positive");
```

### Access Control
```rust
// Require authentication for state changes
owner.require_auth();

// Check permissions
assert!(is_authorized(&caller), "Not authorized");
```

### Safe Arithmetic
```rust
// Use checked arithmetic
let result = a.checked_add(b).expect("Overflow");
```

## Deployment

### Testnet Deployment
```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/petchain_stellar.wasm \
  --network testnet
```

### Mainnet Considerations
- Thorough testing on testnet
- Security audit
- Gas optimization
- Upgrade strategy
