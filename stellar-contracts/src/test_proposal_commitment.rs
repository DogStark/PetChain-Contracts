use crate::{
    ContractError, PetChainContract, PetChainContractClient, ProposalAction, SystemKey,
    MultiSigProposal,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Error, Vec};

fn setup(env: &Env) -> (PetChainContractClient, Address, Address) {
    env.mock_all_auths();
    let id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &id);
    let admin = Address::generate(env);
    let mut admins = Vec::new(env);
    admins.push_back(admin.clone());
    client.init_multisig(&admin, &admins, &1);
    (client, admin, id)
}

fn commitment(env: &Env, id: &Address, pid: u64) -> BytesN<32> {
    env.as_contract(id, || {
        env.storage()
            .instance()
            .get(&SystemKey::ProposalCommitment(pid))
            .unwrap()
    })
}

#[test]
fn commitment_is_stored_and_differs_per_action() {
    let env = Env::default();
    let (client, admin, id) = setup(&env);
    let (a, b, c) = (
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    );
    let p1 = client.propose_signer_rotation(&admin, &a, &b);
    let p2 = client.propose_signer_rotation(&admin, &a, &c);
    assert_ne!(commitment(&env, &id, p1), commitment(&env, &id, p2));
}

#[test]
fn execution_rejects_mutated_action() {
    let env = Env::default();
    let (client, admin, id) = setup(&env);
    let (a, b, c) = (
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    );
    let pid = client.propose_signer_rotation(&admin, &a, &b);
    env.as_contract(&id, || {
        let mut p: MultiSigProposal = env
            .storage()
            .instance()
            .get(&SystemKey::Proposal(pid))
            .unwrap();
        p.action = ProposalAction::RotateSigner((a.clone(), c.clone()));
        env.storage().instance().set(&SystemKey::Proposal(pid), &p);
    });
    let res = client.try_execute_proposal(&pid);
    assert_eq!(
        res.unwrap_err().unwrap_err(),
        Error::from_contract_error(ContractError::InvalidState as u32)
    );
}
