use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Vec};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum LitterError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidInput = 4,
    SireNotFound = 5,
    DamNotFound = 6,
    IncompatibleParents = 7,
    DuplicateOffspring = 8,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LitterRecord {
    pub sire: Address,
    pub dam: Address,
    pub offspring: Vec<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataKey {
    pub litter: u64,
}

#[contract]
pub struct LitterContract;

#[contractimpl]
impl LitterContract {
    pub fn register_litter(
        env: Env,
        litter_id: u64,
        sire: Address,
        dam: Address,
        offspring: Vec<u64>,
    ) -> Result<(), LitterError> {
        sire.require_auth();

        if !Self::pet_exists(&env, &sire) {
            return Err(LitterError::SireNotFound);
        }
        if !Self::pet_exists(&env, &dam) {
            return Err(LitterError::DamNotFound);
        }
        if !Self::parents_compatible(&env, &sire, &dam) {
            return Err(LitterError::IncompatibleParents);
        }

        // Reject duplicate offspring IDs before any state is written so the
        // pedigree graph cannot be corrupted by a repeated linkage.
        if Self::has_duplicate_offspring(&offspring) {
            return Err(LitterError::DuplicateOffspring);
        }

        let record = LitterRecord {
            sire,
            dam,
            offspring,
        };
        env.storage()
            .persistent()
            .set(&DataKey { litter: litter_id }, &record);

        Ok(())
    }

    fn has_duplicate_offspring(offspring: &Vec<u64>) -> bool {
        let len = offspring.len();
        for i in 0..len {
            let current = offspring.get(i).unwrap();
            for j in (i + 1)..len {
                if current == offspring.get(j).unwrap() {
                    return true;
                }
            }
        }
        false
    }

    fn pet_exists(env: &Env, pet: &Address) -> bool {
        env.storage().persistent().has(pet)
    }

    fn parents_compatible(env: &Env, sire: &Address, dam: &Address) -> bool {
        sire != dam && env.storage().persistent().has(sire) && env.storage().persistent().has(dam)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn register_litter_rejects_duplicate_offspring() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, LitterContract);
        let client = LitterContractClient::new(&env, &contract_id);

        let sire = Address::generate(&env);
        let dam = Address::generate(&env);
        env.as_contract(&contract_id, || {
            env.storage().persistent().set(&sire, &true);
            env.storage().persistent().set(&dam, &true);
        });

        let mut offspring = Vec::new(&env);
        offspring.push_back(1u64);
        offspring.push_back(2u64);
        offspring.push_back(1u64);

        let result = client.try_register_litter(&1u64, &sire, &dam, &offspring);
        assert_eq!(result, Err(Ok(LitterError::DuplicateOffspring)));

        // State must remain unchanged after a rejected registration.
        env.as_contract(&contract_id, || {
            assert!(!env
                .storage()
                .persistent()
                .has(&DataKey { litter: 1u64 }));
        });
    }
}
