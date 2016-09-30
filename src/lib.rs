//! **arrayvec** provides the types `ArrayVec` and `ArrayString`:
//! array-backed vector and string types, which store their contents inline.
//!
//! The **arrayvec** crate has the following cargo feature flags:
//!
//! - `std`
//!   - Optional, enabled by default
//!   - Requires Rust 1.6 *to disable*
//!   - Use libstd
//!
//! - `use_union`
//!   - Optional
//!   - Requires Rust nightly channel
//!   - Use the unstable feature untagged unions for the internal implementation,
//!     which has reduced space overhead
//!
//! - `use_generic_array`
//!   - Optional
//!   - Requires Rust stable channel
//!   - Depend on generic-array and allow using it just like a fixed
//!     size array for ArrayVec storage.
#![cfg_attr(not(feature="std"), no_std)]

extern crate odds;
extern crate nodrop;

#[cfg(feature = "use_generic_array")]
extern crate generic_array;

#[cfg(not(feature="std"))]
extern crate core as std;

use std::fmt;
#[cfg(feature="std")]
use std::error::Error;
#[cfg(feature="std")]
use std::any::Any; // core but unused

mod array;
mod copy;
mod string;
mod vec;
mod raw;

pub use array::Array;
pub use copy::ArrayVecCopy;
pub use odds::IndexRange as RangeArgument;
pub use raw::Drain;
pub use string::ArrayString;
pub use vec::ArrayVec;
pub use vec::IntoIter;

/// Error value indicating insufficient capacity
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct CapacityError<T = ()> {
    element: T,
}

impl<T> CapacityError<T> {
    fn new(element: T) -> CapacityError<T> {
        CapacityError {
            element: element,
        }
    }

    /// Extract the overflowing element
    pub fn element(self) -> T {
        self.element
    }

    /// Convert into a `CapacityError` that does not carry an element.
    pub fn simplify(self) -> CapacityError {
        CapacityError { element: () }
    }
}

const CAPERROR: &'static str = "insufficient capacity";

#[cfg(feature="std")]
/// Requires `features="std"`.
impl<T: Any> Error for CapacityError<T> {
    fn description(&self) -> &str {
        CAPERROR
    }
}

impl<T> fmt::Display for CapacityError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", CAPERROR)
    }
}

impl<T> fmt::Debug for CapacityError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", "CapacityError", CAPERROR)
    }
}
