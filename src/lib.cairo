pub mod contracts {
    pub mod petchain;
    pub mod interface;
    pub mod types;
}
pub mod components {
    pub mod pet_owner {
        pub mod petowner_component;
        pub mod mock_pet_owner;
        pub mod IPetOwner;
        pub mod pet_owner_test;
    }
}

pub mod base {
    pub mod types;
}
