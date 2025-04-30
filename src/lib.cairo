pub mod contracts {
    pub mod petchain;
    pub mod interface;
    pub mod types;
}
pub mod components {
    pub mod pet_owner {
        pub mod pet_owner;
        pub mod mock;
        pub mod interface;
        pub mod test;
        pub mod types;
    }

    pub mod veterinary_professional {
        pub mod vet;
        pub mod mock;
        pub mod interface;
        pub mod test;
        pub mod types;
    }
}
