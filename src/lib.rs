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

pub trait LenUint: Add + Sub + Copy + PartialOrd + PartialEq + private::Sealed  {
    const MAX: usize;
    const ZERO: Self;
    fn from_usize(n: usize) -> Self;
    fn to_usize(self) -> usize;
}
macro_rules! impl_lenuint {
    ($ty: path) => {
        impl $crate::private::Sealed for $ty {}
        impl $crate::LenUint for $ty {
            const MAX: usize = <$ty>::MAX as usize;
            const ZERO: Self = 0;
            fn from_usize(n: usize) -> Self { n as $ty }
            fn to_usize(self) -> usize { self as usize }
        }
    };
}
mod private {
    pub trait Sealed {}
    impl_lenuint!(u8);
    impl_lenuint!(u16);
    impl_lenuint!(u32);
    #[cfg(target_pointer_width = "64")]
    impl_lenuint!(u64);
    impl_lenuint!(usize);
}
macro_rules! assert_capacity_limit {
    ($ty: path, $cap:expr) => {
        if $cap > <$ty as LenUint>::MAX {
            panic!("ArrayVec: capacity {} is too large for {}::MAX={}", CAP, std::any::type_name::<$ty>(), <$ty as LenUint>::MAX)
        }
    }
}

macro_rules! assert_capacity_limit_const {
    ($ty: path, $cap:expr) => {
        if $cap > <$ty as LenUint>::MAX {
            panic!("ArrayVec: capacity is too large for LenUint::MAX")
        }
    }
}
pub type DefaultLenUint = u32;
mod arrayvec_impl;
mod arrayvec;
mod array_string;
mod char;
mod errors;
mod utils;

use core::ops::Sub;
use std::ops::Add;
pub use crate::array_string::ArrayString;
pub use crate::errors::CapacityError;

pub use crate::arrayvec::{ArrayVec, IntoIter, Drain};
