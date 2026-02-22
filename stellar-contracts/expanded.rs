#![feature(prelude_import)]
#![no_std]
#![allow(clippy::too_many_arguments)]
#[macro_use]
extern crate core;
#[prelude_import]
use core::prelude::rust_2021::*;
use soroban_sdk::xdr::{FromXdr, ToXdr};
use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Bytes, BytesN, Env, String, Symbol,
    Vec,
};
pub enum Species {
    Other,
    Dog,
    Cat,
    Bird,
}
#[automatically_derived]
impl ::core::clone::Clone for Species {
    #[inline]
    fn clone(&self) -> Species {
        match self {
            Species::Other => Species::Other,
            Species::Dog => Species::Dog,
            Species::Cat => Species::Cat,
            Species::Bird => Species::Bird,
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Species {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                Species::Other => "Other",
                Species::Dog => "Dog",
                Species::Cat => "Cat",
                Species::Bird => "Bird",
            },
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for Species {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Species {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Species {
    #[inline]
    fn eq(&self, other: &Species) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
pub static __SPEC_XDR_TYPE_SPECIES: [u8; 96usize] = Species::spec_xdr();
impl Species {
    pub const fn spec_xdr() -> [u8; 96usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\x07Species\0\0\0\0\x04\0\0\0\0\0\0\0\0\0\0\0\x05Other\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x03Dog\0\0\0\0\0\0\0\0\0\0\0\0\x03Cat\0\0\0\0\0\0\0\0\0\0\0\0\x04Bird"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for Species {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &["Other", "Dog", "Cat", "Bird"];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Other
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Dog
                }
                2 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Cat
                }
                3 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Bird
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, Species> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &Species,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            Species::Other => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Other")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            Species::Dog => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Dog")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            Species::Cat => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Cat")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            Species::Bird => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Bird")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub enum Gender {
    NotSpecified,
    Male,
    Female,
    Unknown,
}
#[automatically_derived]
impl ::core::clone::Clone for Gender {
    #[inline]
    fn clone(&self) -> Gender {
        match self {
            Gender::NotSpecified => Gender::NotSpecified,
            Gender::Male => Gender::Male,
            Gender::Female => Gender::Female,
            Gender::Unknown => Gender::Unknown,
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Gender {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                Gender::NotSpecified => "NotSpecified",
                Gender::Male => "Male",
                Gender::Female => "Female",
                Gender::Unknown => "Unknown",
            },
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for Gender {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Gender {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Gender {
    #[inline]
    fn eq(&self, other: &Gender) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
pub static __SPEC_XDR_TYPE_GENDER: [u8; 108usize] = Gender::spec_xdr();
impl Gender {
    pub const fn spec_xdr() -> [u8; 108usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\x06Gender\0\0\0\0\0\x04\0\0\0\0\0\0\0\0\0\0\0\x0cNotSpecified\0\0\0\0\0\0\0\0\0\0\0\x04Male\0\0\0\0\0\0\0\0\0\0\0\x06Female\0\0\0\0\0\0\0\0\0\0\0\0\0\x07Unknown\0"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for Gender {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &[
            "NotSpecified",
            "Male",
            "Female",
            "Unknown",
        ];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::NotSpecified
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Male
                }
                2 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Female
                }
                3 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Unknown
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, Gender> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &Gender,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            Gender::NotSpecified => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"NotSpecified")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            Gender::Male => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Male")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            Gender::Female => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Female")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            Gender::Unknown => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Unknown")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub enum PrivacyLevel {
    Public,
    Restricted,
    Private,
}
#[automatically_derived]
impl ::core::clone::Clone for PrivacyLevel {
    #[inline]
    fn clone(&self) -> PrivacyLevel {
        match self {
            PrivacyLevel::Public => PrivacyLevel::Public,
            PrivacyLevel::Restricted => PrivacyLevel::Restricted,
            PrivacyLevel::Private => PrivacyLevel::Private,
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for PrivacyLevel {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                PrivacyLevel::Public => "Public",
                PrivacyLevel::Restricted => "Restricted",
                PrivacyLevel::Private => "Private",
            },
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PrivacyLevel {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PrivacyLevel {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PrivacyLevel {
    #[inline]
    fn eq(&self, other: &PrivacyLevel) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
pub static __SPEC_XDR_TYPE_PRIVACYLEVEL: [u8; 96usize] = PrivacyLevel::spec_xdr();
impl PrivacyLevel {
    pub const fn spec_xdr() -> [u8; 96usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\x0cPrivacyLevel\0\0\0\x03\0\0\0\0\0\0\0\0\0\0\0\x06Public\0\0\0\0\0\0\0\0\0\0\0\0\0\nRestricted\0\0\0\0\0\0\0\0\0\0\0\0\0\x07Private\0"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for PrivacyLevel {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &["Public", "Restricted", "Private"];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Public
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Restricted
                }
                2 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Private
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, PrivacyLevel> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &PrivacyLevel,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            PrivacyLevel::Public => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Public")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            PrivacyLevel::Restricted => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Restricted")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            PrivacyLevel::Private => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Private")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub enum AccessAction {
    Read,
    Write,
    Grant,
    Revoke,
}
#[automatically_derived]
impl ::core::clone::Clone for AccessAction {
    #[inline]
    fn clone(&self) -> AccessAction {
        match self {
            AccessAction::Read => AccessAction::Read,
            AccessAction::Write => AccessAction::Write,
            AccessAction::Grant => AccessAction::Grant,
            AccessAction::Revoke => AccessAction::Revoke,
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for AccessAction {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                AccessAction::Read => "Read",
                AccessAction::Write => "Write",
                AccessAction::Grant => "Grant",
                AccessAction::Revoke => "Revoke",
            },
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for AccessAction {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for AccessAction {}
#[automatically_derived]
impl ::core::cmp::PartialEq for AccessAction {
    #[inline]
    fn eq(&self, other: &AccessAction) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
pub static __SPEC_XDR_TYPE_ACCESSACTION: [u8; 108usize] = AccessAction::spec_xdr();
impl AccessAction {
    pub const fn spec_xdr() -> [u8; 108usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\x0cAccessAction\0\0\0\x04\0\0\0\0\0\0\0\0\0\0\0\x04Read\0\0\0\0\0\0\0\0\0\0\0\x05Write\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x05Grant\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x06Revoke\0\0"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for AccessAction {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &["Read", "Write", "Grant", "Revoke"];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Read
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Write
                }
                2 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Grant
                }
                3 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Revoke
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, AccessAction> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &AccessAction,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            AccessAction::Read => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Read")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            AccessAction::Write => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Write")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            AccessAction::Grant => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Grant")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            AccessAction::Revoke => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Revoke")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub struct AccessLog {
    pub id: u64,
    pub pet_id: u64,
    pub user: Address,
    pub action: AccessAction,
    pub timestamp: u64,
    pub details: String,
}
#[automatically_derived]
impl ::core::clone::Clone for AccessLog {
    #[inline]
    fn clone(&self) -> AccessLog {
        AccessLog {
            id: ::core::clone::Clone::clone(&self.id),
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            user: ::core::clone::Clone::clone(&self.user),
            action: ::core::clone::Clone::clone(&self.action),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
            details: ::core::clone::Clone::clone(&self.details),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for AccessLog {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "id",
            "pet_id",
            "user",
            "action",
            "timestamp",
            "details",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.id,
            &self.pet_id,
            &self.user,
            &self.action,
            &self.timestamp,
            &&self.details,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(f, "AccessLog", names, values)
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for AccessLog {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<Address>;
        let _: ::core::cmp::AssertParamIsEq<AccessAction>;
        let _: ::core::cmp::AssertParamIsEq<String>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for AccessLog {}
#[automatically_derived]
impl ::core::cmp::PartialEq for AccessLog {
    #[inline]
    fn eq(&self, other: &AccessLog) -> bool {
        self.id == other.id && self.pet_id == other.pet_id
            && self.timestamp == other.timestamp && self.user == other.user
            && self.action == other.action && self.details == other.details
    }
}
pub static __SPEC_XDR_TYPE_ACCESSLOG: [u8; 164usize] = AccessLog::spec_xdr();
impl AccessLog {
    pub const fn spec_xdr() -> [u8; 164usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\tAccessLog\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\x06action\0\0\0\0\x07\xd0\0\0\0\x0cAccessAction\0\0\0\0\0\0\0\x07details\0\0\0\0\x10\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\ttimestamp\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\x04user\0\0\0\x13"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for AccessLog {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 6usize] = [
            "action",
            "details",
            "id",
            "pet_id",
            "timestamp",
            "user",
        ];
        let mut vals: [Val; 6usize] = [Val::VOID.to_val(); 6usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            action: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            details: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            id: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[3].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            timestamp: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            user: vals[5].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, AccessLog> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &AccessLog,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 6usize] = [
            "action",
            "details",
            "id",
            "pet_id",
            "timestamp",
            "user",
        ];
        let vals: [Val; 6usize] = [
            (&val.action).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.details).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.timestamp).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.user).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct EmergencyContactInfo {
    pub name: String,
    pub phone: String,
    pub relationship: String,
}
#[automatically_derived]
impl ::core::clone::Clone for EmergencyContactInfo {
    #[inline]
    fn clone(&self) -> EmergencyContactInfo {
        EmergencyContactInfo {
            name: ::core::clone::Clone::clone(&self.name),
            phone: ::core::clone::Clone::clone(&self.phone),
            relationship: ::core::clone::Clone::clone(&self.relationship),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for EmergencyContactInfo {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "EmergencyContactInfo",
            "name",
            &self.name,
            "phone",
            &self.phone,
            "relationship",
            &&self.relationship,
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for EmergencyContactInfo {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<String>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for EmergencyContactInfo {}
#[automatically_derived]
impl ::core::cmp::PartialEq for EmergencyContactInfo {
    #[inline]
    fn eq(&self, other: &EmergencyContactInfo) -> bool {
        self.name == other.name && self.phone == other.phone
            && self.relationship == other.relationship
    }
}
pub static __SPEC_XDR_TYPE_EMERGENCYCONTACTINFO: [u8; 100usize] = EmergencyContactInfo::spec_xdr();
impl EmergencyContactInfo {
    pub const fn spec_xdr() -> [u8; 100usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x14EmergencyContactInfo\0\0\0\x03\0\0\0\0\0\0\0\x04name\0\0\0\x10\0\0\0\0\0\0\0\x05phone\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x0crelationship\0\0\0\x10"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>
for EmergencyContactInfo {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 3usize] = ["name", "phone", "relationship"];
        let mut vals: [Val; 3usize] = [Val::VOID.to_val(); 3usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            name: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            phone: vals[1].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            relationship: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, EmergencyContactInfo>
for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &EmergencyContactInfo,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 3usize] = ["name", "phone", "relationship"];
        let vals: [Val; 3usize] = [
            (&val.name).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.phone).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.relationship).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct EmergencyContact {
    pub name: String,
    pub phone: String,
    pub email: String,
    pub relationship: String,
    pub is_primary: bool,
}
#[automatically_derived]
impl ::core::clone::Clone for EmergencyContact {
    #[inline]
    fn clone(&self) -> EmergencyContact {
        EmergencyContact {
            name: ::core::clone::Clone::clone(&self.name),
            phone: ::core::clone::Clone::clone(&self.phone),
            email: ::core::clone::Clone::clone(&self.email),
            relationship: ::core::clone::Clone::clone(&self.relationship),
            is_primary: ::core::clone::Clone::clone(&self.is_primary),
        }
    }
}
pub static __SPEC_XDR_TYPE_EMERGENCYCONTACT: [u8; 140usize] = EmergencyContact::spec_xdr();
impl EmergencyContact {
    pub const fn spec_xdr() -> [u8; 140usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x10EmergencyContact\0\0\0\x05\0\0\0\0\0\0\0\x05email\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\nis_primary\0\0\0\0\0\x01\0\0\0\0\0\0\0\x04name\0\0\0\x10\0\0\0\0\0\0\0\x05phone\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x0crelationship\0\0\0\x10"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for EmergencyContact {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 5usize] = [
            "email",
            "is_primary",
            "name",
            "phone",
            "relationship",
        ];
        let mut vals: [Val; 5usize] = [Val::VOID.to_val(); 5usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            email: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            is_primary: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            name: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            phone: vals[3].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            relationship: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, EmergencyContact> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &EmergencyContact,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 5usize] = [
            "email",
            "is_primary",
            "name",
            "phone",
            "relationship",
        ];
        let vals: [Val; 5usize] = [
            (&val.email).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.is_primary).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.name).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.phone).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.relationship).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct EncryptedData {
    pub nonce: Bytes,
    pub ciphertext: Bytes,
}
#[automatically_derived]
impl ::core::clone::Clone for EncryptedData {
    #[inline]
    fn clone(&self) -> EncryptedData {
        EncryptedData {
            nonce: ::core::clone::Clone::clone(&self.nonce),
            ciphertext: ::core::clone::Clone::clone(&self.ciphertext),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for EncryptedData {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "EncryptedData",
            "nonce",
            &self.nonce,
            "ciphertext",
            &&self.ciphertext,
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for EncryptedData {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<Bytes>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for EncryptedData {}
#[automatically_derived]
impl ::core::cmp::PartialEq for EncryptedData {
    #[inline]
    fn eq(&self, other: &EncryptedData) -> bool {
        self.nonce == other.nonce && self.ciphertext == other.ciphertext
    }
}
pub static __SPEC_XDR_TYPE_ENCRYPTEDDATA: [u8; 80usize] = EncryptedData::spec_xdr();
impl EncryptedData {
    pub const fn spec_xdr() -> [u8; 80usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\rEncryptedData\0\0\0\0\0\0\x02\0\0\0\0\0\0\0\nciphertext\0\0\0\0\0\x0e\0\0\0\0\0\0\0\x05nonce\0\0\0\0\0\0\x0e"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for EncryptedData {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 2usize] = ["ciphertext", "nonce"];
        let mut vals: [Val; 2usize] = [Val::VOID.to_val(); 2usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            ciphertext: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            nonce: vals[1].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, EncryptedData> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &EncryptedData,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 2usize] = ["ciphertext", "nonce"];
        let vals: [Val; 2usize] = [
            (&val.ciphertext).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.nonce).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct Pet {
    pub id: u64,
    pub owner: Address,
    pub privacy_level: PrivacyLevel,
    pub encrypted_name: EncryptedData,
    pub encrypted_birthday: EncryptedData,
    pub encrypted_breed: EncryptedData,
    pub encrypted_emergency_contacts: EncryptedData,
    pub encrypted_medical_alerts: EncryptedData,
    pub name: String,
    pub birthday: String,
    pub breed: String,
    pub emergency_contacts: Vec<EmergencyContact>,
    pub medical_alerts: String,
    pub active: bool,
    pub created_at: u64,
    pub updated_at: u64,
    pub new_owner: Address,
    pub species: Species,
    pub gender: Gender,
    pub color: String,
    pub weight: u32,
    pub microchip_id: Option<String>,
    pub photo_hashes: Vec<String>,
}
#[automatically_derived]
impl ::core::clone::Clone for Pet {
    #[inline]
    fn clone(&self) -> Pet {
        Pet {
            id: ::core::clone::Clone::clone(&self.id),
            owner: ::core::clone::Clone::clone(&self.owner),
            privacy_level: ::core::clone::Clone::clone(&self.privacy_level),
            encrypted_name: ::core::clone::Clone::clone(&self.encrypted_name),
            encrypted_birthday: ::core::clone::Clone::clone(&self.encrypted_birthday),
            encrypted_breed: ::core::clone::Clone::clone(&self.encrypted_breed),
            encrypted_emergency_contacts: ::core::clone::Clone::clone(
                &self.encrypted_emergency_contacts,
            ),
            encrypted_medical_alerts: ::core::clone::Clone::clone(
                &self.encrypted_medical_alerts,
            ),
            name: ::core::clone::Clone::clone(&self.name),
            birthday: ::core::clone::Clone::clone(&self.birthday),
            breed: ::core::clone::Clone::clone(&self.breed),
            emergency_contacts: ::core::clone::Clone::clone(&self.emergency_contacts),
            medical_alerts: ::core::clone::Clone::clone(&self.medical_alerts),
            active: ::core::clone::Clone::clone(&self.active),
            created_at: ::core::clone::Clone::clone(&self.created_at),
            updated_at: ::core::clone::Clone::clone(&self.updated_at),
            new_owner: ::core::clone::Clone::clone(&self.new_owner),
            species: ::core::clone::Clone::clone(&self.species),
            gender: ::core::clone::Clone::clone(&self.gender),
            color: ::core::clone::Clone::clone(&self.color),
            weight: ::core::clone::Clone::clone(&self.weight),
            microchip_id: ::core::clone::Clone::clone(&self.microchip_id),
            photo_hashes: ::core::clone::Clone::clone(&self.photo_hashes),
        }
    }
}
pub static __SPEC_XDR_TYPE_PET: [u8; 760usize] = Pet::spec_xdr();
impl Pet {
    pub const fn spec_xdr() -> [u8; 760usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x03Pet\0\0\0\0\x17\0\0\0\0\0\0\0\x06active\0\0\0\0\0\x01\0\0\0\0\0\0\0\x08birthday\0\0\0\x10\0\0\0\0\0\0\0\x05breed\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x05color\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\ncreated_at\0\0\0\0\0\x06\0\0\0\0\0\0\0\x12emergency_contacts\0\0\0\0\x03\xea\0\0\x07\xd0\0\0\0\x10EmergencyContact\0\0\0\0\0\0\0\x12encrypted_birthday\0\0\0\0\x07\xd0\0\0\0\rEncryptedData\0\0\0\0\0\0\0\0\0\0\x0fencrypted_breed\0\0\0\x07\xd0\0\0\0\rEncryptedData\0\0\0\0\0\0\0\0\0\0\x1cencrypted_emergency_contacts\0\0\x07\xd0\0\0\0\rEncryptedData\0\0\0\0\0\0\0\0\0\0\x18encrypted_medical_alerts\0\0\x07\xd0\0\0\0\rEncryptedData\0\0\0\0\0\0\0\0\0\0\x0eencrypted_name\0\0\0\0\x07\xd0\0\0\0\rEncryptedData\0\0\0\0\0\0\0\0\0\0\x06gender\0\0\0\0\x07\xd0\0\0\0\x06Gender\0\0\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0emedical_alerts\0\0\0\0\0\x10\0\0\0\0\0\0\0\x0cmicrochip_id\0\0\x03\xe8\0\0\0\x10\0\0\0\0\0\0\0\x04name\0\0\0\x10\0\0\0\0\0\0\0\tnew_owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x05owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x0cphoto_hashes\0\0\x03\xea\0\0\0\x10\0\0\0\0\0\0\0\rprivacy_level\0\0\0\0\0\x07\xd0\0\0\0\x0cPrivacyLevel\0\0\0\0\0\0\0\x07species\0\0\0\x07\xd0\0\0\0\x07Species\0\0\0\0\0\0\0\0\nupdated_at\0\0\0\0\0\x06\0\0\0\0\0\0\0\x06weight\0\0\0\0\0\x04"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for Pet {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 23usize] = [
            "active",
            "birthday",
            "breed",
            "color",
            "created_at",
            "emergency_contacts",
            "encrypted_birthday",
            "encrypted_breed",
            "encrypted_emergency_contacts",
            "encrypted_medical_alerts",
            "encrypted_name",
            "gender",
            "id",
            "medical_alerts",
            "microchip_id",
            "name",
            "new_owner",
            "owner",
            "photo_hashes",
            "privacy_level",
            "species",
            "updated_at",
            "weight",
        ];
        let mut vals: [Val; 23usize] = [Val::VOID.to_val(); 23usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            active: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            birthday: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            breed: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            color: vals[3].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            created_at: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            emergency_contacts: vals[5]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            encrypted_birthday: vals[6]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            encrypted_breed: vals[7]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            encrypted_emergency_contacts: vals[8]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            encrypted_medical_alerts: vals[9]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            encrypted_name: vals[10]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            gender: vals[11]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            id: vals[12].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            medical_alerts: vals[13]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            microchip_id: vals[14]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            name: vals[15].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            new_owner: vals[16]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            owner: vals[17].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            photo_hashes: vals[18]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            privacy_level: vals[19]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            species: vals[20]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            updated_at: vals[21]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            weight: vals[22].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, Pet> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &Pet,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 23usize] = [
            "active",
            "birthday",
            "breed",
            "color",
            "created_at",
            "emergency_contacts",
            "encrypted_birthday",
            "encrypted_breed",
            "encrypted_emergency_contacts",
            "encrypted_medical_alerts",
            "encrypted_name",
            "gender",
            "id",
            "medical_alerts",
            "microchip_id",
            "name",
            "new_owner",
            "owner",
            "photo_hashes",
            "privacy_level",
            "species",
            "updated_at",
            "weight",
        ];
        let vals: [Val; 23usize] = [
            (&val.active).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.birthday).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.breed).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.color).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.created_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.emergency_contacts).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.encrypted_birthday).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.encrypted_breed).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.encrypted_emergency_contacts)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.encrypted_medical_alerts)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.encrypted_name).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.gender).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.medical_alerts).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.microchip_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.name).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.new_owner).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.owner).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.photo_hashes).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.privacy_level).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.species).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.updated_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.weight).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct PetProfile {
    pub id: u64,
    pub owner: Address,
    pub privacy_level: PrivacyLevel,
    pub name: String,
    pub birthday: String,
    pub active: bool,
    pub created_at: u64,
    pub updated_at: u64,
    pub new_owner: Address,
    pub species: Species,
    pub gender: Gender,
    pub breed: String,
    pub color: String,
    pub weight: u32,
    pub microchip_id: Option<String>,
}
#[automatically_derived]
impl ::core::clone::Clone for PetProfile {
    #[inline]
    fn clone(&self) -> PetProfile {
        PetProfile {
            id: ::core::clone::Clone::clone(&self.id),
            owner: ::core::clone::Clone::clone(&self.owner),
            privacy_level: ::core::clone::Clone::clone(&self.privacy_level),
            name: ::core::clone::Clone::clone(&self.name),
            birthday: ::core::clone::Clone::clone(&self.birthday),
            active: ::core::clone::Clone::clone(&self.active),
            created_at: ::core::clone::Clone::clone(&self.created_at),
            updated_at: ::core::clone::Clone::clone(&self.updated_at),
            new_owner: ::core::clone::Clone::clone(&self.new_owner),
            species: ::core::clone::Clone::clone(&self.species),
            gender: ::core::clone::Clone::clone(&self.gender),
            breed: ::core::clone::Clone::clone(&self.breed),
            color: ::core::clone::Clone::clone(&self.color),
            weight: ::core::clone::Clone::clone(&self.weight),
            microchip_id: ::core::clone::Clone::clone(&self.microchip_id),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for PetProfile {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "id",
            "owner",
            "privacy_level",
            "name",
            "birthday",
            "active",
            "created_at",
            "updated_at",
            "new_owner",
            "species",
            "gender",
            "breed",
            "color",
            "weight",
            "microchip_id",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.id,
            &self.owner,
            &self.privacy_level,
            &self.name,
            &self.birthday,
            &self.active,
            &self.created_at,
            &self.updated_at,
            &self.new_owner,
            &self.species,
            &self.gender,
            &self.breed,
            &self.color,
            &self.weight,
            &&self.microchip_id,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(
            f,
            "PetProfile",
            names,
            values,
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PetProfile {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<Address>;
        let _: ::core::cmp::AssertParamIsEq<PrivacyLevel>;
        let _: ::core::cmp::AssertParamIsEq<String>;
        let _: ::core::cmp::AssertParamIsEq<bool>;
        let _: ::core::cmp::AssertParamIsEq<Species>;
        let _: ::core::cmp::AssertParamIsEq<Gender>;
        let _: ::core::cmp::AssertParamIsEq<u32>;
        let _: ::core::cmp::AssertParamIsEq<Option<String>>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PetProfile {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PetProfile {
    #[inline]
    fn eq(&self, other: &PetProfile) -> bool {
        self.id == other.id && self.active == other.active
            && self.created_at == other.created_at && self.updated_at == other.updated_at
            && self.weight == other.weight && self.owner == other.owner
            && self.privacy_level == other.privacy_level && self.name == other.name
            && self.birthday == other.birthday && self.new_owner == other.new_owner
            && self.species == other.species && self.gender == other.gender
            && self.breed == other.breed && self.color == other.color
            && self.microchip_id == other.microchip_id
    }
}
pub static __SPEC_XDR_TYPE_PETPROFILE: [u8; 392usize] = PetProfile::spec_xdr();
impl PetProfile {
    pub const fn spec_xdr() -> [u8; 392usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\nPetProfile\0\0\0\0\0\x0f\0\0\0\0\0\0\0\x06active\0\0\0\0\0\x01\0\0\0\0\0\0\0\x08birthday\0\0\0\x10\0\0\0\0\0\0\0\x05breed\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x05color\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\ncreated_at\0\0\0\0\0\x06\0\0\0\0\0\0\0\x06gender\0\0\0\0\x07\xd0\0\0\0\x06Gender\0\0\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0cmicrochip_id\0\0\x03\xe8\0\0\0\x10\0\0\0\0\0\0\0\x04name\0\0\0\x10\0\0\0\0\0\0\0\tnew_owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x05owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\rprivacy_level\0\0\0\0\0\x07\xd0\0\0\0\x0cPrivacyLevel\0\0\0\0\0\0\0\x07species\0\0\0\x07\xd0\0\0\0\x07Species\0\0\0\0\0\0\0\0\nupdated_at\0\0\0\0\0\x06\0\0\0\0\0\0\0\x06weight\0\0\0\0\0\x04"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for PetProfile {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 15usize] = [
            "active",
            "birthday",
            "breed",
            "color",
            "created_at",
            "gender",
            "id",
            "microchip_id",
            "name",
            "new_owner",
            "owner",
            "privacy_level",
            "species",
            "updated_at",
            "weight",
        ];
        let mut vals: [Val; 15usize] = [Val::VOID.to_val(); 15usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            active: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            birthday: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            breed: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            color: vals[3].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            created_at: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            gender: vals[5].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            id: vals[6].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            microchip_id: vals[7]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            name: vals[8].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            new_owner: vals[9]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            owner: vals[10].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            privacy_level: vals[11]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            species: vals[12]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            updated_at: vals[13]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            weight: vals[14].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, PetProfile> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &PetProfile,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 15usize] = [
            "active",
            "birthday",
            "breed",
            "color",
            "created_at",
            "gender",
            "id",
            "microchip_id",
            "name",
            "new_owner",
            "owner",
            "privacy_level",
            "species",
            "updated_at",
            "weight",
        ];
        let vals: [Val; 15usize] = [
            (&val.active).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.birthday).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.breed).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.color).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.created_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.gender).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.microchip_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.name).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.new_owner).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.owner).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.privacy_level).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.species).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.updated_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.weight).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct PetOwner {
    pub owner_address: Address,
    pub privacy_level: PrivacyLevel,
    pub encrypted_name: EncryptedData,
    pub encrypted_email: EncryptedData,
    pub encrypted_emergency_contact: EncryptedData,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_pet_owner: bool,
}
#[automatically_derived]
impl ::core::clone::Clone for PetOwner {
    #[inline]
    fn clone(&self) -> PetOwner {
        PetOwner {
            owner_address: ::core::clone::Clone::clone(&self.owner_address),
            privacy_level: ::core::clone::Clone::clone(&self.privacy_level),
            encrypted_name: ::core::clone::Clone::clone(&self.encrypted_name),
            encrypted_email: ::core::clone::Clone::clone(&self.encrypted_email),
            encrypted_emergency_contact: ::core::clone::Clone::clone(
                &self.encrypted_emergency_contact,
            ),
            created_at: ::core::clone::Clone::clone(&self.created_at),
            updated_at: ::core::clone::Clone::clone(&self.updated_at),
            is_pet_owner: ::core::clone::Clone::clone(&self.is_pet_owner),
        }
    }
}
pub static __SPEC_XDR_TYPE_PETOWNER: [u8; 328usize] = PetOwner::spec_xdr();
impl PetOwner {
    pub const fn spec_xdr() -> [u8; 328usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x08PetOwner\0\0\0\x08\0\0\0\0\0\0\0\ncreated_at\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0fencrypted_email\0\0\0\x07\xd0\0\0\0\rEncryptedData\0\0\0\0\0\0\0\0\0\0\x1bencrypted_emergency_contact\0\0\0\x07\xd0\0\0\0\rEncryptedData\0\0\0\0\0\0\0\0\0\0\x0eencrypted_name\0\0\0\0\x07\xd0\0\0\0\rEncryptedData\0\0\0\0\0\0\0\0\0\0\x0cis_pet_owner\0\0\0\x01\0\0\0\0\0\0\0\rowner_address\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\rprivacy_level\0\0\0\0\0\x07\xd0\0\0\0\x0cPrivacyLevel\0\0\0\0\0\0\0\nupdated_at\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for PetOwner {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 8usize] = [
            "created_at",
            "encrypted_email",
            "encrypted_emergency_contact",
            "encrypted_name",
            "is_pet_owner",
            "owner_address",
            "privacy_level",
            "updated_at",
        ];
        let mut vals: [Val; 8usize] = [Val::VOID.to_val(); 8usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            created_at: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            encrypted_email: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            encrypted_emergency_contact: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            encrypted_name: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            is_pet_owner: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            owner_address: vals[5]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            privacy_level: vals[6]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            updated_at: vals[7]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, PetOwner> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &PetOwner,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 8usize] = [
            "created_at",
            "encrypted_email",
            "encrypted_emergency_contact",
            "encrypted_name",
            "is_pet_owner",
            "owner_address",
            "privacy_level",
            "updated_at",
        ];
        let vals: [Val; 8usize] = [
            (&val.created_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.encrypted_email).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.encrypted_emergency_contact)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.encrypted_name).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.is_pet_owner).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.owner_address).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.privacy_level).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.updated_at).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct ClinicInfo {
    pub clinic_name: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub operating_hours: String,
    pub emergency_available: bool,
}
#[automatically_derived]
impl ::core::clone::Clone for ClinicInfo {
    #[inline]
    fn clone(&self) -> ClinicInfo {
        ClinicInfo {
            clinic_name: ::core::clone::Clone::clone(&self.clinic_name),
            address: ::core::clone::Clone::clone(&self.address),
            phone: ::core::clone::Clone::clone(&self.phone),
            email: ::core::clone::Clone::clone(&self.email),
            operating_hours: ::core::clone::Clone::clone(&self.operating_hours),
            emergency_available: ::core::clone::Clone::clone(&self.emergency_available),
        }
    }
}
pub static __SPEC_XDR_TYPE_CLINICINFO: [u8; 176usize] = ClinicInfo::spec_xdr();
impl ClinicInfo {
    pub const fn spec_xdr() -> [u8; 176usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\nClinicInfo\0\0\0\0\0\x06\0\0\0\0\0\0\0\x07address\0\0\0\0\x10\0\0\0\0\0\0\0\x0bclinic_name\0\0\0\0\x10\0\0\0\0\0\0\0\x05email\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x13emergency_available\0\0\0\0\x01\0\0\0\0\0\0\0\x0foperating_hours\0\0\0\0\x10\0\0\0\0\0\0\0\x05phone\0\0\0\0\0\0\x10"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for ClinicInfo {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 6usize] = [
            "address",
            "clinic_name",
            "email",
            "emergency_available",
            "operating_hours",
            "phone",
        ];
        let mut vals: [Val; 6usize] = [Val::VOID.to_val(); 6usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            address: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            clinic_name: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            email: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            emergency_available: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            operating_hours: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            phone: vals[5].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, ClinicInfo> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &ClinicInfo,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 6usize] = [
            "address",
            "clinic_name",
            "email",
            "emergency_available",
            "operating_hours",
            "phone",
        ];
        let vals: [Val; 6usize] = [
            (&val.address).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.clinic_name).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.email).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.emergency_available).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.operating_hours).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.phone).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct Vet {
    pub address: Address,
    pub name: String,
    pub license_number: String,
    pub specialization: String,
    pub verified: bool,
    pub clinic_info: Option<ClinicInfo>,
}
#[automatically_derived]
impl ::core::clone::Clone for Vet {
    #[inline]
    fn clone(&self) -> Vet {
        Vet {
            address: ::core::clone::Clone::clone(&self.address),
            name: ::core::clone::Clone::clone(&self.name),
            license_number: ::core::clone::Clone::clone(&self.license_number),
            specialization: ::core::clone::Clone::clone(&self.specialization),
            verified: ::core::clone::Clone::clone(&self.verified),
            clinic_info: ::core::clone::Clone::clone(&self.clinic_info),
        }
    }
}
pub static __SPEC_XDR_TYPE_VET: [u8; 180usize] = Vet::spec_xdr();
impl Vet {
    pub const fn spec_xdr() -> [u8; 180usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x03Vet\0\0\0\0\x06\0\0\0\0\0\0\0\x07address\0\0\0\0\x13\0\0\0\0\0\0\0\x0bclinic_info\0\0\0\x03\xe8\0\0\x07\xd0\0\0\0\nClinicInfo\0\0\0\0\0\0\0\0\0\x0elicense_number\0\0\0\0\0\x10\0\0\0\0\0\0\0\x04name\0\0\0\x10\0\0\0\0\0\0\0\x0especialization\0\0\0\0\0\x10\0\0\0\0\0\0\0\x08verified\0\0\0\x01"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for Vet {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 6usize] = [
            "address",
            "clinic_info",
            "license_number",
            "name",
            "specialization",
            "verified",
        ];
        let mut vals: [Val; 6usize] = [Val::VOID.to_val(); 6usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            address: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            clinic_info: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            license_number: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            name: vals[3].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            specialization: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            verified: vals[5]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, Vet> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &Vet,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 6usize] = [
            "address",
            "clinic_info",
            "license_number",
            "name",
            "specialization",
            "verified",
        ];
        let vals: [Val; 6usize] = [
            (&val.address).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.clinic_info).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.license_number).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.name).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.specialization).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.verified).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub enum VaccineType {
    Rabies,
    Parvovirus,
    Leukemia,
    Bordetella,
    Other,
}
#[automatically_derived]
impl ::core::clone::Clone for VaccineType {
    #[inline]
    fn clone(&self) -> VaccineType {
        match self {
            VaccineType::Rabies => VaccineType::Rabies,
            VaccineType::Parvovirus => VaccineType::Parvovirus,
            VaccineType::Leukemia => VaccineType::Leukemia,
            VaccineType::Bordetella => VaccineType::Bordetella,
            VaccineType::Other => VaccineType::Other,
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for VaccineType {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                VaccineType::Rabies => "Rabies",
                VaccineType::Parvovirus => "Parvovirus",
                VaccineType::Leukemia => "Leukemia",
                VaccineType::Bordetella => "Bordetella",
                VaccineType::Other => "Other",
            },
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for VaccineType {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for VaccineType {}
#[automatically_derived]
impl ::core::cmp::PartialEq for VaccineType {
    #[inline]
    fn eq(&self, other: &VaccineType) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
pub static __SPEC_XDR_TYPE_VACCINETYPE: [u8; 140usize] = VaccineType::spec_xdr();
impl VaccineType {
    pub const fn spec_xdr() -> [u8; 140usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\x0bVaccineType\0\0\0\0\x05\0\0\0\0\0\0\0\0\0\0\0\x06Rabies\0\0\0\0\0\0\0\0\0\0\0\0\0\nParvovirus\0\0\0\0\0\0\0\0\0\0\0\0\0\x08Leukemia\0\0\0\0\0\0\0\0\0\0\0\nBordetella\0\0\0\0\0\0\0\0\0\0\0\0\0\x05Other\0\0\0"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for VaccineType {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &[
            "Rabies",
            "Parvovirus",
            "Leukemia",
            "Bordetella",
            "Other",
        ];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Rabies
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Parvovirus
                }
                2 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Leukemia
                }
                3 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Bordetella
                }
                4 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Other
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, VaccineType> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &VaccineType,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            VaccineType::Rabies => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Rabies")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            VaccineType::Parvovirus => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Parvovirus")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            VaccineType::Leukemia => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Leukemia")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            VaccineType::Bordetella => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Bordetella")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            VaccineType::Other => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Other")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub struct Vaccination {
    pub id: u64,
    pub pet_id: u64,
    pub veterinarian: Address,
    pub vaccine_type: VaccineType,
    pub vaccine_name: Option<String>,
    pub encrypted_vaccine_name: EncryptedData,
    pub administered_at: u64,
    pub next_due_date: u64,
    pub batch_number: Option<String>,
    pub encrypted_batch_number: EncryptedData,
    pub created_at: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for Vaccination {
    #[inline]
    fn clone(&self) -> Vaccination {
        Vaccination {
            id: ::core::clone::Clone::clone(&self.id),
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            veterinarian: ::core::clone::Clone::clone(&self.veterinarian),
            vaccine_type: ::core::clone::Clone::clone(&self.vaccine_type),
            vaccine_name: ::core::clone::Clone::clone(&self.vaccine_name),
            encrypted_vaccine_name: ::core::clone::Clone::clone(
                &self.encrypted_vaccine_name,
            ),
            administered_at: ::core::clone::Clone::clone(&self.administered_at),
            next_due_date: ::core::clone::Clone::clone(&self.next_due_date),
            batch_number: ::core::clone::Clone::clone(&self.batch_number),
            encrypted_batch_number: ::core::clone::Clone::clone(
                &self.encrypted_batch_number,
            ),
            created_at: ::core::clone::Clone::clone(&self.created_at),
        }
    }
}
pub static __SPEC_XDR_TYPE_VACCINATION: [u8; 380usize] = Vaccination::spec_xdr();
impl Vaccination {
    pub const fn spec_xdr() -> [u8; 380usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x0bVaccination\0\0\0\0\x0b\0\0\0\0\0\0\0\x0fadministered_at\0\0\0\0\x06\0\0\0\0\0\0\0\x0cbatch_number\0\0\x03\xe8\0\0\0\x10\0\0\0\0\0\0\0\ncreated_at\0\0\0\0\0\x06\0\0\0\0\0\0\0\x16encrypted_batch_number\0\0\0\0\x07\xd0\0\0\0\rEncryptedData\0\0\0\0\0\0\0\0\0\0\x16encrypted_vaccine_name\0\0\0\0\x07\xd0\0\0\0\rEncryptedData\0\0\0\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0\0\0\0\rnext_due_date\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0cvaccine_name\0\0\x03\xe8\0\0\0\x10\0\0\0\0\0\0\0\x0cvaccine_type\0\0\x07\xd0\0\0\0\x0bVaccineType\0\0\0\0\0\0\0\0\x0cveterinarian\0\0\0\x13"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for Vaccination {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 11usize] = [
            "administered_at",
            "batch_number",
            "created_at",
            "encrypted_batch_number",
            "encrypted_vaccine_name",
            "id",
            "next_due_date",
            "pet_id",
            "vaccine_name",
            "vaccine_type",
            "veterinarian",
        ];
        let mut vals: [Val; 11usize] = [Val::VOID.to_val(); 11usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            administered_at: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            batch_number: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            created_at: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            encrypted_batch_number: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            encrypted_vaccine_name: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            id: vals[5].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            next_due_date: vals[6]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[7].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            vaccine_name: vals[8]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            vaccine_type: vals[9]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            veterinarian: vals[10]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, Vaccination> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &Vaccination,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 11usize] = [
            "administered_at",
            "batch_number",
            "created_at",
            "encrypted_batch_number",
            "encrypted_vaccine_name",
            "id",
            "next_due_date",
            "pet_id",
            "vaccine_name",
            "vaccine_type",
            "veterinarian",
        ];
        let vals: [Val; 11usize] = [
            (&val.administered_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.batch_number).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.created_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.encrypted_batch_number)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.encrypted_vaccine_name)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.next_due_date).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.vaccine_name).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.vaccine_type).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.veterinarian).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct TagLinkedEvent {
    pub tag_id: BytesN<32>,
    pub pet_id: u64,
    pub owner: Address,
    pub timestamp: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for TagLinkedEvent {
    #[inline]
    fn clone(&self) -> TagLinkedEvent {
        TagLinkedEvent {
            tag_id: ::core::clone::Clone::clone(&self.tag_id),
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            owner: ::core::clone::Clone::clone(&self.owner),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
        }
    }
}
pub static __SPEC_XDR_TYPE_TAGLINKEDEVENT: [u8; 124usize] = TagLinkedEvent::spec_xdr();
impl TagLinkedEvent {
    pub const fn spec_xdr() -> [u8; 124usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x0eTagLinkedEvent\0\0\0\0\0\x04\0\0\0\0\0\0\0\x05owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x06tag_id\0\0\0\0\x03\xee\0\0\0 \0\0\0\0\0\0\0\ttimestamp\0\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for TagLinkedEvent {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 4usize] = ["owner", "pet_id", "tag_id", "timestamp"];
        let mut vals: [Val; 4usize] = [Val::VOID.to_val(); 4usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            owner: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[1].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            tag_id: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            timestamp: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, TagLinkedEvent> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &TagLinkedEvent,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 4usize] = ["owner", "pet_id", "tag_id", "timestamp"];
        let vals: [Val; 4usize] = [
            (&val.owner).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.tag_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.timestamp).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct TagDeactivatedEvent {
    pub tag_id: BytesN<32>,
    pub pet_id: u64,
    pub deactivated_by: Address,
    pub timestamp: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for TagDeactivatedEvent {
    #[inline]
    fn clone(&self) -> TagDeactivatedEvent {
        TagDeactivatedEvent {
            tag_id: ::core::clone::Clone::clone(&self.tag_id),
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            deactivated_by: ::core::clone::Clone::clone(&self.deactivated_by),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
        }
    }
}
pub static __SPEC_XDR_TYPE_TAGDEACTIVATEDEVENT: [u8; 136usize] = TagDeactivatedEvent::spec_xdr();
impl TagDeactivatedEvent {
    pub const fn spec_xdr() -> [u8; 136usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x13TagDeactivatedEvent\0\0\0\0\x04\0\0\0\0\0\0\0\x0edeactivated_by\0\0\0\0\0\x13\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x06tag_id\0\0\0\0\x03\xee\0\0\0 \0\0\0\0\0\0\0\ttimestamp\0\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>
for TagDeactivatedEvent {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 4usize] = [
            "deactivated_by",
            "pet_id",
            "tag_id",
            "timestamp",
        ];
        let mut vals: [Val; 4usize] = [Val::VOID.to_val(); 4usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            deactivated_by: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[1].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            tag_id: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            timestamp: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, TagDeactivatedEvent>
for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &TagDeactivatedEvent,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 4usize] = [
            "deactivated_by",
            "pet_id",
            "tag_id",
            "timestamp",
        ];
        let vals: [Val; 4usize] = [
            (&val.deactivated_by).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.tag_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.timestamp).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct ContractVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
#[automatically_derived]
impl ::core::clone::Clone for ContractVersion {
    #[inline]
    fn clone(&self) -> ContractVersion {
        ContractVersion {
            major: ::core::clone::Clone::clone(&self.major),
            minor: ::core::clone::Clone::clone(&self.minor),
            patch: ::core::clone::Clone::clone(&self.patch),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for ContractVersion {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "ContractVersion",
            "major",
            &self.major,
            "minor",
            &self.minor,
            "patch",
            &&self.patch,
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ContractVersion {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u32>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ContractVersion {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ContractVersion {
    #[inline]
    fn eq(&self, other: &ContractVersion) -> bool {
        self.major == other.major && self.minor == other.minor
            && self.patch == other.patch
    }
}
pub static __SPEC_XDR_TYPE_CONTRACTVERSION: [u8; 96usize] = ContractVersion::spec_xdr();
impl ContractVersion {
    pub const fn spec_xdr() -> [u8; 96usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x0fContractVersion\0\0\0\0\x03\0\0\0\0\0\0\0\x05major\0\0\0\0\0\0\x04\0\0\0\0\0\0\0\x05minor\0\0\0\0\0\0\x04\0\0\0\0\0\0\0\x05patch\0\0\0\0\0\0\x04"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for ContractVersion {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 3usize] = ["major", "minor", "patch"];
        let mut vals: [Val; 3usize] = [Val::VOID.to_val(); 3usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            major: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            minor: vals[1].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            patch: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, ContractVersion> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &ContractVersion,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 3usize] = ["major", "minor", "patch"];
        let vals: [Val; 3usize] = [
            (&val.major).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.minor).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.patch).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct UpgradeProposal {
    pub id: u64,
    pub proposed_by: Address,
    pub new_wasm_hash: BytesN<32>,
    pub proposed_at: u64,
    pub approved: bool,
    pub executed: bool,
}
#[automatically_derived]
impl ::core::clone::Clone for UpgradeProposal {
    #[inline]
    fn clone(&self) -> UpgradeProposal {
        UpgradeProposal {
            id: ::core::clone::Clone::clone(&self.id),
            proposed_by: ::core::clone::Clone::clone(&self.proposed_by),
            new_wasm_hash: ::core::clone::Clone::clone(&self.new_wasm_hash),
            proposed_at: ::core::clone::Clone::clone(&self.proposed_at),
            approved: ::core::clone::Clone::clone(&self.approved),
            executed: ::core::clone::Clone::clone(&self.executed),
        }
    }
}
pub static __SPEC_XDR_TYPE_UPGRADEPROPOSAL: [u8; 172usize] = UpgradeProposal::spec_xdr();
impl UpgradeProposal {
    pub const fn spec_xdr() -> [u8; 172usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x0fUpgradeProposal\0\0\0\0\x06\0\0\0\0\0\0\0\x08approved\0\0\0\x01\0\0\0\0\0\0\0\x08executed\0\0\0\x01\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0\0\0\0\rnew_wasm_hash\0\0\0\0\0\x03\xee\0\0\0 \0\0\0\0\0\0\0\x0bproposed_at\0\0\0\0\x06\0\0\0\0\0\0\0\x0bproposed_by\0\0\0\0\x13"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for UpgradeProposal {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 6usize] = [
            "approved",
            "executed",
            "id",
            "new_wasm_hash",
            "proposed_at",
            "proposed_by",
        ];
        let mut vals: [Val; 6usize] = [Val::VOID.to_val(); 6usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            approved: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            executed: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            id: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            new_wasm_hash: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            proposed_at: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            proposed_by: vals[5]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, UpgradeProposal> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &UpgradeProposal,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 6usize] = [
            "approved",
            "executed",
            "id",
            "new_wasm_hash",
            "proposed_at",
            "proposed_by",
        ];
        let vals: [Val; 6usize] = [
            (&val.approved).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.executed).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.new_wasm_hash).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.proposed_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.proposed_by).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct TagReactivatedEvent {
    pub tag_id: BytesN<32>,
    pub pet_id: u64,
    pub reactivated_by: Address,
    pub timestamp: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for TagReactivatedEvent {
    #[inline]
    fn clone(&self) -> TagReactivatedEvent {
        TagReactivatedEvent {
            tag_id: ::core::clone::Clone::clone(&self.tag_id),
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            reactivated_by: ::core::clone::Clone::clone(&self.reactivated_by),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
        }
    }
}
pub static __SPEC_XDR_TYPE_TAGREACTIVATEDEVENT: [u8; 136usize] = TagReactivatedEvent::spec_xdr();
impl TagReactivatedEvent {
    pub const fn spec_xdr() -> [u8; 136usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x13TagReactivatedEvent\0\0\0\0\x04\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0ereactivated_by\0\0\0\0\0\x13\0\0\0\0\0\0\0\x06tag_id\0\0\0\0\x03\xee\0\0\0 \0\0\0\0\0\0\0\ttimestamp\0\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>
for TagReactivatedEvent {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 4usize] = [
            "pet_id",
            "reactivated_by",
            "tag_id",
            "timestamp",
        ];
        let mut vals: [Val; 4usize] = [Val::VOID.to_val(); 4usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            pet_id: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            reactivated_by: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            tag_id: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            timestamp: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, TagReactivatedEvent>
for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &TagReactivatedEvent,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 4usize] = [
            "pet_id",
            "reactivated_by",
            "tag_id",
            "timestamp",
        ];
        let vals: [Val; 4usize] = [
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.reactivated_by).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.tag_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.timestamp).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct PetTag {
    pub tag_id: BytesN<32>,
    pub pet_id: u64,
    pub owner: Address,
    pub message: String,
    pub is_active: bool,
    pub linked_at: u64,
    pub updated_at: u64,
    pub tag_message: String,
    pub created_at: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for PetTag {
    #[inline]
    fn clone(&self) -> PetTag {
        PetTag {
            tag_id: ::core::clone::Clone::clone(&self.tag_id),
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            owner: ::core::clone::Clone::clone(&self.owner),
            message: ::core::clone::Clone::clone(&self.message),
            is_active: ::core::clone::Clone::clone(&self.is_active),
            linked_at: ::core::clone::Clone::clone(&self.linked_at),
            updated_at: ::core::clone::Clone::clone(&self.updated_at),
            tag_message: ::core::clone::Clone::clone(&self.tag_message),
            created_at: ::core::clone::Clone::clone(&self.created_at),
        }
    }
}
pub static __SPEC_XDR_TYPE_PETTAG: [u8; 232usize] = PetTag::spec_xdr();
impl PetTag {
    pub const fn spec_xdr() -> [u8; 232usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x06PetTag\0\0\0\0\0\t\0\0\0\0\0\0\0\ncreated_at\0\0\0\0\0\x06\0\0\0\0\0\0\0\tis_active\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\tlinked_at\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\x07message\0\0\0\0\x10\0\0\0\0\0\0\0\x05owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x06tag_id\0\0\0\0\x03\xee\0\0\0 \0\0\0\0\0\0\0\x0btag_message\0\0\0\0\x10\0\0\0\0\0\0\0\nupdated_at\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for PetTag {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 9usize] = [
            "created_at",
            "is_active",
            "linked_at",
            "message",
            "owner",
            "pet_id",
            "tag_id",
            "tag_message",
            "updated_at",
        ];
        let mut vals: [Val; 9usize] = [Val::VOID.to_val(); 9usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            created_at: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            is_active: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            linked_at: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            message: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            owner: vals[4].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[5].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            tag_id: vals[6].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            tag_message: vals[7]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            updated_at: vals[8]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, PetTag> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &PetTag,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 9usize] = [
            "created_at",
            "is_active",
            "linked_at",
            "message",
            "owner",
            "pet_id",
            "tag_id",
            "tag_message",
            "updated_at",
        ];
        let vals: [Val; 9usize] = [
            (&val.created_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.is_active).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.linked_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.message).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.owner).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.tag_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.tag_message).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.updated_at).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub enum DataKey {
    Pet(u64),
    PetCount,
    PetOwner(Address),
    OwnerPetIndex((Address, u64)),
    PetCountByOwner(Address),
    SpeciesPetCount(String),
    SpeciesPetIndex((String, u64)),
    Vet(Address),
    VetLicense(String),
    Admin,
    ContractVersion,
    UpgradeProposal(u64),
    UpgradeProposalCount,
    AccessGrant((u64, Address)),
    AccessGrantCount(u64),
    AccessGrantIndex((u64, u64)),
    TemporaryCustody(u64),
}
pub static __SPEC_XDR_TYPE_DATAKEY: [u8; 604usize] = DataKey::spec_xdr();
impl DataKey {
    pub const fn spec_xdr() -> [u8; 604usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\x07DataKey\0\0\0\0\x11\0\0\0\x01\0\0\0\0\0\0\0\x03Pet\0\0\0\0\x01\0\0\0\x06\0\0\0\0\0\0\0\0\0\0\0\x08PetCount\0\0\0\x01\0\0\0\0\0\0\0\x08PetOwner\0\0\0\x01\0\0\0\x13\0\0\0\x01\0\0\0\0\0\0\0\rOwnerPetIndex\0\0\0\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\0\x13\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\x0fPetCountByOwner\0\0\0\0\x01\0\0\0\x13\0\0\0\x01\0\0\0\0\0\0\0\x0fSpeciesPetCount\0\0\0\0\x01\0\0\0\x10\0\0\0\x01\0\0\0\0\0\0\0\x0fSpeciesPetIndex\0\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\0\x10\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\x03Vet\0\0\0\0\x01\0\0\0\x13\0\0\0\x01\0\0\0\0\0\0\0\nVetLicense\0\0\0\0\0\x01\0\0\0\x10\0\0\0\0\0\0\0\0\0\0\0\x05Admin\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x0fContractVersion\0\0\0\0\x01\0\0\0\0\0\0\0\x0fUpgradeProposal\0\0\0\0\x01\0\0\0\x06\0\0\0\0\0\0\0\0\0\0\0\x14UpgradeProposalCount\0\0\0\x01\0\0\0\0\0\0\0\x0bAccessGrant\0\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\0\x06\0\0\0\x13\0\0\0\x01\0\0\0\0\0\0\0\x10AccessGrantCount\0\0\0\x01\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\x10AccessGrantIndex\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\0\x06\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\x10TemporaryCustody\0\0\0\x01\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for DataKey {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &[
            "Pet",
            "PetCount",
            "PetOwner",
            "OwnerPetIndex",
            "PetCountByOwner",
            "SpeciesPetCount",
            "SpeciesPetIndex",
            "Vet",
            "VetLicense",
            "Admin",
            "ContractVersion",
            "UpgradeProposal",
            "UpgradeProposalCount",
            "AccessGrant",
            "AccessGrantCount",
            "AccessGrantIndex",
            "TemporaryCustody",
        ];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Pet(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetCount
                }
                2 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetOwner(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                3 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::OwnerPetIndex(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                4 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetCountByOwner(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                5 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::SpeciesPetCount(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                6 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::SpeciesPetIndex(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                7 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Vet(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                8 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::VetLicense(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                9 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Admin
                }
                10 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::ContractVersion
                }
                11 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::UpgradeProposal(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                12 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::UpgradeProposalCount
                }
                13 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::AccessGrant(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                14 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::AccessGrantCount(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                15 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::AccessGrantIndex(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                16 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::TemporaryCustody(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, DataKey> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &DataKey,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            DataKey::Pet(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Pet")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::PetCount => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetCount")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::PetOwner(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetOwner")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::OwnerPetIndex(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"OwnerPetIndex")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::PetCountByOwner(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetCountByOwner")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::SpeciesPetCount(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"SpeciesPetCount")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::SpeciesPetIndex(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"SpeciesPetIndex")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::Vet(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Vet")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::VetLicense(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"VetLicense")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::Admin => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Admin")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::ContractVersion => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"ContractVersion")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::UpgradeProposal(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"UpgradeProposal")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::UpgradeProposalCount => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"UpgradeProposalCount")?
                        .to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::AccessGrant(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"AccessGrant")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::AccessGrantCount(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"AccessGrantCount")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::AccessGrantIndex(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"AccessGrantIndex")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            DataKey::TemporaryCustody(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"TemporaryCustody")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub enum MedicalKey {
    LabResult(u64),
    LabResultCount,
    PetLabResultIndex((u64, u64)),
    PetLabResultCount(u64),
    MedicalRecord(u64),
    MedicalRecordCount,
    PetMedicalRecordIndex((u64, u64)),
    PetMedicalRecordCount(u64),
    GlobalMedication(u64),
    MedicationCount,
    PetMedicationCount(u64),
    PetMedicationIndex((u64, u64)),
    Vaccination(u64),
    VaccinationCount,
    PetVaccinationCount(u64),
    PetVaccinationByIndex((u64, u64)),
}
pub static __SPEC_XDR_TYPE_MEDICALKEY: [u8; 664usize] = MedicalKey::spec_xdr();
impl MedicalKey {
    pub const fn spec_xdr() -> [u8; 664usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\nMedicalKey\0\0\0\0\0\x10\0\0\0\x01\0\0\0\0\0\0\0\tLabResult\0\0\0\0\0\0\x01\0\0\0\x06\0\0\0\0\0\0\0\0\0\0\0\x0eLabResultCount\0\0\0\0\0\x01\0\0\0\0\0\0\0\x11PetLabResultIndex\0\0\0\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\0\x06\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\x11PetLabResultCount\0\0\0\0\0\0\x01\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\rMedicalRecord\0\0\0\0\0\0\x01\0\0\0\x06\0\0\0\0\0\0\0\0\0\0\0\x12MedicalRecordCount\0\0\0\0\0\x01\0\0\0\0\0\0\0\x15PetMedicalRecordIndex\0\0\0\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\0\x06\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\x15PetMedicalRecordCount\0\0\0\0\0\0\x01\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\x10GlobalMedication\0\0\0\x01\0\0\0\x06\0\0\0\0\0\0\0\0\0\0\0\x0fMedicationCount\0\0\0\0\x01\0\0\0\0\0\0\0\x12PetMedicationCount\0\0\0\0\0\x01\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\x12PetMedicationIndex\0\0\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\0\x06\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\x0bVaccination\0\0\0\0\x01\0\0\0\x06\0\0\0\0\0\0\0\0\0\0\0\x10VaccinationCount\0\0\0\x01\0\0\0\0\0\0\0\x13PetVaccinationCount\0\0\0\0\x01\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\x15PetVaccinationByIndex\0\0\0\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\0\x06\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for MedicalKey {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &[
            "LabResult",
            "LabResultCount",
            "PetLabResultIndex",
            "PetLabResultCount",
            "MedicalRecord",
            "MedicalRecordCount",
            "PetMedicalRecordIndex",
            "PetMedicalRecordCount",
            "GlobalMedication",
            "MedicationCount",
            "PetMedicationCount",
            "PetMedicationIndex",
            "Vaccination",
            "VaccinationCount",
            "PetVaccinationCount",
            "PetVaccinationByIndex",
        ];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::LabResult(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::LabResultCount
                }
                2 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetLabResultIndex(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                3 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetLabResultCount(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                4 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::MedicalRecord(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                5 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::MedicalRecordCount
                }
                6 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetMedicalRecordIndex(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                7 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetMedicalRecordCount(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                8 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::GlobalMedication(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                9 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::MedicationCount
                }
                10 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetMedicationCount(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                11 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetMedicationIndex(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                12 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Vaccination(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                13 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::VaccinationCount
                }
                14 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetVaccinationCount(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                15 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetVaccinationByIndex(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, MedicalKey> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &MedicalKey,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            MedicalKey::LabResult(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"LabResult")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            MedicalKey::LabResultCount => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"LabResultCount")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            MedicalKey::PetLabResultIndex(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetLabResultIndex")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            MedicalKey::PetLabResultCount(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetLabResultCount")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            MedicalKey::MedicalRecord(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"MedicalRecord")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            MedicalKey::MedicalRecordCount => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"MedicalRecordCount")?
                        .to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            MedicalKey::PetMedicalRecordIndex(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetMedicalRecordIndex")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            MedicalKey::PetMedicalRecordCount(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetMedicalRecordCount")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            MedicalKey::GlobalMedication(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"GlobalMedication")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            MedicalKey::MedicationCount => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"MedicationCount")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            MedicalKey::PetMedicationCount(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetMedicationCount")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            MedicalKey::PetMedicationIndex(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetMedicationIndex")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            MedicalKey::Vaccination(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Vaccination")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            MedicalKey::VaccinationCount => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"VaccinationCount")?
                        .to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            MedicalKey::PetVaccinationCount(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetVaccinationCount")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            MedicalKey::PetVaccinationByIndex(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetVaccinationByIndex")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub enum ReviewKey {
    VetReview(u64),
    VetReviewCount,
    VetReviewByVetIndex((Address, u64)),
    VetReviewCountByVet(Address),
    VetReviewByOwnerVet((Address, Address)),
}
pub static __SPEC_XDR_TYPE_REVIEWKEY: [u8; 236usize] = ReviewKey::spec_xdr();
impl ReviewKey {
    pub const fn spec_xdr() -> [u8; 236usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\tReviewKey\0\0\0\0\0\0\x05\0\0\0\x01\0\0\0\0\0\0\0\tVetReview\0\0\0\0\0\0\x01\0\0\0\x06\0\0\0\0\0\0\0\0\0\0\0\x0eVetReviewCount\0\0\0\0\0\x01\0\0\0\0\0\0\0\x13VetReviewByVetIndex\0\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\0\x13\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\x13VetReviewCountByVet\0\0\0\0\x01\0\0\0\x13\0\0\0\x01\0\0\0\0\0\0\0\x13VetReviewByOwnerVet\0\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\0\x13\0\0\0\x13"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for ReviewKey {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &[
            "VetReview",
            "VetReviewCount",
            "VetReviewByVetIndex",
            "VetReviewCountByVet",
            "VetReviewByOwnerVet",
        ];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::VetReview(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::VetReviewCount
                }
                2 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::VetReviewByVetIndex(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                3 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::VetReviewCountByVet(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                4 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::VetReviewByOwnerVet(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, ReviewKey> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &ReviewKey,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            ReviewKey::VetReview(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"VetReview")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            ReviewKey::VetReviewCount => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"VetReviewCount")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            ReviewKey::VetReviewByVetIndex(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"VetReviewByVetIndex")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            ReviewKey::VetReviewCountByVet(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"VetReviewCountByVet")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            ReviewKey::VetReviewByOwnerVet(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"VetReviewByOwnerVet")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub enum AlertKey {
    LostPetAlert(u64),
    LostPetAlertCount,
    ActiveLostPetAlerts,
    AlertSightings(u64),
}
pub static __SPEC_XDR_TYPE_ALERTKEY: [u8; 160usize] = AlertKey::spec_xdr();
impl AlertKey {
    pub const fn spec_xdr() -> [u8; 160usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\x08AlertKey\0\0\0\x04\0\0\0\x01\0\0\0\0\0\0\0\x0cLostPetAlert\0\0\0\x01\0\0\0\x06\0\0\0\0\0\0\0\0\0\0\0\x11LostPetAlertCount\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x13ActiveLostPetAlerts\0\0\0\0\x01\0\0\0\0\0\0\0\x0eAlertSightings\0\0\0\0\0\x01\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for AlertKey {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &[
            "LostPetAlert",
            "LostPetAlertCount",
            "ActiveLostPetAlerts",
            "AlertSightings",
        ];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::LostPetAlert(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::LostPetAlertCount
                }
                2 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::ActiveLostPetAlerts
                }
                3 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::AlertSightings(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, AlertKey> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &AlertKey,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            AlertKey::LostPetAlert(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"LostPetAlert")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            AlertKey::LostPetAlertCount => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"LostPetAlertCount")?
                        .to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            AlertKey::ActiveLostPetAlerts => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"ActiveLostPetAlerts")?
                        .to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            AlertKey::AlertSightings(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"AlertSightings")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub enum ConsentKey {
    Consent(u64),
    ConsentCount,
    PetConsentIndex((u64, u64)),
    PetConsentCount(u64),
}
pub static __SPEC_XDR_TYPE_CONSENTKEY: [u8; 168usize] = ConsentKey::spec_xdr();
impl ConsentKey {
    pub const fn spec_xdr() -> [u8; 168usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\nConsentKey\0\0\0\0\0\x04\0\0\0\x01\0\0\0\0\0\0\0\x07Consent\0\0\0\0\x01\0\0\0\x06\0\0\0\0\0\0\0\0\0\0\0\x0cConsentCount\0\0\0\x01\0\0\0\0\0\0\0\x0fPetConsentIndex\0\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\0\x06\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\x0fPetConsentCount\0\0\0\0\x01\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for ConsentKey {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &[
            "Consent",
            "ConsentCount",
            "PetConsentIndex",
            "PetConsentCount",
        ];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Consent(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::ConsentCount
                }
                2 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetConsentIndex(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                3 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetConsentCount(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, ConsentKey> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &ConsentKey,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            ConsentKey::Consent(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Consent")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            ConsentKey::ConsentCount => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"ConsentCount")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            ConsentKey::PetConsentIndex(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetConsentIndex")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            ConsentKey::PetConsentCount(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetConsentCount")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub enum TreatmentKey {
    Treatment(u64),
    TreatmentCount,
    PetTreatmentCount(u64),
    PetTreatmentIndex((u64, u64)),
}
pub static __SPEC_XDR_TYPE_TREATMENTKEY: [u8; 184usize] = TreatmentKey::spec_xdr();
impl TreatmentKey {
    pub const fn spec_xdr() -> [u8; 184usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\x0cTreatmentKey\0\0\0\x04\0\0\0\x01\0\0\0\0\0\0\0\tTreatment\0\0\0\0\0\0\x01\0\0\0\x06\0\0\0\0\0\0\0\0\0\0\0\x0eTreatmentCount\0\0\0\0\0\x01\0\0\0\0\0\0\0\x11PetTreatmentCount\0\0\0\0\0\0\x01\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\x11PetTreatmentIndex\0\0\0\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\0\x06\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for TreatmentKey {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &[
            "Treatment",
            "TreatmentCount",
            "PetTreatmentCount",
            "PetTreatmentIndex",
        ];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Treatment(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::TreatmentCount
                }
                2 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetTreatmentCount(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                3 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetTreatmentIndex(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, TreatmentKey> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &TreatmentKey,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            TreatmentKey::Treatment(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Treatment")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            TreatmentKey::TreatmentCount => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"TreatmentCount")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            TreatmentKey::PetTreatmentCount(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetTreatmentCount")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            TreatmentKey::PetTreatmentIndex(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetTreatmentIndex")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub enum TagKey {
    Tag(BytesN<32>),
    PetTagId(u64),
    TagNonce,
    PetTagCount,
}
pub static __SPEC_XDR_TYPE_TAGKEY: [u8; 128usize] = TagKey::spec_xdr();
impl TagKey {
    pub const fn spec_xdr() -> [u8; 128usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\x06TagKey\0\0\0\0\0\x04\0\0\0\x01\0\0\0\0\0\0\0\x03Tag\0\0\0\0\x01\0\0\x03\xee\0\0\0 \0\0\0\x01\0\0\0\0\0\0\0\x08PetTagId\0\0\0\x01\0\0\0\x06\0\0\0\0\0\0\0\0\0\0\0\x08TagNonce\0\0\0\0\0\0\0\0\0\0\0\x0bPetTagCount\0"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for TagKey {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &[
            "Tag",
            "PetTagId",
            "TagNonce",
            "PetTagCount",
        ];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Tag(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                1 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetTagId(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                2 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::TagNonce
                }
                3 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetTagCount
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, TagKey> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &TagKey,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            TagKey::Tag(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Tag")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            TagKey::PetTagId(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetTagId")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            TagKey::TagNonce => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"TagNonce")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            TagKey::PetTagCount => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetTagCount")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub enum SystemKey {
    PetOwnershipRecord(u64),
    OwnershipRecordCount,
    PetOwnershipRecordCount(u64),
    PetOwnershipRecordIndex((u64, u64)),
    Admins,
    AdminThreshold,
    Proposal(u64),
    ProposalCount,
    VetAvailability((Address, u64)),
    VetAvailabilityCount(Address),
    VetAvailabilityByDate((Address, u64)),
}
pub static __SPEC_XDR_TYPE_SYSTEMKEY: [u8; 452usize] = SystemKey::spec_xdr();
impl SystemKey {
    pub const fn spec_xdr() -> [u8; 452usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\tSystemKey\0\0\0\0\0\0\x0b\0\0\0\x01\0\0\0\0\0\0\0\x12PetOwnershipRecord\0\0\0\0\0\x01\0\0\0\x06\0\0\0\0\0\0\0\0\0\0\0\x14OwnershipRecordCount\0\0\0\x01\0\0\0\0\0\0\0\x17PetOwnershipRecordCount\0\0\0\0\x01\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\x17PetOwnershipRecordIndex\0\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\0\x06\0\0\0\x06\0\0\0\0\0\0\0\0\0\0\0\x06Admins\0\0\0\0\0\0\0\0\0\0\0\0\0\x0eAdminThreshold\0\0\0\0\0\x01\0\0\0\0\0\0\0\x08Proposal\0\0\0\x01\0\0\0\x06\0\0\0\0\0\0\0\0\0\0\0\rProposalCount\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x0fVetAvailability\0\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\0\x13\0\0\0\x06\0\0\0\x01\0\0\0\0\0\0\0\x14VetAvailabilityCount\0\0\0\x01\0\0\0\x13\0\0\0\x01\0\0\0\0\0\0\0\x15VetAvailabilityByDate\0\0\0\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\0\x13\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for SystemKey {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &[
            "PetOwnershipRecord",
            "OwnershipRecordCount",
            "PetOwnershipRecordCount",
            "PetOwnershipRecordIndex",
            "Admins",
            "AdminThreshold",
            "Proposal",
            "ProposalCount",
            "VetAvailability",
            "VetAvailabilityCount",
            "VetAvailabilityByDate",
        ];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetOwnershipRecord(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::OwnershipRecordCount
                }
                2 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetOwnershipRecordCount(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                3 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PetOwnershipRecordIndex(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                4 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Admins
                }
                5 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::AdminThreshold
                }
                6 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Proposal(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                7 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::ProposalCount
                }
                8 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::VetAvailability(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                9 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::VetAvailabilityCount(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                10 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::VetAvailabilityByDate(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, SystemKey> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &SystemKey,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            SystemKey::PetOwnershipRecord(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetOwnershipRecord")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            SystemKey::OwnershipRecordCount => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"OwnershipRecordCount")?
                        .to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            SystemKey::PetOwnershipRecordCount(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetOwnershipRecordCount")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            SystemKey::PetOwnershipRecordIndex(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PetOwnershipRecordIndex")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            SystemKey::Admins => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Admins")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            SystemKey::AdminThreshold => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"AdminThreshold")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            SystemKey::Proposal(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Proposal")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            SystemKey::ProposalCount => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"ProposalCount")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            SystemKey::VetAvailability(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"VetAvailability")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            SystemKey::VetAvailabilityCount(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"VetAvailabilityCount")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            SystemKey::VetAvailabilityByDate(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"VetAvailabilityByDate")?
                        .to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub enum AlertStatus {
    Active,
    Found,
    Cancelled,
}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for AlertStatus {}
#[automatically_derived]
impl ::core::clone::Clone for AlertStatus {
    #[inline]
    fn clone(&self) -> AlertStatus {
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for AlertStatus {}
#[automatically_derived]
impl ::core::fmt::Debug for AlertStatus {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                AlertStatus::Active => "Active",
                AlertStatus::Found => "Found",
                AlertStatus::Cancelled => "Cancelled",
            },
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for AlertStatus {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for AlertStatus {}
#[automatically_derived]
impl ::core::cmp::PartialEq for AlertStatus {
    #[inline]
    fn eq(&self, other: &AlertStatus) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
pub static __SPEC_XDR_TYPE_ALERTSTATUS: [u8; 96usize] = AlertStatus::spec_xdr();
impl AlertStatus {
    pub const fn spec_xdr() -> [u8; 96usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\x0bAlertStatus\0\0\0\0\x03\0\0\0\0\0\0\0\0\0\0\0\x06Active\0\0\0\0\0\0\0\0\0\0\0\0\0\x05Found\0\0\0\0\0\0\0\0\0\0\0\0\0\0\tCancelled\0\0\0"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for AlertStatus {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &["Active", "Found", "Cancelled"];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Active
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Found
                }
                2 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Cancelled
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, AlertStatus> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &AlertStatus,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            AlertStatus::Active => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Active")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            AlertStatus::Found => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Found")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            AlertStatus::Cancelled => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Cancelled")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub struct LostPetAlert {
    pub id: u64,
    pub pet_id: u64,
    pub reported_by: Address,
    pub reported_date: u64,
    pub last_seen_location: String,
    pub reward_amount: Option<u64>,
    pub status: AlertStatus,
    pub found_date: Option<u64>,
}
#[automatically_derived]
impl ::core::clone::Clone for LostPetAlert {
    #[inline]
    fn clone(&self) -> LostPetAlert {
        LostPetAlert {
            id: ::core::clone::Clone::clone(&self.id),
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            reported_by: ::core::clone::Clone::clone(&self.reported_by),
            reported_date: ::core::clone::Clone::clone(&self.reported_date),
            last_seen_location: ::core::clone::Clone::clone(&self.last_seen_location),
            reward_amount: ::core::clone::Clone::clone(&self.reward_amount),
            status: ::core::clone::Clone::clone(&self.status),
            found_date: ::core::clone::Clone::clone(&self.found_date),
        }
    }
}
pub static __SPEC_XDR_TYPE_LOSTPETALERT: [u8; 248usize] = LostPetAlert::spec_xdr();
impl LostPetAlert {
    pub const fn spec_xdr() -> [u8; 248usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x0cLostPetAlert\0\0\0\x08\0\0\0\0\0\0\0\nfound_date\0\0\0\0\x03\xe8\0\0\0\x06\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x12last_seen_location\0\0\0\0\0\x10\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0breported_by\0\0\0\0\x13\0\0\0\0\0\0\0\rreported_date\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\rreward_amount\0\0\0\0\0\x03\xe8\0\0\0\x06\0\0\0\0\0\0\0\x06status\0\0\0\0\x07\xd0\0\0\0\x0bAlertStatus\0"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for LostPetAlert {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 8usize] = [
            "found_date",
            "id",
            "last_seen_location",
            "pet_id",
            "reported_by",
            "reported_date",
            "reward_amount",
            "status",
        ];
        let mut vals: [Val; 8usize] = [Val::VOID.to_val(); 8usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            found_date: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            id: vals[1].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            last_seen_location: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[3].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            reported_by: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            reported_date: vals[5]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            reward_amount: vals[6]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            status: vals[7].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, LostPetAlert> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &LostPetAlert,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 8usize] = [
            "found_date",
            "id",
            "last_seen_location",
            "pet_id",
            "reported_by",
            "reported_date",
            "reward_amount",
            "status",
        ];
        let vals: [Val; 8usize] = [
            (&val.found_date).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.last_seen_location).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.reported_by).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.reported_date).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.reward_amount).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.status).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct SightingReport {
    pub alert_id: u64,
    pub reporter: Address,
    pub location: String,
    pub timestamp: u64,
    pub description: String,
}
#[automatically_derived]
impl ::core::clone::Clone for SightingReport {
    #[inline]
    fn clone(&self) -> SightingReport {
        SightingReport {
            alert_id: ::core::clone::Clone::clone(&self.alert_id),
            reporter: ::core::clone::Clone::clone(&self.reporter),
            location: ::core::clone::Clone::clone(&self.location),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
            description: ::core::clone::Clone::clone(&self.description),
        }
    }
}
pub static __SPEC_XDR_TYPE_SIGHTINGREPORT: [u8; 144usize] = SightingReport::spec_xdr();
impl SightingReport {
    pub const fn spec_xdr() -> [u8; 144usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x0eSightingReport\0\0\0\0\0\x05\0\0\0\0\0\0\0\x08alert_id\0\0\0\x06\0\0\0\0\0\0\0\x0bdescription\0\0\0\0\x10\0\0\0\0\0\0\0\x08location\0\0\0\x10\0\0\0\0\0\0\0\x08reporter\0\0\0\x13\0\0\0\0\0\0\0\ttimestamp\0\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for SightingReport {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 5usize] = [
            "alert_id",
            "description",
            "location",
            "reporter",
            "timestamp",
        ];
        let mut vals: [Val; 5usize] = [Val::VOID.to_val(); 5usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            alert_id: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            description: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            location: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            reporter: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            timestamp: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, SightingReport> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &SightingReport,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 5usize] = [
            "alert_id",
            "description",
            "location",
            "reporter",
            "timestamp",
        ];
        let vals: [Val; 5usize] = [
            (&val.alert_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.description).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.location).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.reporter).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.timestamp).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct AvailabilitySlot {
    pub vet_address: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub available: bool,
}
#[automatically_derived]
impl ::core::clone::Clone for AvailabilitySlot {
    #[inline]
    fn clone(&self) -> AvailabilitySlot {
        AvailabilitySlot {
            vet_address: ::core::clone::Clone::clone(&self.vet_address),
            start_time: ::core::clone::Clone::clone(&self.start_time),
            end_time: ::core::clone::Clone::clone(&self.end_time),
            available: ::core::clone::Clone::clone(&self.available),
        }
    }
}
pub static __SPEC_XDR_TYPE_AVAILABILITYSLOT: [u8; 128usize] = AvailabilitySlot::spec_xdr();
impl AvailabilitySlot {
    pub const fn spec_xdr() -> [u8; 128usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x10AvailabilitySlot\0\0\0\x04\0\0\0\0\0\0\0\tavailable\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x08end_time\0\0\0\x06\0\0\0\0\0\0\0\nstart_time\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for AvailabilitySlot {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 4usize] = [
            "available",
            "end_time",
            "start_time",
            "vet_address",
        ];
        let mut vals: [Val; 4usize] = [Val::VOID.to_val(); 4usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            available: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            end_time: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            start_time: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            vet_address: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, AvailabilitySlot> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &AvailabilitySlot,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 4usize] = [
            "available",
            "end_time",
            "start_time",
            "vet_address",
        ];
        let vals: [Val; 4usize] = [
            (&val.available).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.end_time).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.start_time).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.vet_address).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub enum ConsentType {
    Insurance,
    Research,
    PublicHealth,
    Other,
}
#[automatically_derived]
impl ::core::clone::Clone for ConsentType {
    #[inline]
    fn clone(&self) -> ConsentType {
        match self {
            ConsentType::Insurance => ConsentType::Insurance,
            ConsentType::Research => ConsentType::Research,
            ConsentType::PublicHealth => ConsentType::PublicHealth,
            ConsentType::Other => ConsentType::Other,
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for ConsentType {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                ConsentType::Insurance => "Insurance",
                ConsentType::Research => "Research",
                ConsentType::PublicHealth => "PublicHealth",
                ConsentType::Other => "Other",
            },
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ConsentType {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ConsentType {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ConsentType {
    #[inline]
    fn eq(&self, other: &ConsentType) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
pub static __SPEC_XDR_TYPE_CONSENTTYPE: [u8; 120usize] = ConsentType::spec_xdr();
impl ConsentType {
    pub const fn spec_xdr() -> [u8; 120usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\x0bConsentType\0\0\0\0\x04\0\0\0\0\0\0\0\0\0\0\0\tInsurance\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x08Research\0\0\0\0\0\0\0\0\0\0\0\x0cPublicHealth\0\0\0\0\0\0\0\0\0\0\0\x05Other\0\0\0"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for ConsentType {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &[
            "Insurance",
            "Research",
            "PublicHealth",
            "Other",
        ];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Insurance
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Research
                }
                2 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::PublicHealth
                }
                3 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Other
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, ConsentType> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &ConsentType,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            ConsentType::Insurance => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Insurance")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            ConsentType::Research => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Research")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            ConsentType::PublicHealth => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"PublicHealth")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            ConsentType::Other => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Other")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub struct Consent {
    pub id: u64,
    pub pet_id: u64,
    pub owner: Address,
    pub consent_type: ConsentType,
    pub granted_to: Address,
    pub granted_at: u64,
    pub revoked_at: Option<u64>,
    pub is_active: bool,
}
#[automatically_derived]
impl ::core::clone::Clone for Consent {
    #[inline]
    fn clone(&self) -> Consent {
        Consent {
            id: ::core::clone::Clone::clone(&self.id),
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            owner: ::core::clone::Clone::clone(&self.owner),
            consent_type: ::core::clone::Clone::clone(&self.consent_type),
            granted_to: ::core::clone::Clone::clone(&self.granted_to),
            granted_at: ::core::clone::Clone::clone(&self.granted_at),
            revoked_at: ::core::clone::Clone::clone(&self.revoked_at),
            is_active: ::core::clone::Clone::clone(&self.is_active),
        }
    }
}
pub static __SPEC_XDR_TYPE_CONSENT: [u8; 224usize] = Consent::spec_xdr();
impl Consent {
    pub const fn spec_xdr() -> [u8; 224usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x07Consent\0\0\0\0\x08\0\0\0\0\0\0\0\x0cconsent_type\0\0\x07\xd0\0\0\0\x0bConsentType\0\0\0\0\0\0\0\0\ngranted_at\0\0\0\0\0\x06\0\0\0\0\0\0\0\ngranted_to\0\0\0\0\0\x13\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0\0\0\0\tis_active\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x05owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\nrevoked_at\0\0\0\0\x03\xe8\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for Consent {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 8usize] = [
            "consent_type",
            "granted_at",
            "granted_to",
            "id",
            "is_active",
            "owner",
            "pet_id",
            "revoked_at",
        ];
        let mut vals: [Val; 8usize] = [Val::VOID.to_val(); 8usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            consent_type: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            granted_at: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            granted_to: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            id: vals[3].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            is_active: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            owner: vals[5].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[6].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            revoked_at: vals[7]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, Consent> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &Consent,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 8usize] = [
            "consent_type",
            "granted_at",
            "granted_to",
            "id",
            "is_active",
            "owner",
            "pet_id",
            "revoked_at",
        ];
        let vals: [Val; 8usize] = [
            (&val.consent_type).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.granted_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.granted_to).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.is_active).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.owner).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.revoked_at).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct LabResult {
    pub id: u64,
    pub pet_id: u64,
    pub test_type: String,
    pub date: u64,
    pub results: String,
    pub vet_address: Address,
    pub reference_ranges: String,
    pub attachment_hash: Option<String>,
    pub medical_record_id: Option<u64>,
}
#[automatically_derived]
impl ::core::clone::Clone for LabResult {
    #[inline]
    fn clone(&self) -> LabResult {
        LabResult {
            id: ::core::clone::Clone::clone(&self.id),
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            test_type: ::core::clone::Clone::clone(&self.test_type),
            date: ::core::clone::Clone::clone(&self.date),
            results: ::core::clone::Clone::clone(&self.results),
            vet_address: ::core::clone::Clone::clone(&self.vet_address),
            reference_ranges: ::core::clone::Clone::clone(&self.reference_ranges),
            attachment_hash: ::core::clone::Clone::clone(&self.attachment_hash),
            medical_record_id: ::core::clone::Clone::clone(&self.medical_record_id),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for LabResult {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "id",
            "pet_id",
            "test_type",
            "date",
            "results",
            "vet_address",
            "reference_ranges",
            "attachment_hash",
            "medical_record_id",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.id,
            &self.pet_id,
            &self.test_type,
            &self.date,
            &self.results,
            &self.vet_address,
            &self.reference_ranges,
            &self.attachment_hash,
            &&self.medical_record_id,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(f, "LabResult", names, values)
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for LabResult {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<String>;
        let _: ::core::cmp::AssertParamIsEq<Address>;
        let _: ::core::cmp::AssertParamIsEq<Option<String>>;
        let _: ::core::cmp::AssertParamIsEq<Option<u64>>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for LabResult {}
#[automatically_derived]
impl ::core::cmp::PartialEq for LabResult {
    #[inline]
    fn eq(&self, other: &LabResult) -> bool {
        self.id == other.id && self.pet_id == other.pet_id && self.date == other.date
            && self.test_type == other.test_type && self.results == other.results
            && self.vet_address == other.vet_address
            && self.reference_ranges == other.reference_ranges
            && self.attachment_hash == other.attachment_hash
            && self.medical_record_id == other.medical_record_id
    }
}
pub static __SPEC_XDR_TYPE_LABRESULT: [u8; 248usize] = LabResult::spec_xdr();
impl LabResult {
    pub const fn spec_xdr() -> [u8; 248usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\tLabResult\0\0\0\0\0\0\t\0\0\0\0\0\0\0\x0fattachment_hash\0\0\0\x03\xe8\0\0\0\x10\0\0\0\0\0\0\0\x04date\0\0\0\x06\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x11medical_record_id\0\0\0\0\0\x03\xe8\0\0\0\x06\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x10reference_ranges\0\0\0\x10\0\0\0\0\0\0\0\x07results\0\0\0\0\x10\0\0\0\0\0\0\0\ttest_type\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for LabResult {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 9usize] = [
            "attachment_hash",
            "date",
            "id",
            "medical_record_id",
            "pet_id",
            "reference_ranges",
            "results",
            "test_type",
            "vet_address",
        ];
        let mut vals: [Val; 9usize] = [Val::VOID.to_val(); 9usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            attachment_hash: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            date: vals[1].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            id: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            medical_record_id: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[4].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            reference_ranges: vals[5]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            results: vals[6]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            test_type: vals[7]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            vet_address: vals[8]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, LabResult> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &LabResult,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 9usize] = [
            "attachment_hash",
            "date",
            "id",
            "medical_record_id",
            "pet_id",
            "reference_ranges",
            "results",
            "test_type",
            "vet_address",
        ];
        let vals: [Val; 9usize] = [
            (&val.attachment_hash).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.date).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.medical_record_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.reference_ranges).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.results).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.test_type).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.vet_address).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct VaccinationSummary {
    pub is_fully_current: bool,
    pub overdue_types: Vec<VaccineType>,
    pub upcoming_count: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for VaccinationSummary {
    #[inline]
    fn clone(&self) -> VaccinationSummary {
        VaccinationSummary {
            is_fully_current: ::core::clone::Clone::clone(&self.is_fully_current),
            overdue_types: ::core::clone::Clone::clone(&self.overdue_types),
            upcoming_count: ::core::clone::Clone::clone(&self.upcoming_count),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for VaccinationSummary {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "VaccinationSummary",
            "is_fully_current",
            &self.is_fully_current,
            "overdue_types",
            &self.overdue_types,
            "upcoming_count",
            &&self.upcoming_count,
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for VaccinationSummary {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<bool>;
        let _: ::core::cmp::AssertParamIsEq<Vec<VaccineType>>;
        let _: ::core::cmp::AssertParamIsEq<u64>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for VaccinationSummary {}
#[automatically_derived]
impl ::core::cmp::PartialEq for VaccinationSummary {
    #[inline]
    fn eq(&self, other: &VaccinationSummary) -> bool {
        self.is_fully_current == other.is_fully_current
            && self.upcoming_count == other.upcoming_count
            && self.overdue_types == other.overdue_types
    }
}
pub static __SPEC_XDR_TYPE_VACCINATIONSUMMARY: [u8; 144usize] = VaccinationSummary::spec_xdr();
impl VaccinationSummary {
    pub const fn spec_xdr() -> [u8; 144usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x12VaccinationSummary\0\0\0\0\0\x03\0\0\0\0\0\0\0\x10is_fully_current\0\0\0\x01\0\0\0\0\0\0\0\roverdue_types\0\0\0\0\0\x03\xea\0\0\x07\xd0\0\0\0\x0bVaccineType\0\0\0\0\0\0\0\0\x0eupcoming_count\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for VaccinationSummary {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 3usize] = [
            "is_fully_current",
            "overdue_types",
            "upcoming_count",
        ];
        let mut vals: [Val; 3usize] = [Val::VOID.to_val(); 3usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            is_fully_current: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            overdue_types: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            upcoming_count: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, VaccinationSummary> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &VaccinationSummary,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 3usize] = [
            "is_fully_current",
            "overdue_types",
            "upcoming_count",
        ];
        let vals: [Val; 3usize] = [
            (&val.is_fully_current).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.overdue_types).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.upcoming_count).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub enum AccessLevel {
    None,
    Basic,
    Full,
}
#[automatically_derived]
impl ::core::clone::Clone for AccessLevel {
    #[inline]
    fn clone(&self) -> AccessLevel {
        match self {
            AccessLevel::None => AccessLevel::None,
            AccessLevel::Basic => AccessLevel::Basic,
            AccessLevel::Full => AccessLevel::Full,
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for AccessLevel {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                AccessLevel::None => "None",
                AccessLevel::Basic => "Basic",
                AccessLevel::Full => "Full",
            },
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for AccessLevel {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for AccessLevel {}
#[automatically_derived]
impl ::core::cmp::PartialEq for AccessLevel {
    #[inline]
    fn eq(&self, other: &AccessLevel) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
pub static __SPEC_XDR_TYPE_ACCESSLEVEL: [u8; 84usize] = AccessLevel::spec_xdr();
impl AccessLevel {
    pub const fn spec_xdr() -> [u8; 84usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\x0bAccessLevel\0\0\0\0\x03\0\0\0\0\0\0\0\0\0\0\0\x04None\0\0\0\0\0\0\0\0\0\0\0\x05Basic\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x04Full"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for AccessLevel {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &["None", "Basic", "Full"];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::None
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Basic
                }
                2 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Full
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, AccessLevel> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &AccessLevel,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            AccessLevel::None => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"None")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            AccessLevel::Basic => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Basic")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            AccessLevel::Full => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Full")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub struct AccessGrant {
    pub pet_id: u64,
    pub granter: Address,
    pub grantee: Address,
    pub access_level: AccessLevel,
    pub granted_at: u64,
    pub expires_at: Option<u64>,
    pub is_active: bool,
}
#[automatically_derived]
impl ::core::clone::Clone for AccessGrant {
    #[inline]
    fn clone(&self) -> AccessGrant {
        AccessGrant {
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            granter: ::core::clone::Clone::clone(&self.granter),
            grantee: ::core::clone::Clone::clone(&self.grantee),
            access_level: ::core::clone::Clone::clone(&self.access_level),
            granted_at: ::core::clone::Clone::clone(&self.granted_at),
            expires_at: ::core::clone::Clone::clone(&self.expires_at),
            is_active: ::core::clone::Clone::clone(&self.is_active),
        }
    }
}
pub static __SPEC_XDR_TYPE_ACCESSGRANT: [u8; 208usize] = AccessGrant::spec_xdr();
impl AccessGrant {
    pub const fn spec_xdr() -> [u8; 208usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x0bAccessGrant\0\0\0\0\x07\0\0\0\0\0\0\0\x0caccess_level\0\0\x07\xd0\0\0\0\x0bAccessLevel\0\0\0\0\0\0\0\0\nexpires_at\0\0\0\0\x03\xe8\0\0\0\x06\0\0\0\0\0\0\0\ngranted_at\0\0\0\0\0\x06\0\0\0\0\0\0\0\x07grantee\0\0\0\0\x13\0\0\0\0\0\0\0\x07granter\0\0\0\0\x13\0\0\0\0\0\0\0\tis_active\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for AccessGrant {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 7usize] = [
            "access_level",
            "expires_at",
            "granted_at",
            "grantee",
            "granter",
            "is_active",
            "pet_id",
        ];
        let mut vals: [Val; 7usize] = [Val::VOID.to_val(); 7usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            access_level: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            expires_at: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            granted_at: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            grantee: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            granter: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            is_active: vals[5]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[6].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, AccessGrant> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &AccessGrant,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 7usize] = [
            "access_level",
            "expires_at",
            "granted_at",
            "grantee",
            "granter",
            "is_active",
            "pet_id",
        ];
        let vals: [Val; 7usize] = [
            (&val.access_level).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.expires_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.granted_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.grantee).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.granter).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.is_active).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct TemporaryCustody {
    pub pet_id: u64,
    pub owner: Address,
    pub custodian: Address,
    pub start_date: u64,
    pub end_date: u64,
    pub permissions: Vec<String>,
    pub is_active: bool,
}
#[automatically_derived]
impl ::core::clone::Clone for TemporaryCustody {
    #[inline]
    fn clone(&self) -> TemporaryCustody {
        TemporaryCustody {
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            owner: ::core::clone::Clone::clone(&self.owner),
            custodian: ::core::clone::Clone::clone(&self.custodian),
            start_date: ::core::clone::Clone::clone(&self.start_date),
            end_date: ::core::clone::Clone::clone(&self.end_date),
            permissions: ::core::clone::Clone::clone(&self.permissions),
            is_active: ::core::clone::Clone::clone(&self.is_active),
        }
    }
}
pub static __SPEC_XDR_TYPE_TEMPORARYCUSTODY: [u8; 196usize] = TemporaryCustody::spec_xdr();
impl TemporaryCustody {
    pub const fn spec_xdr() -> [u8; 196usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x10TemporaryCustody\0\0\0\x07\0\0\0\0\0\0\0\tcustodian\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x08end_date\0\0\0\x06\0\0\0\0\0\0\0\tis_active\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x05owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x0bpermissions\0\0\0\x03\xea\0\0\0\x10\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\nstart_date\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for TemporaryCustody {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 7usize] = [
            "custodian",
            "end_date",
            "is_active",
            "owner",
            "permissions",
            "pet_id",
            "start_date",
        ];
        let mut vals: [Val; 7usize] = [Val::VOID.to_val(); 7usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            custodian: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            end_date: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            is_active: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            owner: vals[3].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            permissions: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[5].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            start_date: vals[6]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, TemporaryCustody> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &TemporaryCustody,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 7usize] = [
            "custodian",
            "end_date",
            "is_active",
            "owner",
            "permissions",
            "pet_id",
            "start_date",
        ];
        let vals: [Val; 7usize] = [
            (&val.custodian).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.end_date).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.is_active).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.owner).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.permissions).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.start_date).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct Medication {
    pub id: u64,
    pub pet_id: u64,
    pub name: String,
    pub dosage: String,
    pub frequency: String,
    pub start_date: u64,
    pub end_date: Option<u64>,
    pub prescribing_vet: Address,
    pub active: bool,
}
#[automatically_derived]
impl ::core::clone::Clone for Medication {
    #[inline]
    fn clone(&self) -> Medication {
        Medication {
            id: ::core::clone::Clone::clone(&self.id),
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            name: ::core::clone::Clone::clone(&self.name),
            dosage: ::core::clone::Clone::clone(&self.dosage),
            frequency: ::core::clone::Clone::clone(&self.frequency),
            start_date: ::core::clone::Clone::clone(&self.start_date),
            end_date: ::core::clone::Clone::clone(&self.end_date),
            prescribing_vet: ::core::clone::Clone::clone(&self.prescribing_vet),
            active: ::core::clone::Clone::clone(&self.active),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Medication {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "id",
            "pet_id",
            "name",
            "dosage",
            "frequency",
            "start_date",
            "end_date",
            "prescribing_vet",
            "active",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.id,
            &self.pet_id,
            &self.name,
            &self.dosage,
            &self.frequency,
            &self.start_date,
            &self.end_date,
            &self.prescribing_vet,
            &&self.active,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(
            f,
            "Medication",
            names,
            values,
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for Medication {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<String>;
        let _: ::core::cmp::AssertParamIsEq<Option<u64>>;
        let _: ::core::cmp::AssertParamIsEq<Address>;
        let _: ::core::cmp::AssertParamIsEq<bool>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Medication {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Medication {
    #[inline]
    fn eq(&self, other: &Medication) -> bool {
        self.id == other.id && self.pet_id == other.pet_id
            && self.start_date == other.start_date && self.active == other.active
            && self.name == other.name && self.dosage == other.dosage
            && self.frequency == other.frequency && self.end_date == other.end_date
            && self.prescribing_vet == other.prescribing_vet
    }
}
pub static __SPEC_XDR_TYPE_MEDICATION: [u8; 224usize] = Medication::spec_xdr();
impl Medication {
    pub const fn spec_xdr() -> [u8; 224usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\nMedication\0\0\0\0\0\t\0\0\0\0\0\0\0\x06active\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06dosage\0\0\0\0\0\x10\0\0\0\0\0\0\0\x08end_date\0\0\x03\xe8\0\0\0\x06\0\0\0\0\0\0\0\tfrequency\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x04name\0\0\0\x10\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0fprescribing_vet\0\0\0\0\x13\0\0\0\0\0\0\0\nstart_date\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for Medication {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 9usize] = [
            "active",
            "dosage",
            "end_date",
            "frequency",
            "id",
            "name",
            "pet_id",
            "prescribing_vet",
            "start_date",
        ];
        let mut vals: [Val; 9usize] = [Val::VOID.to_val(); 9usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            active: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            dosage: vals[1].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            end_date: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            frequency: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            id: vals[4].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            name: vals[5].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[6].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            prescribing_vet: vals[7]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            start_date: vals[8]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, Medication> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &Medication,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 9usize] = [
            "active",
            "dosage",
            "end_date",
            "frequency",
            "id",
            "name",
            "pet_id",
            "prescribing_vet",
            "start_date",
        ];
        let vals: [Val; 9usize] = [
            (&val.active).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.dosage).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.end_date).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.frequency).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.name).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.prescribing_vet).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.start_date).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct MedicalRecord {
    pub id: u64,
    pub pet_id: u64,
    pub vet_address: Address,
    pub diagnosis: String,
    pub treatment: String,
    pub medications: String,
    pub date: u64,
    pub notes: String,
}
#[automatically_derived]
impl ::core::clone::Clone for MedicalRecord {
    #[inline]
    fn clone(&self) -> MedicalRecord {
        MedicalRecord {
            id: ::core::clone::Clone::clone(&self.id),
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            vet_address: ::core::clone::Clone::clone(&self.vet_address),
            diagnosis: ::core::clone::Clone::clone(&self.diagnosis),
            treatment: ::core::clone::Clone::clone(&self.treatment),
            medications: ::core::clone::Clone::clone(&self.medications),
            date: ::core::clone::Clone::clone(&self.date),
            notes: ::core::clone::Clone::clone(&self.notes),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for MedicalRecord {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "id",
            "pet_id",
            "vet_address",
            "diagnosis",
            "treatment",
            "medications",
            "date",
            "notes",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.id,
            &self.pet_id,
            &self.vet_address,
            &self.diagnosis,
            &self.treatment,
            &self.medications,
            &self.date,
            &&self.notes,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(
            f,
            "MedicalRecord",
            names,
            values,
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for MedicalRecord {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<Address>;
        let _: ::core::cmp::AssertParamIsEq<String>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for MedicalRecord {}
#[automatically_derived]
impl ::core::cmp::PartialEq for MedicalRecord {
    #[inline]
    fn eq(&self, other: &MedicalRecord) -> bool {
        self.id == other.id && self.pet_id == other.pet_id && self.date == other.date
            && self.vet_address == other.vet_address && self.diagnosis == other.diagnosis
            && self.treatment == other.treatment && self.medications == other.medications
            && self.notes == other.notes
    }
}
pub static __SPEC_XDR_TYPE_MEDICALRECORD: [u8; 204usize] = MedicalRecord::spec_xdr();
impl MedicalRecord {
    pub const fn spec_xdr() -> [u8; 204usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\rMedicalRecord\0\0\0\0\0\0\x08\0\0\0\0\0\0\0\x04date\0\0\0\x06\0\0\0\0\0\0\0\tdiagnosis\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0bmedications\0\0\0\0\x10\0\0\0\0\0\0\0\x05notes\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\ttreatment\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for MedicalRecord {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 8usize] = [
            "date",
            "diagnosis",
            "id",
            "medications",
            "notes",
            "pet_id",
            "treatment",
            "vet_address",
        ];
        let mut vals: [Val; 8usize] = [Val::VOID.to_val(); 8usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            date: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            diagnosis: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            id: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            medications: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            notes: vals[4].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[5].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            treatment: vals[6]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            vet_address: vals[7]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, MedicalRecord> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &MedicalRecord,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 8usize] = [
            "date",
            "diagnosis",
            "id",
            "medications",
            "notes",
            "pet_id",
            "treatment",
            "vet_address",
        ];
        let vals: [Val; 8usize] = [
            (&val.date).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.diagnosis).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.medications).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.notes).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.treatment).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.vet_address).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct VaccinationInput {
    pub pet_id: u64,
    pub vaccine_type: VaccineType,
    pub vaccine_name: String,
    pub administered_at: u64,
    pub next_due_date: u64,
    pub batch_number: String,
}
#[automatically_derived]
impl ::core::clone::Clone for VaccinationInput {
    #[inline]
    fn clone(&self) -> VaccinationInput {
        VaccinationInput {
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            vaccine_type: ::core::clone::Clone::clone(&self.vaccine_type),
            vaccine_name: ::core::clone::Clone::clone(&self.vaccine_name),
            administered_at: ::core::clone::Clone::clone(&self.administered_at),
            next_due_date: ::core::clone::Clone::clone(&self.next_due_date),
            batch_number: ::core::clone::Clone::clone(&self.batch_number),
        }
    }
}
pub static __SPEC_XDR_TYPE_VACCINATIONINPUT: [u8; 200usize] = VaccinationInput::spec_xdr();
impl VaccinationInput {
    pub const fn spec_xdr() -> [u8; 200usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x10VaccinationInput\0\0\0\x06\0\0\0\0\0\0\0\x0fadministered_at\0\0\0\0\x06\0\0\0\0\0\0\0\x0cbatch_number\0\0\0\x10\0\0\0\0\0\0\0\rnext_due_date\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0cvaccine_name\0\0\0\x10\0\0\0\0\0\0\0\x0cvaccine_type\0\0\x07\xd0\0\0\0\x0bVaccineType\0"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for VaccinationInput {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 6usize] = [
            "administered_at",
            "batch_number",
            "next_due_date",
            "pet_id",
            "vaccine_name",
            "vaccine_type",
        ];
        let mut vals: [Val; 6usize] = [Val::VOID.to_val(); 6usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            administered_at: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            batch_number: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            next_due_date: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[3].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            vaccine_name: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            vaccine_type: vals[5]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, VaccinationInput> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &VaccinationInput,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 6usize] = [
            "administered_at",
            "batch_number",
            "next_due_date",
            "pet_id",
            "vaccine_name",
            "vaccine_type",
        ];
        let vals: [Val; 6usize] = [
            (&val.administered_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.batch_number).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.next_due_date).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.vaccine_name).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.vaccine_type).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct MedicalRecordInput {
    pub pet_id: u64,
    pub diagnosis: String,
    pub treatment: String,
    pub medications: String,
    pub notes: String,
}
#[automatically_derived]
impl ::core::clone::Clone for MedicalRecordInput {
    #[inline]
    fn clone(&self) -> MedicalRecordInput {
        MedicalRecordInput {
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            diagnosis: ::core::clone::Clone::clone(&self.diagnosis),
            treatment: ::core::clone::Clone::clone(&self.treatment),
            medications: ::core::clone::Clone::clone(&self.medications),
            notes: ::core::clone::Clone::clone(&self.notes),
        }
    }
}
pub static __SPEC_XDR_TYPE_MEDICALRECORDINPUT: [u8; 152usize] = MedicalRecordInput::spec_xdr();
impl MedicalRecordInput {
    pub const fn spec_xdr() -> [u8; 152usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x12MedicalRecordInput\0\0\0\0\0\x05\0\0\0\0\0\0\0\tdiagnosis\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x0bmedications\0\0\0\0\x10\0\0\0\0\0\0\0\x05notes\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\ttreatment\0\0\0\0\0\0\x10"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for MedicalRecordInput {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 5usize] = [
            "diagnosis",
            "medications",
            "notes",
            "pet_id",
            "treatment",
        ];
        let mut vals: [Val; 5usize] = [Val::VOID.to_val(); 5usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            diagnosis: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            medications: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            notes: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[3].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            treatment: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, MedicalRecordInput> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &MedicalRecordInput,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 5usize] = [
            "diagnosis",
            "medications",
            "notes",
            "pet_id",
            "treatment",
        ];
        let vals: [Val; 5usize] = [
            (&val.diagnosis).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.medications).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.notes).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.treatment).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct VetReview {
    pub id: u64,
    pub vet_address: Address,
    pub reviewer: Address,
    pub rating: u32,
    pub comment: String,
    pub date: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for VetReview {
    #[inline]
    fn clone(&self) -> VetReview {
        VetReview {
            id: ::core::clone::Clone::clone(&self.id),
            vet_address: ::core::clone::Clone::clone(&self.vet_address),
            reviewer: ::core::clone::Clone::clone(&self.reviewer),
            rating: ::core::clone::Clone::clone(&self.rating),
            comment: ::core::clone::Clone::clone(&self.comment),
            date: ::core::clone::Clone::clone(&self.date),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for VetReview {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "id",
            "vet_address",
            "reviewer",
            "rating",
            "comment",
            "date",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.id,
            &self.vet_address,
            &self.reviewer,
            &self.rating,
            &self.comment,
            &&self.date,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(f, "VetReview", names, values)
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for VetReview {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<Address>;
        let _: ::core::cmp::AssertParamIsEq<u32>;
        let _: ::core::cmp::AssertParamIsEq<String>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for VetReview {}
#[automatically_derived]
impl ::core::cmp::PartialEq for VetReview {
    #[inline]
    fn eq(&self, other: &VetReview) -> bool {
        self.id == other.id && self.rating == other.rating && self.date == other.date
            && self.vet_address == other.vet_address && self.reviewer == other.reviewer
            && self.comment == other.comment
    }
}
pub static __SPEC_XDR_TYPE_VETREVIEW: [u8; 148usize] = VetReview::spec_xdr();
impl VetReview {
    pub const fn spec_xdr() -> [u8; 148usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\tVetReview\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\x07comment\0\0\0\0\x10\0\0\0\0\0\0\0\x04date\0\0\0\x06\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x06rating\0\0\0\0\0\x04\0\0\0\0\0\0\0\x08reviewer\0\0\0\x13\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for VetReview {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 6usize] = [
            "comment",
            "date",
            "id",
            "rating",
            "reviewer",
            "vet_address",
        ];
        let mut vals: [Val; 6usize] = [Val::VOID.to_val(); 6usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            comment: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            date: vals[1].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            id: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            rating: vals[3].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            reviewer: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            vet_address: vals[5]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, VetReview> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &VetReview,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 6usize] = [
            "comment",
            "date",
            "id",
            "rating",
            "reviewer",
            "vet_address",
        ];
        let vals: [Val; 6usize] = [
            (&val.comment).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.date).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.rating).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.reviewer).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.vet_address).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct OwnershipRecord {
    pub pet_id: u64,
    pub previous_owner: Address,
    pub new_owner: Address,
    pub transfer_date: u64,
    pub transfer_reason: String,
}
#[automatically_derived]
impl ::core::clone::Clone for OwnershipRecord {
    #[inline]
    fn clone(&self) -> OwnershipRecord {
        OwnershipRecord {
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            previous_owner: ::core::clone::Clone::clone(&self.previous_owner),
            new_owner: ::core::clone::Clone::clone(&self.new_owner),
            transfer_date: ::core::clone::Clone::clone(&self.transfer_date),
            transfer_reason: ::core::clone::Clone::clone(&self.transfer_reason),
        }
    }
}
pub static __SPEC_XDR_TYPE_OWNERSHIPRECORD: [u8; 164usize] = OwnershipRecord::spec_xdr();
impl OwnershipRecord {
    pub const fn spec_xdr() -> [u8; 164usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x0fOwnershipRecord\0\0\0\0\x05\0\0\0\0\0\0\0\tnew_owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0eprevious_owner\0\0\0\0\0\x13\0\0\0\0\0\0\0\rtransfer_date\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0ftransfer_reason\0\0\0\0\x10"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for OwnershipRecord {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 5usize] = [
            "new_owner",
            "pet_id",
            "previous_owner",
            "transfer_date",
            "transfer_reason",
        ];
        let mut vals: [Val; 5usize] = [Val::VOID.to_val(); 5usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            new_owner: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[1].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            previous_owner: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            transfer_date: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            transfer_reason: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, OwnershipRecord> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &OwnershipRecord,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 5usize] = [
            "new_owner",
            "pet_id",
            "previous_owner",
            "transfer_date",
            "transfer_reason",
        ];
        let vals: [Val; 5usize] = [
            (&val.new_owner).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.previous_owner).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.transfer_date).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.transfer_reason).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub enum ProposalAction {
    UpgradeContract(BytesN<32>),
    VerifyVet(Address),
    RevokeVet(Address),
    ChangeAdmin((Vec<Address>, u32)),
}
#[automatically_derived]
impl ::core::clone::Clone for ProposalAction {
    #[inline]
    fn clone(&self) -> ProposalAction {
        match self {
            ProposalAction::UpgradeContract(__self_0) => {
                ProposalAction::UpgradeContract(::core::clone::Clone::clone(__self_0))
            }
            ProposalAction::VerifyVet(__self_0) => {
                ProposalAction::VerifyVet(::core::clone::Clone::clone(__self_0))
            }
            ProposalAction::RevokeVet(__self_0) => {
                ProposalAction::RevokeVet(::core::clone::Clone::clone(__self_0))
            }
            ProposalAction::ChangeAdmin(__self_0) => {
                ProposalAction::ChangeAdmin(::core::clone::Clone::clone(__self_0))
            }
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for ProposalAction {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ProposalAction::UpgradeContract(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "UpgradeContract",
                    &__self_0,
                )
            }
            ProposalAction::VerifyVet(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "VerifyVet",
                    &__self_0,
                )
            }
            ProposalAction::RevokeVet(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "RevokeVet",
                    &__self_0,
                )
            }
            ProposalAction::ChangeAdmin(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "ChangeAdmin",
                    &__self_0,
                )
            }
        }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for ProposalAction {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<BytesN<32>>;
        let _: ::core::cmp::AssertParamIsEq<Address>;
        let _: ::core::cmp::AssertParamIsEq<(Vec<Address>, u32)>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ProposalAction {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ProposalAction {
    #[inline]
    fn eq(&self, other: &ProposalAction) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (
                    ProposalAction::UpgradeContract(__self_0),
                    ProposalAction::UpgradeContract(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    ProposalAction::VerifyVet(__self_0),
                    ProposalAction::VerifyVet(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    ProposalAction::RevokeVet(__self_0),
                    ProposalAction::RevokeVet(__arg1_0),
                ) => __self_0 == __arg1_0,
                (
                    ProposalAction::ChangeAdmin(__self_0),
                    ProposalAction::ChangeAdmin(__arg1_0),
                ) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
pub static __SPEC_XDR_TYPE_PROPOSALACTION: [u8; 188usize] = ProposalAction::spec_xdr();
impl ProposalAction {
    pub const fn spec_xdr() -> [u8; 188usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\x0eProposalAction\0\0\0\0\0\x04\0\0\0\x01\0\0\0\0\0\0\0\x0fUpgradeContract\0\0\0\0\x01\0\0\x03\xee\0\0\0 \0\0\0\x01\0\0\0\0\0\0\0\tVerifyVet\0\0\0\0\0\0\x01\0\0\0\x13\0\0\0\x01\0\0\0\0\0\0\0\tRevokeVet\0\0\0\0\0\0\x01\0\0\0\x13\0\0\0\x01\0\0\0\0\0\0\0\x0bChangeAdmin\0\0\0\0\x01\0\0\x03\xed\0\0\0\x02\0\0\x03\xea\0\0\0\x13\0\0\0\x04"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for ProposalAction {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &[
            "UpgradeContract",
            "VerifyVet",
            "RevokeVet",
            "ChangeAdmin",
        ];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::UpgradeContract(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                1 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::VerifyVet(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                2 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::RevokeVet(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                3 => {
                    if iter.len() > 1usize {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::ChangeAdmin(
                        iter
                            .next()
                            .ok_or(soroban_sdk::ConversionError)??
                            .try_into_val(env)?,
                    )
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, ProposalAction> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &ProposalAction,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            ProposalAction::UpgradeContract(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"UpgradeContract")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            ProposalAction::VerifyVet(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"VerifyVet")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            ProposalAction::RevokeVet(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"RevokeVet")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            ProposalAction::ChangeAdmin(ref value0) => {
                let tup: (soroban_sdk::Val, soroban_sdk::Val) = (
                    soroban_sdk::Symbol::try_from_val(env, &"ChangeAdmin")?.to_val(),
                    value0.try_into_val(env)?,
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub struct MultiSigProposal {
    pub id: u64,
    pub action: ProposalAction,
    pub proposed_by: Address,
    pub approvals: Vec<Address>,
    pub required_approvals: u32,
    pub created_at: u64,
    pub expires_at: u64,
    pub executed: bool,
}
#[automatically_derived]
impl ::core::clone::Clone for MultiSigProposal {
    #[inline]
    fn clone(&self) -> MultiSigProposal {
        MultiSigProposal {
            id: ::core::clone::Clone::clone(&self.id),
            action: ::core::clone::Clone::clone(&self.action),
            proposed_by: ::core::clone::Clone::clone(&self.proposed_by),
            approvals: ::core::clone::Clone::clone(&self.approvals),
            required_approvals: ::core::clone::Clone::clone(&self.required_approvals),
            created_at: ::core::clone::Clone::clone(&self.created_at),
            expires_at: ::core::clone::Clone::clone(&self.expires_at),
            executed: ::core::clone::Clone::clone(&self.executed),
        }
    }
}
pub static __SPEC_XDR_TYPE_MULTISIGPROPOSAL: [u8; 244usize] = MultiSigProposal::spec_xdr();
impl MultiSigProposal {
    pub const fn spec_xdr() -> [u8; 244usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x10MultiSigProposal\0\0\0\x08\0\0\0\0\0\0\0\x06action\0\0\0\0\x07\xd0\0\0\0\x0eProposalAction\0\0\0\0\0\0\0\0\0\tapprovals\0\0\0\0\0\x03\xea\0\0\0\x13\0\0\0\0\0\0\0\ncreated_at\0\0\0\0\0\x06\0\0\0\0\0\0\0\x08executed\0\0\0\x01\0\0\0\0\0\0\0\nexpires_at\0\0\0\0\0\x06\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0bproposed_by\0\0\0\0\x13\0\0\0\0\0\0\0\x12required_approvals\0\0\0\0\0\x04"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for MultiSigProposal {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 8usize] = [
            "action",
            "approvals",
            "created_at",
            "executed",
            "expires_at",
            "id",
            "proposed_by",
            "required_approvals",
        ];
        let mut vals: [Val; 8usize] = [Val::VOID.to_val(); 8usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            action: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            approvals: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            created_at: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            executed: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            expires_at: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            id: vals[5].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            proposed_by: vals[6]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            required_approvals: vals[7]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, MultiSigProposal> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &MultiSigProposal,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 8usize] = [
            "action",
            "approvals",
            "created_at",
            "executed",
            "expires_at",
            "id",
            "proposed_by",
            "required_approvals",
        ];
        let vals: [Val; 8usize] = [
            (&val.action).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.approvals).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.created_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.executed).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.expires_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.proposed_by).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.required_approvals).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub enum TreatmentType {
    Surgery,
    Therapy,
    Emergency,
    Routine,
    Other,
}
#[automatically_derived]
impl ::core::clone::Clone for TreatmentType {
    #[inline]
    fn clone(&self) -> TreatmentType {
        match self {
            TreatmentType::Surgery => TreatmentType::Surgery,
            TreatmentType::Therapy => TreatmentType::Therapy,
            TreatmentType::Emergency => TreatmentType::Emergency,
            TreatmentType::Routine => TreatmentType::Routine,
            TreatmentType::Other => TreatmentType::Other,
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for TreatmentType {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                TreatmentType::Surgery => "Surgery",
                TreatmentType::Therapy => "Therapy",
                TreatmentType::Emergency => "Emergency",
                TreatmentType::Routine => "Routine",
                TreatmentType::Other => "Other",
            },
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for TreatmentType {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for TreatmentType {}
#[automatically_derived]
impl ::core::cmp::PartialEq for TreatmentType {
    #[inline]
    fn eq(&self, other: &TreatmentType) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
pub static __SPEC_XDR_TYPE_TREATMENTTYPE: [u8; 140usize] = TreatmentType::spec_xdr();
impl TreatmentType {
    pub const fn spec_xdr() -> [u8; 140usize] {
        *b"\0\0\0\x02\0\0\0\0\0\0\0\0\0\0\0\rTreatmentType\0\0\0\0\0\0\x05\0\0\0\0\0\0\0\0\0\0\0\x07Surgery\0\0\0\0\0\0\0\0\0\0\0\0\x07Therapy\0\0\0\0\0\0\0\0\0\0\0\0\tEmergency\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x07Routine\0\0\0\0\0\0\0\0\0\0\0\0\x05Other\0\0\0"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for TreatmentType {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{EnvBase, TryIntoVal, TryFromVal};
        const CASES: &'static [&'static str] = &[
            "Surgery",
            "Therapy",
            "Emergency",
            "Routine",
            "Other",
        ];
        let vec: soroban_sdk::Vec<soroban_sdk::Val> = val.try_into_val(env)?;
        let mut iter = vec.try_iter();
        let discriminant: soroban_sdk::Symbol = iter
            .next()
            .ok_or(soroban_sdk::ConversionError)??
            .try_into_val(env)
            .map_err(|_| soroban_sdk::ConversionError)?;
        Ok(
            match u32::from(
                env.symbol_index_in_strs(discriminant.to_symbol_val(), CASES)?,
            ) as usize
            {
                0 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Surgery
                }
                1 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Therapy
                }
                2 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Emergency
                }
                3 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Routine
                }
                4 => {
                    if iter.len() > 0 {
                        return Err(soroban_sdk::ConversionError);
                    }
                    Self::Other
                }
                _ => Err(soroban_sdk::ConversionError {})?,
            },
        )
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, TreatmentType> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    #[inline(always)]
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &TreatmentType,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, TryFromVal};
        match val {
            TreatmentType::Surgery => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Surgery")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            TreatmentType::Therapy => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Therapy")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            TreatmentType::Emergency => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Emergency")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            TreatmentType::Routine => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Routine")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
            TreatmentType::Other => {
                let tup: (soroban_sdk::Val,) = (
                    soroban_sdk::Symbol::try_from_val(env, &"Other")?.to_val(),
                );
                tup.try_into_val(env).map_err(Into::into)
            }
        }
    }
}
pub struct Treatment {
    pub id: u64,
    pub pet_id: u64,
    pub treatment_type: TreatmentType,
    pub date: u64,
    pub vet_address: Address,
    pub notes: String,
    pub cost: Option<i128>,
    pub outcome: String,
}
#[automatically_derived]
impl ::core::clone::Clone for Treatment {
    #[inline]
    fn clone(&self) -> Treatment {
        Treatment {
            id: ::core::clone::Clone::clone(&self.id),
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            treatment_type: ::core::clone::Clone::clone(&self.treatment_type),
            date: ::core::clone::Clone::clone(&self.date),
            vet_address: ::core::clone::Clone::clone(&self.vet_address),
            notes: ::core::clone::Clone::clone(&self.notes),
            cost: ::core::clone::Clone::clone(&self.cost),
            outcome: ::core::clone::Clone::clone(&self.outcome),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Treatment {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "id",
            "pet_id",
            "treatment_type",
            "date",
            "vet_address",
            "notes",
            "cost",
            "outcome",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.id,
            &self.pet_id,
            &self.treatment_type,
            &self.date,
            &self.vet_address,
            &self.notes,
            &self.cost,
            &&self.outcome,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(f, "Treatment", names, values)
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for Treatment {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<TreatmentType>;
        let _: ::core::cmp::AssertParamIsEq<Address>;
        let _: ::core::cmp::AssertParamIsEq<String>;
        let _: ::core::cmp::AssertParamIsEq<Option<i128>>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Treatment {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Treatment {
    #[inline]
    fn eq(&self, other: &Treatment) -> bool {
        self.id == other.id && self.pet_id == other.pet_id && self.date == other.date
            && self.treatment_type == other.treatment_type
            && self.vet_address == other.vet_address && self.notes == other.notes
            && self.cost == other.cost && self.outcome == other.outcome
    }
}
pub static __SPEC_XDR_TYPE_TREATMENT: [u8; 216usize] = Treatment::spec_xdr();
impl Treatment {
    pub const fn spec_xdr() -> [u8; 216usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\tTreatment\0\0\0\0\0\0\x08\0\0\0\0\0\0\0\x04cost\0\0\x03\xe8\0\0\0\x0b\0\0\0\0\0\0\0\x04date\0\0\0\x06\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x05notes\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x07outcome\0\0\0\0\x10\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0etreatment_type\0\0\0\0\x07\xd0\0\0\0\rTreatmentType\0\0\0\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for Treatment {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 8usize] = [
            "cost",
            "date",
            "id",
            "notes",
            "outcome",
            "pet_id",
            "treatment_type",
            "vet_address",
        ];
        let mut vals: [Val; 8usize] = [Val::VOID.to_val(); 8usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            cost: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            date: vals[1].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            id: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            notes: vals[3].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            outcome: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[5].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            treatment_type: vals[6]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            vet_address: vals[7]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, Treatment> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &Treatment,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 8usize] = [
            "cost",
            "date",
            "id",
            "notes",
            "outcome",
            "pet_id",
            "treatment_type",
            "vet_address",
        ];
        let vals: [Val; 8usize] = [
            (&val.cost).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.date).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.notes).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.outcome).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.treatment_type).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.vet_address).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct TreatmentAddedEvent {
    pub treatment_id: u64,
    pub pet_id: u64,
    pub vet_address: Address,
    pub treatment_type: TreatmentType,
    pub timestamp: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for TreatmentAddedEvent {
    #[inline]
    fn clone(&self) -> TreatmentAddedEvent {
        TreatmentAddedEvent {
            treatment_id: ::core::clone::Clone::clone(&self.treatment_id),
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            vet_address: ::core::clone::Clone::clone(&self.vet_address),
            treatment_type: ::core::clone::Clone::clone(&self.treatment_type),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for TreatmentAddedEvent {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field5_finish(
            f,
            "TreatmentAddedEvent",
            "treatment_id",
            &self.treatment_id,
            "pet_id",
            &self.pet_id,
            "vet_address",
            &self.vet_address,
            "treatment_type",
            &self.treatment_type,
            "timestamp",
            &&self.timestamp,
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for TreatmentAddedEvent {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<Address>;
        let _: ::core::cmp::AssertParamIsEq<TreatmentType>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for TreatmentAddedEvent {}
#[automatically_derived]
impl ::core::cmp::PartialEq for TreatmentAddedEvent {
    #[inline]
    fn eq(&self, other: &TreatmentAddedEvent) -> bool {
        self.treatment_id == other.treatment_id && self.pet_id == other.pet_id
            && self.timestamp == other.timestamp && self.vet_address == other.vet_address
            && self.treatment_type == other.treatment_type
    }
}
pub static __SPEC_XDR_TYPE_TREATMENTADDEDEVENT: [u8; 180usize] = TreatmentAddedEvent::spec_xdr();
impl TreatmentAddedEvent {
    pub const fn spec_xdr() -> [u8; 180usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x13TreatmentAddedEvent\0\0\0\0\x05\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\ttimestamp\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0ctreatment_id\0\0\0\x06\0\0\0\0\0\0\0\x0etreatment_type\0\0\0\0\x07\xd0\0\0\0\rTreatmentType\0\0\0\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>
for TreatmentAddedEvent {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 5usize] = [
            "pet_id",
            "timestamp",
            "treatment_id",
            "treatment_type",
            "vet_address",
        ];
        let mut vals: [Val; 5usize] = [Val::VOID.to_val(); 5usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            pet_id: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            timestamp: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            treatment_id: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            treatment_type: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            vet_address: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, TreatmentAddedEvent>
for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &TreatmentAddedEvent,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 5usize] = [
            "pet_id",
            "timestamp",
            "treatment_id",
            "treatment_type",
            "vet_address",
        ];
        let vals: [Val; 5usize] = [
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.timestamp).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.treatment_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.treatment_type).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.vet_address).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct InsurancePolicy {
    pub policy_id: String,
    pub provider: String,
    pub coverage_type: String,
    pub premium: u64,
    pub coverage_limit: u64,
    pub start_date: u64,
    pub expiry_date: u64,
    pub active: bool,
}
#[automatically_derived]
impl ::core::clone::Clone for InsurancePolicy {
    #[inline]
    fn clone(&self) -> InsurancePolicy {
        InsurancePolicy {
            policy_id: ::core::clone::Clone::clone(&self.policy_id),
            provider: ::core::clone::Clone::clone(&self.provider),
            coverage_type: ::core::clone::Clone::clone(&self.coverage_type),
            premium: ::core::clone::Clone::clone(&self.premium),
            coverage_limit: ::core::clone::Clone::clone(&self.coverage_limit),
            start_date: ::core::clone::Clone::clone(&self.start_date),
            expiry_date: ::core::clone::Clone::clone(&self.expiry_date),
            active: ::core::clone::Clone::clone(&self.active),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for InsurancePolicy {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "policy_id",
            "provider",
            "coverage_type",
            "premium",
            "coverage_limit",
            "start_date",
            "expiry_date",
            "active",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.policy_id,
            &self.provider,
            &self.coverage_type,
            &self.premium,
            &self.coverage_limit,
            &self.start_date,
            &self.expiry_date,
            &&self.active,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(
            f,
            "InsurancePolicy",
            names,
            values,
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for InsurancePolicy {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<String>;
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<bool>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for InsurancePolicy {}
#[automatically_derived]
impl ::core::cmp::PartialEq for InsurancePolicy {
    #[inline]
    fn eq(&self, other: &InsurancePolicy) -> bool {
        self.premium == other.premium && self.coverage_limit == other.coverage_limit
            && self.start_date == other.start_date
            && self.expiry_date == other.expiry_date && self.active == other.active
            && self.policy_id == other.policy_id && self.provider == other.provider
            && self.coverage_type == other.coverage_type
    }
}
pub static __SPEC_XDR_TYPE_INSURANCEPOLICY: [u8; 224usize] = InsurancePolicy::spec_xdr();
impl InsurancePolicy {
    pub const fn spec_xdr() -> [u8; 224usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x0fInsurancePolicy\0\0\0\0\x08\0\0\0\0\0\0\0\x06active\0\0\0\0\0\x01\0\0\0\0\0\0\0\x0ecoverage_limit\0\0\0\0\0\x06\0\0\0\0\0\0\0\rcoverage_type\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x0bexpiry_date\0\0\0\0\x06\0\0\0\0\0\0\0\tpolicy_id\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x07premium\0\0\0\0\x06\0\0\0\0\0\0\0\x08provider\0\0\0\x10\0\0\0\0\0\0\0\nstart_date\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for InsurancePolicy {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 8usize] = [
            "active",
            "coverage_limit",
            "coverage_type",
            "expiry_date",
            "policy_id",
            "premium",
            "provider",
            "start_date",
        ];
        let mut vals: [Val; 8usize] = [Val::VOID.to_val(); 8usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            active: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            coverage_limit: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            coverage_type: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            expiry_date: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            policy_id: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            premium: vals[5]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            provider: vals[6]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            start_date: vals[7]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, InsurancePolicy> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &InsurancePolicy,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 8usize] = [
            "active",
            "coverage_limit",
            "coverage_type",
            "expiry_date",
            "policy_id",
            "premium",
            "provider",
            "start_date",
        ];
        let vals: [Val; 8usize] = [
            (&val.active).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.coverage_limit).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.coverage_type).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.expiry_date).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.policy_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.premium).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.provider).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.start_date).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct InsuranceAddedEvent {
    pub pet_id: u64,
    pub policy_id: String,
    pub provider: String,
    pub timestamp: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for InsuranceAddedEvent {
    #[inline]
    fn clone(&self) -> InsuranceAddedEvent {
        InsuranceAddedEvent {
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            policy_id: ::core::clone::Clone::clone(&self.policy_id),
            provider: ::core::clone::Clone::clone(&self.provider),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for InsuranceAddedEvent {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field4_finish(
            f,
            "InsuranceAddedEvent",
            "pet_id",
            &self.pet_id,
            "policy_id",
            &self.policy_id,
            "provider",
            &self.provider,
            "timestamp",
            &&self.timestamp,
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for InsuranceAddedEvent {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<String>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for InsuranceAddedEvent {}
#[automatically_derived]
impl ::core::cmp::PartialEq for InsuranceAddedEvent {
    #[inline]
    fn eq(&self, other: &InsuranceAddedEvent) -> bool {
        self.pet_id == other.pet_id && self.timestamp == other.timestamp
            && self.policy_id == other.policy_id && self.provider == other.provider
    }
}
pub static __SPEC_XDR_TYPE_INSURANCEADDEDEVENT: [u8; 128usize] = InsuranceAddedEvent::spec_xdr();
impl InsuranceAddedEvent {
    pub const fn spec_xdr() -> [u8; 128usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x13InsuranceAddedEvent\0\0\0\0\x04\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\tpolicy_id\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x08provider\0\0\0\x10\0\0\0\0\0\0\0\ttimestamp\0\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>
for InsuranceAddedEvent {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 4usize] = [
            "pet_id",
            "policy_id",
            "provider",
            "timestamp",
        ];
        let mut vals: [Val; 4usize] = [Val::VOID.to_val(); 4usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            pet_id: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            policy_id: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            provider: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            timestamp: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, InsuranceAddedEvent>
for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &InsuranceAddedEvent,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 4usize] = [
            "pet_id",
            "policy_id",
            "provider",
            "timestamp",
        ];
        let vals: [Val; 4usize] = [
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.policy_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.provider).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.timestamp).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct InsuranceUpdatedEvent {
    pub pet_id: u64,
    pub policy_id: String,
    pub active: bool,
    pub timestamp: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for InsuranceUpdatedEvent {
    #[inline]
    fn clone(&self) -> InsuranceUpdatedEvent {
        InsuranceUpdatedEvent {
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            policy_id: ::core::clone::Clone::clone(&self.policy_id),
            active: ::core::clone::Clone::clone(&self.active),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for InsuranceUpdatedEvent {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field4_finish(
            f,
            "InsuranceUpdatedEvent",
            "pet_id",
            &self.pet_id,
            "policy_id",
            &self.policy_id,
            "active",
            &self.active,
            "timestamp",
            &&self.timestamp,
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for InsuranceUpdatedEvent {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<String>;
        let _: ::core::cmp::AssertParamIsEq<bool>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for InsuranceUpdatedEvent {}
#[automatically_derived]
impl ::core::cmp::PartialEq for InsuranceUpdatedEvent {
    #[inline]
    fn eq(&self, other: &InsuranceUpdatedEvent) -> bool {
        self.pet_id == other.pet_id && self.active == other.active
            && self.timestamp == other.timestamp && self.policy_id == other.policy_id
    }
}
pub static __SPEC_XDR_TYPE_INSURANCEUPDATEDEVENT: [u8; 132usize] = InsuranceUpdatedEvent::spec_xdr();
impl InsuranceUpdatedEvent {
    pub const fn spec_xdr() -> [u8; 132usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x15InsuranceUpdatedEvent\0\0\0\0\0\0\x04\0\0\0\0\0\0\0\x06active\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\tpolicy_id\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\ttimestamp\0\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>
for InsuranceUpdatedEvent {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 4usize] = [
            "active",
            "pet_id",
            "policy_id",
            "timestamp",
        ];
        let mut vals: [Val; 4usize] = [Val::VOID.to_val(); 4usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            active: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[1].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            policy_id: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            timestamp: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, InsuranceUpdatedEvent>
for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &InsuranceUpdatedEvent,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 4usize] = [
            "active",
            "pet_id",
            "policy_id",
            "timestamp",
        ];
        let vals: [Val; 4usize] = [
            (&val.active).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.policy_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.timestamp).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct AccessGrantedEvent {
    pub pet_id: u64,
    pub granter: Address,
    pub grantee: Address,
    pub access_level: AccessLevel,
    pub expires_at: Option<u64>,
    pub timestamp: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for AccessGrantedEvent {
    #[inline]
    fn clone(&self) -> AccessGrantedEvent {
        AccessGrantedEvent {
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            granter: ::core::clone::Clone::clone(&self.granter),
            grantee: ::core::clone::Clone::clone(&self.grantee),
            access_level: ::core::clone::Clone::clone(&self.access_level),
            expires_at: ::core::clone::Clone::clone(&self.expires_at),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
        }
    }
}
pub static __SPEC_XDR_TYPE_ACCESSGRANTEDEVENT: [u8; 192usize] = AccessGrantedEvent::spec_xdr();
impl AccessGrantedEvent {
    pub const fn spec_xdr() -> [u8; 192usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x12AccessGrantedEvent\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0caccess_level\0\0\x07\xd0\0\0\0\x0bAccessLevel\0\0\0\0\0\0\0\0\nexpires_at\0\0\0\0\x03\xe8\0\0\0\x06\0\0\0\0\0\0\0\x07grantee\0\0\0\0\x13\0\0\0\0\0\0\0\x07granter\0\0\0\0\x13\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\ttimestamp\0\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for AccessGrantedEvent {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 6usize] = [
            "access_level",
            "expires_at",
            "grantee",
            "granter",
            "pet_id",
            "timestamp",
        ];
        let mut vals: [Val; 6usize] = [Val::VOID.to_val(); 6usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            access_level: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            expires_at: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            grantee: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            granter: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[4].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            timestamp: vals[5]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, AccessGrantedEvent> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &AccessGrantedEvent,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 6usize] = [
            "access_level",
            "expires_at",
            "grantee",
            "granter",
            "pet_id",
            "timestamp",
        ];
        let vals: [Val; 6usize] = [
            (&val.access_level).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.expires_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.grantee).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.granter).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.timestamp).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct AccessRevokedEvent {
    pub pet_id: u64,
    pub granter: Address,
    pub grantee: Address,
    pub timestamp: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for AccessRevokedEvent {
    #[inline]
    fn clone(&self) -> AccessRevokedEvent {
        AccessRevokedEvent {
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            granter: ::core::clone::Clone::clone(&self.granter),
            grantee: ::core::clone::Clone::clone(&self.grantee),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
        }
    }
}
pub static __SPEC_XDR_TYPE_ACCESSREVOKEDEVENT: [u8; 124usize] = AccessRevokedEvent::spec_xdr();
impl AccessRevokedEvent {
    pub const fn spec_xdr() -> [u8; 124usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x12AccessRevokedEvent\0\0\0\0\0\x04\0\0\0\0\0\0\0\x07grantee\0\0\0\0\x13\0\0\0\0\0\0\0\x07granter\0\0\0\0\x13\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\ttimestamp\0\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for AccessRevokedEvent {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 4usize] = [
            "grantee",
            "granter",
            "pet_id",
            "timestamp",
        ];
        let mut vals: [Val; 4usize] = [Val::VOID.to_val(); 4usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            grantee: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            granter: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            timestamp: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, AccessRevokedEvent> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &AccessRevokedEvent,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 4usize] = [
            "grantee",
            "granter",
            "pet_id",
            "timestamp",
        ];
        let vals: [Val; 4usize] = [
            (&val.grantee).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.granter).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.timestamp).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct AccessExpiredEvent {
    pub pet_id: u64,
    pub grantee: Address,
    pub expired_at: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for AccessExpiredEvent {
    #[inline]
    fn clone(&self) -> AccessExpiredEvent {
        AccessExpiredEvent {
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            grantee: ::core::clone::Clone::clone(&self.grantee),
            expired_at: ::core::clone::Clone::clone(&self.expired_at),
        }
    }
}
pub static __SPEC_XDR_TYPE_ACCESSEXPIREDEVENT: [u8; 104usize] = AccessExpiredEvent::spec_xdr();
impl AccessExpiredEvent {
    pub const fn spec_xdr() -> [u8; 104usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x12AccessExpiredEvent\0\0\0\0\0\x03\0\0\0\0\0\0\0\nexpired_at\0\0\0\0\0\x06\0\0\0\0\0\0\0\x07grantee\0\0\0\0\x13\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for AccessExpiredEvent {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 3usize] = ["expired_at", "grantee", "pet_id"];
        let mut vals: [Val; 3usize] = [Val::VOID.to_val(); 3usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            expired_at: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            grantee: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, AccessExpiredEvent> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &AccessExpiredEvent,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 3usize] = ["expired_at", "grantee", "pet_id"];
        let vals: [Val; 3usize] = [
            (&val.expired_at).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.grantee).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct PetRegisteredEvent {
    pub pet_id: u64,
    pub owner: Address,
    pub name: String,
    pub species: Species,
    pub timestamp: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for PetRegisteredEvent {
    #[inline]
    fn clone(&self) -> PetRegisteredEvent {
        PetRegisteredEvent {
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            owner: ::core::clone::Clone::clone(&self.owner),
            name: ::core::clone::Clone::clone(&self.name),
            species: ::core::clone::Clone::clone(&self.species),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for PetRegisteredEvent {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field5_finish(
            f,
            "PetRegisteredEvent",
            "pet_id",
            &self.pet_id,
            "owner",
            &self.owner,
            "name",
            &self.name,
            "species",
            &self.species,
            "timestamp",
            &&self.timestamp,
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PetRegisteredEvent {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<Address>;
        let _: ::core::cmp::AssertParamIsEq<String>;
        let _: ::core::cmp::AssertParamIsEq<Species>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PetRegisteredEvent {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PetRegisteredEvent {
    #[inline]
    fn eq(&self, other: &PetRegisteredEvent) -> bool {
        self.pet_id == other.pet_id && self.timestamp == other.timestamp
            && self.owner == other.owner && self.name == other.name
            && self.species == other.species
    }
}
pub static __SPEC_XDR_TYPE_PETREGISTEREDEVENT: [u8; 152usize] = PetRegisteredEvent::spec_xdr();
impl PetRegisteredEvent {
    pub const fn spec_xdr() -> [u8; 152usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x12PetRegisteredEvent\0\0\0\0\0\x05\0\0\0\0\0\0\0\x04name\0\0\0\x10\0\0\0\0\0\0\0\x05owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x07species\0\0\0\x07\xd0\0\0\0\x07Species\0\0\0\0\0\0\0\0\ttimestamp\0\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> for PetRegisteredEvent {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 5usize] = [
            "name",
            "owner",
            "pet_id",
            "species",
            "timestamp",
        ];
        let mut vals: [Val; 5usize] = [Val::VOID.to_val(); 5usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            name: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            owner: vals[1].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            species: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            timestamp: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, PetRegisteredEvent> for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &PetRegisteredEvent,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 5usize] = [
            "name",
            "owner",
            "pet_id",
            "species",
            "timestamp",
        ];
        let vals: [Val; 5usize] = [
            (&val.name).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.owner).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.species).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.timestamp).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct VaccinationAddedEvent {
    pub vaccine_id: u64,
    pub pet_id: u64,
    pub veterinarian: Address,
    pub vaccine_type: VaccineType,
    pub next_due_date: u64,
    pub timestamp: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for VaccinationAddedEvent {
    #[inline]
    fn clone(&self) -> VaccinationAddedEvent {
        VaccinationAddedEvent {
            vaccine_id: ::core::clone::Clone::clone(&self.vaccine_id),
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            veterinarian: ::core::clone::Clone::clone(&self.veterinarian),
            vaccine_type: ::core::clone::Clone::clone(&self.vaccine_type),
            next_due_date: ::core::clone::Clone::clone(&self.next_due_date),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for VaccinationAddedEvent {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "vaccine_id",
            "pet_id",
            "veterinarian",
            "vaccine_type",
            "next_due_date",
            "timestamp",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.vaccine_id,
            &self.pet_id,
            &self.veterinarian,
            &self.vaccine_type,
            &self.next_due_date,
            &&self.timestamp,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(
            f,
            "VaccinationAddedEvent",
            names,
            values,
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for VaccinationAddedEvent {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<Address>;
        let _: ::core::cmp::AssertParamIsEq<VaccineType>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for VaccinationAddedEvent {}
#[automatically_derived]
impl ::core::cmp::PartialEq for VaccinationAddedEvent {
    #[inline]
    fn eq(&self, other: &VaccinationAddedEvent) -> bool {
        self.vaccine_id == other.vaccine_id && self.pet_id == other.pet_id
            && self.next_due_date == other.next_due_date
            && self.timestamp == other.timestamp
            && self.veterinarian == other.veterinarian
            && self.vaccine_type == other.vaccine_type
    }
}
pub static __SPEC_XDR_TYPE_VACCINATIONADDEDEVENT: [u8; 204usize] = VaccinationAddedEvent::spec_xdr();
impl VaccinationAddedEvent {
    pub const fn spec_xdr() -> [u8; 204usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x15VaccinationAddedEvent\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\rnext_due_date\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\ttimestamp\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\nvaccine_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0cvaccine_type\0\0\x07\xd0\0\0\0\x0bVaccineType\0\0\0\0\0\0\0\0\x0cveterinarian\0\0\0\x13"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>
for VaccinationAddedEvent {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 6usize] = [
            "next_due_date",
            "pet_id",
            "timestamp",
            "vaccine_id",
            "vaccine_type",
            "veterinarian",
        ];
        let mut vals: [Val; 6usize] = [Val::VOID.to_val(); 6usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            next_due_date: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[1].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            timestamp: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            vaccine_id: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            vaccine_type: vals[4]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            veterinarian: vals[5]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, VaccinationAddedEvent>
for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &VaccinationAddedEvent,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 6usize] = [
            "next_due_date",
            "pet_id",
            "timestamp",
            "vaccine_id",
            "vaccine_type",
            "veterinarian",
        ];
        let vals: [Val; 6usize] = [
            (&val.next_due_date).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.timestamp).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.vaccine_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.vaccine_type).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.veterinarian).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct PetOwnershipTransferredEvent {
    pub pet_id: u64,
    pub old_owner: Address,
    pub new_owner: Address,
    pub timestamp: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for PetOwnershipTransferredEvent {
    #[inline]
    fn clone(&self) -> PetOwnershipTransferredEvent {
        PetOwnershipTransferredEvent {
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            old_owner: ::core::clone::Clone::clone(&self.old_owner),
            new_owner: ::core::clone::Clone::clone(&self.new_owner),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for PetOwnershipTransferredEvent {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field4_finish(
            f,
            "PetOwnershipTransferredEvent",
            "pet_id",
            &self.pet_id,
            "old_owner",
            &self.old_owner,
            "new_owner",
            &self.new_owner,
            "timestamp",
            &&self.timestamp,
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PetOwnershipTransferredEvent {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<Address>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PetOwnershipTransferredEvent {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PetOwnershipTransferredEvent {
    #[inline]
    fn eq(&self, other: &PetOwnershipTransferredEvent) -> bool {
        self.pet_id == other.pet_id && self.timestamp == other.timestamp
            && self.old_owner == other.old_owner && self.new_owner == other.new_owner
    }
}
pub static __SPEC_XDR_TYPE_PETOWNERSHIPTRANSFERREDEVENT: [u8; 140usize] = PetOwnershipTransferredEvent::spec_xdr();
impl PetOwnershipTransferredEvent {
    pub const fn spec_xdr() -> [u8; 140usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x1cPetOwnershipTransferredEvent\0\0\0\x04\0\0\0\0\0\0\0\tnew_owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\told_owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\ttimestamp\0\0\0\0\0\0\x06"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>
for PetOwnershipTransferredEvent {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 4usize] = [
            "new_owner",
            "old_owner",
            "pet_id",
            "timestamp",
        ];
        let mut vals: [Val; 4usize] = [Val::VOID.to_val(); 4usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            new_owner: vals[0]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            old_owner: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            pet_id: vals[2].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            timestamp: vals[3]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, PetOwnershipTransferredEvent>
for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &PetOwnershipTransferredEvent,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 4usize] = [
            "new_owner",
            "old_owner",
            "pet_id",
            "timestamp",
        ];
        let vals: [Val; 4usize] = [
            (&val.new_owner).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.old_owner).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.timestamp).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct MedicalRecordAddedEvent {
    pub pet_id: u64,
    pub updated_by: Address,
    pub timestamp: u64,
}
#[automatically_derived]
impl ::core::clone::Clone for MedicalRecordAddedEvent {
    #[inline]
    fn clone(&self) -> MedicalRecordAddedEvent {
        MedicalRecordAddedEvent {
            pet_id: ::core::clone::Clone::clone(&self.pet_id),
            updated_by: ::core::clone::Clone::clone(&self.updated_by),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for MedicalRecordAddedEvent {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "MedicalRecordAddedEvent",
            "pet_id",
            &self.pet_id,
            "updated_by",
            &self.updated_by,
            "timestamp",
            &&self.timestamp,
        )
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for MedicalRecordAddedEvent {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<Address>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for MedicalRecordAddedEvent {}
#[automatically_derived]
impl ::core::cmp::PartialEq for MedicalRecordAddedEvent {
    #[inline]
    fn eq(&self, other: &MedicalRecordAddedEvent) -> bool {
        self.pet_id == other.pet_id && self.timestamp == other.timestamp
            && self.updated_by == other.updated_by
    }
}
pub static __SPEC_XDR_TYPE_MEDICALRECORDADDEDEVENT: [u8; 112usize] = MedicalRecordAddedEvent::spec_xdr();
impl MedicalRecordAddedEvent {
    pub const fn spec_xdr() -> [u8; 112usize] {
        *b"\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\x17MedicalRecordAddedEvent\0\0\0\0\x03\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\ttimestamp\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\nupdated_by\0\0\0\0\0\x13"
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>
for MedicalRecordAddedEvent {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &soroban_sdk::Val,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val, MapObject};
        const KEYS: [&'static str; 3usize] = ["pet_id", "timestamp", "updated_by"];
        let mut vals: [Val; 3usize] = [Val::VOID.to_val(); 3usize];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals).map_err(|_| ConversionError)?;
        Ok(Self {
            pet_id: vals[0].try_into_val(env).map_err(|_| soroban_sdk::ConversionError)?,
            timestamp: vals[1]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
            updated_by: vals[2]
                .try_into_val(env)
                .map_err(|_| soroban_sdk::ConversionError)?,
        })
    }
}
impl soroban_sdk::TryFromVal<soroban_sdk::Env, MedicalRecordAddedEvent>
for soroban_sdk::Val {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(
        env: &soroban_sdk::Env,
        val: &MedicalRecordAddedEvent,
    ) -> Result<Self, soroban_sdk::ConversionError> {
        use soroban_sdk::{TryIntoVal, EnvBase, ConversionError, Val};
        const KEYS: [&'static str; 3usize] = ["pet_id", "timestamp", "updated_by"];
        let vals: [Val; 3usize] = [
            (&val.pet_id).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.timestamp).try_into_val(env).map_err(|_| ConversionError)?,
            (&val.updated_by).try_into_val(env).map_err(|_| ConversionError)?,
        ];
        Ok(env.map_new_from_slices(&KEYS, &vals).map_err(|_| ConversionError)?.into())
    }
}
pub struct PetChainContract;
///PetChainContractClient is a client for calling the contract defined in "PetChainContract".
pub struct PetChainContractClient<'a> {
    pub env: soroban_sdk::Env,
    pub address: soroban_sdk::Address,
    #[doc(hidden)]
    _phantom: core::marker::PhantomData<&'a ()>,
}
impl<'a> PetChainContractClient<'a> {
    pub fn new(env: &soroban_sdk::Env, address: &soroban_sdk::Address) -> Self {
        Self {
            env: env.clone(),
            address: address.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}
impl PetChainContract {
    fn log_access(
        env: &Env,
        pet_id: u64,
        user: Address,
        action: AccessAction,
        details: String,
    ) {
        let key = (Symbol::new(env, "access_logs"), pet_id);
        let mut logs: Vec<AccessLog> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        let id = logs.len() as u64;
        let log = AccessLog {
            id,
            pet_id,
            user,
            action,
            timestamp: env.ledger().timestamp(),
            details,
        };
        logs.push_back(log);
        env.storage().persistent().set(&key, &logs);
    }
    fn require_admin(env: &Env) {
        if let Some(legacy_admin) = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
        {
            legacy_admin.require_auth();
            return;
        }
        let admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&SystemKey::Admins)
            .expect("Admins not set");
        if admins.is_empty() {
            {
                ::core::panicking::panic_fmt(format_args!("No admins configured"));
            };
        }
        let admin = admins.get(0).expect("No admins configured");
        admin.require_auth();
    }
    fn require_admin_auth(env: &Env, admin: &Address) {
        if let Some(legacy_admin) = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
        {
            if &legacy_admin == admin {
                admin.require_auth();
                return;
            }
        }
        let admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&SystemKey::Admins)
            .expect("Admins not set");
        if !admins.contains(admin.clone()) {
            {
                ::core::panicking::panic_fmt(format_args!("Address is not an admin"));
            };
        }
        admin.require_auth();
    }
    pub fn init_admin(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin)
            || env.storage().instance().has(&SystemKey::Admins)
        {
            {
                ::core::panicking::panic_fmt(format_args!("Admin already set"));
            };
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }
    pub fn init_multisig(
        env: Env,
        invoker: Address,
        admins: Vec<Address>,
        threshold: u32,
    ) {
        if env.storage().instance().has(&DataKey::Admin)
            || env.storage().instance().has(&SystemKey::Admins)
        {
            {
                ::core::panicking::panic_fmt(format_args!("Admin already set"));
            };
        }
        if threshold == 0 || threshold > admins.len() {
            {
                ::core::panicking::panic_fmt(format_args!("Invalid threshold"));
            };
        }
        invoker.require_auth();
        if !admins.contains(invoker) {
            {
                ::core::panicking::panic_fmt(
                    format_args!("Invoker must be in the initial admin list"),
                );
            };
        }
        env.storage().instance().set(&SystemKey::Admins, &admins);
        env.storage().instance().set(&SystemKey::AdminThreshold, &threshold);
    }
    #[allow(clippy::too_many_arguments)]
    pub fn register_pet(
        env: Env,
        owner: Address,
        name: String,
        birthday: String,
        gender: Gender,
        species: Species,
        breed: String,
        color: String,
        weight: u32,
        microchip_id: Option<String>,
        privacy_level: PrivacyLevel,
    ) -> u64 {
        owner.require_auth();
        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetCount)
            .unwrap_or(0);
        let pet_id = pet_count + 1;
        let timestamp = env.ledger().timestamp();
        let key = Self::get_encryption_key(&env);
        let name_bytes = name.to_xdr(&env);
        let (name_nonce, name_ciphertext) = encrypt_sensitive_data(
            &env,
            &name_bytes,
            &key,
        );
        let encrypted_name = EncryptedData {
            nonce: name_nonce,
            ciphertext: name_ciphertext,
        };
        let birthday_bytes = birthday.to_xdr(&env);
        let (birthday_nonce, birthday_ciphertext) = encrypt_sensitive_data(
            &env,
            &birthday_bytes,
            &key,
        );
        let encrypted_birthday = EncryptedData {
            nonce: birthday_nonce,
            ciphertext: birthday_ciphertext,
        };
        let breed_bytes = breed.to_xdr(&env);
        let (breed_nonce, breed_ciphertext) = encrypt_sensitive_data(
            &env,
            &breed_bytes,
            &key,
        );
        let encrypted_breed = EncryptedData {
            nonce: breed_nonce,
            ciphertext: breed_ciphertext,
        };
        let empty_alerts_bytes = Bytes::from_slice(&env, "".as_bytes());
        let (alerts_nonce, alerts_ciphertext) = encrypt_sensitive_data(
            &env,
            &empty_alerts_bytes,
            &key,
        );
        let encrypted_medical_alerts = EncryptedData {
            nonce: alerts_nonce,
            ciphertext: alerts_ciphertext,
        };
        let empty_contacts = Vec::<EmergencyContact>::new(&env);
        let contacts_bytes = empty_contacts.to_xdr(&env);
        let (contacts_nonce, contacts_ciphertext) = encrypt_sensitive_data(
            &env,
            &contacts_bytes,
            &key,
        );
        let encrypted_emergency_contacts = EncryptedData {
            nonce: contacts_nonce,
            ciphertext: contacts_ciphertext,
        };
        let pet = Pet {
            id: pet_id,
            owner: owner.clone(),
            privacy_level,
            encrypted_name,
            encrypted_birthday,
            encrypted_breed,
            encrypted_emergency_contacts,
            encrypted_medical_alerts,
            name: String::from_str(&env, ""),
            birthday: String::from_str(&env, ""),
            breed: String::from_str(&env, ""),
            emergency_contacts: Vec::<EmergencyContact>::new(&env),
            medical_alerts: String::from_str(&env, ""),
            active: false,
            created_at: timestamp,
            updated_at: timestamp,
            new_owner: owner.clone(),
            species: species.clone(),
            gender,
            color,
            weight,
            microchip_id,
            photo_hashes: Vec::new(&env),
        };
        env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        env.storage().instance().set(&DataKey::PetCount, &pet_id);
        Self::log_ownership_change(
            &env,
            pet_id,
            owner.clone(),
            owner.clone(),
            String::from_str(&env, "Initial Registration"),
        );
        let owner_pet_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetCountByOwner(owner.clone()))
            .unwrap_or(0) + 1;
        env.storage()
            .instance()
            .set(&DataKey::PetCountByOwner(owner.clone()), &owner_pet_count);
        env.storage()
            .instance()
            .set(&DataKey::OwnerPetIndex((owner.clone(), owner_pet_count)), &pet_id);
        let species_key = Self::species_to_string(&env, &species);
        let species_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::SpeciesPetCount(species_key.clone()))
            .unwrap_or(0) + 1;
        env.storage()
            .instance()
            .set(&DataKey::SpeciesPetCount(species_key.clone()), &species_count);
        env.storage()
            .instance()
            .set(&DataKey::SpeciesPetIndex((species_key, species_count)), &pet_id);
        env.events()
            .publish(
                (String::from_str(&env, "PetRegistered"), pet_id),
                PetRegisteredEvent {
                    pet_id,
                    owner,
                    name: String::from_str(&env, "PROTECTED"),
                    species,
                    timestamp,
                },
            );
        pet_id
    }
    #[allow(clippy::too_many_arguments)]
    pub fn update_pet_profile(
        env: Env,
        id: u64,
        name: String,
        birthday: String,
        gender: Gender,
        species: Species,
        breed: String,
        color: String,
        weight: u32,
        microchip_id: Option<String>,
        privacy_level: PrivacyLevel,
    ) -> bool {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            pet.owner.require_auth();
            let key = Self::get_encryption_key(&env);
            let name_bytes = name.to_xdr(&env);
            let (name_nonce, name_ciphertext) = encrypt_sensitive_data(
                &env,
                &name_bytes,
                &key,
            );
            pet.encrypted_name = EncryptedData {
                nonce: name_nonce,
                ciphertext: name_ciphertext,
            };
            let birthday_bytes = birthday.to_xdr(&env);
            let (birthday_nonce, birthday_ciphertext) = encrypt_sensitive_data(
                &env,
                &birthday_bytes,
                &key,
            );
            pet.encrypted_birthday = EncryptedData {
                nonce: birthday_nonce,
                ciphertext: birthday_ciphertext,
            };
            let breed_bytes = breed.to_xdr(&env);
            let (breed_nonce, breed_ciphertext) = encrypt_sensitive_data(
                &env,
                &breed_bytes,
                &key,
            );
            pet.encrypted_breed = EncryptedData {
                nonce: breed_nonce,
                ciphertext: breed_ciphertext,
            };
            pet.gender = gender;
            pet.species = species;
            pet.privacy_level = privacy_level;
            pet.color = color;
            pet.weight = weight;
            pet.microchip_id = microchip_id;
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(id), &pet);
            Self::log_access(
                &env,
                id,
                pet.owner,
                AccessAction::Write,
                String::from_str(&env, "Pet profile updated"),
            );
            true
        } else {
            false
        }
    }
    pub fn get_pet(env: Env, id: u64) -> Option<PetProfile> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            let _current_user = env.current_contract_address();
            let _is_authorized_for_full_data = false;
            let key = Self::get_encryption_key(&env);
            let decrypted_name = decrypt_sensitive_data(
                    &env,
                    &pet.encrypted_name.ciphertext,
                    &pet.encrypted_name.nonce,
                    &key,
                )
                .unwrap_or(Bytes::new(&env));
            let name = String::from_xdr(&env, &decrypted_name)
                .unwrap_or(String::from_str(&env, "Error"));
            let decrypted_birthday = decrypt_sensitive_data(
                    &env,
                    &pet.encrypted_birthday.ciphertext,
                    &pet.encrypted_birthday.nonce,
                    &key,
                )
                .unwrap_or(Bytes::new(&env));
            let birthday = String::from_xdr(&env, &decrypted_birthday)
                .unwrap_or(String::from_str(&env, "Error"));
            let decrypted_breed = decrypt_sensitive_data(
                    &env,
                    &pet.encrypted_breed.ciphertext,
                    &pet.encrypted_breed.nonce,
                    &key,
                )
                .unwrap_or(Bytes::new(&env));
            let breed = String::from_xdr(&env, &decrypted_breed)
                .unwrap_or(String::from_str(&env, "Error"));
            let profile = PetProfile {
                id: pet.id,
                owner: pet.owner,
                privacy_level: pet.privacy_level,
                name,
                birthday,
                active: pet.active,
                created_at: pet.created_at,
                updated_at: pet.updated_at,
                new_owner: pet.new_owner,
                species: pet.species,
                gender: pet.gender,
                breed,
                color: pet.color,
                weight: pet.weight,
                microchip_id: pet.microchip_id,
            };
            Self::log_access(
                &env,
                id,
                env.current_contract_address(),
                AccessAction::Read,
                String::from_str(&env, "Pet profile accessed"),
            );
            Some(profile)
        } else {
            None
        }
    }
    pub fn is_pet_active(env: Env, id: u64) -> bool {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            pet.active
        } else {
            false
        }
    }
    pub fn get_pet_owner(env: Env, id: u64) -> Option<Address> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            Some(pet.owner)
        } else {
            None
        }
    }
    pub fn activate_pet(env: Env, id: u64) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            pet.active = true;
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(id), &pet);
        }
    }
    pub fn deactivate_pet(env: Env, id: u64) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            pet.owner.require_auth();
            pet.active = false;
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(id), &pet);
        }
    }
    pub fn add_pet_photo(env: Env, pet_id: u64, photo_hash: String) -> bool {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.owner.require_auth();
            Self::validate_ipfs_hash(&photo_hash);
            pet.photo_hashes.push_back(photo_hash);
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
            true
        } else {
            false
        }
    }
    pub fn get_pet_photos(env: Env, pet_id: u64) -> Vec<String> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.photo_hashes
        } else {
            Vec::new(&env)
        }
    }
    pub fn transfer_pet_ownership(env: Env, id: u64, to: Address) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            pet.owner.require_auth();
            pet.new_owner = to;
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(id), &pet);
        }
    }
    pub fn accept_pet_transfer(env: Env, id: u64) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            pet.new_owner.require_auth();
            let old_owner = pet.owner.clone();
            Self::remove_pet_from_owner_index(&env, &old_owner, id);
            pet.owner = pet.new_owner.clone();
            pet.updated_at = env.ledger().timestamp();
            Self::add_pet_to_owner_index(&env, &pet.owner, id);
            env.storage().instance().set(&DataKey::Pet(id), &pet);
            Self::log_ownership_change(
                &env,
                id,
                old_owner.clone(),
                pet.owner.clone(),
                String::from_str(&env, "Ownership Transfer"),
            );
            env.events()
                .publish(
                    (String::from_str(&env, "PetOwnershipTransferred"), id),
                    PetOwnershipTransferredEvent {
                        pet_id: id,
                        old_owner,
                        new_owner: pet.owner.clone(),
                        timestamp: pet.updated_at,
                    },
                );
        }
    }
    fn remove_pet_from_owner_index(env: &Env, owner: &Address, pet_id: u64) {
        let count = Self::get_owner_pet_count(env, owner);
        if count == 0 {
            return;
        }
        let mut remove_index: Option<u64> = None;
        for i in 1..=count {
            if let Some(pid) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::OwnerPetIndex((owner.clone(), i)))
            {
                if pid == pet_id {
                    remove_index = Some(i);
                    break;
                }
            }
        }
        if let Some(idx) = remove_index {
            if idx != count {
                let last_pet_id = env
                    .storage()
                    .instance()
                    .get::<DataKey, u64>(&DataKey::OwnerPetIndex((owner.clone(), count)))
                    .unwrap();
                env.storage()
                    .instance()
                    .set(&DataKey::OwnerPetIndex((owner.clone(), idx)), &last_pet_id);
            }
            env.storage()
                .instance()
                .remove(&DataKey::OwnerPetIndex((owner.clone(), count)));
            env.storage()
                .instance()
                .set(&DataKey::PetCountByOwner(owner.clone()), &(count - 1));
        }
    }
    fn add_pet_to_owner_index(env: &Env, owner: &Address, pet_id: u64) {
        let count = Self::get_owner_pet_count(env, owner);
        let new_count = count + 1;
        env.storage()
            .instance()
            .set(&DataKey::PetCountByOwner(owner.clone()), &new_count);
        env.storage()
            .instance()
            .set(&DataKey::OwnerPetIndex((owner.clone(), new_count)), &pet_id);
    }
    pub fn register_pet_owner(
        env: Env,
        owner: Address,
        name: String,
        email: String,
        emergency_contact: String,
    ) {
        owner.require_auth();
        let key = Self::get_encryption_key(&env);
        let timestamp = env.ledger().timestamp();
        let name_bytes = name.to_xdr(&env);
        let (name_nonce, name_ciphertext) = encrypt_sensitive_data(
            &env,
            &name_bytes,
            &key,
        );
        let encrypted_name = EncryptedData {
            nonce: name_nonce,
            ciphertext: name_ciphertext,
        };
        let email_bytes = email.to_xdr(&env);
        let (email_nonce, email_ciphertext) = encrypt_sensitive_data(
            &env,
            &email_bytes,
            &key,
        );
        let encrypted_email = EncryptedData {
            nonce: email_nonce,
            ciphertext: email_ciphertext,
        };
        let contact_bytes = emergency_contact.to_xdr(&env);
        let (contact_nonce, contact_ciphertext) = encrypt_sensitive_data(
            &env,
            &contact_bytes,
            &key,
        );
        let encrypted_emergency_contact = EncryptedData {
            nonce: contact_nonce,
            ciphertext: contact_ciphertext,
        };
        let pet_owner = PetOwner {
            owner_address: owner.clone(),
            privacy_level: PrivacyLevel::Public,
            encrypted_name,
            encrypted_email,
            encrypted_emergency_contact,
            created_at: timestamp,
            updated_at: timestamp,
            is_pet_owner: true,
        };
        env.storage().instance().set(&DataKey::PetOwner(owner), &pet_owner);
    }
    pub fn is_owner_registered(env: Env, owner: Address) -> bool {
        if let Some(pet_owner) = env
            .storage()
            .instance()
            .get::<DataKey, PetOwner>(&DataKey::PetOwner(owner))
        {
            pet_owner.is_pet_owner
        } else {
            false
        }
    }
    pub fn update_owner_profile(
        env: Env,
        owner: Address,
        name: String,
        email: String,
        emergency_contact: String,
    ) -> bool {
        owner.require_auth();
        if let Some(mut pet_owner) = env
            .storage()
            .instance()
            .get::<DataKey, PetOwner>(&DataKey::PetOwner(owner.clone()))
        {
            let key = Self::get_encryption_key(&env);
            let name_bytes = name.to_xdr(&env);
            let (name_nonce, name_ciphertext) = encrypt_sensitive_data(
                &env,
                &name_bytes,
                &key,
            );
            pet_owner.encrypted_name = EncryptedData {
                nonce: name_nonce,
                ciphertext: name_ciphertext,
            };
            let email_bytes = email.to_xdr(&env);
            let (email_nonce, email_ciphertext) = encrypt_sensitive_data(
                &env,
                &email_bytes,
                &key,
            );
            pet_owner.encrypted_email = EncryptedData {
                nonce: email_nonce,
                ciphertext: email_ciphertext,
            };
            let contact_bytes = emergency_contact.to_xdr(&env);
            let (contact_nonce, contact_ciphertext) = encrypt_sensitive_data(
                &env,
                &contact_bytes,
                &key,
            );
            pet_owner.encrypted_emergency_contact = EncryptedData {
                nonce: contact_nonce,
                ciphertext: contact_ciphertext,
            };
            pet_owner.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::PetOwner(owner), &pet_owner);
            true
        } else {
            false
        }
    }
    pub fn register_vet(
        env: Env,
        vet_address: Address,
        name: String,
        license_number: String,
        specialization: String,
    ) -> bool {
        vet_address.require_auth();
        if env.storage().instance().has(&DataKey::VetLicense(license_number.clone())) {
            {
                ::core::panicking::panic_fmt(format_args!("License already registered"));
            };
        }
        if env.storage().instance().has(&DataKey::Vet(vet_address.clone())) {
            {
                ::core::panicking::panic_fmt(format_args!("Vet already registered"));
            };
        }
        let vet = Vet {
            address: vet_address.clone(),
            name,
            license_number: license_number.clone(),
            specialization,
            specializations: Vec::new(&env),
            certifications: Vec::new(&env),
            verified: false,
            clinic_info: None,
        };
        env.storage().instance().set(&DataKey::Vet(vet_address.clone()), &vet);
        env.storage().instance().set(&DataKey::VetLicense(license_number), &vet_address);
        true
    }
    pub fn verify_vet(env: Env, admin: Address, vet_address: Address) -> bool {
        Self::require_admin_auth(&env, &admin);
        Self::_verify_vet_internal(&env, vet_address)
    }
    fn _verify_vet_internal(env: &Env, vet_address: Address) -> bool {
        if let Some(mut vet) = env
            .storage()
            .instance()
            .get::<DataKey, Vet>(&DataKey::Vet(vet_address))
        {
            vet.verified = true;
            env.storage().instance().set(&DataKey::Vet(vet.address.clone()), &vet);
            true
        } else {
            false
        }
    }
    pub fn revoke_vet_license(env: Env, admin: Address, vet_address: Address) -> bool {
        Self::require_admin_auth(&env, &admin);
        Self::_revoke_vet_internal(&env, vet_address)
    }
    fn _revoke_vet_internal(env: &Env, vet_address: Address) -> bool {
        if let Some(mut vet) = env
            .storage()
            .instance()
            .get::<DataKey, Vet>(&DataKey::Vet(vet_address))
        {
            vet.verified = false;
            env.storage().instance().set(&DataKey::Vet(vet.address.clone()), &vet);
            true
        } else {
            false
        }
    }
    pub fn is_verified_vet(env: Env, vet_address: Address) -> bool {
        env.storage()
            .instance()
            .get::<DataKey, Vet>(&DataKey::Vet(vet_address))
            .map(|vet| vet.verified)
            .unwrap_or(false)
    }
    pub fn get_vet(env: Env, vet_address: Address) -> Option<Vet> {
        env.storage().instance().get(&DataKey::Vet(vet_address))
    }
    pub fn get_vet_by_license(env: Env, license_number: String) -> Option<Vet> {
        let vet_address: Option<Address> = env
            .storage()
            .instance()
            .get(&DataKey::VetLicense(license_number));
        vet_address.and_then(|address| Self::get_vet(env, address))
    }
    /// Update clinic info for a vet. Only the vet can update their own clinic info.
    pub fn update_clinic_info(
        env: Env,
        vet_address: Address,
        clinic_info: ClinicInfo,
    ) -> bool {
        vet_address.require_auth();
        if let Some(mut vet) = env
            .storage()
            .instance()
            .get::<_, Vet>(&DataKey::Vet(vet_address.clone()))
        {
            vet.clinic_info = Some(clinic_info);
            env.storage().instance().set(&DataKey::Vet(vet_address), &vet);
            true
        } else {
            {
                ::core::panicking::panic_fmt(format_args!("Vet not found"));
            };
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn add_vaccination(
        env: Env,
        pet_id: u64,
        veterinarian: Address,
        vaccine_type: VaccineType,
        vaccine_name: String,
        administered_at: u64,
        next_due_date: u64,
        batch_number: String,
    ) -> u64 {
        veterinarian.require_auth();
        if !Self::is_verified_vet(env.clone(), veterinarian.clone()) {
            {
                ::core::panicking::panic_fmt(format_args!("Veterinarian not verified"));
            };
        }
        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        let vaccine_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::VaccinationCount)
            .unwrap_or(0);
        let vaccine_id = vaccine_count + 1;
        let now = env.ledger().timestamp();
        let key = Self::get_encryption_key(&env);
        let vname_bytes = vaccine_name.to_xdr(&env);
        let (vname_nonce, vname_ciphertext) = encrypt_sensitive_data(
            &env,
            &vname_bytes,
            &key,
        );
        let encrypted_vaccine_name = EncryptedData {
            nonce: vname_nonce,
            ciphertext: vname_ciphertext,
        };
        let batch_bytes = batch_number.to_xdr(&env);
        let (batch_nonce, batch_ciphertext) = encrypt_sensitive_data(
            &env,
            &batch_bytes,
            &key,
        );
        let encrypted_batch_number = EncryptedData {
            nonce: batch_nonce,
            ciphertext: batch_ciphertext,
        };
        let record = Vaccination {
            id: vaccine_id,
            pet_id,
            veterinarian: veterinarian.clone(),
            vaccine_type: vaccine_type.clone(),
            vaccine_name: None,
            encrypted_vaccine_name,
            administered_at,
            next_due_date,
            batch_number: None,
            encrypted_batch_number,
            created_at: now,
        };
        env.storage().instance().set(&MedicalKey::Vaccination(vaccine_id), &record);
        env.storage().instance().set(&MedicalKey::VaccinationCount, &vaccine_id);
        let pet_vax_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);
        let new_pet_vax_count = pet_vax_count + 1;
        env.storage()
            .instance()
            .set(&MedicalKey::PetVaccinationCount(pet_id), &new_pet_vax_count);
        env.storage()
            .instance()
            .set(
                &MedicalKey::PetVaccinationByIndex((pet_id, new_pet_vax_count)),
                &vaccine_id,
            );
        env.events()
            .publish(
                (String::from_str(&env, "VaccinationAdded"), pet_id),
                VaccinationAddedEvent {
                    vaccine_id,
                    pet_id,
                    veterinarian,
                    vaccine_type,
                    next_due_date,
                    timestamp: now,
                },
            );
        vaccine_id
    }
    pub fn get_vaccinations(env: Env, vaccine_id: u64) -> Option<Vaccination> {
        if let Some(record) = env
            .storage()
            .instance()
            .get::<MedicalKey, Vaccination>(&MedicalKey::Vaccination(vaccine_id))
        {
            let key = Self::get_encryption_key(&env);
            let name_bytes = decrypt_sensitive_data(
                    &env,
                    &record.encrypted_vaccine_name.ciphertext,
                    &record.encrypted_vaccine_name.nonce,
                    &key,
                )
                .unwrap_or(Bytes::new(&env));
            let vaccine_name = String::from_xdr(&env, &name_bytes)
                .unwrap_or(String::from_str(&env, "Error"));
            let batch_bytes = decrypt_sensitive_data(
                    &env,
                    &record.encrypted_batch_number.ciphertext,
                    &record.encrypted_batch_number.nonce,
                    &key,
                )
                .unwrap_or(Bytes::new(&env));
            let batch_number = String::from_xdr(&env, &batch_bytes)
                .unwrap_or(String::from_str(&env, "Error"));
            let mut decrypted = record.clone();
            decrypted.vaccine_name = Some(vaccine_name);
            decrypted.batch_number = Some(batch_number);
            Some(decrypted)
        } else {
            None
        }
    }
    pub fn get_vaccination_history(env: Env, pet_id: u64) -> Vec<Vaccination> {
        if env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(pet_id)).is_none()
        {
            return Vec::new(&env);
        }
        let _vax_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);
        let count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);
        let mut history = Vec::new(&env);
        for i in 1..=count {
            if let Some(vid) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vax) = Self::get_vaccinations(env.clone(), vid) {
                    history.push_back(vax);
                }
            }
        }
        history
    }
    pub fn get_upcoming_vaccinations(
        env: Env,
        pet_id: u64,
        days_threshold: u64,
    ) -> Vec<Vaccination> {
        let current_time = env.ledger().timestamp();
        let threshold = current_time + (days_threshold * 86400);
        let history = Self::get_vaccination_history(env.clone(), pet_id);
        let mut upcoming = Vec::new(&env);
        for vax in history.iter() {
            if vax.next_due_date <= threshold {
                upcoming.push_back(vax);
            }
        }
        upcoming
    }
    pub fn is_vaccination_current(
        env: Env,
        pet_id: u64,
        vaccine_type: VaccineType,
    ) -> bool {
        let current_time = env.ledger().timestamp();
        let history = Self::get_vaccination_history(env, pet_id);
        let mut most_recent: Option<Vaccination> = None;
        for vax in history.iter() {
            if vax.vaccine_type == vaccine_type {
                match most_recent.clone() {
                    Some(current) => {
                        if vax.administered_at > current.administered_at {
                            most_recent = Some(vax);
                        }
                    }
                    None => most_recent = Some(vax),
                }
            }
        }
        if let Some(vax) = most_recent {
            vax.next_due_date > current_time
        } else {
            false
        }
    }
    pub fn get_overdue_vaccinations(env: Env, pet_id: u64) -> Vec<VaccineType> {
        let current_time = env.ledger().timestamp();
        let history = Self::get_vaccination_history(env.clone(), pet_id);
        let mut overdue = Vec::new(&env);
        for vax in history.iter() {
            if vax.next_due_date < current_time {
                overdue.push_back(vax.vaccine_type);
            }
        }
        overdue
    }
    fn generate_tag_id(env: &Env, pet_id: u64, _owner: &Address) -> BytesN<32> {
        let nonce: u64 = env.storage().instance().get(&TagKey::TagNonce).unwrap_or(0);
        let new_nonce = nonce + 1;
        env.storage().instance().set(&TagKey::TagNonce, &new_nonce);
        let timestamp = env.ledger().timestamp();
        let sequence = env.ledger().sequence();
        let mut preimage = Bytes::new(env);
        for byte in pet_id.to_be_bytes() {
            preimage.push_back(byte);
        }
        for byte in new_nonce.to_be_bytes() {
            preimage.push_back(byte);
        }
        for byte in timestamp.to_be_bytes() {
            preimage.push_back(byte);
        }
        for byte in sequence.to_be_bytes() {
            preimage.push_back(byte);
        }
        env.crypto().sha256(&preimage).into()
    }
    pub fn link_tag_to_pet(env: Env, pet_id: u64) -> BytesN<32> {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();
        if env
            .storage()
            .instance()
            .get::<DataKey, BytesN<32>>(&TagKey::PetTagId(pet_id))
            .is_some()
        {
            {
                ::core::panicking::panic_fmt(
                    format_args!("Pet already has a linked tag"),
                );
            };
        }
        let tag_id = Self::generate_tag_id(&env, pet_id, &pet.owner);
        let now = env.ledger().timestamp();
        let pet_tag = PetTag {
            tag_id: tag_id.clone(),
            pet_id,
            owner: pet.owner.clone(),
            message: String::from_str(&env, ""),
            is_active: true,
            linked_at: now,
            updated_at: now,
            tag_message: String::from_str(&env, ""),
            created_at: now,
        };
        env.storage().instance().set(&TagKey::Tag(tag_id.clone()), &pet_tag);
        env.storage().instance().set(&TagKey::PetTagId(pet_id), &tag_id);
        let count: u64 = env.storage().instance().get(&TagKey::PetTagCount).unwrap_or(0);
        env.storage().instance().set(&TagKey::PetTagCount, &(count + 1));
        env.events()
            .publish(
                (String::from_str(&env, "TAG_LINKED"),),
                TagLinkedEvent {
                    tag_id: tag_id.clone(),
                    pet_id,
                    owner: pet.owner.clone(),
                    timestamp: now,
                },
            );
        tag_id
    }
    pub fn get_pet_by_tag(env: Env, tag_id: BytesN<32>) -> Option<PetProfile> {
        if let Some(tag) = env
            .storage()
            .instance()
            .get::<TagKey, PetTag>(&TagKey::Tag(tag_id))
        {
            if !tag.is_active {
                return None;
            }
            Self::get_pet(env, tag.pet_id)
        } else {
            None
        }
    }
    pub fn get_tag(env: Env, tag_id: BytesN<32>) -> Option<PetTag> {
        env.storage().instance().get(&TagKey::Tag(tag_id))
    }
    pub fn get_tag_by_pet(env: Env, pet_id: u64) -> Option<BytesN<32>> {
        env.storage().instance().get(&TagKey::PetTagId(pet_id))
    }
    pub fn update_tag_message(env: Env, tag_id: BytesN<32>, message: String) -> bool {
        if let Some(mut tag) = env
            .storage()
            .instance()
            .get::<TagKey, PetTag>(&TagKey::Tag(tag_id.clone()))
        {
            let pet = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(tag.pet_id))
                .expect("Pet not found");
            pet.owner.require_auth();
            tag.message = message;
            tag.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&TagKey::Tag(tag_id), &tag);
            true
        } else {
            false
        }
    }
    pub fn deactivate_tag(env: Env, tag_id: BytesN<32>) -> bool {
        if let Some(mut tag) = env
            .storage()
            .instance()
            .get::<TagKey, PetTag>(&TagKey::Tag(tag_id.clone()))
        {
            let pet = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(tag.pet_id))
                .expect("Pet not found");
            pet.owner.require_auth();
            tag.is_active = false;
            tag.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&TagKey::Tag(tag_id.clone()), &tag);
            env.events()
                .publish(
                    (String::from_str(&env, "TAG_DEACTIVATED"),),
                    TagDeactivatedEvent {
                        tag_id,
                        pet_id: tag.pet_id,
                        deactivated_by: pet.owner,
                        timestamp: env.ledger().timestamp(),
                    },
                );
            true
        } else {
            false
        }
    }
    pub fn reactivate_tag(env: Env, tag_id: BytesN<32>) -> bool {
        if let Some(mut tag) = env
            .storage()
            .instance()
            .get::<TagKey, PetTag>(&TagKey::Tag(tag_id.clone()))
        {
            let pet = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(tag.pet_id))
                .expect("Pet not found");
            pet.owner.require_auth();
            tag.is_active = true;
            tag.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&TagKey::Tag(tag_id.clone()), &tag);
            env.events()
                .publish(
                    (String::from_str(&env, "TAG_REACTIVATED"),),
                    TagReactivatedEvent {
                        tag_id,
                        pet_id: tag.pet_id,
                        reactivated_by: pet.owner,
                        timestamp: env.ledger().timestamp(),
                    },
                );
            true
        } else {
            false
        }
    }
    pub fn is_tag_active(env: Env, tag_id: BytesN<32>) -> bool {
        if let Some(tag) = env
            .storage()
            .instance()
            .get::<TagKey, PetTag>(&TagKey::Tag(tag_id))
        {
            tag.is_active
        } else {
            false
        }
    }
    fn get_owner_pet_count(env: &Env, owner: &Address) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::PetCountByOwner(owner.clone()))
            .unwrap_or(0)
    }
    fn species_to_string(env: &Env, species: &Species) -> String {
        match species {
            Species::Other => String::from_str(env, "Other"),
            Species::Dog => String::from_str(env, "Dog"),
            Species::Cat => String::from_str(env, "Cat"),
            Species::Bird => String::from_str(env, "Bird"),
        }
    }
    fn validate_ipfs_hash(hash: &String) {
        let len = hash.len();
        if !(32_u32..=128_u32).contains(&len) {
            {
                ::core::panicking::panic_fmt(
                    format_args!("Invalid IPFS hash: length must be 32-128 chars"),
                );
            };
        }
    }
    fn get_encryption_key(env: &Env) -> Bytes {
        Bytes::from_array(env, &[0u8; 32])
    }
    fn log_ownership_change(
        env: &Env,
        pet_id: u64,
        previous_owner: Address,
        new_owner: Address,
        reason: String,
    ) {
        let global_count: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::OwnershipRecordCount)
            .unwrap_or(0);
        let record_id = global_count + 1;
        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::PetOwnershipRecordCount(pet_id))
            .unwrap_or(0);
        let new_pet_count = pet_count + 1;
        let record = OwnershipRecord {
            pet_id,
            previous_owner,
            new_owner,
            transfer_date: env.ledger().timestamp(),
            transfer_reason: reason,
        };
        env.storage().instance().set(&SystemKey::PetOwnershipRecord(record_id), &record);
        env.storage().instance().set(&SystemKey::OwnershipRecordCount, &record_id);
        env.storage()
            .instance()
            .set(&SystemKey::PetOwnershipRecordCount(pet_id), &new_pet_count);
        env.storage()
            .instance()
            .set(
                &SystemKey::PetOwnershipRecordIndex((pet_id, new_pet_count)),
                &record_id,
            );
    }
    pub fn get_ownership_history(env: Env, pet_id: u64) -> Vec<OwnershipRecord> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::PetOwnershipRecordCount(pet_id))
            .unwrap_or(0);
        let mut history = Vec::new(&env);
        for i in 1..=count {
            if let Some(record_id) = env
                .storage()
                .instance()
                .get::<SystemKey, u64>(&SystemKey::PetOwnershipRecordIndex((pet_id, i)))
            {
                if let Some(record) = env
                    .storage()
                    .instance()
                    .get::<
                        SystemKey,
                        OwnershipRecord,
                    >(&SystemKey::PetOwnershipRecord(record_id))
                {
                    history.push_back(record);
                }
            }
        }
        history
    }
    pub fn set_emergency_contacts(
        env: Env,
        pet_id: u64,
        contacts: Vec<EmergencyContact>,
        medical_notes: String,
    ) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.owner.require_auth();
            let key = Self::get_encryption_key(&env);
            let contacts_bytes = contacts.to_xdr(&env);
            let (c_nonce, c_cipher) = encrypt_sensitive_data(
                &env,
                &contacts_bytes,
                &key,
            );
            pet.encrypted_emergency_contacts = EncryptedData {
                nonce: c_nonce,
                ciphertext: c_cipher,
            };
            let notes_bytes = medical_notes.to_xdr(&env);
            let (n_nonce, n_cipher) = encrypt_sensitive_data(&env, &notes_bytes, &key);
            pet.encrypted_medical_alerts = EncryptedData {
                nonce: n_nonce,
                ciphertext: n_cipher,
            };
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        } else {
            {
                ::core::panicking::panic_fmt(format_args!("Pet not found"));
            };
        }
    }
    pub fn get_emergency_info(
        env: Env,
        pet_id: u64,
    ) -> Option<(Vec<EmergencyContact>, String)> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            let key = Self::get_encryption_key(&env);
            let c_bytes = decrypt_sensitive_data(
                    &env,
                    &pet.encrypted_emergency_contacts.ciphertext,
                    &pet.encrypted_emergency_contacts.nonce,
                    &key,
                )
                .unwrap_or(Bytes::new(&env));
            let contacts = Vec::<EmergencyContact>::from_xdr(&env, &c_bytes)
                .unwrap_or(Vec::new(&env));
            let n_bytes = decrypt_sensitive_data(
                    &env,
                    &pet.encrypted_medical_alerts.ciphertext,
                    &pet.encrypted_medical_alerts.nonce,
                    &key,
                )
                .unwrap_or(Bytes::new(&env));
            let notes = String::from_xdr(&env, &n_bytes)
                .unwrap_or(String::from_str(&env, ""));
            Some((contacts, notes))
        } else {
            None
        }
    }
    /// Get emergency contacts for a pet (publicly accessible - no auth required for emergency responders)
    pub fn get_emergency_contacts(env: Env, pet_id: u64) -> Vec<EmergencyContact> {
        if let Some(pet) = env.storage().instance().get::<_, Pet>(&DataKey::Pet(pet_id))
        {
            let key = Self::get_encryption_key(&env);
            let c_bytes = decrypt_sensitive_data(
                    &env,
                    &pet.encrypted_emergency_contacts.ciphertext,
                    &pet.encrypted_emergency_contacts.nonce,
                    &key,
                )
                .unwrap_or(Bytes::new(&env));
            Vec::<EmergencyContact>::from_xdr(&env, &c_bytes).unwrap_or(Vec::new(&env))
        } else {
            Vec::new(&env)
        }
    }
    pub fn get_accessible_pets(env: Env, user: Address) -> Vec<u64> {
        user.require_auth();
        let mut accessible_pets = Vec::new(&env);
        let count = Self::get_owner_pet_count(&env, &user);
        for i in 1..=count {
            if let Some(pid) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::OwnerPetIndex((user.clone(), i)))
            {
                accessible_pets.push_back(pid);
            }
        }
        accessible_pets
    }
    pub fn get_all_pets_by_owner(env: Env, owner: Address) -> Vec<PetProfile> {
        let count = Self::get_owner_pet_count(&env, &owner);
        let mut pets = Vec::new(&env);
        for i in 1..=count {
            if let Some(pid) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::OwnerPetIndex((owner.clone(), i)))
            {
                if let Some(pet) = Self::get_pet(env.clone(), pid) {
                    pets.push_back(pet);
                }
            }
        }
        pets
    }
    pub fn get_pets_by_owner(env: Env, owner: Address) -> Vec<PetProfile> {
        Self::get_all_pets_by_owner(env, owner)
    }
    pub fn get_pets_by_species(env: Env, species: String) -> Vec<PetProfile> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::SpeciesPetCount(species.clone()))
            .unwrap_or(0);
        let mut pets = Vec::new(&env);
        for i in 1..=count {
            if let Some(pid) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::SpeciesPetIndex((species.clone(), i)))
            {
                if let Some(pet) = Self::get_pet(env.clone(), pid) {
                    pets.push_back(pet);
                }
            }
        }
        pets
    }
    pub fn get_active_pets(env: Env) -> Vec<PetProfile> {
        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetCount)
            .unwrap_or(0);
        let mut pets = Vec::new(&env);
        for id in 1..=pet_count {
            if let Some(pet) = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(id))
            {
                if pet.active {
                    if let Some(profile) = Self::get_pet(env.clone(), id) {
                        pets.push_back(profile);
                    }
                }
            }
        }
        pets
    }
    pub fn grant_access(
        env: Env,
        pet_id: u64,
        grantee: Address,
        access_level: AccessLevel,
        expires_at: Option<u64>,
    ) -> bool {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();
        let granter = pet.owner.clone();
        let now = env.ledger().timestamp();
        let grant = AccessGrant {
            pet_id,
            granter: granter.clone(),
            grantee: grantee.clone(),
            access_level: access_level.clone(),
            granted_at: now,
            expires_at,
            is_active: true,
        };
        env.storage()
            .instance()
            .set(&DataKey::AccessGrant((pet_id, grantee.clone())), &grant);
        let grant_count = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::AccessGrantCount(pet_id))
            .unwrap_or(0);
        let new_count = grant_count + 1;
        env.storage().instance().set(&DataKey::AccessGrantCount(pet_id), &new_count);
        env.storage()
            .instance()
            .set(&DataKey::AccessGrantIndex((pet_id, new_count)), &grantee);
        env.events()
            .publish(
                (String::from_str(&env, "AccessGranted"), pet_id),
                AccessGrantedEvent {
                    pet_id,
                    granter: granter.clone(),
                    grantee,
                    access_level,
                    expires_at,
                    timestamp: now,
                },
            );
        Self::log_access(
            &env,
            pet_id,
            granter,
            AccessAction::Grant,
            String::from_str(&env, "Access granted"),
        );
        true
    }
    pub fn revoke_access(env: Env, pet_id: u64, grantee: Address) -> bool {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();
        let granter = pet.owner.clone();
        let key = DataKey::AccessGrant((pet_id, grantee.clone()));
        if let Some(mut grant) = env
            .storage()
            .instance()
            .get::<DataKey, AccessGrant>(&key)
        {
            grant.is_active = false;
            grant.access_level = AccessLevel::None;
            env.storage().instance().set(&key, &grant);
            env.events()
                .publish(
                    (String::from_str(&env, "AccessRevoked"), pet_id),
                    AccessRevokedEvent {
                        pet_id,
                        granter: granter.clone(),
                        grantee,
                        timestamp: env.ledger().timestamp(),
                    },
                );
            Self::log_access(
                &env,
                pet_id,
                granter,
                AccessAction::Revoke,
                String::from_str(&env, "Access revoked"),
            );
            true
        } else {
            false
        }
    }
    pub fn grant_temporary_custody(
        env: Env,
        pet_id: u64,
        custodian: Address,
        start_date: u64,
        end_date: u64,
        permissions: Vec<String>,
    ) -> TemporaryCustody {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();
        let custody = TemporaryCustody {
            pet_id,
            owner: pet.owner,
            custodian,
            start_date,
            end_date,
            permissions,
            is_active: true,
        };
        env.storage().instance().set(&DataKey::TemporaryCustody(pet_id), &custody);
        custody
    }
    pub fn revoke_temporary_custody(env: Env, pet_id: u64) {
        let mut custody: TemporaryCustody = env
            .storage()
            .instance()
            .get(&DataKey::TemporaryCustody(pet_id))
            .expect("Temporary custody not found");
        custody.owner.require_auth();
        custody.is_active = false;
        env.storage().instance().set(&DataKey::TemporaryCustody(pet_id), &custody);
    }
    pub fn is_custody_valid(env: Env, pet_id: u64) -> bool {
        let custody: TemporaryCustody = env
            .storage()
            .instance()
            .get(&DataKey::TemporaryCustody(pet_id))
            .expect("Temporary custody not found");
        let current_time = env.ledger().timestamp();
        custody.is_active && current_time <= custody.end_date
    }
    pub fn add_medical_record(
        env: Env,
        pet_id: u64,
        vet_address: Address,
        diagnosis: String,
        treatment: String,
        medications: String,
        notes: String,
    ) -> u64 {
        vet_address.require_auth();
        if !Self::is_verified_vet(env.clone(), vet_address.clone()) {
            {
                ::core::panicking::panic_fmt(format_args!("Veterinarian not verified"));
            };
        }
        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        let count = env
            .storage()
            .instance()
            .get::<MedicalKey, u64>(&MedicalKey::MedicalRecordCount)
            .unwrap_or(0);
        let id = count + 1;
        env.storage().instance().set(&MedicalKey::MedicalRecordCount, &id);
        let now = env.ledger().timestamp();
        let record = MedicalRecord {
            id,
            pet_id,
            vet_address: vet_address.clone(),
            diagnosis,
            treatment,
            medications,
            date: now,
            notes,
        };
        env.storage().instance().set(&MedicalKey::MedicalRecord(id), &record);
        let pet_record_count = env
            .storage()
            .instance()
            .get::<MedicalKey, u64>(&MedicalKey::PetMedicalRecordCount(pet_id))
            .unwrap_or(0);
        let new_pet_record_count = pet_record_count + 1;
        env.storage()
            .instance()
            .set(&MedicalKey::PetMedicalRecordCount(pet_id), &new_pet_record_count);
        env.storage()
            .instance()
            .set(
                &MedicalKey::PetMedicalRecordIndex((pet_id, new_pet_record_count)),
                &id,
            );
        env.events()
            .publish(
                (String::from_str(&env, "MedicalRecordAdded"), pet_id),
                MedicalRecordAddedEvent {
                    pet_id,
                    updated_by: veterinarian.clone(),
                    timestamp: now,
                },
            );
        Self::log_access(
            &env,
            pet_id,
            veterinarian,
            AccessAction::Write,
            String::from_str(&env, "Medical record added"),
        );
        id
    }
    pub fn update_medical_record(
        env: Env,
        record_id: u64,
        diagnosis: String,
        treatment: String,
        medications: String,
        notes: String,
    ) -> bool {
        if let Some(mut record) = env
            .storage()
            .instance()
            .get::<MedicalKey, MedicalRecord>(&MedicalKey::MedicalRecord(record_id))
        {
            record.vet_address.require_auth();
            record.diagnosis = diagnosis;
            record.treatment = treatment;
            record.medications = medications;
            record.notes = notes;
            record.date = env.ledger().timestamp();
            env.storage().instance().set(&MedicalKey::MedicalRecord(record_id), &record);
            Self::log_access(
                &env,
                record.pet_id,
                record.veterinarian,
                AccessAction::Write,
                String::from_str(&env, "Medical record updated"),
            );
            true
        } else {
            false
        }
    }
    pub fn get_medical_record(env: Env, record_id: u64) -> Option<MedicalRecord> {
        let record: Option<MedicalRecord> = env
            .storage()
            .instance()
            .get(&MedicalKey::MedicalRecord(record_id));
        if let Some(ref r) = record {
            Self::log_access(
                &env,
                r.pet_id,
                env.current_contract_address(),
                AccessAction::Read,
                String::from_str(&env, "Medical record accessed"),
            );
        }
        record
    }
    pub fn get_pet_medical_records(env: Env, pet_id: u64) -> Vec<MedicalRecord> {
        let count = env
            .storage()
            .instance()
            .get::<MedicalKey, u64>(&MedicalKey::PetMedicalRecordCount(pet_id))
            .unwrap_or(0);
        let mut records = Vec::new(&env);
        for i in 1..=count {
            if let Some(rid) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetMedicalRecordIndex((pet_id, i)))
            {
                if let Some(record) = Self::get_medical_record(env.clone(), rid) {
                    records.push_back(record);
                }
            }
        }
        Self::log_access(
            &env,
            pet_id,
            env.current_contract_address(),
            AccessAction::Read,
            String::from_str(&env, "Pet medical records accessed"),
        );
        records
    }
    pub fn get_access_logs(env: Env, pet_id: u64) -> Vec<AccessLog> {
        let key = (Symbol::new(&env, "access_logs"), pet_id);
        env.storage().persistent().get(&key).unwrap_or(Vec::new(&env))
    }
    pub fn check_access(env: Env, pet_id: u64, user: Address) -> AccessLevel {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            if pet.owner == user {
                return AccessLevel::Full;
            }
            if let Some(grant) = env
                .storage()
                .instance()
                .get::<DataKey, AccessGrant>(&DataKey::AccessGrant((pet_id, user)))
            {
                if !grant.is_active {
                    return AccessLevel::None;
                }
                if let Some(exp) = grant.expires_at {
                    if env.ledger().timestamp() >= exp {
                        return AccessLevel::None;
                    }
                }
                return grant.access_level;
            }
        }
        AccessLevel::None
    }
    pub fn get_authorized_users(env: Env, pet_id: u64) -> Vec<Address> {
        let count = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::AccessGrantCount(pet_id))
            .unwrap_or(0);
        let mut users = Vec::new(&env);
        for i in 1..=count {
            if let Some(grantee) = env
                .storage()
                .instance()
                .get::<DataKey, Address>(&DataKey::AccessGrantIndex((pet_id, i)))
            {
                if Self::check_access(env.clone(), pet_id, grantee.clone())
                    != AccessLevel::None
                {
                    users.push_back(grantee);
                }
            }
        }
        users
    }
    pub fn get_access_grant(
        env: Env,
        pet_id: u64,
        grantee: Address,
    ) -> Option<AccessGrant> {
        env.storage().instance().get(&DataKey::AccessGrant((pet_id, grantee)))
    }
    pub fn add_lab_result(
        env: Env,
        pet_id: u64,
        vet_address: Address,
        test_type: String,
        results: String,
        reference_ranges: String,
        attachment_hash: Option<String>,
        medical_record_id: Option<u64>,
    ) -> u64 {
        vet_address.require_auth();
        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        let count = env
            .storage()
            .instance()
            .get::<MedicalKey, u64>(&MedicalKey::LabResultCount)
            .unwrap_or(0);
        let id = count + 1;
        env.storage().instance().set(&MedicalKey::LabResultCount, &id);
        let result = LabResult {
            id,
            pet_id,
            test_type,
            date: env.ledger().timestamp(),
            results,
            vet_address,
            reference_ranges,
            attachment_hash,
            medical_record_id,
        };
        env.storage().instance().set(&MedicalKey::LabResult(id), &result);
        let p_count = env
            .storage()
            .instance()
            .get::<MedicalKey, u64>(&MedicalKey::PetLabResultCount(pet_id))
            .unwrap_or(0);
        let new_p = p_count + 1;
        env.storage().instance().set(&MedicalKey::PetLabResultCount(pet_id), &new_p);
        env.storage()
            .instance()
            .set(&MedicalKey::PetLabResultIndex((pet_id, new_p)), &id);
        id
    }
    pub fn get_lab_result(env: Env, lab_result_id: u64) -> Option<LabResult> {
        env.storage().instance().get(&MedicalKey::LabResult(lab_result_id))
    }
    pub fn get_lab_results(env: Env, pet_id: u64) -> Vec<LabResult> {
        let count = env
            .storage()
            .instance()
            .get::<MedicalKey, u64>(&MedicalKey::PetLabResultCount(pet_id))
            .unwrap_or(0);
        let mut res = Vec::new(&env);
        for i in 1..=count {
            if let Some(lid) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetLabResultIndex((pet_id, i)))
            {
                if let Some(r) = Self::get_lab_result(env.clone(), lid) {
                    res.push_back(r);
                }
            }
        }
        res
    }
    #[allow(clippy::too_many_arguments)]
    pub fn add_medication_to_record(
        env: Env,
        record_id: u64,
        name: String,
        dosage: String,
        frequency: String,
        start_date: u64,
        end_date: u64,
        prescribing_vet: Address,
    ) -> bool {
        if let Some(mut record) = env
            .storage()
            .instance()
            .get::<MedicalKey, MedicalRecord>(&MedicalKey::MedicalRecord(record_id))
        {
            prescribing_vet.require_auth();
            let med = Medication {
                id: 0,
                pet_id: record.pet_id,
                name,
                dosage,
                frequency,
                start_date,
                end_date: Some(end_date),
                prescribing_vet,
                active: true,
            };
            record.medications.push_back(med);
            record.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&MedicalKey::MedicalRecord(record_id), &record);
            true
        } else {
            false
        }
    }
    pub fn mark_record_med_completed(env: Env, record_id: u64, med_index: u32) -> bool {
        if let Some(mut record) = env
            .storage()
            .instance()
            .get::<MedicalKey, MedicalRecord>(&MedicalKey::MedicalRecord(record_id))
        {
            let _pet = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(record.pet_id))
                .expect("Pet not found");
            record.veterinarian.require_auth();
            if let Some(mut med) = record.medications.get(med_index) {
                med.active = false;
                record.medications.set(med_index, med);
                record.updated_at = env.ledger().timestamp();
                env.storage()
                    .instance()
                    .set(&MedicalKey::MedicalRecord(record_id), &record);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn get_active_record_meds(env: Env, pet_id: u64) -> Vec<Medication> {
        let records = Self::get_pet_medical_records(env.clone(), pet_id);
        let mut active_meds = Vec::new(&env);
        for record in records.iter() {
            for med in record.medications.iter() {
                if med.active {
                    active_meds.push_back(med);
                }
            }
        }
        active_meds
    }
    pub fn get_record_med_history(env: Env, pet_id: u64) -> Vec<Medication> {
        let records = Self::get_pet_medical_records(env.clone(), pet_id);
        let mut history = Vec::new(&env);
        for record in records.iter() {
            for med in record.medications.iter() {
                history.push_back(med);
            }
        }
        history
    }
    pub fn batch_add_vaccinations(
        env: Env,
        veterinarian: Address,
        vaccinations: Vec<VaccinationInput>,
    ) -> Vec<u64> {
        veterinarian.require_auth();
        if !Self::is_verified_vet(env.clone(), veterinarian.clone()) {
            {
                ::core::panicking::panic_fmt(format_args!("Veterinarian not verified"));
            };
        }
        let mut ids = Vec::new(&env);
        for input in vaccinations.iter() {
            let id = Self::add_vaccination(
                env.clone(),
                input.pet_id,
                veterinarian.clone(),
                input.vaccine_type,
                input.vaccine_name,
                input.administered_at,
                input.next_due_date,
                input.batch_number,
            );
            ids.push_back(id);
        }
        ids
    }
    pub fn batch_add_records(
        env: Env,
        veterinarian: Address,
        records: Vec<MedicalRecordInput>,
    ) -> Vec<u64> {
        veterinarian.require_auth();
        let mut ids = Vec::new(&env);
        for input in records.iter() {
            let id = Self::add_medical_record(
                env.clone(),
                input.pet_id,
                veterinarian.clone(),
                input.diagnosis,
                input.treatment,
                input.medications,
                input.notes,
            );
            ids.push_back(id);
        }
        ids
    }
    /// Report a pet as lost
    pub fn report_lost(
        env: Env,
        pet_id: u64,
        last_seen_location: String,
        reward_amount: Option<u64>,
    ) -> u64 {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();
        let alert_count: u64 = env
            .storage()
            .instance()
            .get(&AlertKey::LostPetAlertCount)
            .unwrap_or(0);
        let alert_id = alert_count + 1;
        let alert = LostPetAlert {
            id: alert_id,
            pet_id,
            reported_by: pet.owner.clone(),
            reported_date: env.ledger().timestamp(),
            last_seen_location,
            reward_amount,
            status: AlertStatus::Active,
            found_date: None,
        };
        env.storage().instance().set(&AlertKey::LostPetAlert(alert_id), &alert);
        env.storage().instance().set(&AlertKey::LostPetAlertCount, &alert_id);
        let mut active_alerts: Vec<u64> = env
            .storage()
            .instance()
            .get(&AlertKey::ActiveLostPetAlerts)
            .unwrap_or(Vec::new(&env));
        active_alerts.push_back(alert_id);
        env.storage().instance().set(&AlertKey::ActiveLostPetAlerts, &active_alerts);
        alert_id
    }
    /// Report a sighting of a lost pet
    pub fn report_sighting(
        env: Env,
        alert_id: u64,
        location: String,
        description: String,
    ) -> bool {
        let reporter = env.current_contract_address();
        let sighting = SightingReport {
            alert_id,
            reporter,
            location,
            timestamp: env.ledger().timestamp(),
            description,
        };
        let key = AlertKey::AlertSightings(alert_id);
        let mut sightings: Vec<SightingReport> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or(Vec::new(&env));
        sightings.push_back(sighting);
        env.storage().instance().set(&key, &sightings);
        true
    }
    /// Mark a lost pet as found
    pub fn report_found(env: Env, alert_id: u64) -> bool {
        let key = AlertKey::LostPetAlert(alert_id);
        let mut alert: LostPetAlert = env
            .storage()
            .instance()
            .get(&key)
            .expect("Alert not found");
        alert.reported_by.require_auth();
        if alert.status != AlertStatus::Active {
            {
                ::core::panicking::panic_fmt(format_args!("Alert is not active"));
            };
        }
        alert.status = AlertStatus::Found;
        alert.found_date = Some(env.ledger().timestamp());
        env.storage().instance().set(&key, &alert);
        let mut active_alerts: Vec<u64> = env
            .storage()
            .instance()
            .get(&AlertKey::ActiveLostPetAlerts)
            .unwrap_or(Vec::new(&env));
        if let Some(pos) = active_alerts.iter().position(|id| id == alert_id) {
            active_alerts.remove(pos as u32);
            env.storage().instance().set(&AlertKey::ActiveLostPetAlerts, &active_alerts);
        }
        true
    }
    /// Cancel a lost pet alert
    pub fn cancel_lost_alert(env: Env, alert_id: u64) -> bool {
        let key = AlertKey::LostPetAlert(alert_id);
        let mut alert: LostPetAlert = env
            .storage()
            .instance()
            .get(&key)
            .expect("Alert not found");
        alert.reported_by.require_auth();
        if alert.status != AlertStatus::Active {
            {
                ::core::panicking::panic_fmt(format_args!("Alert is not active"));
            };
        }
        alert.status = AlertStatus::Cancelled;
        env.storage().instance().set(&key, &alert);
        let mut active_alerts: Vec<u64> = env
            .storage()
            .instance()
            .get(&AlertKey::ActiveLostPetAlerts)
            .unwrap_or(Vec::new(&env));
        if let Some(pos) = active_alerts.iter().position(|id| id == alert_id) {
            active_alerts.remove(pos as u32);
            env.storage().instance().set(&AlertKey::ActiveLostPetAlerts, &active_alerts);
        }
        true
    }
    /// Get all active lost pet alerts
    pub fn get_active_alerts(env: Env) -> Vec<LostPetAlert> {
        let active_ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&AlertKey::ActiveLostPetAlerts)
            .unwrap_or(Vec::new(&env));
        let mut active_alerts = Vec::new(&env);
        for id in active_ids.iter() {
            if let Some(alert) = env
                .storage()
                .instance()
                .get::<AlertKey, LostPetAlert>(&AlertKey::LostPetAlert(id))
            {
                if alert.status == AlertStatus::Active {
                    active_alerts.push_back(alert);
                }
            }
        }
        active_alerts
    }
    /// Get a specific alert by ID
    pub fn get_alert(env: Env, alert_id: u64) -> Option<LostPetAlert> {
        env.storage().instance().get(&AlertKey::LostPetAlert(alert_id))
    }
    /// Get sightings for a specific alert
    pub fn get_alert_sightings(env: Env, alert_id: u64) -> Vec<SightingReport> {
        env.storage()
            .instance()
            .get(&AlertKey::AlertSightings(alert_id))
            .unwrap_or(Vec::new(&env))
    }
    /// Get alerts for a specific pet
    pub fn get_pet_alerts(env: Env, pet_id: u64) -> Vec<LostPetAlert> {
        let alert_count: u64 = env
            .storage()
            .instance()
            .get(&AlertKey::LostPetAlertCount)
            .unwrap_or(0);
        let mut pet_alerts = Vec::new(&env);
        for i in 1..=alert_count {
            if let Some(alert) = env
                .storage()
                .instance()
                .get::<AlertKey, LostPetAlert>(&AlertKey::LostPetAlert(i))
            {
                if alert.pet_id == pet_id {
                    pet_alerts.push_back(alert);
                }
            }
        }
        pet_alerts
    }
    /// Set availability slots for a vet (only verified vets can set their availability)
    pub fn set_availability(
        env: Env,
        vet_address: Address,
        start_time: u64,
        end_time: u64,
    ) -> u64 {
        vet_address.require_auth();
        if !Self::is_verified_vet(env.clone(), vet_address.clone()) {
            {
                ::core::panicking::panic_fmt(format_args!("Vet not verified"));
            };
        }
        let slot_count: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::VetAvailabilityCount(vet_address.clone()))
            .unwrap_or(0);
        let slot_index = slot_count + 1;
        let slot = AvailabilitySlot {
            vet_address: vet_address.clone(),
            start_time,
            end_time,
            available: true,
        };
        env.storage()
            .instance()
            .set(&SystemKey::VetAvailability((vet_address.clone(), slot_index)), &slot);
        env.storage()
            .instance()
            .set(&SystemKey::VetAvailabilityCount(vet_address.clone()), &slot_index);
        let date = Self::get_date_from_timestamp(start_time);
        let date_key = SystemKey::VetAvailabilityByDate((vet_address.clone(), date));
        let mut date_slots: Vec<u64> = env
            .storage()
            .instance()
            .get(&date_key)
            .unwrap_or(Vec::new(&env));
        date_slots.push_back(slot_index);
        env.storage().instance().set(&date_key, &date_slots);
        slot_index
    }
    /// Get available slots for a vet on a specific date
    pub fn get_available_slots(
        env: Env,
        vet_address: Address,
        date: u64,
    ) -> Vec<AvailabilitySlot> {
        let date_key = SystemKey::VetAvailabilityByDate((vet_address.clone(), date));
        let slot_indices: Vec<u64> = env
            .storage()
            .instance()
            .get(&date_key)
            .unwrap_or(Vec::new(&env));
        let mut available_slots = Vec::new(&env);
        for index in slot_indices.iter() {
            if let Some(slot) = env
                .storage()
                .instance()
                .get::<
                    SystemKey,
                    AvailabilitySlot,
                >(&SystemKey::VetAvailability((vet_address.clone(), index)))
            {
                if slot.available {
                    available_slots.push_back(slot);
                }
            }
        }
        available_slots
    }
    pub fn grant_consent(
        env: Env,
        pet_id: u64,
        owner: Address,
        consent_type: ConsentType,
        granted_to: Address,
    ) -> u64 {
        owner.require_auth();
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        if pet.owner != owner {
            {
                ::core::panicking::panic_fmt(format_args!("Not the pet owner"));
            };
        }
        let count: u64 = env
            .storage()
            .instance()
            .get(&ConsentKey::ConsentCount)
            .unwrap_or(0);
        let consent_id = count + 1;
        let now = env.ledger().timestamp();
        let consent = Consent {
            id: consent_id,
            pet_id,
            owner,
            consent_type,
            granted_to,
            granted_at: now,
            revoked_at: None,
            is_active: true,
        };
        env.storage().instance().set(&ConsentKey::Consent(consent_id), &consent);
        env.storage().instance().set(&ConsentKey::ConsentCount, &consent_id);
        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&ConsentKey::PetConsentCount(pet_id))
            .unwrap_or(0);
        let new_pet_count = pet_count + 1;
        env.storage()
            .instance()
            .set(&ConsentKey::PetConsentCount(pet_id), &new_pet_count);
        env.storage()
            .instance()
            .set(&ConsentKey::PetConsentIndex((pet_id, new_pet_count)), &consent_id);
        consent_id
    }
    pub fn revoke_consent(env: Env, consent_id: u64, owner: Address) -> bool {
        owner.require_auth();
        if let Some(mut consent) = env
            .storage()
            .instance()
            .get::<ConsentKey, Consent>(&ConsentKey::Consent(consent_id))
        {
            if consent.owner != owner {
                {
                    ::core::panicking::panic_fmt(format_args!("Not the consent owner"));
                };
            }
            if !consent.is_active {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Consent already revoked"),
                    );
                };
            }
            consent.is_active = false;
            consent.revoked_at = Some(env.ledger().timestamp());
            env.storage().instance().set(&ConsentKey::Consent(consent_id), &consent);
            true
        } else {
            false
        }
    }
    pub fn get_consent_history(env: Env, pet_id: u64) -> Vec<Consent> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&ConsentKey::PetConsentCount(pet_id))
            .unwrap_or(0);
        let mut history = Vec::new(&env);
        for i in 1..=count {
            if let Some(consent_id) = env
                .storage()
                .instance()
                .get::<ConsentKey, u64>(&ConsentKey::PetConsentIndex((pet_id, i)))
            {
                if let Some(consent) = env
                    .storage()
                    .instance()
                    .get::<ConsentKey, Consent>(&ConsentKey::Consent(consent_id))
                {
                    history.push_back(consent);
                }
            }
        }
        history
    }
    /// Book a slot (mark as unavailable)
    pub fn book_slot(env: Env, vet_address: Address, slot_index: u64) -> bool {
        let key = SystemKey::VetAvailability((vet_address.clone(), slot_index));
        if let Some(mut slot) = env
            .storage()
            .instance()
            .get::<SystemKey, AvailabilitySlot>(&key)
        {
            if !slot.available {
                {
                    ::core::panicking::panic_fmt(format_args!("Slot already booked"));
                };
            }
            slot.available = false;
            env.storage().instance().set(&key, &slot);
            true
        } else {
            false
        }
    }
    /// Helper: Extract date from timestamp (yyyyMMdd format)
    fn get_date_from_timestamp(timestamp: u64) -> u64 {
        timestamp / 86400
    }
    pub fn get_version(env: Env) -> ContractVersion {
        env.storage()
            .instance()
            .get(&DataKey::ContractVersion)
            .unwrap_or(ContractVersion {
                major: 1,
                minor: 0,
                patch: 0,
            })
    }
    pub fn upgrade_contract(env: Env, new_wasm_hash: BytesN<32>) {
        Self::require_admin(&env);
        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }
    pub fn propose_upgrade(
        env: Env,
        proposer: Address,
        new_wasm_hash: BytesN<32>,
    ) -> u64 {
        Self::require_admin(&env);
        proposer.require_auth();
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::UpgradeProposalCount)
            .unwrap_or(0);
        let proposal_id = count + 1;
        let proposal = UpgradeProposal {
            id: proposal_id,
            proposed_by: proposer,
            new_wasm_hash,
            proposed_at: env.ledger().timestamp(),
            approved: false,
            executed: false,
        };
        env.storage().instance().set(&DataKey::UpgradeProposal(proposal_id), &proposal);
        env.storage().instance().set(&DataKey::UpgradeProposalCount, &proposal_id);
        proposal_id
    }
    pub fn approve_upgrade(env: Env, proposal_id: u64) -> bool {
        Self::require_admin(&env);
        if let Some(mut proposal) = env
            .storage()
            .instance()
            .get::<DataKey, UpgradeProposal>(&DataKey::UpgradeProposal(proposal_id))
        {
            if proposal.executed {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Proposal already executed"),
                    );
                };
            }
            proposal.approved = true;
            env.storage()
                .instance()
                .set(&DataKey::UpgradeProposal(proposal_id), &proposal);
            true
        } else {
            false
        }
    }
    pub fn get_upgrade_proposal(env: Env, proposal_id: u64) -> Option<UpgradeProposal> {
        env.storage().instance().get(&DataKey::UpgradeProposal(proposal_id))
    }
    pub fn migrate_version(env: Env, major: u32, minor: u32, patch: u32) {
        Self::require_admin(&env);
        let version = ContractVersion {
            major,
            minor,
            patch,
        };
        env.storage().instance().set(&DataKey::ContractVersion, &version);
    }
    pub fn propose_action(
        env: Env,
        proposer: Address,
        action: ProposalAction,
        expires_in: u64,
    ) -> u64 {
        Self::require_admin_auth(&env, &proposer);
        let count: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::ProposalCount)
            .unwrap_or(0);
        let proposal_id = count + 1;
        let threshold = env
            .storage()
            .instance()
            .get::<SystemKey, u32>(&SystemKey::AdminThreshold)
            .unwrap_or(1);
        let mut approvals = Vec::new(&env);
        approvals.push_back(proposer.clone());
        let now = env.ledger().timestamp();
        let proposal = MultiSigProposal {
            id: proposal_id,
            action,
            proposed_by: proposer,
            approvals,
            required_approvals: threshold,
            created_at: now,
            expires_at: now + expires_in,
            executed: false,
        };
        env.storage().instance().set(&SystemKey::Proposal(proposal_id), &proposal);
        env.storage().instance().set(&SystemKey::ProposalCount, &proposal_id);
        proposal_id
    }
    pub fn approve_proposal(env: Env, admin: Address, proposal_id: u64) {
        Self::require_admin_auth(&env, &admin);
        let mut proposal: MultiSigProposal = env
            .storage()
            .instance()
            .get(&SystemKey::Proposal(proposal_id))
            .expect("Proposal not found");
        if proposal.executed {
            {
                ::core::panicking::panic_fmt(format_args!("Proposal already executed"));
            };
        }
        if env.ledger().timestamp() > proposal.expires_at {
            {
                ::core::panicking::panic_fmt(format_args!("Proposal expired"));
            };
        }
        if proposal.approvals.contains(admin.clone()) {
            {
                ::core::panicking::panic_fmt(format_args!("Admin already approved"));
            };
        }
        proposal.approvals.push_back(admin);
        env.storage().instance().set(&SystemKey::Proposal(proposal_id), &proposal);
    }
    pub fn execute_proposal(env: Env, proposal_id: u64) {
        let mut proposal: MultiSigProposal = env
            .storage()
            .instance()
            .get(&SystemKey::Proposal(proposal_id))
            .expect("Proposal not found");
        if proposal.executed {
            {
                ::core::panicking::panic_fmt(format_args!("Proposal already executed"));
            };
        }
        if env.ledger().timestamp() > proposal.expires_at {
            {
                ::core::panicking::panic_fmt(format_args!("Proposal expired"));
            };
        }
        if proposal.approvals.len() < proposal.required_approvals {
            {
                ::core::panicking::panic_fmt(format_args!("Threshold not met"));
            };
        }
        match proposal.action.clone() {
            ProposalAction::VerifyVet(addr) => {
                Self::_verify_vet_internal(&env, addr);
            }
            ProposalAction::RevokeVet(addr) => {
                Self::_revoke_vet_internal(&env, addr);
            }
            ProposalAction::UpgradeContract(_code_hash) => {}
            ProposalAction::ChangeAdmin(params) => {
                let (admins, threshold) = params;
                if threshold == 0 || threshold > admins.len() {
                    {
                        ::core::panicking::panic_fmt(format_args!("Invalid threshold"));
                    };
                }
                env.storage().instance().set(&SystemKey::Admins, &admins);
                env.storage().instance().set(&SystemKey::AdminThreshold, &threshold);
                env.storage().instance().remove(&DataKey::Admin);
            }
        }
        proposal.executed = true;
        env.storage().instance().set(&SystemKey::Proposal(proposal_id), &proposal);
    }
    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<MultiSigProposal> {
        env.storage().instance().get(&SystemKey::Proposal(proposal_id))
    }
    pub fn add_vet_review(
        env: Env,
        reviewer: Address,
        vet: Address,
        rating: u32,
        comment: String,
    ) -> u64 {
        reviewer.require_auth();
        if !(1..=5).contains(&rating) {
            {
                ::core::panicking::panic_fmt(
                    format_args!("Rating must be between 1 and 5"),
                );
            };
        }
        if env
            .storage()
            .instance()
            .has(&ReviewKey::VetReviewByOwnerVet((reviewer.clone(), vet.clone())))
        {
            {
                ::core::panicking::panic_fmt(
                    format_args!("You have already reviewed this veterinarian"),
                );
            };
        }
        let count: u64 = env
            .storage()
            .instance()
            .get(&ReviewKey::VetReviewCount)
            .unwrap_or(0);
        let id = count + 1;
        let review = VetReview {
            id,
            vet_address: vet.clone(),
            reviewer: reviewer.clone(),
            rating,
            comment,
            date: env.ledger().timestamp(),
        };
        env.storage().instance().set(&ReviewKey::VetReview(id), &review);
        env.storage().instance().set(&ReviewKey::VetReviewCount, &id);
        let vet_count: u64 = env
            .storage()
            .instance()
            .get(&ReviewKey::VetReviewCountByVet(vet.clone()))
            .unwrap_or(0);
        let new_vet_count = vet_count + 1;
        env.storage()
            .instance()
            .set(&ReviewKey::VetReviewCountByVet(vet.clone()), &new_vet_count);
        env.storage()
            .instance()
            .set(&ReviewKey::VetReviewByVetIndex((vet.clone(), new_vet_count)), &id);
        env.storage()
            .instance()
            .set(&ReviewKey::VetReviewByOwnerVet((reviewer, vet)), &id);
        id
    }
    pub fn get_vet_reviews(env: Env, vet: Address) -> Vec<VetReview> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&ReviewKey::VetReviewCountByVet(vet.clone()))
            .unwrap_or(0);
        let mut reviews = Vec::new(&env);
        for i in 1..=count {
            if let Some(review_id) = env
                .storage()
                .instance()
                .get::<ReviewKey, u64>(&ReviewKey::VetReviewByVetIndex((vet.clone(), i)))
            {
                if let Some(review) = env
                    .storage()
                    .instance()
                    .get::<ReviewKey, VetReview>(&ReviewKey::VetReview(review_id))
                {
                    reviews.push_back(review);
                }
            }
        }
        reviews
    }
    pub fn get_vet_average_rating(env: Env, vet: Address) -> u32 {
        let reviews = Self::get_vet_reviews(env.clone(), vet);
        if reviews.is_empty() {
            return 0;
        }
        let mut total = 0u32;
        for review in reviews.iter() {
            total += review.rating;
        }
        total / reviews.len()
    }
    pub fn add_medication(
        env: Env,
        pet_id: u64,
        name: String,
        dosage: String,
        frequency: String,
        start_date: u64,
        end_date: Option<u64>,
        prescribing_vet: Address,
    ) -> u64 {
        prescribing_vet.require_auth();
        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        let count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::MedicationCount)
            .unwrap_or(0);
        let id = count + 1;
        let medication = Medication {
            id,
            pet_id,
            name,
            dosage,
            frequency,
            start_date,
            end_date,
            prescribing_vet: prescribing_vet.clone(),
            active: true,
        };
        env.storage().instance().set(&MedicalKey::GlobalMedication(id), &medication);
        env.storage().instance().set(&MedicalKey::MedicationCount, &id);
        let pet_med_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetMedicationCount(pet_id))
            .unwrap_or(0);
        let new_count = pet_med_count + 1;
        env.storage()
            .instance()
            .set(&MedicalKey::PetMedicationCount(pet_id), &new_count);
        env.storage()
            .instance()
            .set(&MedicalKey::PetMedicationIndex((pet_id, new_count)), &id);
        id
    }
    pub fn get_active_medications(env: Env, pet_id: u64) -> Vec<Medication> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetMedicationCount(pet_id))
            .unwrap_or(0);
        let mut active_meds = Vec::new(&env);
        for i in 1..=count {
            if let Some(med_id) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetMedicationIndex((pet_id, i)))
            {
                if let Some(med) = env
                    .storage()
                    .instance()
                    .get::<MedicalKey, Medication>(&MedicalKey::GlobalMedication(med_id))
                {
                    if med.active {
                        active_meds.push_back(med);
                    }
                }
            }
        }
        active_meds
    }
    pub fn mark_medication_completed(env: Env, medication_id: u64) {
        if let Some(mut med) = env
            .storage()
            .instance()
            .get::<MedicalKey, Medication>(&MedicalKey::GlobalMedication(medication_id))
        {
            med.prescribing_vet.require_auth();
            med.active = false;
            if med.end_date.is_none() {
                med.end_date = Some(env.ledger().timestamp());
            }
            env.storage()
                .instance()
                .set(&MedicalKey::GlobalMedication(medication_id), &med);
        } else {
            {
                ::core::panicking::panic_fmt(format_args!("Medication not found"));
            };
        }
    }
    pub fn add_treatment(
        env: Env,
        pet_id: u64,
        vet_address: Address,
        treatment_type: TreatmentType,
        date: u64,
        notes: String,
        cost: Option<i128>,
        outcome: String,
    ) -> u64 {
        vet_address.require_auth();
        if !Self::is_verified_vet(env.clone(), vet_address.clone()) {
            {
                ::core::panicking::panic_fmt(format_args!("Veterinarian not verified"));
            };
        }
        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        let treatment_count: u64 = env
            .storage()
            .instance()
            .get(&TreatmentKey::TreatmentCount)
            .unwrap_or(0);
        let treatment_id = treatment_count + 1;
        let now = env.ledger().timestamp();
        let treatment = Treatment {
            id: treatment_id,
            pet_id,
            treatment_type: treatment_type.clone(),
            date,
            vet_address: vet_address.clone(),
            notes,
            cost,
            outcome,
        };
        env.storage().instance().set(&TreatmentKey::Treatment(treatment_id), &treatment);
        env.storage().instance().set(&TreatmentKey::TreatmentCount, &treatment_id);
        let pet_treatment_count: u64 = env
            .storage()
            .instance()
            .get(&TreatmentKey::PetTreatmentCount(pet_id))
            .unwrap_or(0);
        let new_pet_treatment_count = pet_treatment_count + 1;
        env.storage()
            .instance()
            .set(&TreatmentKey::PetTreatmentCount(pet_id), &new_pet_treatment_count);
        env.storage()
            .instance()
            .set(
                &TreatmentKey::PetTreatmentIndex((pet_id, new_pet_treatment_count)),
                &treatment_id,
            );
        env.events()
            .publish(
                (String::from_str(&env, "TreatmentAdded"), pet_id),
                TreatmentAddedEvent {
                    treatment_id,
                    pet_id,
                    vet_address,
                    treatment_type,
                    timestamp: now,
                },
            );
        treatment_id
    }
    pub fn get_treatment(env: Env, treatment_id: u64) -> Option<Treatment> {
        env.storage()
            .instance()
            .get::<TreatmentKey, Treatment>(&TreatmentKey::Treatment(treatment_id))
    }
    pub fn get_treatment_history(env: Env, pet_id: u64) -> Vec<Treatment> {
        if env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(pet_id)).is_none()
        {
            return Vec::new(&env);
        }
        let count: u64 = env
            .storage()
            .instance()
            .get(&TreatmentKey::PetTreatmentCount(pet_id))
            .unwrap_or(0);
        let mut history = Vec::new(&env);
        for i in 1..=count {
            if let Some(tid) = env
                .storage()
                .instance()
                .get::<TreatmentKey, u64>(&TreatmentKey::PetTreatmentIndex((pet_id, i)))
            {
                if let Some(treatment) = env
                    .storage()
                    .instance()
                    .get::<TreatmentKey, Treatment>(&TreatmentKey::Treatment(tid))
                {
                    history.push_back(treatment);
                }
            }
        }
        history
    }
    pub fn get_treatments_by_type(
        env: Env,
        pet_id: u64,
        treatment_type: TreatmentType,
    ) -> Vec<Treatment> {
        let history = Self::get_treatment_history(env.clone(), pet_id);
        let mut filtered = Vec::new(&env);
        for treatment in history.iter() {
            if treatment.treatment_type == treatment_type {
                filtered.push_back(treatment);
            }
        }
        filtered
    }
    pub fn add_insurance_policy(
        env: Env,
        pet_id: u64,
        policy_id: String,
        provider: String,
        coverage_type: String,
        premium: u64,
        coverage_limit: u64,
        expiry_date: u64,
    ) -> bool {
        if env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(pet_id)).is_none()
        {
            return false;
        }
        let start_date = env.ledger().timestamp();
        let policy = InsurancePolicy {
            policy_id: policy_id.clone(),
            provider: provider.clone(),
            coverage_type,
            premium,
            coverage_limit,
            start_date,
            expiry_date,
            active: true,
        };
        env.storage().instance().set(&InsuranceKey::Policy(pet_id), &policy);
        env.events()
            .publish(
                (String::from_str(&env, "InsuranceAdded"), pet_id),
                InsuranceAddedEvent {
                    pet_id,
                    policy_id,
                    provider,
                    timestamp: start_date,
                },
            );
        true
    }
    pub fn get_pet_insurance(env: Env, pet_id: u64) -> Option<InsurancePolicy> {
        env.storage()
            .instance()
            .get::<InsuranceKey, InsurancePolicy>(&InsuranceKey::Policy(pet_id))
    }
    pub fn update_insurance_status(env: Env, pet_id: u64, active: bool) -> bool {
        if let Some(mut policy) = env
            .storage()
            .instance()
            .get::<InsuranceKey, InsurancePolicy>(&InsuranceKey::Policy(pet_id))
        {
            policy.active = active;
            env.storage().instance().set(&InsuranceKey::Policy(pet_id), &policy);
            env.events()
                .publish(
                    (String::from_str(&env, "InsuranceUpdated"), pet_id),
                    InsuranceUpdatedEvent {
                        pet_id,
                        policy_id: policy.policy_id,
                        active,
                        timestamp: env.ledger().timestamp(),
                    },
                );
            return true;
        }
        false
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_INIT_ADMIN: [u8; 52usize] = PetChainContract::spec_xdr_init_admin();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_init_admin() -> [u8; 52usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\ninit_admin\0\0\0\0\0\x01\0\0\0\0\0\0\0\x05admin\0\0\0\0\0\0\x13\0\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_INIT_MULTISIG: [u8; 104usize] = PetChainContract::spec_xdr_init_multisig();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_init_multisig() -> [u8; 104usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\rinit_multisig\0\0\0\0\0\0\x03\0\0\0\0\0\0\0\x07invoker\0\0\0\0\x13\0\0\0\0\0\0\0\x06admins\0\0\0\0\x03\xea\0\0\0\x13\0\0\0\0\0\0\0\tthreshold\0\0\0\0\0\0\x04\0\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(clippy::too_many_arguments)]
pub static __SPEC_XDR_FN_REGISTER_PET: [u8; 288usize] = PetChainContract::spec_xdr_register_pet();
impl PetChainContract {
    #[allow(non_snake_case)]
    #[allow(clippy::too_many_arguments)]
    pub const fn spec_xdr_register_pet() -> [u8; 288usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0cregister_pet\0\0\0\n\0\0\0\0\0\0\0\x05owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x04name\0\0\0\x10\0\0\0\0\0\0\0\x08birthday\0\0\0\x10\0\0\0\0\0\0\0\x06gender\0\0\0\0\x07\xd0\0\0\0\x06Gender\0\0\0\0\0\0\0\0\0\x07species\0\0\0\x07\xd0\0\0\0\x07Species\0\0\0\0\0\0\0\0\x05breed\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x05color\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x06weight\0\0\0\0\0\x04\0\0\0\0\0\0\0\x0cmicrochip_id\0\0\x03\xe8\0\0\0\x10\0\0\0\0\0\0\0\rprivacy_level\0\0\0\0\0\x07\xd0\0\0\0\x0cPrivacyLevel\0\0\0\x01\0\0\0\x06"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(clippy::too_many_arguments)]
pub static __SPEC_XDR_FN_UPDATE_PET_PROFILE: [u8; 292usize] = PetChainContract::spec_xdr_update_pet_profile();
impl PetChainContract {
    #[allow(non_snake_case)]
    #[allow(clippy::too_many_arguments)]
    pub const fn spec_xdr_update_pet_profile() -> [u8; 292usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x12update_pet_profile\0\0\0\0\0\n\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x04name\0\0\0\x10\0\0\0\0\0\0\0\x08birthday\0\0\0\x10\0\0\0\0\0\0\0\x06gender\0\0\0\0\x07\xd0\0\0\0\x06Gender\0\0\0\0\0\0\0\0\0\x07species\0\0\0\x07\xd0\0\0\0\x07Species\0\0\0\0\0\0\0\0\x05breed\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x05color\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x06weight\0\0\0\0\0\x04\0\0\0\0\0\0\0\x0cmicrochip_id\0\0\x03\xe8\0\0\0\x10\0\0\0\0\0\0\0\rprivacy_level\0\0\0\0\0\x07\xd0\0\0\0\x0cPrivacyLevel\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_PET: [u8; 68usize] = PetChainContract::spec_xdr_get_pet();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_pet() -> [u8; 68usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x07get_pet\0\0\0\0\x01\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xe8\0\0\x07\xd0\0\0\0\nPetProfile\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_IS_PET_ACTIVE: [u8; 56usize] = PetChainContract::spec_xdr_is_pet_active();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_is_pet_active() -> [u8; 56usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\ris_pet_active\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_PET_OWNER: [u8; 60usize] = PetChainContract::spec_xdr_get_pet_owner();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_pet_owner() -> [u8; 60usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\rget_pet_owner\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xe8\0\0\0\x13"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_ACTIVATE_PET: [u8; 48usize] = PetChainContract::spec_xdr_activate_pet();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_activate_pet() -> [u8; 48usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0cactivate_pet\0\0\0\x01\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_DEACTIVATE_PET: [u8; 52usize] = PetChainContract::spec_xdr_deactivate_pet();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_deactivate_pet() -> [u8; 52usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0edeactivate_pet\0\0\0\0\0\x01\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_ADD_PET_PHOTO: [u8; 84usize] = PetChainContract::spec_xdr_add_pet_photo();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_add_pet_photo() -> [u8; 84usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\radd_pet_photo\0\0\0\0\0\0\x02\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\nphoto_hash\0\0\0\0\0\x10\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_PET_PHOTOS: [u8; 64usize] = PetChainContract::spec_xdr_get_pet_photos();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_pet_photos() -> [u8; 64usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0eget_pet_photos\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\0\x10"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_TRANSFER_PET_OWNERSHIP: [u8; 76usize] = PetChainContract::spec_xdr_transfer_pet_ownership();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_transfer_pet_ownership() -> [u8; 76usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x16transfer_pet_ownership\0\0\0\0\0\x02\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x02to\0\0\0\0\0\x13\0\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_ACCEPT_PET_TRANSFER: [u8; 56usize] = PetChainContract::spec_xdr_accept_pet_transfer();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_accept_pet_transfer() -> [u8; 56usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x13accept_pet_transfer\0\0\0\0\x01\0\0\0\0\0\0\0\x02id\0\0\0\0\0\x06\0\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_REGISTER_PET_OWNER: [u8; 128usize] = PetChainContract::spec_xdr_register_pet_owner();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_register_pet_owner() -> [u8; 128usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x12register_pet_owner\0\0\0\0\0\x04\0\0\0\0\0\0\0\x05owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x04name\0\0\0\x10\0\0\0\0\0\0\0\x05email\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x11emergency_contact\0\0\0\0\0\0\x10\0\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_IS_OWNER_REGISTERED: [u8; 64usize] = PetChainContract::spec_xdr_is_owner_registered();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_is_owner_registered() -> [u8; 64usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x13is_owner_registered\0\0\0\0\x01\0\0\0\0\0\0\0\x05owner\0\0\0\0\0\0\x13\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_UPDATE_OWNER_PROFILE: [u8; 132usize] = PetChainContract::spec_xdr_update_owner_profile();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_update_owner_profile() -> [u8; 132usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x14update_owner_profile\0\0\0\x04\0\0\0\0\0\0\0\x05owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x04name\0\0\0\x10\0\0\0\0\0\0\0\x05email\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x11emergency_contact\0\0\0\0\0\0\x10\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_REGISTER_VET: [u8; 132usize] = PetChainContract::spec_xdr_register_vet();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_register_vet() -> [u8; 132usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0cregister_vet\0\0\0\x04\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13\0\0\0\0\0\0\0\x04name\0\0\0\x10\0\0\0\0\0\0\0\x0elicense_number\0\0\0\0\0\x10\0\0\0\0\0\0\0\x0especialization\0\0\0\0\0\x10\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_VERIFY_VET: [u8; 80usize] = PetChainContract::spec_xdr_verify_vet();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_verify_vet() -> [u8; 80usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\nverify_vet\0\0\0\0\0\x02\0\0\0\0\0\0\0\x05admin\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_REVOKE_VET_LICENSE: [u8; 88usize] = PetChainContract::spec_xdr_revoke_vet_license();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_revoke_vet_license() -> [u8; 88usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x12revoke_vet_license\0\0\0\0\0\x02\0\0\0\0\0\0\0\x05admin\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_IS_VERIFIED_VET: [u8; 64usize] = PetChainContract::spec_xdr_is_verified_vet();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_is_verified_vet() -> [u8; 64usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0fis_verified_vet\0\0\0\0\x01\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_VET: [u8; 68usize] = PetChainContract::spec_xdr_get_vet();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_vet() -> [u8; 68usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x07get_vet\0\0\0\0\x01\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13\0\0\0\x01\0\0\x03\xe8\0\0\x07\xd0\0\0\0\x03Vet\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_VET_BY_LICENSE: [u8; 84usize] = PetChainContract::spec_xdr_get_vet_by_license();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_vet_by_license() -> [u8; 84usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x12get_vet_by_license\0\0\0\0\0\x01\0\0\0\0\0\0\0\x0elicense_number\0\0\0\0\0\x10\0\0\0\x01\0\0\x03\xe8\0\0\x07\xd0\0\0\0\x03Vet\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
/// Update clinic info for a vet. Only the vet can update their own clinic info.
pub static __SPEC_XDR_FN_UPDATE_CLINIC_INFO: [u8; 184usize] = PetChainContract::spec_xdr_update_clinic_info();
impl PetChainContract {
    #[allow(non_snake_case)]
    /// Update clinic info for a vet. Only the vet can update their own clinic info.
    pub const fn spec_xdr_update_clinic_info() -> [u8; 184usize] {
        *b"\0\0\0\0\0\0\0LUpdate clinic info for a vet. Only the vet can update their own clinic info.\0\0\0\x12update_clinic_info\0\0\0\0\0\x02\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13\0\0\0\0\0\0\0\x0bclinic_info\0\0\0\x07\xd0\0\0\0\nClinicInfo\0\0\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(clippy::too_many_arguments)]
pub static __SPEC_XDR_FN_ADD_VACCINATION: [u8; 228usize] = PetChainContract::spec_xdr_add_vaccination();
impl PetChainContract {
    #[allow(non_snake_case)]
    #[allow(clippy::too_many_arguments)]
    pub const fn spec_xdr_add_vaccination() -> [u8; 228usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0fadd_vaccination\0\0\0\0\x07\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0cveterinarian\0\0\0\x13\0\0\0\0\0\0\0\x0cvaccine_type\0\0\x07\xd0\0\0\0\x0bVaccineType\0\0\0\0\0\0\0\0\x0cvaccine_name\0\0\0\x10\0\0\0\0\0\0\0\x0fadministered_at\0\0\0\0\x06\0\0\0\0\0\0\0\rnext_due_date\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0cbatch_number\0\0\0\x10\0\0\0\x01\0\0\0\x06"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_VACCINATIONS: [u8; 84usize] = PetChainContract::spec_xdr_get_vaccinations();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_vaccinations() -> [u8; 84usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x10get_vaccinations\0\0\0\x01\0\0\0\0\0\0\0\nvaccine_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xe8\0\0\x07\xd0\0\0\0\x0bVaccination\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_VACCINATION_HISTORY: [u8; 88usize] = PetChainContract::spec_xdr_get_vaccination_history();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_vaccination_history() -> [u8; 88usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x17get_vaccination_history\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\x0bVaccination\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_UPCOMING_VACCINATIONS: [u8; 120usize] = PetChainContract::spec_xdr_get_upcoming_vaccinations();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_upcoming_vaccinations() -> [u8; 120usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x19get_upcoming_vaccinations\0\0\0\0\0\0\x02\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0edays_threshold\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\x0bVaccination\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_IS_VACCINATION_CURRENT: [u8; 108usize] = PetChainContract::spec_xdr_is_vaccination_current();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_is_vaccination_current() -> [u8; 108usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x16is_vaccination_current\0\0\0\0\0\x02\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0cvaccine_type\0\0\x07\xd0\0\0\0\x0bVaccineType\0\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_OVERDUE_VACCINATIONS: [u8; 88usize] = PetChainContract::spec_xdr_get_overdue_vaccinations();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_overdue_vaccinations() -> [u8; 88usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x18get_overdue_vaccinations\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\x0bVaccineType\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_LINK_TAG_TO_PET: [u8; 64usize] = PetChainContract::spec_xdr_link_tag_to_pet();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_link_tag_to_pet() -> [u8; 64usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0flink_tag_to_pet\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xee\0\0\0 "
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_PET_BY_TAG: [u8; 84usize] = PetChainContract::spec_xdr_get_pet_by_tag();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_pet_by_tag() -> [u8; 84usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0eget_pet_by_tag\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06tag_id\0\0\0\0\x03\xee\0\0\0 \0\0\0\x01\0\0\x03\xe8\0\0\x07\xd0\0\0\0\nPetProfile\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_TAG: [u8; 72usize] = PetChainContract::spec_xdr_get_tag();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_tag() -> [u8; 72usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x07get_tag\0\0\0\0\x01\0\0\0\0\0\0\0\x06tag_id\0\0\0\0\x03\xee\0\0\0 \0\0\0\x01\0\0\x03\xe8\0\0\x07\xd0\0\0\0\x06PetTag\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_TAG_BY_PET: [u8; 68usize] = PetChainContract::spec_xdr_get_tag_by_pet();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_tag_by_pet() -> [u8; 68usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0eget_tag_by_pet\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xe8\0\0\x03\xee\0\0\0 "
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_UPDATE_TAG_MESSAGE: [u8; 88usize] = PetChainContract::spec_xdr_update_tag_message();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_update_tag_message() -> [u8; 88usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x12update_tag_message\0\0\0\0\0\x02\0\0\0\0\0\0\0\x06tag_id\0\0\0\0\x03\xee\0\0\0 \0\0\0\0\0\0\0\x07message\0\0\0\0\x10\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_DEACTIVATE_TAG: [u8; 64usize] = PetChainContract::spec_xdr_deactivate_tag();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_deactivate_tag() -> [u8; 64usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0edeactivate_tag\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06tag_id\0\0\0\0\x03\xee\0\0\0 \0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_REACTIVATE_TAG: [u8; 64usize] = PetChainContract::spec_xdr_reactivate_tag();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_reactivate_tag() -> [u8; 64usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0ereactivate_tag\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06tag_id\0\0\0\0\x03\xee\0\0\0 \0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_IS_TAG_ACTIVE: [u8; 64usize] = PetChainContract::spec_xdr_is_tag_active();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_is_tag_active() -> [u8; 64usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\ris_tag_active\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06tag_id\0\0\0\0\x03\xee\0\0\0 \0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_OWNERSHIP_HISTORY: [u8; 92usize] = PetChainContract::spec_xdr_get_ownership_history();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_ownership_history() -> [u8; 92usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x15get_ownership_history\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\x0fOwnershipRecord\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_SET_EMERGENCY_CONTACTS: [u8; 136usize] = PetChainContract::spec_xdr_set_emergency_contacts();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_set_emergency_contacts() -> [u8; 136usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x16set_emergency_contacts\0\0\0\0\0\x03\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x08contacts\0\0\x03\xea\0\0\x07\xd0\0\0\0\x10EmergencyContact\0\0\0\0\0\0\0\rmedical_notes\0\0\0\0\0\0\x10\0\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_EMERGENCY_INFO: [u8; 104usize] = PetChainContract::spec_xdr_get_emergency_info();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_emergency_info() -> [u8; 104usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x12get_emergency_info\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xe8\0\0\x03\xed\0\0\0\x02\0\0\x03\xea\0\0\x07\xd0\0\0\0\x10EmergencyContact\0\0\0\x10"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
/// Get emergency contacts for a pet (publicly accessible - no auth required for emergency responders)
pub static __SPEC_XDR_FN_GET_EMERGENCY_CONTACTS: [u8; 192usize] = PetChainContract::spec_xdr_get_emergency_contacts();
impl PetChainContract {
    #[allow(non_snake_case)]
    /// Get emergency contacts for a pet (publicly accessible - no auth required for emergency responders)
    pub const fn spec_xdr_get_emergency_contacts() -> [u8; 192usize] {
        *b"\0\0\0\0\0\0\0bGet emergency contacts for a pet (publicly accessible - no auth required for emergency responders)\0\0\0\0\0\x16get_emergency_contacts\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\x10EmergencyContact"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_ACCESSIBLE_PETS: [u8; 64usize] = PetChainContract::spec_xdr_get_accessible_pets();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_accessible_pets() -> [u8; 64usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x13get_accessible_pets\0\0\0\0\x01\0\0\0\0\0\0\0\x04user\0\0\0\x13\0\0\0\x01\0\0\x03\xea\0\0\0\x06"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_ALL_PETS_BY_OWNER: [u8; 88usize] = PetChainContract::spec_xdr_get_all_pets_by_owner();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_all_pets_by_owner() -> [u8; 88usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x15get_all_pets_by_owner\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x05owner\0\0\0\0\0\0\x13\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\nPetProfile\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_PETS_BY_OWNER: [u8; 84usize] = PetChainContract::spec_xdr_get_pets_by_owner();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_pets_by_owner() -> [u8; 84usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x11get_pets_by_owner\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x05owner\0\0\0\0\0\0\x13\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\nPetProfile\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_PETS_BY_SPECIES: [u8; 84usize] = PetChainContract::spec_xdr_get_pets_by_species();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_pets_by_species() -> [u8; 84usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x13get_pets_by_species\0\0\0\0\x01\0\0\0\0\0\0\0\x07species\0\0\0\0\x10\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\nPetProfile\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_ACTIVE_PETS: [u8; 60usize] = PetChainContract::spec_xdr_get_active_pets();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_active_pets() -> [u8; 60usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0fget_active_pets\0\0\0\0\0\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\nPetProfile\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GRANT_ACCESS: [u8; 144usize] = PetChainContract::spec_xdr_grant_access();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_grant_access() -> [u8; 144usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0cgrant_access\0\0\0\x04\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x07grantee\0\0\0\0\x13\0\0\0\0\0\0\0\x0caccess_level\0\0\x07\xd0\0\0\0\x0bAccessLevel\0\0\0\0\0\0\0\0\nexpires_at\0\0\0\0\x03\xe8\0\0\0\x06\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_REVOKE_ACCESS: [u8; 80usize] = PetChainContract::spec_xdr_revoke_access();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_revoke_access() -> [u8; 80usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\rrevoke_access\0\0\0\0\0\0\x02\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x07grantee\0\0\0\0\x13\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GRANT_TEMPORARY_CUSTODY: [u8; 184usize] = PetChainContract::spec_xdr_grant_temporary_custody();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_grant_temporary_custody() -> [u8; 184usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x17grant_temporary_custody\0\0\0\0\x05\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\tcustodian\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\nstart_date\0\0\0\0\0\x06\0\0\0\0\0\0\0\x08end_date\0\0\0\x06\0\0\0\0\0\0\0\x0bpermissions\0\0\0\x03\xea\0\0\0\x10\0\0\0\x01\0\0\x07\xd0\0\0\0\x10TemporaryCustody"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_REVOKE_TEMPORARY_CUSTODY: [u8; 64usize] = PetChainContract::spec_xdr_revoke_temporary_custody();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_revoke_temporary_custody() -> [u8; 64usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x18revoke_temporary_custody\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_IS_CUSTODY_VALID: [u8; 60usize] = PetChainContract::spec_xdr_is_custody_valid();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_is_custody_valid() -> [u8; 60usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x10is_custody_valid\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_ADD_MEDICAL_RECORD: [u8; 180usize] = PetChainContract::spec_xdr_add_medical_record();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_add_medical_record() -> [u8; 180usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x12add_medical_record\0\0\0\0\0\x06\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13\0\0\0\0\0\0\0\tdiagnosis\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\ttreatment\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x0bmedications\0\0\0\0\x10\0\0\0\0\0\0\0\x05notes\0\0\0\0\0\0\x10\0\0\0\x01\0\0\0\x06"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_UPDATE_MEDICAL_RECORD: [u8; 164usize] = PetChainContract::spec_xdr_update_medical_record();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_update_medical_record() -> [u8; 164usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x15update_medical_record\0\0\0\0\0\0\x05\0\0\0\0\0\0\0\trecord_id\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\tdiagnosis\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\ttreatment\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x0bmedications\0\0\0\0\x10\0\0\0\0\0\0\0\x05notes\0\0\0\0\0\0\x10\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_MEDICAL_RECORD: [u8; 92usize] = PetChainContract::spec_xdr_get_medical_record();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_medical_record() -> [u8; 92usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x12get_medical_record\0\0\0\0\0\x01\0\0\0\0\0\0\0\trecord_id\0\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xe8\0\0\x07\xd0\0\0\0\rMedicalRecord\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_PET_MEDICAL_RECORDS: [u8; 92usize] = PetChainContract::spec_xdr_get_pet_medical_records();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_pet_medical_records() -> [u8; 92usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x17get_pet_medical_records\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\rMedicalRecord\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_ACCESS_LOGS: [u8; 80usize] = PetChainContract::spec_xdr_get_access_logs();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_access_logs() -> [u8; 80usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0fget_access_logs\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\tAccessLog\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_CHECK_ACCESS: [u8; 88usize] = PetChainContract::spec_xdr_check_access();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_check_access() -> [u8; 88usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0ccheck_access\0\0\0\x02\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x04user\0\0\0\x13\0\0\0\x01\0\0\x07\xd0\0\0\0\x0bAccessLevel\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_AUTHORIZED_USERS: [u8; 68usize] = PetChainContract::spec_xdr_get_authorized_users();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_authorized_users() -> [u8; 68usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x14get_authorized_users\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\0\x13"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_ACCESS_GRANT: [u8; 100usize] = PetChainContract::spec_xdr_get_access_grant();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_access_grant() -> [u8; 100usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x10get_access_grant\0\0\0\x02\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x07grantee\0\0\0\0\x13\0\0\0\x01\0\0\x03\xe8\0\0\x07\xd0\0\0\0\x0bAccessGrant\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_ADD_LAB_RESULT: [u8; 224usize] = PetChainContract::spec_xdr_add_lab_result();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_add_lab_result() -> [u8; 224usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0eadd_lab_result\0\0\0\0\0\x07\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13\0\0\0\0\0\0\0\ttest_type\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x07results\0\0\0\0\x10\0\0\0\0\0\0\0\x10reference_ranges\0\0\0\x10\0\0\0\0\0\0\0\x0fattachment_hash\0\0\0\x03\xe8\0\0\0\x10\0\0\0\0\0\0\0\x11medical_record_id\0\0\0\0\0\x03\xe8\0\0\0\x06\0\0\0\x01\0\0\0\x06"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_LAB_RESULT: [u8; 88usize] = PetChainContract::spec_xdr_get_lab_result();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_lab_result() -> [u8; 88usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0eget_lab_result\0\0\0\0\0\x01\0\0\0\0\0\0\0\rlab_result_id\0\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xe8\0\0\x07\xd0\0\0\0\tLabResult\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_LAB_RESULTS: [u8; 80usize] = PetChainContract::spec_xdr_get_lab_results();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_lab_results() -> [u8; 80usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0fget_lab_results\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\tLabResult\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(clippy::too_many_arguments)]
pub static __SPEC_XDR_FN_ADD_MEDICATION_TO_RECORD: [u8; 204usize] = PetChainContract::spec_xdr_add_medication_to_record();
impl PetChainContract {
    #[allow(non_snake_case)]
    #[allow(clippy::too_many_arguments)]
    pub const fn spec_xdr_add_medication_to_record() -> [u8; 204usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x18add_medication_to_record\0\0\0\x07\0\0\0\0\0\0\0\trecord_id\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\x04name\0\0\0\x10\0\0\0\0\0\0\0\x06dosage\0\0\0\0\0\x10\0\0\0\0\0\0\0\tfrequency\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\nstart_date\0\0\0\0\0\x06\0\0\0\0\0\0\0\x08end_date\0\0\0\x06\0\0\0\0\0\0\0\x0fprescribing_vet\0\0\0\0\x13\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_MARK_RECORD_MED_COMPLETED: [u8; 100usize] = PetChainContract::spec_xdr_mark_record_med_completed();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_mark_record_med_completed() -> [u8; 100usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x19mark_record_med_completed\0\0\0\0\0\0\x02\0\0\0\0\0\0\0\trecord_id\0\0\0\0\0\0\x06\0\0\0\0\0\0\0\tmed_index\0\0\0\0\0\0\x04\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_ACTIVE_RECORD_MEDS: [u8; 88usize] = PetChainContract::spec_xdr_get_active_record_meds();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_active_record_meds() -> [u8; 88usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x16get_active_record_meds\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\nMedication\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_RECORD_MED_HISTORY: [u8; 88usize] = PetChainContract::spec_xdr_get_record_med_history();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_record_med_history() -> [u8; 88usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x16get_record_med_history\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\nMedication\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_BATCH_ADD_VACCINATIONS: [u8; 124usize] = PetChainContract::spec_xdr_batch_add_vaccinations();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_batch_add_vaccinations() -> [u8; 124usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x16batch_add_vaccinations\0\0\0\0\0\x02\0\0\0\0\0\0\0\x0cveterinarian\0\0\0\x13\0\0\0\0\0\0\0\x0cvaccinations\0\0\x03\xea\0\0\x07\xd0\0\0\0\x10VaccinationInput\0\0\0\x01\0\0\x03\xea\0\0\0\x06"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_BATCH_ADD_RECORDS: [u8; 120usize] = PetChainContract::spec_xdr_batch_add_records();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_batch_add_records() -> [u8; 120usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x11batch_add_records\0\0\0\0\0\0\x02\0\0\0\0\0\0\0\x0cveterinarian\0\0\0\x13\0\0\0\0\0\0\0\x07records\0\0\0\x03\xea\0\0\x07\xd0\0\0\0\x12MedicalRecordInput\0\0\0\0\0\x01\0\0\x03\xea\0\0\0\x06"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
/// Report a pet as lost
pub static __SPEC_XDR_FN_REPORT_LOST: [u8; 140usize] = PetChainContract::spec_xdr_report_lost();
impl PetChainContract {
    #[allow(non_snake_case)]
    /// Report a pet as lost
    pub const fn spec_xdr_report_lost() -> [u8; 140usize] {
        *b"\0\0\0\0\0\0\0\x14Report a pet as lost\0\0\0\x0breport_lost\0\0\0\0\x03\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x12last_seen_location\0\0\0\0\0\x10\0\0\0\0\0\0\0\rreward_amount\0\0\0\0\0\x03\xe8\0\0\0\x06\0\0\0\x01\0\0\0\x06"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
/// Report a sighting of a lost pet
pub static __SPEC_XDR_FN_REPORT_SIGHTING: [u8; 136usize] = PetChainContract::spec_xdr_report_sighting();
impl PetChainContract {
    #[allow(non_snake_case)]
    /// Report a sighting of a lost pet
    pub const fn spec_xdr_report_sighting() -> [u8; 136usize] {
        *b"\0\0\0\0\0\0\0\x1fReport a sighting of a lost pet\0\0\0\0\x0freport_sighting\0\0\0\0\x03\0\0\0\0\0\0\0\x08alert_id\0\0\0\x06\0\0\0\0\0\0\0\x08location\0\0\0\x10\0\0\0\0\0\0\0\x0bdescription\0\0\0\0\x10\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
/// Mark a lost pet as found
pub static __SPEC_XDR_FN_REPORT_FOUND: [u8; 80usize] = PetChainContract::spec_xdr_report_found();
impl PetChainContract {
    #[allow(non_snake_case)]
    /// Mark a lost pet as found
    pub const fn spec_xdr_report_found() -> [u8; 80usize] {
        *b"\0\0\0\0\0\0\0\x18Mark a lost pet as found\0\0\0\x0creport_found\0\0\0\x01\0\0\0\0\0\0\0\x08alert_id\0\0\0\x06\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
/// Cancel a lost pet alert
pub static __SPEC_XDR_FN_CANCEL_LOST_ALERT: [u8; 88usize] = PetChainContract::spec_xdr_cancel_lost_alert();
impl PetChainContract {
    #[allow(non_snake_case)]
    /// Cancel a lost pet alert
    pub const fn spec_xdr_cancel_lost_alert() -> [u8; 88usize] {
        *b"\0\0\0\0\0\0\0\x17Cancel a lost pet alert\0\0\0\0\x11cancel_lost_alert\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x08alert_id\0\0\0\x06\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
/// Get all active lost pet alerts
pub static __SPEC_XDR_FN_GET_ACTIVE_ALERTS: [u8; 96usize] = PetChainContract::spec_xdr_get_active_alerts();
impl PetChainContract {
    #[allow(non_snake_case)]
    /// Get all active lost pet alerts
    pub const fn spec_xdr_get_active_alerts() -> [u8; 96usize] {
        *b"\0\0\0\0\0\0\0\x1eGet all active lost pet alerts\0\0\0\0\0\x11get_active_alerts\0\0\0\0\0\0\0\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\x0cLostPetAlert"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
/// Get a specific alert by ID
pub static __SPEC_XDR_FN_GET_ALERT: [u8; 104usize] = PetChainContract::spec_xdr_get_alert();
impl PetChainContract {
    #[allow(non_snake_case)]
    /// Get a specific alert by ID
    pub const fn spec_xdr_get_alert() -> [u8; 104usize] {
        *b"\0\0\0\0\0\0\0\x1aGet a specific alert by ID\0\0\0\0\0\tget_alert\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x08alert_id\0\0\0\x06\0\0\0\x01\0\0\x03\xe8\0\0\x07\xd0\0\0\0\x0cLostPetAlert"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
/// Get sightings for a specific alert
pub static __SPEC_XDR_FN_GET_ALERT_SIGHTINGS: [u8; 124usize] = PetChainContract::spec_xdr_get_alert_sightings();
impl PetChainContract {
    #[allow(non_snake_case)]
    /// Get sightings for a specific alert
    pub const fn spec_xdr_get_alert_sightings() -> [u8; 124usize] {
        *b"\0\0\0\0\0\0\0\"Get sightings for a specific alert\0\0\0\0\0\x13get_alert_sightings\0\0\0\0\x01\0\0\0\0\0\0\0\x08alert_id\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\x0eSightingReport\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
/// Get alerts for a specific pet
pub static __SPEC_XDR_FN_GET_PET_ALERTS: [u8; 112usize] = PetChainContract::spec_xdr_get_pet_alerts();
impl PetChainContract {
    #[allow(non_snake_case)]
    /// Get alerts for a specific pet
    pub const fn spec_xdr_get_pet_alerts() -> [u8; 112usize] {
        *b"\0\0\0\0\0\0\0\x1dGet alerts for a specific pet\0\0\0\0\0\0\x0eget_pet_alerts\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\x0cLostPetAlert"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
/// Set availability slots for a vet (only verified vets can set their availability)
pub static __SPEC_XDR_FN_SET_AVAILABILITY: [u8; 188usize] = PetChainContract::spec_xdr_set_availability();
impl PetChainContract {
    #[allow(non_snake_case)]
    /// Set availability slots for a vet (only verified vets can set their availability)
    pub const fn spec_xdr_set_availability() -> [u8; 188usize] {
        *b"\0\0\0\0\0\0\0PSet availability slots for a vet (only verified vets can set their availability)\0\0\0\x10set_availability\0\0\0\x03\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13\0\0\0\0\0\0\0\nstart_time\0\0\0\0\0\x06\0\0\0\0\0\0\0\x08end_time\0\0\0\x06\0\0\0\x01\0\0\0\x06"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
/// Get available slots for a vet on a specific date
pub static __SPEC_XDR_FN_GET_AVAILABLE_SLOTS: [u8; 156usize] = PetChainContract::spec_xdr_get_available_slots();
impl PetChainContract {
    #[allow(non_snake_case)]
    /// Get available slots for a vet on a specific date
    pub const fn spec_xdr_get_available_slots() -> [u8; 156usize] {
        *b"\0\0\0\0\0\0\00Get available slots for a vet on a specific date\0\0\0\x13get_available_slots\0\0\0\0\x02\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13\0\0\0\0\0\0\0\x04date\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\x10AvailabilitySlot"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GRANT_CONSENT: [u8; 144usize] = PetChainContract::spec_xdr_grant_consent();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_grant_consent() -> [u8; 144usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\rgrant_consent\0\0\0\0\0\0\x04\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x05owner\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x0cconsent_type\0\0\x07\xd0\0\0\0\x0bConsentType\0\0\0\0\0\0\0\0\ngranted_to\0\0\0\0\0\x13\0\0\0\x01\0\0\0\x06"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_REVOKE_CONSENT: [u8; 84usize] = PetChainContract::spec_xdr_revoke_consent();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_revoke_consent() -> [u8; 84usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0erevoke_consent\0\0\0\0\0\x02\0\0\0\0\0\0\0\nconsent_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x05owner\0\0\0\0\0\0\x13\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_CONSENT_HISTORY: [u8; 80usize] = PetChainContract::spec_xdr_get_consent_history();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_consent_history() -> [u8; 80usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x13get_consent_history\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\x07Consent\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
/// Book a slot (mark as unavailable)
pub static __SPEC_XDR_FN_BOOK_SLOT: [u8; 120usize] = PetChainContract::spec_xdr_book_slot();
impl PetChainContract {
    #[allow(non_snake_case)]
    /// Book a slot (mark as unavailable)
    pub const fn spec_xdr_book_slot() -> [u8; 120usize] {
        *b"\0\0\0\0\0\0\0!Book a slot (mark as unavailable)\0\0\0\0\0\0\tbook_slot\0\0\0\0\0\0\x02\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13\0\0\0\0\0\0\0\nslot_index\0\0\0\0\0\x06\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_VERSION: [u8; 56usize] = PetChainContract::spec_xdr_get_version();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_version() -> [u8; 56usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0bget_version\0\0\0\0\0\0\0\0\x01\0\0\x07\xd0\0\0\0\x0fContractVersion\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_UPGRADE_CONTRACT: [u8; 68usize] = PetChainContract::spec_xdr_upgrade_contract();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_upgrade_contract() -> [u8; 68usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x10upgrade_contract\0\0\0\x01\0\0\0\0\0\0\0\rnew_wasm_hash\0\0\0\0\0\x03\xee\0\0\0 \0\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_PROPOSE_UPGRADE: [u8; 92usize] = PetChainContract::spec_xdr_propose_upgrade();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_propose_upgrade() -> [u8; 92usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0fpropose_upgrade\0\0\0\0\x02\0\0\0\0\0\0\0\x08proposer\0\0\0\x13\0\0\0\0\0\0\0\rnew_wasm_hash\0\0\0\0\0\x03\xee\0\0\0 \0\0\0\x01\0\0\0\x06"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_APPROVE_UPGRADE: [u8; 64usize] = PetChainContract::spec_xdr_approve_upgrade();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_approve_upgrade() -> [u8; 64usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0fapprove_upgrade\0\0\0\0\x01\0\0\0\0\0\0\0\x0bproposal_id\0\0\0\0\x06\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_UPGRADE_PROPOSAL: [u8; 92usize] = PetChainContract::spec_xdr_get_upgrade_proposal();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_upgrade_proposal() -> [u8; 92usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x14get_upgrade_proposal\0\0\0\x01\0\0\0\0\0\0\0\x0bproposal_id\0\0\0\0\x06\0\0\0\x01\0\0\x03\xe8\0\0\x07\xd0\0\0\0\x0fUpgradeProposal\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_MIGRATE_VERSION: [u8; 96usize] = PetChainContract::spec_xdr_migrate_version();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_migrate_version() -> [u8; 96usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0fmigrate_version\0\0\0\0\x03\0\0\0\0\0\0\0\x05major\0\0\0\0\0\0\x04\0\0\0\0\0\0\0\x05minor\0\0\0\0\0\0\x04\0\0\0\0\0\0\0\x05patch\0\0\0\0\0\0\x04\0\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_PROPOSE_ACTION: [u8; 124usize] = PetChainContract::spec_xdr_propose_action();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_propose_action() -> [u8; 124usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0epropose_action\0\0\0\0\0\x03\0\0\0\0\0\0\0\x08proposer\0\0\0\x13\0\0\0\0\0\0\0\x06action\0\0\0\0\x07\xd0\0\0\0\x0eProposalAction\0\0\0\0\0\0\0\0\0\nexpires_in\0\0\0\0\0\x06\0\0\0\x01\0\0\0\x06"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_APPROVE_PROPOSAL: [u8; 80usize] = PetChainContract::spec_xdr_approve_proposal();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_approve_proposal() -> [u8; 80usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x10approve_proposal\0\0\0\x02\0\0\0\0\0\0\0\x05admin\0\0\0\0\0\0\x13\0\0\0\0\0\0\0\x0bproposal_id\0\0\0\0\x06\0\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_EXECUTE_PROPOSAL: [u8; 60usize] = PetChainContract::spec_xdr_execute_proposal();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_execute_proposal() -> [u8; 60usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x10execute_proposal\0\0\0\x01\0\0\0\0\0\0\0\x0bproposal_id\0\0\0\0\x06\0\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_PROPOSAL: [u8; 84usize] = PetChainContract::spec_xdr_get_proposal();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_proposal() -> [u8; 84usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0cget_proposal\0\0\0\x01\0\0\0\0\0\0\0\x0bproposal_id\0\0\0\0\x06\0\0\0\x01\0\0\x03\xe8\0\0\x07\xd0\0\0\0\x10MultiSigProposal"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_ADD_VET_REVIEW: [u8; 116usize] = PetChainContract::spec_xdr_add_vet_review();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_add_vet_review() -> [u8; 116usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0eadd_vet_review\0\0\0\0\0\x04\0\0\0\0\0\0\0\x08reviewer\0\0\0\x13\0\0\0\0\0\0\0\x03vet\0\0\0\0\x13\0\0\0\0\0\0\0\x06rating\0\0\0\0\0\x04\0\0\0\0\0\0\0\x07comment\0\0\0\0\x10\0\0\0\x01\0\0\0\x06"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_VET_REVIEWS: [u8; 76usize] = PetChainContract::spec_xdr_get_vet_reviews();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_vet_reviews() -> [u8; 76usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0fget_vet_reviews\0\0\0\0\x01\0\0\0\0\0\0\0\x03vet\0\0\0\0\x13\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\tVetReview\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_VET_AVERAGE_RATING: [u8; 64usize] = PetChainContract::spec_xdr_get_vet_average_rating();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_vet_average_rating() -> [u8; 64usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x16get_vet_average_rating\0\0\0\0\0\x01\0\0\0\0\0\0\0\x03vet\0\0\0\0\x13\0\0\0\x01\0\0\0\x04"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_ADD_MEDICATION: [u8; 196usize] = PetChainContract::spec_xdr_add_medication();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_add_medication() -> [u8; 196usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x0eadd_medication\0\0\0\0\0\x07\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x04name\0\0\0\x10\0\0\0\0\0\0\0\x06dosage\0\0\0\0\0\x10\0\0\0\0\0\0\0\tfrequency\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\nstart_date\0\0\0\0\0\x06\0\0\0\0\0\0\0\x08end_date\0\0\x03\xe8\0\0\0\x06\0\0\0\0\0\0\0\x0fprescribing_vet\0\0\0\0\x13\0\0\0\x01\0\0\0\x06"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_ACTIVE_MEDICATIONS: [u8; 88usize] = PetChainContract::spec_xdr_get_active_medications();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_active_medications() -> [u8; 88usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x16get_active_medications\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\nMedication\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_MARK_MEDICATION_COMPLETED: [u8; 76usize] = PetChainContract::spec_xdr_mark_medication_completed();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_mark_medication_completed() -> [u8; 76usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x19mark_medication_completed\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\rmedication_id\0\0\0\0\0\0\x06\0\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_ADD_TREATMENT: [u8; 208usize] = PetChainContract::spec_xdr_add_treatment();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_add_treatment() -> [u8; 208usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\radd_treatment\0\0\0\0\0\0\x07\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0bvet_address\0\0\0\0\x13\0\0\0\0\0\0\0\x0etreatment_type\0\0\0\0\x07\xd0\0\0\0\rTreatmentType\0\0\0\0\0\0\0\0\0\0\x04date\0\0\0\x06\0\0\0\0\0\0\0\x05notes\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x04cost\0\0\x03\xe8\0\0\0\x0b\0\0\0\0\0\0\0\x07outcome\0\0\0\0\x10\0\0\0\x01\0\0\0\x06"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_TREATMENT: [u8; 84usize] = PetChainContract::spec_xdr_get_treatment();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_treatment() -> [u8; 84usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\rget_treatment\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x0ctreatment_id\0\0\0\x06\0\0\0\x01\0\0\x03\xe8\0\0\x07\xd0\0\0\0\tTreatment\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_TREATMENT_HISTORY: [u8; 88usize] = PetChainContract::spec_xdr_get_treatment_history();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_treatment_history() -> [u8; 88usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x15get_treatment_history\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\tTreatment\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_TREATMENTS_BY_TYPE: [u8; 136usize] = PetChainContract::spec_xdr_get_treatments_by_type();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_treatments_by_type() -> [u8; 136usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x16get_treatments_by_type\0\0\0\0\0\x02\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0etreatment_type\0\0\0\0\x07\xd0\0\0\0\rTreatmentType\0\0\0\0\0\0\x01\0\0\x03\xea\0\0\x07\xd0\0\0\0\tTreatment\0\0\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_ADD_INSURANCE_POLICY: [u8; 208usize] = PetChainContract::spec_xdr_add_insurance_policy();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_add_insurance_policy() -> [u8; 208usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x14add_insurance_policy\0\0\0\x07\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\tpolicy_id\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x08provider\0\0\0\x10\0\0\0\0\0\0\0\rcoverage_type\0\0\0\0\0\0\x10\0\0\0\0\0\0\0\x07premium\0\0\0\0\x06\0\0\0\0\0\0\0\x0ecoverage_limit\0\0\0\0\0\x06\0\0\0\0\0\0\0\x0bexpiry_date\0\0\0\0\x06\0\0\0\x01\0\0\0\x01"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_GET_PET_INSURANCE: [u8; 88usize] = PetChainContract::spec_xdr_get_pet_insurance();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_get_pet_insurance() -> [u8; 88usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x11get_pet_insurance\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\x01\0\0\x03\xe8\0\0\x07\xd0\0\0\0\x0fInsurancePolicy\0"
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub static __SPEC_XDR_FN_UPDATE_INSURANCE_STATUS: [u8; 88usize] = PetChainContract::spec_xdr_update_insurance_status();
impl PetChainContract {
    #[allow(non_snake_case)]
    pub const fn spec_xdr_update_insurance_status() -> [u8; 88usize] {
        *b"\0\0\0\0\0\0\0\0\0\0\0\x17update_insurance_status\0\0\0\0\x02\0\0\0\0\0\0\0\x06pet_id\0\0\0\0\0\x06\0\0\0\0\0\0\0\x06active\0\0\0\0\0\x01\0\0\0\x01\0\0\0\x01"
    }
}
impl<'a> PetChainContractClient<'a> {
    pub fn init_admin(&self, admin: &Address) -> () {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "init_admin") },
                ::soroban_sdk::Vec::from_array(&self.env, [admin.into_val(&self.env)]),
            );
        res
    }
    pub fn try_init_admin(
        &self,
        admin: &Address,
    ) -> Result<
        Result<
            (),
            <() as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "init_admin") },
                ::soroban_sdk::Vec::from_array(&self.env, [admin.into_val(&self.env)]),
            );
        res
    }
    pub fn init_multisig(
        &self,
        invoker: &Address,
        admins: &Vec<Address>,
        threshold: &u32,
    ) -> () {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "init_multisig") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        invoker.into_val(&self.env),
                        admins.into_val(&self.env),
                        threshold.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_init_multisig(
        &self,
        invoker: &Address,
        admins: &Vec<Address>,
        threshold: &u32,
    ) -> Result<
        Result<
            (),
            <() as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "init_multisig") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        invoker.into_val(&self.env),
                        admins.into_val(&self.env),
                        threshold.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    #[allow(clippy::too_many_arguments)]
    pub fn register_pet(
        &self,
        owner: &Address,
        name: &String,
        birthday: &String,
        gender: &Gender,
        species: &Species,
        breed: &String,
        color: &String,
        weight: &u32,
        microchip_id: &Option<String>,
        privacy_level: &PrivacyLevel,
    ) -> u64 {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "register_pet") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        owner.into_val(&self.env),
                        name.into_val(&self.env),
                        birthday.into_val(&self.env),
                        gender.into_val(&self.env),
                        species.into_val(&self.env),
                        breed.into_val(&self.env),
                        color.into_val(&self.env),
                        weight.into_val(&self.env),
                        microchip_id.into_val(&self.env),
                        privacy_level.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    #[allow(clippy::too_many_arguments)]
    pub fn try_register_pet(
        &self,
        owner: &Address,
        name: &String,
        birthday: &String,
        gender: &Gender,
        species: &Species,
        breed: &String,
        color: &String,
        weight: &u32,
        microchip_id: &Option<String>,
        privacy_level: &PrivacyLevel,
    ) -> Result<
        Result<
            u64,
            <u64 as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "register_pet") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        owner.into_val(&self.env),
                        name.into_val(&self.env),
                        birthday.into_val(&self.env),
                        gender.into_val(&self.env),
                        species.into_val(&self.env),
                        breed.into_val(&self.env),
                        color.into_val(&self.env),
                        weight.into_val(&self.env),
                        microchip_id.into_val(&self.env),
                        privacy_level.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    #[allow(clippy::too_many_arguments)]
    pub fn update_pet_profile(
        &self,
        id: &u64,
        name: &String,
        birthday: &String,
        gender: &Gender,
        species: &Species,
        breed: &String,
        color: &String,
        weight: &u32,
        microchip_id: &Option<String>,
        privacy_level: &PrivacyLevel,
    ) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "update_pet_profile") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        id.into_val(&self.env),
                        name.into_val(&self.env),
                        birthday.into_val(&self.env),
                        gender.into_val(&self.env),
                        species.into_val(&self.env),
                        breed.into_val(&self.env),
                        color.into_val(&self.env),
                        weight.into_val(&self.env),
                        microchip_id.into_val(&self.env),
                        privacy_level.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    #[allow(clippy::too_many_arguments)]
    pub fn try_update_pet_profile(
        &self,
        id: &u64,
        name: &String,
        birthday: &String,
        gender: &Gender,
        species: &Species,
        breed: &String,
        color: &String,
        weight: &u32,
        microchip_id: &Option<String>,
        privacy_level: &PrivacyLevel,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "update_pet_profile") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        id.into_val(&self.env),
                        name.into_val(&self.env),
                        birthday.into_val(&self.env),
                        gender.into_val(&self.env),
                        species.into_val(&self.env),
                        breed.into_val(&self.env),
                        color.into_val(&self.env),
                        weight.into_val(&self.env),
                        microchip_id.into_val(&self.env),
                        privacy_level.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn get_pet(&self, id: &u64) -> Option<PetProfile> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{
                    #[allow(deprecated)]
                    const SYMBOL: soroban_sdk::Symbol = soroban_sdk::Symbol::short(
                        "get_pet",
                    );
                    SYMBOL
                },
                ::soroban_sdk::Vec::from_array(&self.env, [id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_pet(
        &self,
        id: &u64,
    ) -> Result<
        Result<
            Option<PetProfile>,
            <Option<
                PetProfile,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{
                    #[allow(deprecated)]
                    const SYMBOL: soroban_sdk::Symbol = soroban_sdk::Symbol::short(
                        "get_pet",
                    );
                    SYMBOL
                },
                ::soroban_sdk::Vec::from_array(&self.env, [id.into_val(&self.env)]),
            );
        res
    }
    pub fn is_pet_active(&self, id: &u64) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "is_pet_active") },
                ::soroban_sdk::Vec::from_array(&self.env, [id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_is_pet_active(
        &self,
        id: &u64,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "is_pet_active") },
                ::soroban_sdk::Vec::from_array(&self.env, [id.into_val(&self.env)]),
            );
        res
    }
    pub fn get_pet_owner(&self, id: &u64) -> Option<Address> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pet_owner") },
                ::soroban_sdk::Vec::from_array(&self.env, [id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_pet_owner(
        &self,
        id: &u64,
    ) -> Result<
        Result<
            Option<Address>,
            <Option<
                Address,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pet_owner") },
                ::soroban_sdk::Vec::from_array(&self.env, [id.into_val(&self.env)]),
            );
        res
    }
    pub fn activate_pet(&self, id: &u64) -> () {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "activate_pet") },
                ::soroban_sdk::Vec::from_array(&self.env, [id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_activate_pet(
        &self,
        id: &u64,
    ) -> Result<
        Result<
            (),
            <() as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "activate_pet") },
                ::soroban_sdk::Vec::from_array(&self.env, [id.into_val(&self.env)]),
            );
        res
    }
    pub fn deactivate_pet(&self, id: &u64) -> () {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "deactivate_pet") },
                ::soroban_sdk::Vec::from_array(&self.env, [id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_deactivate_pet(
        &self,
        id: &u64,
    ) -> Result<
        Result<
            (),
            <() as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "deactivate_pet") },
                ::soroban_sdk::Vec::from_array(&self.env, [id.into_val(&self.env)]),
            );
        res
    }
    pub fn add_pet_photo(&self, pet_id: &u64, photo_hash: &String) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_pet_photo") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), photo_hash.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_add_pet_photo(
        &self,
        pet_id: &u64,
        photo_hash: &String,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_pet_photo") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), photo_hash.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn get_pet_photos(&self, pet_id: &u64) -> Vec<String> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pet_photos") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_pet_photos(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Vec<String>,
            <Vec<
                String,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pet_photos") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn transfer_pet_ownership(&self, id: &u64, to: &Address) -> () {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "transfer_pet_ownership") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [id.into_val(&self.env), to.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_transfer_pet_ownership(
        &self,
        id: &u64,
        to: &Address,
    ) -> Result<
        Result<
            (),
            <() as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "transfer_pet_ownership") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [id.into_val(&self.env), to.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn accept_pet_transfer(&self, id: &u64) -> () {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "accept_pet_transfer") },
                ::soroban_sdk::Vec::from_array(&self.env, [id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_accept_pet_transfer(
        &self,
        id: &u64,
    ) -> Result<
        Result<
            (),
            <() as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "accept_pet_transfer") },
                ::soroban_sdk::Vec::from_array(&self.env, [id.into_val(&self.env)]),
            );
        res
    }
    pub fn register_pet_owner(
        &self,
        owner: &Address,
        name: &String,
        email: &String,
        emergency_contact: &String,
    ) -> () {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "register_pet_owner") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        owner.into_val(&self.env),
                        name.into_val(&self.env),
                        email.into_val(&self.env),
                        emergency_contact.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_register_pet_owner(
        &self,
        owner: &Address,
        name: &String,
        email: &String,
        emergency_contact: &String,
    ) -> Result<
        Result<
            (),
            <() as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "register_pet_owner") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        owner.into_val(&self.env),
                        name.into_val(&self.env),
                        email.into_val(&self.env),
                        emergency_contact.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn is_owner_registered(&self, owner: &Address) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "is_owner_registered") },
                ::soroban_sdk::Vec::from_array(&self.env, [owner.into_val(&self.env)]),
            );
        res
    }
    pub fn try_is_owner_registered(
        &self,
        owner: &Address,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "is_owner_registered") },
                ::soroban_sdk::Vec::from_array(&self.env, [owner.into_val(&self.env)]),
            );
        res
    }
    pub fn update_owner_profile(
        &self,
        owner: &Address,
        name: &String,
        email: &String,
        emergency_contact: &String,
    ) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "update_owner_profile") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        owner.into_val(&self.env),
                        name.into_val(&self.env),
                        email.into_val(&self.env),
                        emergency_contact.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_update_owner_profile(
        &self,
        owner: &Address,
        name: &String,
        email: &String,
        emergency_contact: &String,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "update_owner_profile") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        owner.into_val(&self.env),
                        name.into_val(&self.env),
                        email.into_val(&self.env),
                        emergency_contact.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn register_vet(
        &self,
        vet_address: &Address,
        name: &String,
        license_number: &String,
        specialization: &String,
    ) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "register_vet") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        vet_address.into_val(&self.env),
                        name.into_val(&self.env),
                        license_number.into_val(&self.env),
                        specialization.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_register_vet(
        &self,
        vet_address: &Address,
        name: &String,
        license_number: &String,
        specialization: &String,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "register_vet") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        vet_address.into_val(&self.env),
                        name.into_val(&self.env),
                        license_number.into_val(&self.env),
                        specialization.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn verify_vet(&self, admin: &Address, vet_address: &Address) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "verify_vet") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [admin.into_val(&self.env), vet_address.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_verify_vet(
        &self,
        admin: &Address,
        vet_address: &Address,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "verify_vet") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [admin.into_val(&self.env), vet_address.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn revoke_vet_license(&self, admin: &Address, vet_address: &Address) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "revoke_vet_license") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [admin.into_val(&self.env), vet_address.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_revoke_vet_license(
        &self,
        admin: &Address,
        vet_address: &Address,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "revoke_vet_license") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [admin.into_val(&self.env), vet_address.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn is_verified_vet(&self, vet_address: &Address) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "is_verified_vet") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [vet_address.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_is_verified_vet(
        &self,
        vet_address: &Address,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "is_verified_vet") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [vet_address.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn get_vet(&self, vet_address: &Address) -> Option<Vet> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{
                    #[allow(deprecated)]
                    const SYMBOL: soroban_sdk::Symbol = soroban_sdk::Symbol::short(
                        "get_vet",
                    );
                    SYMBOL
                },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [vet_address.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_get_vet(
        &self,
        vet_address: &Address,
    ) -> Result<
        Result<
            Option<Vet>,
            <Option<
                Vet,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{
                    #[allow(deprecated)]
                    const SYMBOL: soroban_sdk::Symbol = soroban_sdk::Symbol::short(
                        "get_vet",
                    );
                    SYMBOL
                },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [vet_address.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn get_vet_by_license(&self, license_number: &String) -> Option<Vet> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_vet_by_license") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [license_number.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_get_vet_by_license(
        &self,
        license_number: &String,
    ) -> Result<
        Result<
            Option<Vet>,
            <Option<
                Vet,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_vet_by_license") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [license_number.into_val(&self.env)],
                ),
            );
        res
    }
    /// Update clinic info for a vet. Only the vet can update their own clinic info.
    pub fn update_clinic_info(
        &self,
        vet_address: &Address,
        clinic_info: &ClinicInfo,
    ) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "update_clinic_info") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [vet_address.into_val(&self.env), clinic_info.into_val(&self.env)],
                ),
            );
        res
    }
    /// Update clinic info for a vet. Only the vet can update their own clinic info.
    pub fn try_update_clinic_info(
        &self,
        vet_address: &Address,
        clinic_info: &ClinicInfo,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "update_clinic_info") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [vet_address.into_val(&self.env), clinic_info.into_val(&self.env)],
                ),
            );
        res
    }
    #[allow(clippy::too_many_arguments)]
    pub fn add_vaccination(
        &self,
        pet_id: &u64,
        veterinarian: &Address,
        vaccine_type: &VaccineType,
        vaccine_name: &String,
        administered_at: &u64,
        next_due_date: &u64,
        batch_number: &String,
    ) -> u64 {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_vaccination") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        veterinarian.into_val(&self.env),
                        vaccine_type.into_val(&self.env),
                        vaccine_name.into_val(&self.env),
                        administered_at.into_val(&self.env),
                        next_due_date.into_val(&self.env),
                        batch_number.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    #[allow(clippy::too_many_arguments)]
    pub fn try_add_vaccination(
        &self,
        pet_id: &u64,
        veterinarian: &Address,
        vaccine_type: &VaccineType,
        vaccine_name: &String,
        administered_at: &u64,
        next_due_date: &u64,
        batch_number: &String,
    ) -> Result<
        Result<
            u64,
            <u64 as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_vaccination") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        veterinarian.into_val(&self.env),
                        vaccine_type.into_val(&self.env),
                        vaccine_name.into_val(&self.env),
                        administered_at.into_val(&self.env),
                        next_due_date.into_val(&self.env),
                        batch_number.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn get_vaccinations(&self, vaccine_id: &u64) -> Option<Vaccination> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_vaccinations") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [vaccine_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_get_vaccinations(
        &self,
        vaccine_id: &u64,
    ) -> Result<
        Result<
            Option<Vaccination>,
            <Option<
                Vaccination,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_vaccinations") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [vaccine_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn get_vaccination_history(&self, pet_id: &u64) -> Vec<Vaccination> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_vaccination_history") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_vaccination_history(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Vec<Vaccination>,
            <Vec<
                Vaccination,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_vaccination_history") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn get_upcoming_vaccinations(
        &self,
        pet_id: &u64,
        days_threshold: &u64,
    ) -> Vec<Vaccination> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_upcoming_vaccinations") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), days_threshold.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_get_upcoming_vaccinations(
        &self,
        pet_id: &u64,
        days_threshold: &u64,
    ) -> Result<
        Result<
            Vec<Vaccination>,
            <Vec<
                Vaccination,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_upcoming_vaccinations") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), days_threshold.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn is_vaccination_current(
        &self,
        pet_id: &u64,
        vaccine_type: &VaccineType,
    ) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "is_vaccination_current") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), vaccine_type.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_is_vaccination_current(
        &self,
        pet_id: &u64,
        vaccine_type: &VaccineType,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "is_vaccination_current") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), vaccine_type.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn get_overdue_vaccinations(&self, pet_id: &u64) -> Vec<VaccineType> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_overdue_vaccinations") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_overdue_vaccinations(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Vec<VaccineType>,
            <Vec<
                VaccineType,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_overdue_vaccinations") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn link_tag_to_pet(&self, pet_id: &u64) -> BytesN<32> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "link_tag_to_pet") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_link_tag_to_pet(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            BytesN<32>,
            <BytesN<
                32,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "link_tag_to_pet") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn get_pet_by_tag(&self, tag_id: &BytesN<32>) -> Option<PetProfile> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pet_by_tag") },
                ::soroban_sdk::Vec::from_array(&self.env, [tag_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_pet_by_tag(
        &self,
        tag_id: &BytesN<32>,
    ) -> Result<
        Result<
            Option<PetProfile>,
            <Option<
                PetProfile,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pet_by_tag") },
                ::soroban_sdk::Vec::from_array(&self.env, [tag_id.into_val(&self.env)]),
            );
        res
    }
    pub fn get_tag(&self, tag_id: &BytesN<32>) -> Option<PetTag> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{
                    #[allow(deprecated)]
                    const SYMBOL: soroban_sdk::Symbol = soroban_sdk::Symbol::short(
                        "get_tag",
                    );
                    SYMBOL
                },
                ::soroban_sdk::Vec::from_array(&self.env, [tag_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_tag(
        &self,
        tag_id: &BytesN<32>,
    ) -> Result<
        Result<
            Option<PetTag>,
            <Option<
                PetTag,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{
                    #[allow(deprecated)]
                    const SYMBOL: soroban_sdk::Symbol = soroban_sdk::Symbol::short(
                        "get_tag",
                    );
                    SYMBOL
                },
                ::soroban_sdk::Vec::from_array(&self.env, [tag_id.into_val(&self.env)]),
            );
        res
    }
    pub fn get_tag_by_pet(&self, pet_id: &u64) -> Option<BytesN<32>> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_tag_by_pet") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_tag_by_pet(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Option<BytesN<32>>,
            <Option<
                BytesN<32>,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_tag_by_pet") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn update_tag_message(&self, tag_id: &BytesN<32>, message: &String) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "update_tag_message") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [tag_id.into_val(&self.env), message.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_update_tag_message(
        &self,
        tag_id: &BytesN<32>,
        message: &String,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "update_tag_message") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [tag_id.into_val(&self.env), message.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn deactivate_tag(&self, tag_id: &BytesN<32>) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "deactivate_tag") },
                ::soroban_sdk::Vec::from_array(&self.env, [tag_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_deactivate_tag(
        &self,
        tag_id: &BytesN<32>,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "deactivate_tag") },
                ::soroban_sdk::Vec::from_array(&self.env, [tag_id.into_val(&self.env)]),
            );
        res
    }
    pub fn reactivate_tag(&self, tag_id: &BytesN<32>) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "reactivate_tag") },
                ::soroban_sdk::Vec::from_array(&self.env, [tag_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_reactivate_tag(
        &self,
        tag_id: &BytesN<32>,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "reactivate_tag") },
                ::soroban_sdk::Vec::from_array(&self.env, [tag_id.into_val(&self.env)]),
            );
        res
    }
    pub fn is_tag_active(&self, tag_id: &BytesN<32>) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "is_tag_active") },
                ::soroban_sdk::Vec::from_array(&self.env, [tag_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_is_tag_active(
        &self,
        tag_id: &BytesN<32>,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "is_tag_active") },
                ::soroban_sdk::Vec::from_array(&self.env, [tag_id.into_val(&self.env)]),
            );
        res
    }
    pub fn get_ownership_history(&self, pet_id: &u64) -> Vec<OwnershipRecord> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_ownership_history") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_ownership_history(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Vec<OwnershipRecord>,
            <Vec<
                OwnershipRecord,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_ownership_history") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn set_emergency_contacts(
        &self,
        pet_id: &u64,
        contacts: &Vec<EmergencyContact>,
        medical_notes: &String,
    ) -> () {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "set_emergency_contacts") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        contacts.into_val(&self.env),
                        medical_notes.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_set_emergency_contacts(
        &self,
        pet_id: &u64,
        contacts: &Vec<EmergencyContact>,
        medical_notes: &String,
    ) -> Result<
        Result<
            (),
            <() as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "set_emergency_contacts") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        contacts.into_val(&self.env),
                        medical_notes.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn get_emergency_info(
        &self,
        pet_id: &u64,
    ) -> Option<(Vec<EmergencyContact>, String)> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_emergency_info") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_emergency_info(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Option<(Vec<EmergencyContact>, String)>,
            <Option<
                (Vec<EmergencyContact>, String),
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_emergency_info") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    /// Get emergency contacts for a pet (publicly accessible - no auth required for emergency responders)
    pub fn get_emergency_contacts(&self, pet_id: &u64) -> Vec<EmergencyContact> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_emergency_contacts") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    /// Get emergency contacts for a pet (publicly accessible - no auth required for emergency responders)
    pub fn try_get_emergency_contacts(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Vec<EmergencyContact>,
            <Vec<
                EmergencyContact,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_emergency_contacts") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn get_accessible_pets(&self, user: &Address) -> Vec<u64> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_accessible_pets") },
                ::soroban_sdk::Vec::from_array(&self.env, [user.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_accessible_pets(
        &self,
        user: &Address,
    ) -> Result<
        Result<
            Vec<u64>,
            <Vec<
                u64,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_accessible_pets") },
                ::soroban_sdk::Vec::from_array(&self.env, [user.into_val(&self.env)]),
            );
        res
    }
    pub fn get_all_pets_by_owner(&self, owner: &Address) -> Vec<PetProfile> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_all_pets_by_owner") },
                ::soroban_sdk::Vec::from_array(&self.env, [owner.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_all_pets_by_owner(
        &self,
        owner: &Address,
    ) -> Result<
        Result<
            Vec<PetProfile>,
            <Vec<
                PetProfile,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_all_pets_by_owner") },
                ::soroban_sdk::Vec::from_array(&self.env, [owner.into_val(&self.env)]),
            );
        res
    }
    pub fn get_pets_by_owner(&self, owner: &Address) -> Vec<PetProfile> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pets_by_owner") },
                ::soroban_sdk::Vec::from_array(&self.env, [owner.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_pets_by_owner(
        &self,
        owner: &Address,
    ) -> Result<
        Result<
            Vec<PetProfile>,
            <Vec<
                PetProfile,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pets_by_owner") },
                ::soroban_sdk::Vec::from_array(&self.env, [owner.into_val(&self.env)]),
            );
        res
    }
    pub fn get_pets_by_species(&self, species: &String) -> Vec<PetProfile> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pets_by_species") },
                ::soroban_sdk::Vec::from_array(&self.env, [species.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_pets_by_species(
        &self,
        species: &String,
    ) -> Result<
        Result<
            Vec<PetProfile>,
            <Vec<
                PetProfile,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pets_by_species") },
                ::soroban_sdk::Vec::from_array(&self.env, [species.into_val(&self.env)]),
            );
        res
    }
    pub fn get_active_pets(&self) -> Vec<PetProfile> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_active_pets") },
                ::soroban_sdk::Vec::new(&self.env),
            );
        res
    }
    pub fn try_get_active_pets(
        &self,
    ) -> Result<
        Result<
            Vec<PetProfile>,
            <Vec<
                PetProfile,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_active_pets") },
                ::soroban_sdk::Vec::new(&self.env),
            );
        res
    }
    pub fn grant_access(
        &self,
        pet_id: &u64,
        grantee: &Address,
        access_level: &AccessLevel,
        expires_at: &Option<u64>,
    ) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "grant_access") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        grantee.into_val(&self.env),
                        access_level.into_val(&self.env),
                        expires_at.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_grant_access(
        &self,
        pet_id: &u64,
        grantee: &Address,
        access_level: &AccessLevel,
        expires_at: &Option<u64>,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "grant_access") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        grantee.into_val(&self.env),
                        access_level.into_val(&self.env),
                        expires_at.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn revoke_access(&self, pet_id: &u64, grantee: &Address) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "revoke_access") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), grantee.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_revoke_access(
        &self,
        pet_id: &u64,
        grantee: &Address,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "revoke_access") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), grantee.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn grant_temporary_custody(
        &self,
        pet_id: &u64,
        custodian: &Address,
        start_date: &u64,
        end_date: &u64,
        permissions: &Vec<String>,
    ) -> TemporaryCustody {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "grant_temporary_custody") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        custodian.into_val(&self.env),
                        start_date.into_val(&self.env),
                        end_date.into_val(&self.env),
                        permissions.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_grant_temporary_custody(
        &self,
        pet_id: &u64,
        custodian: &Address,
        start_date: &u64,
        end_date: &u64,
        permissions: &Vec<String>,
    ) -> Result<
        Result<
            TemporaryCustody,
            <TemporaryCustody as soroban_sdk::TryFromVal<
                soroban_sdk::Env,
                soroban_sdk::Val,
            >>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "grant_temporary_custody") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        custodian.into_val(&self.env),
                        start_date.into_val(&self.env),
                        end_date.into_val(&self.env),
                        permissions.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn revoke_temporary_custody(&self, pet_id: &u64) -> () {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "revoke_temporary_custody") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_revoke_temporary_custody(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            (),
            <() as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "revoke_temporary_custody") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn is_custody_valid(&self, pet_id: &u64) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "is_custody_valid") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_is_custody_valid(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "is_custody_valid") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn add_medical_record(
        &self,
        pet_id: &u64,
        vet_address: &Address,
        diagnosis: &String,
        treatment: &String,
        medications: &String,
        notes: &String,
    ) -> u64 {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_medical_record") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        vet_address.into_val(&self.env),
                        diagnosis.into_val(&self.env),
                        treatment.into_val(&self.env),
                        medications.into_val(&self.env),
                        notes.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_add_medical_record(
        &self,
        pet_id: &u64,
        vet_address: &Address,
        diagnosis: &String,
        treatment: &String,
        medications: &String,
        notes: &String,
    ) -> Result<
        Result<
            u64,
            <u64 as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_medical_record") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        vet_address.into_val(&self.env),
                        diagnosis.into_val(&self.env),
                        treatment.into_val(&self.env),
                        medications.into_val(&self.env),
                        notes.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn update_medical_record(
        &self,
        record_id: &u64,
        diagnosis: &String,
        treatment: &String,
        medications: &String,
        notes: &String,
    ) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "update_medical_record") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        record_id.into_val(&self.env),
                        diagnosis.into_val(&self.env),
                        treatment.into_val(&self.env),
                        medications.into_val(&self.env),
                        notes.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_update_medical_record(
        &self,
        record_id: &u64,
        diagnosis: &String,
        treatment: &String,
        medications: &String,
        notes: &String,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "update_medical_record") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        record_id.into_val(&self.env),
                        diagnosis.into_val(&self.env),
                        treatment.into_val(&self.env),
                        medications.into_val(&self.env),
                        notes.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn get_medical_record(&self, record_id: &u64) -> Option<MedicalRecord> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_medical_record") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [record_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_get_medical_record(
        &self,
        record_id: &u64,
    ) -> Result<
        Result<
            Option<MedicalRecord>,
            <Option<
                MedicalRecord,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_medical_record") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [record_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn get_pet_medical_records(&self, pet_id: &u64) -> Vec<MedicalRecord> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pet_medical_records") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_pet_medical_records(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Vec<MedicalRecord>,
            <Vec<
                MedicalRecord,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pet_medical_records") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn get_access_logs(&self, pet_id: &u64) -> Vec<AccessLog> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_access_logs") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_access_logs(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Vec<AccessLog>,
            <Vec<
                AccessLog,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_access_logs") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn check_access(&self, pet_id: &u64, user: &Address) -> AccessLevel {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "check_access") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), user.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_check_access(
        &self,
        pet_id: &u64,
        user: &Address,
    ) -> Result<
        Result<
            AccessLevel,
            <AccessLevel as soroban_sdk::TryFromVal<
                soroban_sdk::Env,
                soroban_sdk::Val,
            >>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "check_access") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), user.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn get_authorized_users(&self, pet_id: &u64) -> Vec<Address> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_authorized_users") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_authorized_users(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Vec<Address>,
            <Vec<
                Address,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_authorized_users") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn get_access_grant(
        &self,
        pet_id: &u64,
        grantee: &Address,
    ) -> Option<AccessGrant> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_access_grant") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), grantee.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_get_access_grant(
        &self,
        pet_id: &u64,
        grantee: &Address,
    ) -> Result<
        Result<
            Option<AccessGrant>,
            <Option<
                AccessGrant,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_access_grant") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), grantee.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn add_lab_result(
        &self,
        pet_id: &u64,
        vet_address: &Address,
        test_type: &String,
        results: &String,
        reference_ranges: &String,
        attachment_hash: &Option<String>,
        medical_record_id: &Option<u64>,
    ) -> u64 {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_lab_result") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        vet_address.into_val(&self.env),
                        test_type.into_val(&self.env),
                        results.into_val(&self.env),
                        reference_ranges.into_val(&self.env),
                        attachment_hash.into_val(&self.env),
                        medical_record_id.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_add_lab_result(
        &self,
        pet_id: &u64,
        vet_address: &Address,
        test_type: &String,
        results: &String,
        reference_ranges: &String,
        attachment_hash: &Option<String>,
        medical_record_id: &Option<u64>,
    ) -> Result<
        Result<
            u64,
            <u64 as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_lab_result") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        vet_address.into_val(&self.env),
                        test_type.into_val(&self.env),
                        results.into_val(&self.env),
                        reference_ranges.into_val(&self.env),
                        attachment_hash.into_val(&self.env),
                        medical_record_id.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn get_lab_result(&self, lab_result_id: &u64) -> Option<LabResult> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_lab_result") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [lab_result_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_get_lab_result(
        &self,
        lab_result_id: &u64,
    ) -> Result<
        Result<
            Option<LabResult>,
            <Option<
                LabResult,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_lab_result") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [lab_result_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn get_lab_results(&self, pet_id: &u64) -> Vec<LabResult> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_lab_results") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_lab_results(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Vec<LabResult>,
            <Vec<
                LabResult,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_lab_results") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    #[allow(clippy::too_many_arguments)]
    pub fn add_medication_to_record(
        &self,
        record_id: &u64,
        name: &String,
        dosage: &String,
        frequency: &String,
        start_date: &u64,
        end_date: &u64,
        prescribing_vet: &Address,
    ) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_medication_to_record") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        record_id.into_val(&self.env),
                        name.into_val(&self.env),
                        dosage.into_val(&self.env),
                        frequency.into_val(&self.env),
                        start_date.into_val(&self.env),
                        end_date.into_val(&self.env),
                        prescribing_vet.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    #[allow(clippy::too_many_arguments)]
    pub fn try_add_medication_to_record(
        &self,
        record_id: &u64,
        name: &String,
        dosage: &String,
        frequency: &String,
        start_date: &u64,
        end_date: &u64,
        prescribing_vet: &Address,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_medication_to_record") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        record_id.into_val(&self.env),
                        name.into_val(&self.env),
                        dosage.into_val(&self.env),
                        frequency.into_val(&self.env),
                        start_date.into_val(&self.env),
                        end_date.into_val(&self.env),
                        prescribing_vet.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn mark_record_med_completed(&self, record_id: &u64, med_index: &u32) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "mark_record_med_completed") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [record_id.into_val(&self.env), med_index.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_mark_record_med_completed(
        &self,
        record_id: &u64,
        med_index: &u32,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "mark_record_med_completed") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [record_id.into_val(&self.env), med_index.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn get_active_record_meds(&self, pet_id: &u64) -> Vec<Medication> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_active_record_meds") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_active_record_meds(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Vec<Medication>,
            <Vec<
                Medication,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_active_record_meds") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn get_record_med_history(&self, pet_id: &u64) -> Vec<Medication> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_record_med_history") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_record_med_history(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Vec<Medication>,
            <Vec<
                Medication,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_record_med_history") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn batch_add_vaccinations(
        &self,
        veterinarian: &Address,
        vaccinations: &Vec<VaccinationInput>,
    ) -> Vec<u64> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "batch_add_vaccinations") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [veterinarian.into_val(&self.env), vaccinations.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_batch_add_vaccinations(
        &self,
        veterinarian: &Address,
        vaccinations: &Vec<VaccinationInput>,
    ) -> Result<
        Result<
            Vec<u64>,
            <Vec<
                u64,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "batch_add_vaccinations") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [veterinarian.into_val(&self.env), vaccinations.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn batch_add_records(
        &self,
        veterinarian: &Address,
        records: &Vec<MedicalRecordInput>,
    ) -> Vec<u64> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "batch_add_records") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [veterinarian.into_val(&self.env), records.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_batch_add_records(
        &self,
        veterinarian: &Address,
        records: &Vec<MedicalRecordInput>,
    ) -> Result<
        Result<
            Vec<u64>,
            <Vec<
                u64,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "batch_add_records") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [veterinarian.into_val(&self.env), records.into_val(&self.env)],
                ),
            );
        res
    }
    /// Report a pet as lost
    pub fn report_lost(
        &self,
        pet_id: &u64,
        last_seen_location: &String,
        reward_amount: &Option<u64>,
    ) -> u64 {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "report_lost") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        last_seen_location.into_val(&self.env),
                        reward_amount.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    /// Report a pet as lost
    pub fn try_report_lost(
        &self,
        pet_id: &u64,
        last_seen_location: &String,
        reward_amount: &Option<u64>,
    ) -> Result<
        Result<
            u64,
            <u64 as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "report_lost") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        last_seen_location.into_val(&self.env),
                        reward_amount.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    /// Report a sighting of a lost pet
    pub fn report_sighting(
        &self,
        alert_id: &u64,
        location: &String,
        description: &String,
    ) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "report_sighting") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        alert_id.into_val(&self.env),
                        location.into_val(&self.env),
                        description.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    /// Report a sighting of a lost pet
    pub fn try_report_sighting(
        &self,
        alert_id: &u64,
        location: &String,
        description: &String,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "report_sighting") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        alert_id.into_val(&self.env),
                        location.into_val(&self.env),
                        description.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    /// Mark a lost pet as found
    pub fn report_found(&self, alert_id: &u64) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "report_found") },
                ::soroban_sdk::Vec::from_array(&self.env, [alert_id.into_val(&self.env)]),
            );
        res
    }
    /// Mark a lost pet as found
    pub fn try_report_found(
        &self,
        alert_id: &u64,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "report_found") },
                ::soroban_sdk::Vec::from_array(&self.env, [alert_id.into_val(&self.env)]),
            );
        res
    }
    /// Cancel a lost pet alert
    pub fn cancel_lost_alert(&self, alert_id: &u64) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "cancel_lost_alert") },
                ::soroban_sdk::Vec::from_array(&self.env, [alert_id.into_val(&self.env)]),
            );
        res
    }
    /// Cancel a lost pet alert
    pub fn try_cancel_lost_alert(
        &self,
        alert_id: &u64,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "cancel_lost_alert") },
                ::soroban_sdk::Vec::from_array(&self.env, [alert_id.into_val(&self.env)]),
            );
        res
    }
    /// Get all active lost pet alerts
    pub fn get_active_alerts(&self) -> Vec<LostPetAlert> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_active_alerts") },
                ::soroban_sdk::Vec::new(&self.env),
            );
        res
    }
    /// Get all active lost pet alerts
    pub fn try_get_active_alerts(
        &self,
    ) -> Result<
        Result<
            Vec<LostPetAlert>,
            <Vec<
                LostPetAlert,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_active_alerts") },
                ::soroban_sdk::Vec::new(&self.env),
            );
        res
    }
    /// Get a specific alert by ID
    pub fn get_alert(&self, alert_id: &u64) -> Option<LostPetAlert> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{
                    #[allow(deprecated)]
                    const SYMBOL: soroban_sdk::Symbol = soroban_sdk::Symbol::short(
                        "get_alert",
                    );
                    SYMBOL
                },
                ::soroban_sdk::Vec::from_array(&self.env, [alert_id.into_val(&self.env)]),
            );
        res
    }
    /// Get a specific alert by ID
    pub fn try_get_alert(
        &self,
        alert_id: &u64,
    ) -> Result<
        Result<
            Option<LostPetAlert>,
            <Option<
                LostPetAlert,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{
                    #[allow(deprecated)]
                    const SYMBOL: soroban_sdk::Symbol = soroban_sdk::Symbol::short(
                        "get_alert",
                    );
                    SYMBOL
                },
                ::soroban_sdk::Vec::from_array(&self.env, [alert_id.into_val(&self.env)]),
            );
        res
    }
    /// Get sightings for a specific alert
    pub fn get_alert_sightings(&self, alert_id: &u64) -> Vec<SightingReport> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_alert_sightings") },
                ::soroban_sdk::Vec::from_array(&self.env, [alert_id.into_val(&self.env)]),
            );
        res
    }
    /// Get sightings for a specific alert
    pub fn try_get_alert_sightings(
        &self,
        alert_id: &u64,
    ) -> Result<
        Result<
            Vec<SightingReport>,
            <Vec<
                SightingReport,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_alert_sightings") },
                ::soroban_sdk::Vec::from_array(&self.env, [alert_id.into_val(&self.env)]),
            );
        res
    }
    /// Get alerts for a specific pet
    pub fn get_pet_alerts(&self, pet_id: &u64) -> Vec<LostPetAlert> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pet_alerts") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    /// Get alerts for a specific pet
    pub fn try_get_pet_alerts(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Vec<LostPetAlert>,
            <Vec<
                LostPetAlert,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pet_alerts") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    /// Set availability slots for a vet (only verified vets can set their availability)
    pub fn set_availability(
        &self,
        vet_address: &Address,
        start_time: &u64,
        end_time: &u64,
    ) -> u64 {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "set_availability") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        vet_address.into_val(&self.env),
                        start_time.into_val(&self.env),
                        end_time.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    /// Set availability slots for a vet (only verified vets can set their availability)
    pub fn try_set_availability(
        &self,
        vet_address: &Address,
        start_time: &u64,
        end_time: &u64,
    ) -> Result<
        Result<
            u64,
            <u64 as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "set_availability") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        vet_address.into_val(&self.env),
                        start_time.into_val(&self.env),
                        end_time.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    /// Get available slots for a vet on a specific date
    pub fn get_available_slots(
        &self,
        vet_address: &Address,
        date: &u64,
    ) -> Vec<AvailabilitySlot> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_available_slots") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [vet_address.into_val(&self.env), date.into_val(&self.env)],
                ),
            );
        res
    }
    /// Get available slots for a vet on a specific date
    pub fn try_get_available_slots(
        &self,
        vet_address: &Address,
        date: &u64,
    ) -> Result<
        Result<
            Vec<AvailabilitySlot>,
            <Vec<
                AvailabilitySlot,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_available_slots") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [vet_address.into_val(&self.env), date.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn grant_consent(
        &self,
        pet_id: &u64,
        owner: &Address,
        consent_type: &ConsentType,
        granted_to: &Address,
    ) -> u64 {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "grant_consent") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        owner.into_val(&self.env),
                        consent_type.into_val(&self.env),
                        granted_to.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_grant_consent(
        &self,
        pet_id: &u64,
        owner: &Address,
        consent_type: &ConsentType,
        granted_to: &Address,
    ) -> Result<
        Result<
            u64,
            <u64 as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "grant_consent") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        owner.into_val(&self.env),
                        consent_type.into_val(&self.env),
                        granted_to.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn revoke_consent(&self, consent_id: &u64, owner: &Address) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "revoke_consent") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [consent_id.into_val(&self.env), owner.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_revoke_consent(
        &self,
        consent_id: &u64,
        owner: &Address,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "revoke_consent") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [consent_id.into_val(&self.env), owner.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn get_consent_history(&self, pet_id: &u64) -> Vec<Consent> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_consent_history") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_consent_history(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Vec<Consent>,
            <Vec<
                Consent,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_consent_history") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    /// Book a slot (mark as unavailable)
    pub fn book_slot(&self, vet_address: &Address, slot_index: &u64) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{
                    #[allow(deprecated)]
                    const SYMBOL: soroban_sdk::Symbol = soroban_sdk::Symbol::short(
                        "book_slot",
                    );
                    SYMBOL
                },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [vet_address.into_val(&self.env), slot_index.into_val(&self.env)],
                ),
            );
        res
    }
    /// Book a slot (mark as unavailable)
    pub fn try_book_slot(
        &self,
        vet_address: &Address,
        slot_index: &u64,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{
                    #[allow(deprecated)]
                    const SYMBOL: soroban_sdk::Symbol = soroban_sdk::Symbol::short(
                        "book_slot",
                    );
                    SYMBOL
                },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [vet_address.into_val(&self.env), slot_index.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn get_version(&self) -> ContractVersion {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_version") },
                ::soroban_sdk::Vec::new(&self.env),
            );
        res
    }
    pub fn try_get_version(
        &self,
    ) -> Result<
        Result<
            ContractVersion,
            <ContractVersion as soroban_sdk::TryFromVal<
                soroban_sdk::Env,
                soroban_sdk::Val,
            >>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_version") },
                ::soroban_sdk::Vec::new(&self.env),
            );
        res
    }
    pub fn upgrade_contract(&self, new_wasm_hash: &BytesN<32>) -> () {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "upgrade_contract") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [new_wasm_hash.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_upgrade_contract(
        &self,
        new_wasm_hash: &BytesN<32>,
    ) -> Result<
        Result<
            (),
            <() as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "upgrade_contract") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [new_wasm_hash.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn propose_upgrade(
        &self,
        proposer: &Address,
        new_wasm_hash: &BytesN<32>,
    ) -> u64 {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "propose_upgrade") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [proposer.into_val(&self.env), new_wasm_hash.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_propose_upgrade(
        &self,
        proposer: &Address,
        new_wasm_hash: &BytesN<32>,
    ) -> Result<
        Result<
            u64,
            <u64 as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "propose_upgrade") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [proposer.into_val(&self.env), new_wasm_hash.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn approve_upgrade(&self, proposal_id: &u64) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "approve_upgrade") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [proposal_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_approve_upgrade(
        &self,
        proposal_id: &u64,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "approve_upgrade") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [proposal_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn get_upgrade_proposal(&self, proposal_id: &u64) -> Option<UpgradeProposal> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_upgrade_proposal") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [proposal_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_get_upgrade_proposal(
        &self,
        proposal_id: &u64,
    ) -> Result<
        Result<
            Option<UpgradeProposal>,
            <Option<
                UpgradeProposal,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_upgrade_proposal") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [proposal_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn migrate_version(&self, major: &u32, minor: &u32, patch: &u32) -> () {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "migrate_version") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        major.into_val(&self.env),
                        minor.into_val(&self.env),
                        patch.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_migrate_version(
        &self,
        major: &u32,
        minor: &u32,
        patch: &u32,
    ) -> Result<
        Result<
            (),
            <() as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "migrate_version") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        major.into_val(&self.env),
                        minor.into_val(&self.env),
                        patch.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn propose_action(
        &self,
        proposer: &Address,
        action: &ProposalAction,
        expires_in: &u64,
    ) -> u64 {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "propose_action") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        proposer.into_val(&self.env),
                        action.into_val(&self.env),
                        expires_in.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_propose_action(
        &self,
        proposer: &Address,
        action: &ProposalAction,
        expires_in: &u64,
    ) -> Result<
        Result<
            u64,
            <u64 as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "propose_action") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        proposer.into_val(&self.env),
                        action.into_val(&self.env),
                        expires_in.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn approve_proposal(&self, admin: &Address, proposal_id: &u64) -> () {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "approve_proposal") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [admin.into_val(&self.env), proposal_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_approve_proposal(
        &self,
        admin: &Address,
        proposal_id: &u64,
    ) -> Result<
        Result<
            (),
            <() as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "approve_proposal") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [admin.into_val(&self.env), proposal_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn execute_proposal(&self, proposal_id: &u64) -> () {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "execute_proposal") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [proposal_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_execute_proposal(
        &self,
        proposal_id: &u64,
    ) -> Result<
        Result<
            (),
            <() as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "execute_proposal") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [proposal_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn get_proposal(&self, proposal_id: &u64) -> Option<MultiSigProposal> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_proposal") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [proposal_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_get_proposal(
        &self,
        proposal_id: &u64,
    ) -> Result<
        Result<
            Option<MultiSigProposal>,
            <Option<
                MultiSigProposal,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_proposal") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [proposal_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn add_vet_review(
        &self,
        reviewer: &Address,
        vet: &Address,
        rating: &u32,
        comment: &String,
    ) -> u64 {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_vet_review") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        reviewer.into_val(&self.env),
                        vet.into_val(&self.env),
                        rating.into_val(&self.env),
                        comment.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_add_vet_review(
        &self,
        reviewer: &Address,
        vet: &Address,
        rating: &u32,
        comment: &String,
    ) -> Result<
        Result<
            u64,
            <u64 as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_vet_review") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        reviewer.into_val(&self.env),
                        vet.into_val(&self.env),
                        rating.into_val(&self.env),
                        comment.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn get_vet_reviews(&self, vet: &Address) -> Vec<VetReview> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_vet_reviews") },
                ::soroban_sdk::Vec::from_array(&self.env, [vet.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_vet_reviews(
        &self,
        vet: &Address,
    ) -> Result<
        Result<
            Vec<VetReview>,
            <Vec<
                VetReview,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_vet_reviews") },
                ::soroban_sdk::Vec::from_array(&self.env, [vet.into_val(&self.env)]),
            );
        res
    }
    pub fn get_vet_average_rating(&self, vet: &Address) -> u32 {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_vet_average_rating") },
                ::soroban_sdk::Vec::from_array(&self.env, [vet.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_vet_average_rating(
        &self,
        vet: &Address,
    ) -> Result<
        Result<
            u32,
            <u32 as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_vet_average_rating") },
                ::soroban_sdk::Vec::from_array(&self.env, [vet.into_val(&self.env)]),
            );
        res
    }
    pub fn add_medication(
        &self,
        pet_id: &u64,
        name: &String,
        dosage: &String,
        frequency: &String,
        start_date: &u64,
        end_date: &Option<u64>,
        prescribing_vet: &Address,
    ) -> u64 {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_medication") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        name.into_val(&self.env),
                        dosage.into_val(&self.env),
                        frequency.into_val(&self.env),
                        start_date.into_val(&self.env),
                        end_date.into_val(&self.env),
                        prescribing_vet.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_add_medication(
        &self,
        pet_id: &u64,
        name: &String,
        dosage: &String,
        frequency: &String,
        start_date: &u64,
        end_date: &Option<u64>,
        prescribing_vet: &Address,
    ) -> Result<
        Result<
            u64,
            <u64 as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_medication") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        name.into_val(&self.env),
                        dosage.into_val(&self.env),
                        frequency.into_val(&self.env),
                        start_date.into_val(&self.env),
                        end_date.into_val(&self.env),
                        prescribing_vet.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn get_active_medications(&self, pet_id: &u64) -> Vec<Medication> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_active_medications") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_active_medications(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Vec<Medication>,
            <Vec<
                Medication,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_active_medications") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn mark_medication_completed(&self, medication_id: &u64) -> () {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "mark_medication_completed") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [medication_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_mark_medication_completed(
        &self,
        medication_id: &u64,
    ) -> Result<
        Result<
            (),
            <() as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "mark_medication_completed") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [medication_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn add_treatment(
        &self,
        pet_id: &u64,
        vet_address: &Address,
        treatment_type: &TreatmentType,
        date: &u64,
        notes: &String,
        cost: &Option<i128>,
        outcome: &String,
    ) -> u64 {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_treatment") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        vet_address.into_val(&self.env),
                        treatment_type.into_val(&self.env),
                        date.into_val(&self.env),
                        notes.into_val(&self.env),
                        cost.into_val(&self.env),
                        outcome.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_add_treatment(
        &self,
        pet_id: &u64,
        vet_address: &Address,
        treatment_type: &TreatmentType,
        date: &u64,
        notes: &String,
        cost: &Option<i128>,
        outcome: &String,
    ) -> Result<
        Result<
            u64,
            <u64 as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_treatment") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        vet_address.into_val(&self.env),
                        treatment_type.into_val(&self.env),
                        date.into_val(&self.env),
                        notes.into_val(&self.env),
                        cost.into_val(&self.env),
                        outcome.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn get_treatment(&self, treatment_id: &u64) -> Option<Treatment> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_treatment") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [treatment_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_get_treatment(
        &self,
        treatment_id: &u64,
    ) -> Result<
        Result<
            Option<Treatment>,
            <Option<
                Treatment,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_treatment") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [treatment_id.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn get_treatment_history(&self, pet_id: &u64) -> Vec<Treatment> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_treatment_history") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_treatment_history(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Vec<Treatment>,
            <Vec<
                Treatment,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_treatment_history") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn get_treatments_by_type(
        &self,
        pet_id: &u64,
        treatment_type: &TreatmentType,
    ) -> Vec<Treatment> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_treatments_by_type") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), treatment_type.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_get_treatments_by_type(
        &self,
        pet_id: &u64,
        treatment_type: &TreatmentType,
    ) -> Result<
        Result<
            Vec<Treatment>,
            <Vec<
                Treatment,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_treatments_by_type") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), treatment_type.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn add_insurance_policy(
        &self,
        pet_id: &u64,
        policy_id: &String,
        provider: &String,
        coverage_type: &String,
        premium: &u64,
        coverage_limit: &u64,
        expiry_date: &u64,
    ) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_insurance_policy") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        policy_id.into_val(&self.env),
                        provider.into_val(&self.env),
                        coverage_type.into_val(&self.env),
                        premium.into_val(&self.env),
                        coverage_limit.into_val(&self.env),
                        expiry_date.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn try_add_insurance_policy(
        &self,
        pet_id: &u64,
        policy_id: &String,
        provider: &String,
        coverage_type: &String,
        premium: &u64,
        coverage_limit: &u64,
        expiry_date: &u64,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "add_insurance_policy") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [
                        pet_id.into_val(&self.env),
                        policy_id.into_val(&self.env),
                        provider.into_val(&self.env),
                        coverage_type.into_val(&self.env),
                        premium.into_val(&self.env),
                        coverage_limit.into_val(&self.env),
                        expiry_date.into_val(&self.env),
                    ],
                ),
            );
        res
    }
    pub fn get_pet_insurance(&self, pet_id: &u64) -> Option<InsurancePolicy> {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pet_insurance") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn try_get_pet_insurance(
        &self,
        pet_id: &u64,
    ) -> Result<
        Result<
            Option<InsurancePolicy>,
            <Option<
                InsurancePolicy,
            > as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "get_pet_insurance") },
                ::soroban_sdk::Vec::from_array(&self.env, [pet_id.into_val(&self.env)]),
            );
        res
    }
    pub fn update_insurance_status(&self, pet_id: &u64, active: &bool) -> bool {
        use core::ops::Not;
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "update_insurance_status") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), active.into_val(&self.env)],
                ),
            );
        res
    }
    pub fn try_update_insurance_status(
        &self,
        pet_id: &u64,
        active: &bool,
    ) -> Result<
        Result<
            bool,
            <bool as soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>>::Error,
        >,
        Result<soroban_sdk::Error, soroban_sdk::InvokeError>,
    > {
        use soroban_sdk::{IntoVal, FromVal};
        let res = self
            .env
            .try_invoke_contract(
                &self.address,
                &{ soroban_sdk::Symbol::new(&self.env, "update_insurance_status") },
                ::soroban_sdk::Vec::from_array(
                    &self.env,
                    [pet_id.into_val(&self.env), active.into_val(&self.env)],
                ),
            );
        res
    }
}
#[doc(hidden)]
pub mod __init_admin {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).init_admin` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::init_admin(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).init_admin` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __init_multisig {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).init_multisig` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::init_multisig(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).init_multisig` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2)
    }
    use super::*;
}
#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
pub mod __register_pet {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).register_pet` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
        arg_7: soroban_sdk::Val,
        arg_8: soroban_sdk::Val,
        arg_9: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::register_pet(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_4),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_5),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_6),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_7),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_8),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_9),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).register_pet` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
        arg_7: soroban_sdk::Val,
        arg_8: soroban_sdk::Val,
        arg_9: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)]
        invoke_raw(
            env,
            arg_0,
            arg_1,
            arg_2,
            arg_3,
            arg_4,
            arg_5,
            arg_6,
            arg_7,
            arg_8,
            arg_9,
        )
    }
    use super::*;
}
#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
pub mod __update_pet_profile {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).update_pet_profile` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
        arg_7: soroban_sdk::Val,
        arg_8: soroban_sdk::Val,
        arg_9: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::update_pet_profile(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_4),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_5),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_6),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_7),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_8),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_9),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).update_pet_profile` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
        arg_7: soroban_sdk::Val,
        arg_8: soroban_sdk::Val,
        arg_9: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)]
        invoke_raw(
            env,
            arg_0,
            arg_1,
            arg_2,
            arg_3,
            arg_4,
            arg_5,
            arg_6,
            arg_7,
            arg_8,
            arg_9,
        )
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_pet {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pet` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_pet(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pet` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __is_pet_active {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).is_pet_active` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::is_pet_active(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).is_pet_active` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_pet_owner {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pet_owner` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_pet_owner(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pet_owner` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __activate_pet {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).activate_pet` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::activate_pet(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).activate_pet` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __deactivate_pet {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).deactivate_pet` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::deactivate_pet(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).deactivate_pet` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __add_pet_photo {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_pet_photo` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::add_pet_photo(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_pet_photo` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_pet_photos {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pet_photos` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_pet_photos(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pet_photos` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __transfer_pet_ownership {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).transfer_pet_ownership` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::transfer_pet_ownership(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).transfer_pet_ownership` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __accept_pet_transfer {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).accept_pet_transfer` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::accept_pet_transfer(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).accept_pet_transfer` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __register_pet_owner {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).register_pet_owner` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::register_pet_owner(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).register_pet_owner` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2, arg_3)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __is_owner_registered {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).is_owner_registered` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::is_owner_registered(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).is_owner_registered` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __update_owner_profile {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).update_owner_profile` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::update_owner_profile(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).update_owner_profile` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2, arg_3)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __register_vet {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).register_vet` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::register_vet(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).register_vet` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2, arg_3)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __verify_vet {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).verify_vet` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::verify_vet(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).verify_vet` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __revoke_vet_license {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).revoke_vet_license` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::revoke_vet_license(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).revoke_vet_license` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __is_verified_vet {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).is_verified_vet` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::is_verified_vet(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).is_verified_vet` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_vet {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_vet` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_vet(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_vet` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_vet_by_license {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_vet_by_license` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_vet_by_license(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_vet_by_license` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
/// Update clinic info for a vet. Only the vet can update their own clinic info.
pub mod __update_clinic_info {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).update_clinic_info` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::update_clinic_info(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).update_clinic_info` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
pub mod __add_vaccination {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_vaccination` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::add_vaccination(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_4),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_5),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_6),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_vaccination` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)]
        invoke_raw(env, arg_0, arg_1, arg_2, arg_3, arg_4, arg_5, arg_6)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_vaccinations {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_vaccinations` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_vaccinations(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_vaccinations` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_vaccination_history {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_vaccination_history` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_vaccination_history(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_vaccination_history` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_upcoming_vaccinations {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_upcoming_vaccinations` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_upcoming_vaccinations(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_upcoming_vaccinations` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __is_vaccination_current {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).is_vaccination_current` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::is_vaccination_current(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).is_vaccination_current` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_overdue_vaccinations {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_overdue_vaccinations` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_overdue_vaccinations(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_overdue_vaccinations` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __link_tag_to_pet {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).link_tag_to_pet` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::link_tag_to_pet(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).link_tag_to_pet` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_pet_by_tag {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pet_by_tag` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_pet_by_tag(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pet_by_tag` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_tag {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_tag` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_tag(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_tag` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_tag_by_pet {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_tag_by_pet` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_tag_by_pet(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_tag_by_pet` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __update_tag_message {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).update_tag_message` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::update_tag_message(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).update_tag_message` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __deactivate_tag {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).deactivate_tag` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::deactivate_tag(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).deactivate_tag` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __reactivate_tag {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).reactivate_tag` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::reactivate_tag(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).reactivate_tag` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __is_tag_active {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).is_tag_active` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::is_tag_active(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).is_tag_active` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_ownership_history {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_ownership_history` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_ownership_history(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_ownership_history` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __set_emergency_contacts {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).set_emergency_contacts` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::set_emergency_contacts(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).set_emergency_contacts` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_emergency_info {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_emergency_info` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_emergency_info(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_emergency_info` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
/// Get emergency contacts for a pet (publicly accessible - no auth required for emergency responders)
pub mod __get_emergency_contacts {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_emergency_contacts` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_emergency_contacts(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_emergency_contacts` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_accessible_pets {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_accessible_pets` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_accessible_pets(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_accessible_pets` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_all_pets_by_owner {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_all_pets_by_owner` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_all_pets_by_owner(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_all_pets_by_owner` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_pets_by_owner {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pets_by_owner` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_pets_by_owner(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pets_by_owner` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_pets_by_species {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pets_by_species` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_pets_by_species(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pets_by_species` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_active_pets {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_active_pets` instead"
    )]
    pub fn invoke_raw(env: soroban_sdk::Env) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_active_pets(env.clone()),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_active_pets` instead"
    )]
    pub extern "C" fn invoke_raw_extern(env: soroban_sdk::Env) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __grant_access {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).grant_access` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::grant_access(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).grant_access` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2, arg_3)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __revoke_access {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).revoke_access` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::revoke_access(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).revoke_access` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __grant_temporary_custody {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).grant_temporary_custody` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::grant_temporary_custody(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_4),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).grant_temporary_custody` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2, arg_3, arg_4)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __revoke_temporary_custody {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).revoke_temporary_custody` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::revoke_temporary_custody(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).revoke_temporary_custody` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __is_custody_valid {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).is_custody_valid` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::is_custody_valid(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).is_custody_valid` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __add_medical_record {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_medical_record` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::add_medical_record(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_4),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_5),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_medical_record` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2, arg_3, arg_4, arg_5)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __update_medical_record {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).update_medical_record` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::update_medical_record(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_4),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).update_medical_record` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2, arg_3, arg_4)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_medical_record {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_medical_record` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_medical_record(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_medical_record` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_pet_medical_records {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pet_medical_records` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_pet_medical_records(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pet_medical_records` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_access_logs {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_access_logs` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_access_logs(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_access_logs` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __check_access {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).check_access` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::check_access(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).check_access` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_authorized_users {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_authorized_users` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_authorized_users(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_authorized_users` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_access_grant {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_access_grant` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_access_grant(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_access_grant` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __add_lab_result {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_lab_result` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::add_lab_result(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_4),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_5),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_6),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_lab_result` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)]
        invoke_raw(env, arg_0, arg_1, arg_2, arg_3, arg_4, arg_5, arg_6)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_lab_result {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_lab_result` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_lab_result(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_lab_result` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_lab_results {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_lab_results` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_lab_results(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_lab_results` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
pub mod __add_medication_to_record {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_medication_to_record` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::add_medication_to_record(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_4),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_5),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_6),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_medication_to_record` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)]
        invoke_raw(env, arg_0, arg_1, arg_2, arg_3, arg_4, arg_5, arg_6)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __mark_record_med_completed {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).mark_record_med_completed` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::mark_record_med_completed(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).mark_record_med_completed` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_active_record_meds {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_active_record_meds` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_active_record_meds(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_active_record_meds` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_record_med_history {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_record_med_history` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_record_med_history(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_record_med_history` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __batch_add_vaccinations {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).batch_add_vaccinations` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::batch_add_vaccinations(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).batch_add_vaccinations` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __batch_add_records {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).batch_add_records` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::batch_add_records(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).batch_add_records` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
/// Report a pet as lost
pub mod __report_lost {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).report_lost` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::report_lost(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).report_lost` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2)
    }
    use super::*;
}
#[doc(hidden)]
/// Report a sighting of a lost pet
pub mod __report_sighting {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).report_sighting` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::report_sighting(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).report_sighting` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2)
    }
    use super::*;
}
#[doc(hidden)]
/// Mark a lost pet as found
pub mod __report_found {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).report_found` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::report_found(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).report_found` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
/// Cancel a lost pet alert
pub mod __cancel_lost_alert {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).cancel_lost_alert` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::cancel_lost_alert(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).cancel_lost_alert` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
/// Get all active lost pet alerts
pub mod __get_active_alerts {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_active_alerts` instead"
    )]
    pub fn invoke_raw(env: soroban_sdk::Env) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_active_alerts(env.clone()),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_active_alerts` instead"
    )]
    pub extern "C" fn invoke_raw_extern(env: soroban_sdk::Env) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env)
    }
    use super::*;
}
#[doc(hidden)]
/// Get a specific alert by ID
pub mod __get_alert {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_alert` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_alert(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_alert` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
/// Get sightings for a specific alert
pub mod __get_alert_sightings {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_alert_sightings` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_alert_sightings(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_alert_sightings` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
/// Get alerts for a specific pet
pub mod __get_pet_alerts {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pet_alerts` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_pet_alerts(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pet_alerts` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
/// Set availability slots for a vet (only verified vets can set their availability)
pub mod __set_availability {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).set_availability` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::set_availability(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).set_availability` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2)
    }
    use super::*;
}
#[doc(hidden)]
/// Get available slots for a vet on a specific date
pub mod __get_available_slots {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_available_slots` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_available_slots(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_available_slots` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __grant_consent {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).grant_consent` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::grant_consent(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).grant_consent` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2, arg_3)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __revoke_consent {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).revoke_consent` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::revoke_consent(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).revoke_consent` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_consent_history {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_consent_history` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_consent_history(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_consent_history` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
/// Book a slot (mark as unavailable)
pub mod __book_slot {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).book_slot` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::book_slot(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).book_slot` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_version {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_version` instead"
    )]
    pub fn invoke_raw(env: soroban_sdk::Env) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_version(env.clone()),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_version` instead"
    )]
    pub extern "C" fn invoke_raw_extern(env: soroban_sdk::Env) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __upgrade_contract {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).upgrade_contract` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::upgrade_contract(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).upgrade_contract` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __propose_upgrade {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).propose_upgrade` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::propose_upgrade(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).propose_upgrade` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __approve_upgrade {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).approve_upgrade` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::approve_upgrade(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).approve_upgrade` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_upgrade_proposal {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_upgrade_proposal` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_upgrade_proposal(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_upgrade_proposal` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __migrate_version {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).migrate_version` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::migrate_version(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).migrate_version` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __propose_action {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).propose_action` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::propose_action(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).propose_action` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __approve_proposal {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).approve_proposal` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::approve_proposal(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).approve_proposal` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __execute_proposal {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).execute_proposal` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::execute_proposal(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).execute_proposal` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_proposal {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_proposal` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_proposal(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_proposal` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __add_vet_review {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_vet_review` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::add_vet_review(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_vet_review` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1, arg_2, arg_3)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_vet_reviews {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_vet_reviews` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_vet_reviews(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_vet_reviews` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_vet_average_rating {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_vet_average_rating` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_vet_average_rating(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_vet_average_rating` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __add_medication {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_medication` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::add_medication(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_4),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_5),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_6),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_medication` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)]
        invoke_raw(env, arg_0, arg_1, arg_2, arg_3, arg_4, arg_5, arg_6)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_active_medications {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_active_medications` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_active_medications(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_active_medications` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __mark_medication_completed {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).mark_medication_completed` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::mark_medication_completed(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).mark_medication_completed` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __add_treatment {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_treatment` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::add_treatment(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_4),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_5),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_6),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_treatment` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)]
        invoke_raw(env, arg_0, arg_1, arg_2, arg_3, arg_4, arg_5, arg_6)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_treatment {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_treatment` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_treatment(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_treatment` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_treatment_history {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_treatment_history` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_treatment_history(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_treatment_history` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_treatments_by_type {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_treatments_by_type` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_treatments_by_type(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_treatments_by_type` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __add_insurance_policy {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_insurance_policy` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::add_insurance_policy(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_2),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_3),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_4),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_5),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_6),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).add_insurance_policy` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
        arg_2: soroban_sdk::Val,
        arg_3: soroban_sdk::Val,
        arg_4: soroban_sdk::Val,
        arg_5: soroban_sdk::Val,
        arg_6: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)]
        invoke_raw(env, arg_0, arg_1, arg_2, arg_3, arg_4, arg_5, arg_6)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __get_pet_insurance {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pet_insurance` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::get_pet_insurance(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).get_pet_insurance` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0)
    }
    use super::*;
}
#[doc(hidden)]
pub mod __update_insurance_status {
    use super::*;
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).update_insurance_status` instead"
    )]
    pub fn invoke_raw(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        <_ as soroban_sdk::IntoVal<
            soroban_sdk::Env,
            soroban_sdk::Val,
        >>::into_val(
            #[allow(deprecated)]
            &<super::PetChainContract>::update_insurance_status(
                env.clone(),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_0),
                ),
                <_ as soroban_sdk::unwrap::UnwrapOptimized>::unwrap_optimized(
                    <_ as soroban_sdk::TryFromValForContractFn<
                        soroban_sdk::Env,
                        soroban_sdk::Val,
                    >>::try_from_val_for_contract_fn(&env, &arg_1),
                ),
            ),
            &env,
        )
    }
    #[deprecated(
        note = "use `PetChainContractClient::new(&env, &contract_id).update_insurance_status` instead"
    )]
    pub extern "C" fn invoke_raw_extern(
        env: soroban_sdk::Env,
        arg_0: soroban_sdk::Val,
        arg_1: soroban_sdk::Val,
    ) -> soroban_sdk::Val {
        #[allow(deprecated)] invoke_raw(env, arg_0, arg_1)
    }
    use super::*;
}
fn encrypt_sensitive_data(env: &Env, data: &Bytes, _key: &Bytes) -> (Bytes, Bytes) {
    let nonce = Bytes::from_array(env, &[0u8; 12]);
    let ciphertext = data.clone();
    (nonce, ciphertext)
}
fn decrypt_sensitive_data(
    _env: &Env,
    ciphertext: &Bytes,
    _nonce: &Bytes,
    _key: &Bytes,
) -> Result<Bytes, ()> {
    Ok(ciphertext.clone())
}
