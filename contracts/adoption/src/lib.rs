#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Symbol};

/// Storage schema version. Bump when the persistent layout changes.
const STORAGE_VERSION: u32 = 1;

/// Maximum length (in bytes) accepted for free-form application notes.
const MAX_NOTES_LEN: u32 = 256;

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ApplicationState {
    Submitted = 0,
    UnderReview = 1,
    Approved = 2,
    Declined = 3,
    Withdrawn = 4,
}

#[contracttype]
#[derive(Clone)]
pub struct Application {
    pub id: u64,
    pub applicant: Address,
    pub pet_id: u64,
    pub state: ApplicationState,
    pub notes_len: u32,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    ApplicationNotFound = 4,
    InvalidTransition = 5,
    TerminalState = 6,
    NotesTooLong = 7,
    Overflow = 8,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Version,
    NextId,
    Application(u64),
}

#[contract]
pub struct AdoptionContract;

#[contractimpl]
impl AdoptionContract {
    /// Initialize the contract with an admin/reviewer authority.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Version) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Version, &STORAGE_VERSION);
        env.storage().instance().set(&DataKey::NextId, &1u64);
        Ok(())
    }

    /// Applicant submits a new adoption application. Starts in `Submitted`.
    pub fn submit_application(
        env: Env,
        applicant: Address,
        pet_id: u64,
        notes_len: u32,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        if notes_len > MAX_NOTES_LEN {
            return Err(Error::NotesTooLong);
        }
        applicant.require_auth();

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextId)
            .unwrap_or(1u64);
        let next = id.checked_add(1).ok_or(Error::Overflow)?;
        env.storage().instance().set(&DataKey::NextId, &next);

        let application = Application {
            id,
            applicant: applicant.clone(),
            pet_id,
            state: ApplicationState::Submitted,
            notes_len,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Application(id), &application);

        env.events().publish(
            (Symbol::new(&env, "app_submitted"), applicant),
            (id, pet_id),
        );
        Ok(id)
    }

    /// Reviewer/admin moves a `Submitted` application into `UnderReview`.
    pub fn start_review(env: Env, reviewer: Address, id: u64) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_reviewer(&env, &reviewer)?;
        reviewer.require_auth();

        let mut application = Self::load(&env, id)?;
        Self::require_state(&application, ApplicationState::Submitted)?;
        application.state = ApplicationState::UnderReview;
        Self::save(&env, &application);

        env.events().publish(
            (Symbol::new(&env, "app_under_review"), reviewer),
            id,
        );
        Ok(())
    }

    /// Reviewer/admin approves an application that is `UnderReview`.
    pub fn approve(env: Env, reviewer: Address, id: u64) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_reviewer(&env, &reviewer)?;
        reviewer.require_auth();

        let mut application = Self::load(&env, id)?;
        Self::require_state(&application, ApplicationState::UnderReview)?;
        application.state = ApplicationState::Approved;
        Self::save(&env, &application);

        env.events()
            .publish((Symbol::new(&env, "app_approved"), reviewer), id);
        Ok(())
    }

    /// Reviewer/admin declines an application that is `Submitted` or `UnderReview`.
    pub fn decline(env: Env, reviewer: Address, id: u64) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_reviewer(&env, &reviewer)?;
        reviewer.require_auth();

        let mut application = Self::load(&env, id)?;
        if application.state != ApplicationState::Submitted
            && application.state != ApplicationState::UnderReview
        {
            return Err(Self::transition_error(&application.state));
        }
        application.state = ApplicationState::Declined;
        Self::save(&env, &application);

        env.events()
            .publish((Symbol::new(&env, "app_declined"), reviewer), id);
        Ok(())
    }

    /// Applicant withdraws their own application while it is non-terminal.
    pub fn withdraw(env: Env, applicant: Address, id: u64) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        applicant.require_auth();

        let mut application = Self::load(&env, id)?;
        if application.applicant != applicant {
            return Err(Error::Unauthorized);
        }
        if application.state != ApplicationState::Submitted
            && application.state != ApplicationState::UnderReview
        {
            return Err(Self::transition_error(&application.state));
        }
        application.state = ApplicationState::Withdrawn;
        Self::save(&env, &application);

        env.events()
            .publish((Symbol::new(&env, "app_withdrawn"), applicant), id);
        Ok(())
    }

    /// Read-only accessor for an application's current state.
    pub fn get_application(env: Env, id: u64) -> Result<Application, Error> {
        Self::load(&env, id)
    }

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Version) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    fn require_reviewer(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if &admin != caller {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    fn require_state(application: &Application, expected: ApplicationState) -> Result<(), Error> {
        if application.state != expected {
            return Err(Self::transition_error(&application.state));
        }
        Ok(())
    }

    /// Terminal states cannot transition further; non-terminal mismatches are invalid.
    fn transition_error(state: &ApplicationState) -> Error {
        match state {
            ApplicationState::Approved
            | ApplicationState::Declined
            | ApplicationState::Withdrawn => Error::TerminalState,
            _ => Error::InvalidTransition,
        }
    }

    fn load(env: &Env, id: u64) -> Result<Application, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Application(id))
            .ok_or(Error::ApplicationNotFound)
    }

    fn save(env: &Env, application: &Application) {
        env.storage()
            .persistent()
            .set(&DataKey::Application(application.id), application);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    fn setup() -> (Env, AdoptionContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, AdoptionContract);
        let client = AdoptionContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, client, admin)
    }

    #[test]
    fn happy_path_submit_review_approve() {
        let (env, client, admin) = setup();
        let applicant = Address::generate(&env);
        let id = client.submit_application(&applicant, &7u64, &10u32);
        assert_eq!(client.get_application(&id).state, ApplicationState::Submitted);
        client.start_review(&admin, &id);
        assert_eq!(client.get_application(&id).state, ApplicationState::UnderReview);
        client.approve(&admin, &id);
        assert_eq!(client.get_application(&id).state, ApplicationState::Approved);
    }

    #[test]
    fn applicant_can_withdraw() {
        let (env, client, _admin) = setup();
        let applicant = Address::generate(&env);
        let id = client.submit_application(&applicant, &1u64, &0u32);
        client.withdraw(&applicant, &id);
        assert_eq!(client.get_application(&id).state, ApplicationState::Withdrawn);
    }

    #[test]
    fn unauthorized_reviewer_rejected() {
        let (env, client, _admin) = setup();
        let applicant = Address::generate(&env);
        let stranger = Address::generate(&env);
        let id = client.submit_application(&applicant, &1u64, &0u32);
        assert_eq!(client.try_start_review(&stranger, &id), Err(Ok(Error::Unauthorized)));
        assert_eq!(client.get_application(&id).state, ApplicationState::Submitted);
    }

    #[test]
    fn non_owner_cannot_withdraw() {
        let (env, client, _admin) = setup();
        let applicant = Address::generate(&env);
        let stranger = Address::generate(&env);
        let id = client.submit_application(&applicant, &1u64, &0u32);
        assert_eq!(client.try_withdraw(&stranger, &id), Err(Ok(Error::Unauthorized)));
        assert_eq!(client.get_application(&id).state, ApplicationState::Submitted);
    }

    #[test]
    fn terminal_states_cannot_transition() {
        let (env, client, admin) = setup();
        let applicant = Address::generate(&env);
        let id = client.submit_application(&applicant, &1u64, &0u32);
        client.decline(&admin, &id);
        assert_eq!(client.try_approve(&admin, &id), Err(Ok(Error::TerminalState)));
        assert_eq!(client.try_withdraw(&applicant, &id), Err(Ok(Error::TerminalState)));
        assert_eq!(client.get_application(&id).state, ApplicationState::Declined);
    }

    #[test]
    fn invalid_transition_from_submitted_to_approve() {
        let (env, client, admin) = setup();
        let applicant = Address::generate(&env);
        let id = client.submit_application(&applicant, &1u64, &0u32);
        assert_eq!(client.try_approve(&admin, &id), Err(Ok(Error::InvalidTransition)));
        assert_eq!(client.get_application(&id).state, ApplicationState::Submitted);
    }

    #[test]
    fn notes_boundary_enforced() {
        let (env, client, _admin) = setup();
        let applicant = Address::generate(&env);
        assert_eq!(
            client.try_submit_application(&applicant, &1u64, &(MAX_NOTES_LEN + 1)),
            Err(Ok(Error::NotesTooLong))
        );
        let id = client.submit_application(&applicant, &1u64, &MAX_NOTES_LEN);
        assert_eq!(client.get_application(&id).notes_len, MAX_NOTES_LEN);
    }

    #[test]
    fn missing_application_errors() {
        let (_env, client, admin) = setup();
        assert_eq!(client.try_start_review(&admin, &999u64), Err(Ok(Error::ApplicationNotFound)));
    }
}
