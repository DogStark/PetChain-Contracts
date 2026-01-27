# Contributing to PetChain Smart Contracts

Thank you for your interest in contributing to PetChain! This guide will help you get started.

## Development Setup

### Prerequisites
- Rust (latest stable)
- Stellar CLI: `cargo install --locked stellar-cli --features opt`

### Setup
1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/PetChain-Contracts.git`
3. Navigate to project: `cd PetChain-Contracts`
4. Build contracts: `cd stellar-contracts && cargo build --target wasm32-unknown-unknown --release`
5. Run tests: `cargo test`

## Code Style

### Rust Guidelines
- Use `cargo fmt` before committing
- Follow Rust naming conventions (snake_case for functions, PascalCase for types)
- Add documentation comments for public functions
- Keep functions focused and small
- Use meaningful variable names

### Contract Patterns
- Always require authentication for state-changing functions
- Use proper error handling with `panic!` for invalid states
- Emit events for important state changes
- Validate inputs at function entry
- Use storage efficiently

## Pull Request Process

1. **Pick an Issue**: Comment on an issue to express interest
2. **Create Branch**: `git checkout -b feature/issue-number-description`
3. **Implement**: Follow the issue requirements and acceptance criteria
4. **Test**: Ensure all tests pass and add new tests for your code
5. **Document**: Update documentation if needed
6. **Submit PR**: Reference the issue number in your PR description

### PR Requirements
- All tests must pass
- Code coverage should not decrease
- Include tests for new functionality
- Update documentation if needed
- Follow the code style guidelines

## Testing

### Running Tests
```bash
cd stellar-contracts
cargo test
```

### Test Requirements
- Unit tests for all public functions
- Test both success and failure cases
- Use descriptive test names
- Aim for >90% code coverage

### Test Structure
```rust
#[test]
fn test_function_name_success_case() {
    // Setup
    let env = Env::default();
    // ... test implementation
    
    // Assert
    assert_eq!(result, expected);
}
```

## Issue Guidelines

### Good First Issues
Look for issues labeled `good-first-issue`:
- Issue #5: Emergency Contact System
- Issue #9: Multi-Pet Owner Support  
- Issue #10: Event Emission System

### Before Starting
- Read the issue description and requirements carefully
- Ask questions in the issue comments if anything is unclear
- Check if someone else is already working on it

## Code Review

### What We Look For
- Correct implementation of requirements
- Proper error handling
- Adequate test coverage
- Clear, readable code
- Security considerations
- Gas efficiency

### Review Process
1. Automated checks (CI/CD)
2. Code review by maintainers
3. Testing on testnet
4. Approval and merge

## Getting Help

- Join our [Telegram community](https://t.me/+Jw8HkvUhinw2YjE0)
- Ask questions in issue comments
- Check existing documentation and issues

## Code of Conduct

- Be respectful and inclusive
- Help others learn and grow
- Focus on constructive feedback
- Follow GitHub's community guidelines

Thank you for contributing to PetChain!