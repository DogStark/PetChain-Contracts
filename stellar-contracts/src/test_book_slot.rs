#[cfg(test)]
mod test_book_slot {
    use crate::{PetChainContract, PetChainContractClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    fn setup_env() -> (Env, PetChainContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn setup_verified_vet(env: &Env, client: &PetChainContractClient) -> Address {
        let admin = Address::generate(env);
        let vet = Address::generate(env);

        let mut admins = soroban_sdk::Vec::new(env);
        admins.push_back(admin.clone());
        client.init_multisig(&admin, &admins, &1u32);

        client.register_vet(
            &vet,
            &String::from_str(env, "Dr. Slot Tester"),
            &String::from_str(env, "LIC-SLOT-001"),
            &String::from_str(env, "General"),
        );
        client.verify_vet(&admin, &vet);
        vet
    }

    fn add_slot(env: &Env, client: &PetChainContractClient, vet: &Address) -> u64 {
        let now = env.ledger().timestamp();
        client.set_availability(vet, &now, &(now + 3600))
    }

    // -------------------------------------------------------
    // Add availability: verified vet can add a slot
    // -------------------------------------------------------
    #[test]
    fn test_add_availability_slot() {
        let (env, client) = setup_env();
        let vet = setup_verified_vet(&env, &client);

        let slot_index = add_slot(&env, &client, &vet);
        assert_eq!(slot_index, 1u64);

        let date = env.ledger().timestamp() / 86400;
        let slots = client.get_available_slots(&vet, &date);
        assert_eq!(slots.len(), 1);
        assert!(slots.get(0).unwrap().available);
    }

    // -------------------------------------------------------
    // Book slot: booking marks the slot unavailable
    // -------------------------------------------------------
    #[test]
    fn test_book_slot_marks_unavailable() {
        let (env, client) = setup_env();
        let vet = setup_verified_vet(&env, &client);
        let slot_index = add_slot(&env, &client, &vet);

        let result = client.book_slot(&vet, &slot_index);
        assert!(result, "book_slot should return true for an available slot");

        let date = env.ledger().timestamp() / 86400;
        let slots = client.get_available_slots(&vet, &date);
        assert!(slots.is_empty(), "Slot should not appear as available after booking");
    }

    // -------------------------------------------------------
    // Double-booking: already-booked slot must panic
    // -------------------------------------------------------
    #[test]
    #[should_panic(expected = "Slot already booked")]
    fn test_cannot_double_book_slot() {
        let (env, client) = setup_env();
        let vet = setup_verified_vet(&env, &client);
        let slot_index = add_slot(&env, &client, &vet);

        client.book_slot(&vet, &slot_index);
        // Second booking on the same slot must panic
        client.book_slot(&vet, &slot_index);
    }

    // -------------------------------------------------------
    // Cancel booking: vet can restore a booked slot
    // -------------------------------------------------------
    #[test]
    fn test_cancel_booking_restores_availability() {
        let (env, client) = setup_env();
        let vet = setup_verified_vet(&env, &client);
        let slot_index = add_slot(&env, &client, &vet);

        client.book_slot(&vet, &slot_index);

        // Slot is booked — cancel it
        let cancelled = client.cancel_booking(&vet, &slot_index);
        assert!(cancelled, "cancel_booking should return true");

        // Slot must be available again
        let date = env.ledger().timestamp() / 86400;
        let slots = client.get_available_slots(&vet, &date);
        assert_eq!(slots.len(), 1);
        assert!(slots.get(0).unwrap().available);
        assert!(
            !slots.is_empty(),
            "Slot should be available before any booking"
        );
        assert_eq!(slots.get(0).unwrap().available, true);

        // A legitimate owner can still book the untouched slot
        let booked = client.book_slot(&vet, &slot_index);
        assert!(
            booked,
            "Legitimate owner should be able to book the available slot"
        );

        // Now the slot must be gone from available list
        let slots_after = client.get_available_slots(&vet, &date);
        assert!(
            slots_after.is_empty(),
            "Slot should be unavailable after legitimate booking"
        );
    }

    // -------------------------------------------------------
    // Cancel unbooked slot must panic
    // -------------------------------------------------------
    #[test]
    #[should_panic(expected = "Slot is not booked")]
    fn test_cancel_unbooked_slot_panics() {
        let (env, client) = setup_env();
        let vet = setup_verified_vet(&env, &client);
        let slot_index = add_slot(&env, &client, &vet);

        // Slot was never booked — cancelling must panic
        client.cancel_booking(&vet, &slot_index);
    }

    // -------------------------------------------------------
    // Non-existent slot: book_slot returns false
    // -------------------------------------------------------
    #[test]
    fn test_book_nonexistent_slot_returns_false() {
        let (env, client) = setup_env();
        let vet = setup_verified_vet(&env, &client);

        let result = client.book_slot(&vet, &999u64);
        assert!(!result, "Booking a non-existent slot should return false");
    }

    // -------------------------------------------------------
    // Multiple slots: each slot is independent
    // -------------------------------------------------------
    #[test]
    fn test_multiple_slots_are_independent() {
        let (env, client) = setup_env();
        let vet = setup_verified_vet(&env, &client);

        let now = env.ledger().timestamp();
        let slot1 = client.set_availability(&vet, &now, &(now + 3600));
        let slot2 = client.set_availability(&vet, &(now + 7200), &(now + 10800));

        // Book only slot1
        client.book_slot(&vet, &slot1);

        let date = now / 86400;
        let slots = client.get_available_slots(&vet, &date);
        // slot2 must still be available
        assert_eq!(slots.len(), 1);
        assert_eq!(slots.get(0).unwrap().start_time, now + 7200);

        // slot2 can still be booked
        assert!(client.book_slot(&vet, &slot2));
    }
}
