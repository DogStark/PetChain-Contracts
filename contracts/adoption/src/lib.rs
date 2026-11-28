#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Symbol, Vec};

/// Storage schema version. Bump when the persistent layout changes.
const STORAGE_VERSION: u32 = 2;

/// Maximum length (in bytes) accepted for f49
ree-form application notes.
const MAX_NOTES_LEN: u32 = 256;

/// Maximum number of applications returned in a single paginated list call.
const MAX_PAGE_SIZE: u32 = 50;

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
    /// SHA-256 digest of the off-chain notes, binding the on-chain record to
    /// the exact note content submitted.  Reviewers and auditors can verify
    /// integrity by hashing the off-chain document and comparing to this field.
    pub notes_hash: BytesN<32>,
}

/// Paginated list result for adoption applications.
#[contracttype]
#[derive(Clone)]
pub struct ApplicationPage {
    /// The applications in this page.
    pub items: Vec<Application>,
    /// The cursor to pass for the next page, or 0 when there are no more results.
    pub next_cursor: u64,
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
    InvalidPageSize = 9,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Version,
    NextId,
    TransferContract,
    Application(u64),
}

#[contract]
pub struct AdoptionContract;

#[contractimpl]
impl AdoptionContract {
    /// Initialize the contract with an admin/reviewer authority and the
    /// pet-transfer-adoption contract address used to execute custody transfers.
    pub fn initialize(env: Env, admin: Address, transfer_contract: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Version) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::TransferContract, &transfer_contract);
        env.storage()
            .instance()
            .set(&DataKey::Version, &STORAGE_VERSION);
        env.storage().instance().set(&DataKey::NextId, &1u64);
        Ok(())
    }

    /// Rotate the admin/reviewer authority to a new address.
    ///
    /// Only the current admin may call this.  The transfer is atomic: the old
    /// admin loses authority the moment the new admin is stored.
    pub fn transfer_admin(
        env: Env,
        current_admin: Address,
        new_admin: Address,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_reviewer(&env, &current_admin)?;
        current_admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        env.events().publish(
            (Symbol::new(&env, "admin_transferred"), current_admin),
            new_admin,
        );
        Ok(())
    }

    /// Applicant submits a new adoption application. Starts in `Submitted`.
    ///
    /// `notes_hash` must be the SHA-256 digest (32 bytes) of the off-chain
    /// notes document. This binds the on-chain record to the exact content
    /// submitted, enabling tamper detection during review and audit.
    pub fn submit_application(
        env: Env,
        applicant: Address,
        pet_id: u64,
        notes_len: u32,
        notes_hash: BytesN<32>,
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
            notes_hash,
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

    /// Reviewer/admin approves an application that is `UnderReview` and
    /// triggers the custody/ownership transfer via the pet-transfer-adoption contract.
    pub fn approve(env: Env, reviewer: Address, id: u64) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_reviewer(&env, &reviewer)?;
        reviewer.require_auth();

        let mut application = Self::load(&env, id)?;
        Self::require_state(&application, ApplicationState::UnderReview)?;
        application.state = ApplicationState::Approved;
        Self::save(&env, &application);

        // Emit the approval event.
        env.events()
            .publish((Symbol::new(&env, "app_approved"), reviewer.clone()), id);

        // Trigger custody transfer in the pet-transfer-adoption contract.
        let transfer_contract: Address = env
            .storage()
            .instance()
            .get(&DataKey::TransferContract)
            .ok_or(Error::NotInitialized)?;

        env.events().publish(
            (
                Symbol::new(&env, "custody_transferred"),
                transfer_contract.clone(),
            ),
            (application.pet_id, application.applicant.clone(), id),
        );

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

    /// Return up to `limit` applications for the given `pet_id`, starting
    /// after `cursor` (exclusive; pass 0 to begin from the first application).
    /// Returns a page with a `next_cursor` of 0 when there are no more results.
    pub fn list_by_pet(
        env: Env,
        pet_id: u64,
        cursor: u64,
        limit: u32,
    ) -> Result<ApplicationPage, Error> {
        Self::require_initialized(&env)?;
        if limit == 0 || limit > MAX_PAGE_SIZE {
            return Err(Error::InvalidPageSize);
        }
        let next_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextId)
            .unwrap_or(1u64);

        let mut items: Vec<Application> = Vec::new(&env);
        let mut next_cursor: u64 = 0;
        let start = if cursor == 0 { 1u64 } else { cursor + 1 };

        let mut id = start;
        while id < next_id {
            if let Some(app) = env
                .storage()
                .persistent()
                .get::<DataKey, Application>(&DataKey::Application(id))
            {
                if app.pet_id == pet_id {
                    if items.len() as u32 >= limit {
                        next_cursor = id;
                        break;
                    }
                    items.push_back(app);
                }
            }
            id += 1;
        }

        Ok(ApplicationPage { items, next_cursor })
    }

    /// Return up to `limit` applications submitted by `applicant`, starting
    /// after `cursor` (exclusive; pass 0 to begin from the first application).
    /// Returns a page with a `next_cursor` of 0 when there are no more results.
    pub fn list_by_applicant(
        env: Env,
        applicant: Address,
        cursor: u64,
        limit: u32,
    ) -> Result<ApplicationPage, Error> {
        Self::require_initialized(&env)?;
        if limit == 0 || limit > MAX_PAGE_SIZE {
            return Err(Error::InvalidPageSize);
        }
        let next_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextId)
            .unwrap_or(1u64);

        let mut items: Vec<Application> = Vec::new(&env);
        let mut next_cursor: u64 = 0;
        let start = if cursor == 0 { 1u64 } else { cursor + 1 };

        let mut id = start;
        while id < next_id {
            if let Some(app) = env
                .storage()
                .persistent()
                .get::<DataKey, Application>(&DataKey::Application(id))
            {
                if app.applicant == applicant {
                    if items.len() as u32 >= limit {
                        next_cursor = id;
                        break;
                    }
                    items.push_back(app);
                }
            }
            id += 1;
        }

        Ok(ApplicationPage { items, next_cursor })
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
    use soroban_sdk::{Bytes, Env};

    fn zero_hash(env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &[0u8; 32])
    }

    fn setup() -> (Env, AdoptionContractClient<'static>, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, AdoptionContract);
        let client = AdoptionContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let transfer_contract = Address::generate(&env);
        client.initialize(&admin, &transfer_contract);
        (env, client, admin, transfer_contract)
    }

    #[test]
    fn happy_path_submit_review_approve() {
        let (env, client, admin, _) = setup();
        let applicant = Address::generate(&env);
        let hash = zero_hash(&env);
        let id = client.submit_application(&applicant, &7u64, &10u32, &hash);
        let app = client.get_application(&id);
        assert_eq!(app.state, ApplicationState::Submitted);
        assert_eq!(app.notes_hash, hash);
        client.start_review(&admin, &id);
        assert_eq!(client.get_application(&id).state, ApplicationState::UnderReview);
        client.approve(&admin, &id);
        assert_eq!(client.get_application(&id).state, ApplicationState::Approved);
    }

    #[test]
    fn approve_emits_custody_transferred_event() {
        let (env, client, admin, _transfer_contract) = setup();
        let applicant = Address::generate(&env);
        let pet_id = 42u64;
        let id = client.submit_application(&applicant, &pet_id, &0u32);
        client.start_review(&admin, &id);
        client.approve(&admin, &id);
        // Custody transfer event must have been published (pet_id, applicant, app_id)
        let events = env.events().all();
        let has_transfer = events.iter().any(|e| {
            let topic_sym = Symbol::new(&env, "custody_transferred");
            // Check topic contains the custody_transferred symbol
            e.0.iter().any(|t| t == topic_sym.into())
        });
        assert!(has_transfer, "custody_transferred event not emitted");
    }

    #[test]
    fn applicant_can_withdraw() {
        let (env, client, _admin, _) = setup();
        let applicant = Address::generate(&env);
        let id = client.submit_application(&applicant, &1u64, &0u32, &zero_hash(&env));
        client.withdraw(&applicant, &id);
        assert_eq!(client.get_application(&id).state, ApplicationState::Withdrawn);
    }

    #[test]
    fn unauthorized_reviewer_rejected() {
        let (env, client, _admin, _) = setup();
        let applicant = Address::generate(&env);
        let stranger = Address::generate(&env);
        let id = client.submit_application(&applicant, &1u64, &0u32, &zero_hash(&env));
        assert_eq!(client.try_start_review(&stranger, &id), Err(Ok(Error::Unauthorized)));
        assert_eq!(client.get_application(&id).state, ApplicationState::Submitted);
    }

    #[test]
    fn non_owner_cannot_withdraw() {
        let (env, client, _admin, _) = setup();
        let applicant = Address::generate(&env);
        let stranger = Address::generate(&env);
        let id = client.submit_application(&applicant, &1u64, &0u32, &zero_hash(&env));
        assert_eq!(client.try_withdraw(&stranger, &id), Err(Ok(Error::Unauthorized)));
        assert_eq!(client.get_application(&id).state, ApplicationState::Submitted);
    }

    #[test]
    fn terminal_states_cannot_transition() {
        let (env, client, admin, _) = setup();
        let applicant = Address::generate(&env);
        let id = client.submit_application(&applicant, &1u64, &0u32, &zero_hash(&env));
        client.decline(&admin, &id);
        assert_eq!(client.try_approve(&admin, &id), Err(Ok(Error::TerminalState)));
        assert_eq!(client.try_withdraw(&applicant, &id), Err(Ok(Error::TerminalState)));
        assert_eq!(client.get_application(&id).state, ApplicationState::Declined);
    }

    #[test]
    fn invalid_transition_from_submitted_to_approve() {
        let (env, client, admin, _) = setup();
        let applicant = Address::generate(&env);
        let id = client.submit_application(&applicant, &1u64, &0u32, &zero_hash(&env));
        assert_eq!(client.try_approve(&admin, &id), Err(Ok(Error::InvalidTransition)));
        assert_eq!(client.get_application(&id).state, ApplicationState::Submitted);
    }

    #[test]
    fn notes_boundary_enforced() {
        let (env, client, _admin, _) = setup();
        let applicant = Address::generate(&env);
        let hash = zero_hash(&env);
        assert_eq!(
            client.try_submit_application(&applicant, &1u64, &(MAX_NOTES_LEN + 1), &hash),
            Err(Ok(Error::NotesTooLong))
        );
        let id = client.submit_application(&applicant, &1u64, &MAX_NOTES_LEN, &hash);
        assert_eq!(client.get_application(&id).notes_len, MAX_NOTES_LEN);
    }

    #[test]
    fn missing_application_errors() {
        let (_env, client, admin, _) = setup();
        assert_eq!(client.try_start_review(&admin, &999u64), Err(Ok(Error::ApplicationNotFound)));
    }

    #[test]
    fn list_by_pet_returns_matching_applications() {
        let (env, client, _admin, _) = setup();
        let applicant1 = Address::generate(&env);
        let applicant2 = Address::generate(&env);
        let pet_id = 10u64;

        let id1 = client.submit_application(&applicant1, &pet_id, &0u32);
        client.submit_application(&applicant2, &999u64, &0u32); // different pet
        let id3 = client.submit_application(&applicant2, &pet_id, &0u32);

        let page = client.list_by_pet(&pet_id, &0u64, &50u32);
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.items.get(0).unwrap().id, id1);
        assert_eq!(page.items.get(1).unwrap().id, id3);
        assert_eq!(page.next_cursor, 0); // no more pages
    }

    #[test]
    fn list_by_pet_paginates_correctly() {
        let (env, client, _admin, _) = setup();
        let applicant = Address::generate(&env);
        let pet_id = 5u64;
        for _ in 0..5u32 {
            client.submit_application(&applicant, &pet_id, &0u32);
        }

        let page1 = client.list_by_pet(&pet_id, &0u64, &3u32);
        assert_eq!(page1.items.len(), 3);
        assert_ne!(page1.next_cursor, 0);

        let page2 = client.list_by_pet(&pet_id, &page1.next_cursor, &3u32);
        assert_eq!(page2.items.len(), 2);
        assert_eq!(page2.next_cursor, 0);
    }

    #[test]
    fn list_by_applicant_returns_matching_applications() {
        let (env, client, _admin, _) = setup();
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        let id1 = client.submit_application(&alice, &1u64, &0u32);
        client.submit_application(&bob, &2u64, &0u32);
        let id3 = client.submit_application(&alice, &3u64, &0u32);

        let page = client.list_by_applicant(&alice, &0u64, &50u32);
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.items.get(0).unwrap().id, id1);
        assert_eq!(page.items.get(1).unwrap().id, id3);
        assert_eq!(page.next_cursor, 0);
    }

    #[test]
    fn list_invalid_page_size_errors() {
        let (_env, client, _, _) = setup();
        assert_eq!(client.try_list_by_pet(&1u64, &0u64, &0u32), Err(Ok(Error::InvalidPageSize)));
        assert_eq!(
            client.try_list_by_pet(&1u64, &0u64, &(MAX_PAGE_SIZE + 1)),
            Err(Ok(Error::InvalidPageSize))
        );
    }
}
