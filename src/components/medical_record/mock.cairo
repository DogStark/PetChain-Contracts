#[starknet::contract]
mod MockMedicalRecordsComponent {
    use petchain::components::medical_record::medical_record::MedicalRecordsComponent;

    component!(
        path: MedicalRecordsComponent, storage: medical_record_storage, event: MedicalRecordEvent,
    );

    #[storage]
    struct Storage {
        #[substorage(v0)]
        medical_record_storage: MedicalRecordsComponent::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        MedicalRecordEvent: MedicalRecordsComponent::Event,
    }


    #[abi(embed_v0)]
    impl Medical = MedicalRecordsComponent::MedicalRecordsImpl<ContractState>;
}

