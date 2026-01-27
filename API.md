# PetChain Smart Contract API

## Overview
The PetChain smart contract manages pet registration, medical records, and ownership on the Stellar network.

## Contract Functions

### Pet Management

#### `register_pet`
Registers a new pet on the blockchain.

**Parameters:**
- `env: Env` - Contract environment
- `owner: Address` - Pet owner's address (requires authentication)
- `name: String` - Pet's name
- `species: String` - Pet's species (e.g., "Dog", "Cat")

**Returns:** `u64` - Unique pet ID

**Example:**
```rust
let pet_id = client.register_pet(&owner_address, &name, &species);
```

#### `get_pet`
Retrieves pet information by ID.

**Parameters:**
- `env: Env` - Contract environment
- `pet_id: u64` - Pet's unique identifier

**Returns:** `Option<Pet>` - Pet data or None if not found

**Example:**
```rust
let pet = client.get_pet(&pet_id);
```

#### `update_pet_status`
Updates a pet's active status.

**Parameters:**
- `env: Env` - Contract environment
- `pet_id: u64` - Pet's unique identifier
- `active: bool` - New active status

**Authentication:** Requires pet owner's signature

**Example:**
```rust
client.update_pet_status(&pet_id, &true);
```

## Data Structures

### Pet
```rust
pub struct Pet {
    pub id: u64,           // Unique identifier
    pub owner: Address,    // Owner's address
    pub name: String,      // Pet's name
    pub species: String,   // Pet's species
    pub active: bool,      // Active status
}
```

### DataKey
```rust
pub enum DataKey {
    Pet(u64),              // Pet data by ID
    PetCount,              // Total pet count
    OwnerPets(Address),    // Pets by owner
}
```

## Error Handling

The contract uses Stellar's built-in error handling:
- Authentication failures throw auth errors
- Invalid pet IDs return `None`
- Storage failures are handled by the Stellar runtime

## Events

Currently, the contract doesn't emit custom events. This will be added in future versions (see Issue #10).

## Gas Costs

Typical operation costs:
- `register_pet`: ~50,000 stroops
- `get_pet`: ~10,000 stroops
- `update_pet_status`: ~30,000 stroops

*Note: Costs may vary based on network conditions*

## Security Considerations

- All state-changing functions require proper authentication
- Pet ownership is immutable after registration
- Only pet owners can update their pet's status
- All data is stored on-chain and publicly readable

## Integration Examples

### JavaScript (StellarJS)
```javascript
import { Contract } from 'stellar-sdk';

const contract = new Contract(contractAddress);
const result = await contract.call('register_pet', owner, name, species);
```

### Rust
```rust
let client = PetChainContractClient::new(&env, &contract_id);
let pet_id = client.register_pet(&owner, &name, &species);
```

## Future Enhancements

See [ISSUES.md](ISSUES.md) for planned features:
- Medical record storage
- Vaccination tracking
- Access control system
- Event emissions