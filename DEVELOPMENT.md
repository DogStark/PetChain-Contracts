# Development Guide

## Architecture Overview

```
PetChain-Contracts/
├── stellar-contracts/     # Stellar smart contracts
│   ├── src/lib.rs        # Main contract code
│   └── Cargo.toml        # Dependencies
├── .github/              # CI/CD and templates
├── ISSUES.md            # Development roadmap
└── API.md               # Contract documentation
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
   - **CONTRACT_ADDRESS**: Obtain from contract deployment
   - **JWT_SECRET**: Generate a secure random string
   - **TOTP_ISSUER**: Set to `PetChain`
   - **DATABASE_URL**: PostgreSQL connection string

### Stellar Testnet Faucet
For development, you'll need test XLM:
1. Visit the [Stellar Testnet Faucet](https://laboratory.stellar.org/#account-creator?network=testnet)
2. Generate a test account
3. Fund it with test XLM

## Smart Contract Structure

### Core Components
- **Pet Registration**: Basic pet data storage
- **Authentication**: Owner-based access control
- **Storage**: Efficient on-chain data management

### Future Components (See ISSUES.md)
- Medical records system
- Vaccination tracking
- Vet verification
- Access control

## Development Workflow

### 1. Issue Selection
- Browse [ISSUES.md](ISSUES.md)
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