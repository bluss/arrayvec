use std::fmt;
#[cfg(feature="std")]
use std::any::Any;
#[cfg(feature="std")]
use std::error::Error;

/// Error value indicating insufficient capacity
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct CapacityError<T = ()> {
    element: T,
}

pub trait PubCrateNew<T> {
    fn new(elt: T) -> Self;
}

impl<T> PubCrateNew<T> for CapacityError<T> {
    fn new(element: T) -> CapacityError<T> {
        CapacityError {
            element: element,
        }
    }
}

impl<T> CapacityError<T> {
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

pub struct OutOfBoundsError {
    _priv: ()
}

impl PubCrateNew<()> for OutOfBoundsError {
    fn new(_: ()) -> Self {
        OutOfBoundsError { _priv: () }
    }
}


impl OutOfBoundsError {
    fn description(&self) -> &'static str {
        "remove index is out of bounds"
    }
}

#[cfg(feature="std")]
/// Requires `features="std"`.
impl Error for OutOfBoundsError {
    fn description(&self) -> &str {
        self.description()
    }
}

impl fmt::Display for OutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl fmt::Debug for OutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OutOfBoundsError: {}", self.description())
    }
}
