# PetMedTracka-Contracts
This repo is specific for handling all smart contract-based contributions. 

## Teck Stack
* Language: Rust
* Network: Stellar

## Getting Started
Check out the [main petchain repo](https://github.com/DogStark/petChain-Frontend) to get a clear overview of the entire PetChain project—its purpose, how it works

### Build and Test Smart Contracts

#### Prerequisites
Install Stellar CLI:
```bash
cargo install --locked stellar-cli --features opt
```

#### Build the Stellar Contracts
```bash
cd stellar-contracts
cargo build --target wasm32-unknown-unknown --release
```
This compiles the Stellar smart contracts. Run this after making changes to confirm everything still compiles correctly.

#### Run Tests
```bash
cd stellar-contracts
cargo test
```
This runs the test suite for the contracts. Use it to make sure your changes don't break existing functionality.

#### Deploy to Testnet
```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/petchain_stellar.wasm \
  --network testnet
```


### Contributing
We're excited to have you contribute! Check out our [Contribution Guide](https://github.com/DogStark/petChain-Frontend/blob/main/contributing.md) to explore:

*  Code of Conduct
*  Step-by-step contribution process 
*  Open tasks and other ways to get involved

---

### Related Repositories

Explore other parts of the PetChain ecosystem:

* [Backend](https://github.com/DogStark/petchain_api)
* [Mobile App (For Pet Owners)](https://github.com/DogStark/PetMedTracka-MobileApp)
* [Frontend (Vet Dashboard)](https://github.com/DogStark/pet-medical-tracka) 


---

### Contact & Community
For feedback, questions or collaboration:

* Contact project lead: [@manlike_HB](https://t.me/manlike_HB), [@llins_x](https://t.me/llins_x) 
* Join Community Chat: [@PetChain Telegram Group](https://t.me/+fLbWYLN8jZw3ZTNk) 
Have questions or feedback?

[Star this repo](https://github.com/DogStark/PetMedTracka-Contracts) to stay updated on new features and releases.

### License
PetChain is licensed under the MIT License.
