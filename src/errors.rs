use std::fmt;
#[cfg(feature="std")]
use std::any::Any;
#[cfg(feature="std")]
use std::error::Error;

use crate::ArrayVec;

/// Error value indicating insufficient capacity
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct CapacityError<T = ()> {
    element: T,
}

impl<T> CapacityError<T> {
    /// Create a new `CapacityError` from `element`.
    pub const fn new(element: T) -> CapacityError<T> {
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
impl<T: Any> Error for CapacityError<T> {}

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

/// Error value indicating that capacity is not completely filled
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct UnderfilledError<T, const CAP: usize>(ArrayVec<T, CAP>);

impl<T, const CAP: usize> UnderfilledError<T, CAP> {
    pub const fn new(inner: ArrayVec<T, CAP>) -> Self {
        Self(inner)
    }

    pub fn take_vec(self) -> ArrayVec<T, CAP> {
        self.0
    }
}

impl<T, const CAP: usize> fmt::Debug for UnderfilledError<T, CAP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "UnderfilledError: capacity is not filled: expected {}, got {}",
            CAP,
            self.0.len()
        )
    }
}

#[cfg(feature="std")]
impl<T, const CAP: usize> Error for UnderfilledError<T, CAP> {}

impl<T, const CAP: usize> fmt::Display for UnderfilledError<T, CAP> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "capacity is not filled: expected {}, got {}",
            CAP,
            self.0.len()
        )
    }
}