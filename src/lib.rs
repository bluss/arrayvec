//! **arrayvec** provides the types `ArrayVec` and `ArrayString`: 
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
//! - `array-sizes-33-128`, `array-sizes-129-255`
//!   - Optional
//!   - Enable more array sizes (see [Array] for more information)
//!
//! - `unstable-const-fn`
//!   - Optional
//!   - Makes [`ArrayVec::new`] and [`ArrayString::new`] `const fn`s,
//!     using the nightly `const_fn` feature.
//!   - Unstable and requires nightly.
//!
//! ## Rust Version
//!
//! This version of arrayvec requires Rust 1.36 or later.
//!
#![doc(html_root_url="https://docs.rs/arrayvec/0.5/")]
#![cfg_attr(not(feature="std"), no_std)]
#![cfg_attr(feature="unstable-const-fn", feature(const_fn))]

#[cfg(feature="serde")]
extern crate serde;

#[cfg(not(feature="std"))]
extern crate core as std;

mod maybe_uninit;
use crate::maybe_uninit::MaybeUninit;

#[cfg(feature="serde")]
use serde::{Serialize, Deserialize, Serializer, Deserializer};

mod array;
mod arrayvec_impl;
mod arrayvec;
mod array_string;
mod char;
mod errors;

pub use crate::array::Array;
pub use crate::array_string::ArrayString;
pub use crate::errors::CapacityError;

pub use crate::arrayvec::{ArrayVec, IntoIter, Drain};
