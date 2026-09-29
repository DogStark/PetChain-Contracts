//! Cross-contract interface compatibility (Issue #1253).
//!
//! Deploys the workspace contracts together into one local Soroban
//! environment and exercises the interfaces they rely on across contract
//! boundaries: authorization, errors, events and state transitions.
//!
//! Contracts under test:
//! - `PetOwnershipContract` — transfers, adoption and escrowed transfers.
//! - `VetRegistryContract` — standalone vet registry.
//! - `PetChainContract` (main, `stellar-contracts/src/lib.rs`) — loaded from its
//!   release WASM via `PETCHAIN_MAIN_WASM`; the CI `cross-contract` job builds
//!   it and runs these tests with `--include-ignored`.
//!
//! The `fixture` module is the compatibility fixture: it fails to compile on
//! signature or enum drift in this crate, and the main contract's return types
//! are decoded into pinned local mirrors, so drift there fails at runtime.
//!
//! Interface versioning (see stellar-contracts/docs/cross-contract-compat.md):
//! - Intentionally versioned: the main contract ABI (`abi-snapshot.txt`,
//!   `docs/abi-migrations.md`, `get_version`) and main-contract event payloads
//!   (`EVENT_SCHEMA_VERSION`, `docs/event-versioning.md`).
//! - Not versioned, append-only: every interface pinned in `fixture` below.
//!   Changing one is a breaking change for the other contracts and clients.

extern crate std;

use crate::vet_registry::{self, VetRegistryContract, VetRegistryContractClient};
use crate::{
    ContractError, PetOwnershipContract, PetOwnershipContractClient, DISPUTE_WINDOW_SECONDS,
};
use soroban_sdk::{
    contracttype,
    testutils::{Address as _, Events, Ledger},
    vec, Address, Env, Error, IntoVal, String, Symbol, TryFromVal, Val, Vec,
};

/// Compile-time compatibility fixture for this crate's shared interfaces.
#[allow(dead_code)]
mod fixture {
    use super::*;
    use crate::TransferType;

    /// Signature drift in any pinned function fails to compile here.
    fn signatures(ownership: &PetOwnershipContractClient, vets: &VetRegistryContractClient) {
        let env = ownership.env.clone();
        let (a, s) = (Address::generate(&env), String::from_str(&env, ""));
        let _: () = ownership.init_trusted_contract(&a, &Vec::new(&env), &1u32);
        let _: bool = ownership.validate_trusted_contract(&a);
        let _: Address = ownership.get_trusted_contract_address();
        let _: () = ownership.create_pet(&1u64, &a);
        let _: () = ownership.initiate_transfer(&1u64, &a);
        let _: () = ownership.accept_transfer(&1u64);
        let _: () = ownership.raise_dispute(&1u64, &a);
        let _: () = ownership.finalize_transfer(&1u64);
        let _: Address = ownership.get_current_owner(&1u64);
        let _: () = vets.init(&a);
        let _: () = vets.register_vet(&a, &s, &s, &s);
        let _: () = vets.verify_vet(&a);
        let _: bool = vets.is_verified_vet(&a);
    }

    /// Adding, removing or renaming a variant fails to compile here.
    pub fn transfer_type(t: &TransferType) -> &'static str {
        match t {
            TransferType::Direct => "Direct",
            TransferType::Adoption => "Adoption",
            TransferType::Multisig => "Multisig",
        }
    }

    /// Error codes other contracts and clients match on.
    pub const OWNERSHIP_ERRORS: [(ContractError, u32); 5] = [
        (ContractError::PetNotFound, 1),
        (ContractError::Unauthorized, 2),
        (ContractError::DisputeWindowNotElapsed, 13),
        (ContractError::TransferAlreadyDisputed, 14),
        (ContractError::UntrustedContract, 19),
    ];
    pub const VET_REGISTRY_ERRORS: [(vet_registry::ContractError, u32); 3] = [
        (vet_registry::ContractError::AlreadyInitialized, 0),
        (vet_registry::ContractError::VetNotFound, 3),
        (vet_registry::ContractError::VetAlreadyVerified, 7),
    ];

    /// Mirror of the main contract's `ContractVersion`; decoding fails if it drifts.
    #[contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct MainContractVersion {
        pub major: u32,
        pub minor: u32,
        pub patch: u32,
    }
}

struct Deployment<'a> {
    env: Env,
    ownership: PetOwnershipContractClient<'a>,
    vets: VetRegistryContractClient<'a>,
    admin: Address,
}

fn deploy<'a>() -> Deployment<'a> {
    let env = Env::default();
    env.mock_all_auths();
    let ownership =
        PetOwnershipContractClient::new(&env, &env.register_contract(None, PetOwnershipContract));
    let vets =
        VetRegistryContractClient::new(&env, &env.register_contract(None, VetRegistryContract));
    let admin = Address::generate(&env);
    vets.init(&admin);
    Deployment {
        env,
        ownership,
        vets,
        admin,
    }
}

fn contract_error(code: u32) -> Error {
    Error::from_contract_error(code)
}

/// True if the last invocation was authorized by `who`.
fn authorized_by(env: &Env, who: &Address) -> bool {
    env.auths().iter().any(|(addr, _)| addr == who)
}

fn last_event_topic(env: &Env) -> Symbol {
    let (_, topics, _) = env.events().all().last().unwrap();
    Symbol::try_from_val(env, &topics.get(0).unwrap()).unwrap()
}

#[test]
fn pinned_error_codes_and_enums_are_stable() {
    for (err, code) in fixture::OWNERSHIP_ERRORS {
        assert_eq!(err as u32, code, "{err:?}");
    }
    for (err, code) in fixture::VET_REGISTRY_ERRORS {
        assert_eq!(err as u32, code, "{err:?}");
    }
    assert_eq!(
        fixture::transfer_type(&crate::TransferType::Direct),
        "Direct"
    );
}

#[test]
fn ownership_trusts_only_its_configured_contract() {
    let d = deploy();
    let signer = Address::generate(&d.env);
    let trusted = d.vets.address.clone();

    d.ownership
        .init_trusted_contract(&trusted, &vec![&d.env, signer.clone()], &1);
    assert!(authorized_by(&d.env, &signer));
    assert!(d.ownership.validate_trusted_contract(&trusted));
    assert_eq!(d.ownership.get_trusted_contract_address(), trusted);

    let other = d.ownership.address.clone();
    assert_eq!(
        d.ownership.try_validate_trusted_contract(&other),
        Err(Ok(contract_error(19)))
    );
}

#[test]
fn vet_registry_interface_round_trip() {
    let d = deploy();
    let vet = Address::generate(&d.env);
    let s = |v: &str| String::from_str(&d.env, v);

    d.vets
        .register_vet(&vet, &s("Dr. Compat"), &s("LIC-COMPAT"), &s("General"));
    assert!(authorized_by(&d.env, &vet));
    assert_eq!(last_event_topic(&d.env), Symbol::new(&d.env, "reg_vet"));
    assert!(!d.vets.is_verified_vet(&vet));

    d.vets.verify_vet(&vet);
    assert!(authorized_by(&d.env, &d.admin));
    assert_eq!(last_event_topic(&d.env), Symbol::new(&d.env, "ver_vet"));
    assert!(d.vets.is_verified_vet(&vet));

    assert_eq!(d.vets.try_verify_vet(&vet), Err(Ok(contract_error(7))));
    assert_eq!(d.vets.try_init(&d.admin), Err(Ok(contract_error(0))));
    assert_eq!(
        d.vets.try_is_verified_vet(&Address::generate(&d.env)),
        Err(Ok(contract_error(3)))
    );
}

#[test]
fn escrowed_transfer_state_machine() {
    let d = deploy();
    let (owner, buyer) = (Address::generate(&d.env), Address::generate(&d.env));

    d.ownership.create_pet(&7, &owner);
    d.ownership.initiate_transfer(&7, &buyer);
    assert!(authorized_by(&d.env, &owner));
    assert_eq!(last_event_topic(&d.env), Symbol::new(&d.env, "xfer_init"));

    d.ownership.accept_transfer(&7);
    assert!(authorized_by(&d.env, &buyer));
    assert_eq!(last_event_topic(&d.env), Symbol::new(&d.env, "xfer_escr"));
    assert_eq!(d.ownership.get_current_owner(&7), owner);

    d.ownership.confirm_custody(&7, &owner);
    d.ownership.confirm_custody(&7, &buyer);

    assert_eq!(
        d.ownership.try_finalize_transfer(&7),
        Err(Ok(contract_error(13)))
    );

    d.env
        .ledger()
        .set_timestamp(d.env.ledger().timestamp() + DISPUTE_WINDOW_SECONDS);
    d.ownership.finalize_transfer(&7);
    assert_eq!(last_event_topic(&d.env), Symbol::new(&d.env, "xfer_fin"));
    assert_eq!(d.ownership.get_current_owner(&7), buyer);
    assert_eq!(
        d.ownership.try_finalize_transfer(&7),
        Err(Ok(contract_error(12)))
    );
}

#[test]
fn disputed_transfer_cannot_be_finalized() {
    let d = deploy();
    let (owner, buyer) = (Address::generate(&d.env), Address::generate(&d.env));
    d.ownership.create_pet(&8, &owner);
    d.ownership.initiate_transfer(&8, &buyer);
    d.ownership.accept_transfer(&8);

    assert_eq!(
        d.ownership
            .try_raise_dispute(&8, &Address::generate(&d.env)),
        Err(Ok(contract_error(2)))
    );
    d.ownership.raise_dispute(&8, &buyer);
    assert!(authorized_by(&d.env, &buyer));
    assert_eq!(last_event_topic(&d.env), Symbol::new(&d.env, "xfer_disp"));

    d.env
        .ledger()
        .set_timestamp(d.env.ledger().timestamp() + DISPUTE_WINDOW_SECONDS);
    assert_eq!(
        d.ownership.try_finalize_transfer(&8),
        Err(Ok(contract_error(14)))
    );
    assert_eq!(d.ownership.get_current_owner(&8), owner);
}

/// Deploys the main contract WASM alongside the others. Run by the CI
/// `cross-contract` job; locally, build the main contract for
/// `wasm32-unknown-unknown` and set `PETCHAIN_MAIN_WASM` to its path.
#[test]
#[ignore = "needs PETCHAIN_MAIN_WASM (run by the CI cross-contract job)"]
fn main_contract_interoperates_with_workspace_contracts() {
    let path = std::env::var("PETCHAIN_MAIN_WASM")
        .expect("set PETCHAIN_MAIN_WASM to the main contract's release WASM");
    let wasm = std::fs::read(&path).expect("read PETCHAIN_MAIN_WASM");

    let d = deploy();
    let env = &d.env;
    // Uploading the WASM is not under test; each main-contract call below runs
    // under a fresh default budget, as it would in its own transaction.
    env.budget().reset_unlimited();
    let main = env.register_contract_wasm(None, wasm.as_slice());
    let call = |f: &str, args: Vec<Val>| -> Val {
        env.budget().reset_default();
        env.invoke_contract(&main, &Symbol::new(env, f), args)
    };
    let s = |v: &str| String::from_str(env, v);

    // Versioned interface: decodes into the pinned mirror type.
    let version = fixture::MainContractVersion::try_from_val(env, &call("get_version", vec![env]))
        .expect("main ContractVersion drifted");
    assert!(version.major >= 1);

    // Ownership contract trusts the deployed main contract, and only it.
    let signer = Address::generate(env);
    d.ownership
        .init_trusted_contract(&main, &vec![env, signer.clone()], &1);
    assert!(d.ownership.validate_trusted_contract(&main));
    assert_eq!(
        d.ownership.try_validate_trusted_contract(&d.vets.address),
        Err(Ok(contract_error(19)))
    );

    // Both vet registries accept the same registration call shape; verification
    // is intentionally admin-parameterised only on the main contract.
    let admin = Address::generate(env);
    let vet = Address::generate(env);
    call("init_admin", vec![env, admin.into_val(env)]);
    let register = vec![
        env,
        vet.into_val(env),
        s("Dr. Main").into_val(env),
        s("LIC-MAIN").into_val(env),
        s("General").into_val(env),
    ];
    let registered: bool = call("register_vet", register).into_val(env);
    assert!(registered);
    assert!(authorized_by(env, &vet));
    d.vets
        .register_vet(&vet, &s("Dr. Main"), &s("LIC-MAIN"), &s("General"));

    let verified: bool = call(
        "verify_vet",
        vec![env, admin.into_val(env), vet.into_val(env)],
    )
    .into_val(env);
    assert!(verified);
    assert!(authorized_by(env, &admin));
    let is_verified: bool = call("is_verified_vet", vec![env, vet.into_val(env)]).into_val(env);
    assert!(is_verified);
    assert!(!d.vets.is_verified_vet(&vet));
}
