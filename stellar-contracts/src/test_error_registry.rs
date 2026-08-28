use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

fn setup_env() -> (Env, PetChainContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init_admin(&admin);

    (env, client, admin)
}

// --- BASIC FUNCTIONALITY ---

#[test]
fn test_set_and_get_error_message() {
    let (env, client, admin) = setup_env();

    // Set an error message in English
    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Pet not found"),
    );

    // Retrieve the message
    let message = client.get_error_message(&3, &String::from_str(&env, "en"));
    assert_eq!(message, Some(String::from_str(&env, "Pet not found")));
}

#[test]
fn test_get_nonexistent_error_message_returns_none() {
    let (env, client, _admin) = setup_env();

    // Try to get a message that doesn't exist
    let message = client.get_error_message(&999, &String::from_str(&env, "en"));
    assert_eq!(message, None);
}

#[test]
fn test_set_error_message_multiple_languages() {
    let (env, client, admin) = setup_env();

    // Set English message
    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Pet not found"),
    );

    // Set Spanish message
    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "es"),
        &String::from_str(&env, "Mascota no encontrada"),
    );

    // Retrieve both messages
    let en_message = client.get_error_message(&3, &String::from_str(&env, "en"));
    let es_message = client.get_error_message(&3, &String::from_str(&env, "es"));

    assert_eq!(en_message, Some(String::from_str(&env, "Pet not found")));
    assert_eq!(
        es_message,
        Some(String::from_str(&env, "Mascota no encontrada"))
    );
}

#[test]
fn test_supported_languages_updated() {
    let (env, client, admin) = setup_env();

    // Initially no languages
    let languages = client.get_supported_languages();
    assert_eq!(languages.len(), 0);

    // Add English message
    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Pet not found"),
    );

    let languages = client.get_supported_languages();
    assert_eq!(languages.len(), 1);
    assert!(languages.contains(&String::from_str(&env, "en")));

    // Add Spanish message
    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "es"),
        &String::from_str(&env, "Mascota no encontrada"),
    );

    let languages = client.get_supported_languages();
    assert_eq!(languages.len(), 2);
    assert!(languages.contains(&String::from_str(&env, "en")));
    assert!(languages.contains(&String::from_str(&env, "es")));
}

#[test]
fn test_overwrite_existing_message() {
    let (env, client, admin) = setup_env();

    // Set initial message
    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Pet not found"),
    );

    // Overwrite with new message
    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Pet record not found in database"),
    );

    // Retrieve the updated message
    let message = client.get_error_message(&3, &String::from_str(&env, "en"));
    assert_eq!(
        message,
        Some(String::from_str(&env, "Pet record not found in database"))
    );
}

// --- BATCH OPERATIONS ---

#[test]
fn test_batch_set_error_messages() {
    let (env, client, admin) = setup_env();

    let mut messages = Vec::new(&env);
    messages.push_back(ErrorMessage {
        code: 1,
        language: String::from_str(&env, "en"),
        message: String::from_str(&env, "Unauthorized"),
    });
    messages.push_back(ErrorMessage {
        code: 2,
        language: String::from_str(&env, "en"),
        message: String::from_str(&env, "Admin not initialized"),
    });
    messages.push_back(ErrorMessage {
        code: 1,
        language: String::from_str(&env, "es"),
        message: String::from_str(&env, "No autorizado"),
    });

    client.batch_set_error_messages(&admin, &messages);

    // Verify all messages were set
    let msg1_en = client.get_error_message(&1, &String::from_str(&env, "en"));
    let msg2_en = client.get_error_message(&2, &String::from_str(&env, "en"));
    let msg1_es = client.get_error_message(&1, &String::from_str(&env, "es"));

    assert_eq!(msg1_en, Some(String::from_str(&env, "Unauthorized")));
    assert_eq!(
        msg2_en,
        Some(String::from_str(&env, "Admin not initialized"))
    );
    assert_eq!(msg1_es, Some(String::from_str(&env, "No autorizado")));
}

#[test]
fn test_initialize_error_messages() {
    let (env, client, admin) = setup_env();

    // Initialize default messages
    client.initialize_error_messages(&admin);

    // Check that English and Spanish are supported
    let languages = client.get_supported_languages();
    assert!(languages.contains(&String::from_str(&env, "en")));
    assert!(languages.contains(&String::from_str(&env, "es")));

    // Check some specific messages
    let msg_en = client.get_error_message(&3, &String::from_str(&env, "en"));
    let msg_es = client.get_error_message(&3, &String::from_str(&env, "es"));

    assert_eq!(msg_en, Some(String::from_str(&env, "Pet not found")));
    assert_eq!(
        msg_es,
        Some(String::from_str(&env, "Mascota no encontrada"))
    );
}

// --- REMOVE MESSAGES ---

#[test]
fn test_remove_error_message() {
    let (env, client, admin) = setup_env();

    // Set a message
    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Pet not found"),
    );

    // Verify it exists
    let message = client.get_error_message(&3, &String::from_str(&env, "en"));
    assert!(message.is_some());

    // Remove it
    client.remove_error_message(&admin, &3, &String::from_str(&env, "en"));

    // Verify it's gone
    let message = client.get_error_message(&3, &String::from_str(&env, "en"));
    assert_eq!(message, None);
}

// --- AUTHORIZATION ---

#[test]
#[should_panic]
fn test_set_error_message_requires_admin() {
    let (env, client, _admin) = setup_env();
    let non_admin = Address::generate(&env);

    // Non-admin should not be able to set messages
    client.set_error_message(
        &non_admin,
        &3,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Pet not found"),
    );
}

#[test]
#[should_panic]
fn test_batch_set_requires_admin() {
    let (env, client, _admin) = setup_env();
    let non_admin = Address::generate(&env);

    let mut messages = Vec::new(&env);
    messages.push_back(ErrorMessage {
        code: 1,
        language: String::from_str(&env, "en"),
        message: String::from_str(&env, "Unauthorized"),
    });

    client.batch_set_error_messages(&non_admin, &messages);
}

#[test]
#[should_panic]
fn test_initialize_requires_admin() {
    let (env, client, _admin) = setup_env();
    let non_admin = Address::generate(&env);

    client.initialize_error_messages(&non_admin);
}

#[test]
#[should_panic]
fn test_remove_requires_admin() {
    let (env, client, admin) = setup_env();
    let non_admin = Address::generate(&env);

    // Set a message as admin
    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Pet not found"),
    );

    // Try to remove as non-admin
    client.remove_error_message(&non_admin, &3, &String::from_str(&env, "en"));
}

// --- VALIDATION ---

#[test]
#[should_panic]
fn test_empty_language_rejected() {
    let (env, client, admin) = setup_env();

    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, ""),
        &String::from_str(&env, "Pet not found"),
    );
}

#[test]
#[should_panic]
fn test_language_too_long_rejected() {
    let (env, client, admin) = setup_env();

    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "verylonglanguagecode"),
        &String::from_str(&env, "Pet not found"),
    );
}

#[test]
#[should_panic]
fn test_empty_message_rejected() {
    let (env, client, admin) = setup_env();

    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "en"),
        &String::from_str(&env, ""),
    );
}

#[test]
#[should_panic]
fn test_message_too_long_rejected() {
    let (env, client, admin) = setup_env();

    // Create a message longer than 500 characters
    let long_message = "a".repeat(501);

    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "en"),
        &String::from_str(&env, &long_message),
    );
}

// --- MULTIPLE ERROR CODES ---

#[test]
fn test_multiple_error_codes_same_language() {
    let (env, client, admin) = setup_env();

    // Set messages for different error codes
    client.set_error_message(
        &admin,
        &1,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Unauthorized"),
    );

    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Pet not found"),
    );

    client.set_error_message(
        &admin,
        &160,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Storage quota exceeded"),
    );

    // Retrieve all messages
    let msg1 = client.get_error_message(&1, &String::from_str(&env, "en"));
    let msg3 = client.get_error_message(&3, &String::from_str(&env, "en"));
    let msg160 = client.get_error_message(&160, &String::from_str(&env, "en"));

    assert_eq!(msg1, Some(String::from_str(&env, "Unauthorized")));
    assert_eq!(msg3, Some(String::from_str(&env, "Pet not found")));
    assert_eq!(
        msg160,
        Some(String::from_str(&env, "Storage quota exceeded"))
    );
}

// --- LANGUAGE ISOLATION ---

#[test]
fn test_languages_are_isolated() {
    let (env, client, admin) = setup_env();

    // Set English message
    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Pet not found"),
    );

    // Spanish message should not exist
    let es_message = client.get_error_message(&3, &String::from_str(&env, "es"));
    assert_eq!(es_message, None);

    // Set Spanish message
    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "es"),
        &String::from_str(&env, "Mascota no encontrada"),
    );

    // Both should now exist independently
    let en_message = client.get_error_message(&3, &String::from_str(&env, "en"));
    let es_message = client.get_error_message(&3, &String::from_str(&env, "es"));

    assert_eq!(en_message, Some(String::from_str(&env, "Pet not found")));
    assert_eq!(
        es_message,
        Some(String::from_str(&env, "Mascota no encontrada"))
    );
}

// --- EDGE CASES ---

#[test]
fn test_language_code_case_sensitive() {
    let (env, client, admin) = setup_env();

    // Set message with lowercase language code
    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Pet not found"),
    );

    // Try to retrieve with uppercase
    let message = client.get_error_message(&3, &String::from_str(&env, "EN"));
    assert_eq!(message, None);

    // Retrieve with correct case
    let message = client.get_error_message(&3, &String::from_str(&env, "en"));
    assert_eq!(message, Some(String::from_str(&env, "Pet not found")));
}

#[test]
fn test_error_code_zero() {
    let (env, client, admin) = setup_env();

    // Error code 0 should work (though not used in practice)
    client.set_error_message(
        &admin,
        &0,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Success"),
    );

    let message = client.get_error_message(&0, &String::from_str(&env, "en"));
    assert_eq!(message, Some(String::from_str(&env, "Success")));
}

#[test]
fn test_large_error_code() {
    let (env, client, admin) = setup_env();

    // Test with large error code
    client.set_error_message(
        &admin,
        &999999,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Custom error"),
    );

    let message = client.get_error_message(&999999, &String::from_str(&env, "en"));
    assert_eq!(message, Some(String::from_str(&env, "Custom error")));
}

// --- REAL-WORLD SCENARIOS ---

#[test]
fn test_complete_error_registry_setup() {
    let (env, client, admin) = setup_env();

    // Initialize default messages
    client.initialize_error_messages(&admin);

    // Add a custom language (French)
    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "fr"),
        &String::from_str(&env, "Animal non trouvé"),
    );

    // Verify all three languages work
    let en = client.get_error_message(&3, &String::from_str(&env, "en"));
    let es = client.get_error_message(&3, &String::from_str(&env, "es"));
    let fr = client.get_error_message(&3, &String::from_str(&env, "fr"));

    assert!(en.is_some());
    assert!(es.is_some());
    assert!(fr.is_some());

    // Check supported languages
    let languages = client.get_supported_languages();
    assert_eq!(languages.len(), 3);
}

#[test]
fn test_update_existing_translation() {
    let (env, client, admin) = setup_env();

    // Set initial translation
    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "en"),
        &String::from_str(&env, "Pet not found"),
    );

    // Update to more descriptive message
    client.set_error_message(
        &admin,
        &3,
        &String::from_str(&env, "en"),
        &String::from_str(
            &env,
            "The requested pet record could not be found in the database",
        ),
    );

    let message = client.get_error_message(&3, &String::from_str(&env, "en"));
    assert_eq!(
        message,
        Some(String::from_str(
            &env,
            "The requested pet record could not be found in the database"
        ))
    );
}
// Append-only addition for issue #793
#[test]
fn test_batch_set_error_messages_at_limit_succeeds() {
    let (env, client, admin) = setup_env();

    let mut messages = Vec::new(&env);
    for i in 0..MAX_BATCH_ERROR_MESSAGES {
        messages.push_back(ErrorMessage {
            code: i as u32,
            language: String::from_str(&env, "en"),
            message: String::from_str(&env, "Test message"),
        });
    }

    client.batch_set_error_messages(&admin, &messages);

    let msg = client.get_error_message(
        &(MAX_BATCH_ERROR_MESSAGES as u32 - 1),
        &String::from_str(&env, "en"),
    );
    assert_eq!(msg, Some(String::from_str(&env, "Test message")));
}

#[test]
#[should_panic]
fn test_batch_set_error_messages_over_limit_fails() {
    let (env, client, admin) = setup_env();

    let mut messages = Vec::new(&env);
    for i in 0..=MAX_BATCH_ERROR_MESSAGES {
        messages.push_back(ErrorMessage {
            code: i as u32,
            language: String::from_str(&env, "en"),
            message: String::from_str(&env, "Test message"),
        });
    }

    client.batch_set_error_messages(&admin, &messages);
}

#[test]
fn test_batch_set_error_messages_empty_succeeds() {
    let (env, client, admin) = setup_env();

    let messages = Vec::new(&env);
    client.batch_set_error_messages(&admin, &messages);

    let languages = client.get_supported_languages();
    assert_eq!(languages.len(), 0);
}

// --- ISSUE #1029: ContractError discriminant snapshot tests ---

#[test]
fn test_contract_error_discriminants() {
    assert_eq!(ContractError::AdminAlreadyApproved as u32, 1);
    assert_eq!(ContractError::AdminAlreadySet as u32, 2);
    assert_eq!(ContractError::AdminNotInitialized as u32, 3);
    assert_eq!(ContractError::AdminsNotSet as u32, 4);
    assert_eq!(ContractError::BatchTooLarge as u32, 5);
    assert_eq!(ContractError::CertificateAlreadyAnchored as u32, 6);
    assert_eq!(ContractError::CounterOverflow as u32, 7);
    assert_eq!(ContractError::InputStringTooLong as u32, 8);
    assert_eq!(ContractError::InvalidBreed as u32, 9);
    assert_eq!(ContractError::InvalidCallerNonce as u32, 10);
    assert_eq!(ContractError::InvalidCertificateHash as u32, 11);
    assert_eq!(ContractError::InvalidInput as u32, 12);
    assert_eq!(ContractError::InvalidIpfsHash as u32, 13);
    assert_eq!(ContractError::InvalidPetName as u32, 14);
    assert_eq!(ContractError::InvalidRating as u32, 15);
    assert_eq!(ContractError::InvalidState as u32, 16);
    assert_eq!(ContractError::InvalidThreshold as u32, 17);
    assert_eq!(ContractError::InvokerNotInAdminList as u32, 18);
    assert_eq!(ContractError::LicenseAlreadyRegistered as u32, 19);
    assert_eq!(ContractError::NoAdminsConfigured as u32, 20);
    assert_eq!(ContractError::NotAnAdmin as u32, 21);
    assert_eq!(ContractError::NotPetOwner as u32, 22);
    assert_eq!(ContractError::PetAlreadyHasLinkedTag as u32, 23);
    assert_eq!(ContractError::PetNotFound as u32, 24);
    assert_eq!(ContractError::StorageQuotaExceeded as u32, 25);
    assert_eq!(ContractError::ThresholdNotMet as u32, 26);
    assert_eq!(ContractError::TooManyItems as u32, 27);
    assert_eq!(ContractError::Unauthorized as u32, 28);
    assert_eq!(ContractError::VaccinationNotFound as u32, 29);
    assert_eq!(ContractError::VetAlreadyRegistered as u32, 30);
    assert_eq!(ContractError::VetNotFound as u32, 31);
    assert_eq!(ContractError::VetNotVerified as u32, 32);
    assert_eq!(ContractError::VeterinarianNotVerified as u32, 33);
    assert_eq!(ContractError::SlotAlreadyBooked as u32, 34);
    assert_eq!(ContractError::DuplicateActivity as u32, 35);
    assert_eq!(ContractError::InbreedingThresholdExceeded as u32, 36);
    assert_eq!(ContractError::SelfBreeding as u32, 37);
    assert_eq!(ContractError::ProposalAlreadyExecuted as u32, 38);
    assert_eq!(ContractError::InvalidNonce as u32, 39);
    assert_eq!(ContractError::ProposalNotFound as u32, 39);
    assert_eq!(ContractError::NoPreviousUpgrade as u32, 41);
    assert_eq!(ContractError::RollbackWindowExpired as u32, 40);
    assert_eq!(ContractError::ProposalExpired as u32, 43);
    assert_eq!(ContractError::ProposalNotApproved as u32, 44);
    assert_eq!(ContractError::QuorumNotMet as u32, 45);
    assert_eq!(ContractError::RateLimitExceeded as u32, 46);
    assert_eq!(ContractError::AlreadyDeleted as u32, 160);
    assert_eq!(ContractError::RecordAlreadyDeleted as u32, 161);
    assert_eq!(ContractError::RetentionPeriodNotMet as u32, 162);
    assert_eq!(ContractError::RecordNotFound as u32, 163);
}
