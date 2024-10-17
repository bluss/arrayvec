//! **arrayvec** provides the types [`ArrayVec`] and [`ArrayString`]: 
//! array-backed vector and string types, which store their contents inline.
//!
//! The arrayvec package has the following cargo features:
//!
//! - `std`
//!   - Optional, enabled by default
//!   - Use libstd; disable to use `no_std` instead.
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

#[cfg(not(target_pointer_width = "16"))]
pub(crate) type LenUint = u32;

#[cfg(target_pointer_width = "16")]
pub(crate) type LenUint = u16;

macro_rules! assert_capacity_limit {
    ($cap:expr) => {
        if std::mem::size_of::<usize>() > std::mem::size_of::<LenUint>() {
            if $cap > LenUint::MAX as usize {
                #[cfg(not(target_pointer_width = "16"))]
                panic!("ArrayVec: largest supported capacity is u32::MAX");
                #[cfg(target_pointer_width = "16")]
                panic!("ArrayVec: largest supported capacity is u16::MAX");
            }
        }
    }
}

macro_rules! assert_capacity_limit_const {
    ($cap:expr) => {
        if std::mem::size_of::<usize>() > std::mem::size_of::<LenUint>() {
            if $cap > LenUint::MAX as usize {
                [/*ArrayVec: largest supported capacity is u32::MAX*/][$cap]
            }
        }
    }
}

macro_rules! assert_length_lt_capacity_const {
    ($len:expr, $cap:expr) => {
        if $len > 0 {
            let _len_check = [(); $cap /*ArrayVec/ArrayString: insufficient capacity */][$len - 1];
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
