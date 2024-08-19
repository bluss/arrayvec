//! **arrayvec** provides the types [`ArrayVec`] and [`ArrayString`]: 
//! array-backed vector and string types, which store their contents inline.
//!
//! The arrayvec package has the following cargo features:
//!
//! - `std`
//!   - Optional, enabled by default
//!   - Use libstd; disable to use `no_std` instead.
//!
//! - `len_u16`
//!   - Optional.
//!   - Use `u16` as length type.
//!
//! - `len_u8`
//!   - Optional.
//!   - Use `u8` as length type.
//!
//! - `serde`
//!   - Optional
//!   - Enable serialization for ArrayVec and ArrayString using serde 1.x
//!
//! - `zeroize`
//!   - Optional
//!   - Implement `Zeroize` for ArrayVec and ArrayString
//!
//! ## Rust Version
//!
//! This version of arrayvec requires Rust 1.51 or later.
//!
#![doc(html_root_url="https://docs.rs/arrayvec/0.7/")]
#![cfg_attr(not(feature="std"), no_std)]

#[cfg(feature="serde")]
extern crate serde;

#[cfg(not(feature="std"))]
extern crate core as std;

#[cfg(all(not(feature="len_u8"), not(feature="len_u16")))]
pub(crate) type LenUint = u32;

#[cfg(feature="len_u16")]
pub(crate) type LenUint = u16;

#[cfg(feature="len_u8")]
pub(crate) type LenUint = u8;

macro_rules! assert_capacity_limit {
    ($cap:expr) => {
        if std::mem::size_of::<usize>() > std::mem::size_of::<LenUint>() {
            if $cap > LenUint::MAX as usize {
                panic!("ArrayVec: largest supported capacity is {}", LenUint::MAX)
            }
        }
    }
}

macro_rules! assert_capacity_limit_const {
    ($cap:expr) => {
        if std::mem::size_of::<usize>() > std::mem::size_of::<LenUint>() {
            if $cap > LenUint::MAX as usize {
                [/*ArrayVec: largest supported capacity is LenUint::MAX*/][$cap]
            }
        }
    }
}

mod arrayvec_impl;
mod arrayvec;
mod array_string;
mod char;
mod errors;
mod utils;

pub use crate::array_string::ArrayString;
pub use crate::errors::CapacityError;

pub use crate::arrayvec::{ArrayVec, IntoIter, Drain};
