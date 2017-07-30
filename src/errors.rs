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

impl<T> CapacityError<T> {
    pub(crate) fn new(element: T) -> CapacityError<T> {
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

pub enum InsertError<T> {
    Full(T),
    OutOfBounds,
}

impl<T> InsertError<T> {
    fn description(&self) -> &'static str {
        match *self {
            InsertError::Full(_) => "ArrayVec is already at full capacity",
            InsertError::OutOfBounds => "index is out of bounds",
        }
    }
}

#[cfg(feature="std")]
/// Requires `features="std"`.
impl<T: Any> Error for InsertError<T> {
    fn description(&self) -> &str {
        self.description()
    }
}

impl<T> fmt::Display for InsertError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl<T> fmt::Debug for InsertError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InsertError::Full(_) => write!(f, "InsertError::Full: ")?,
            InsertError::OutOfBounds => write!(f, "InsertError::OutOfBounds: ")?,
        }
        write!(f, "{}", self.description())
    }
}

pub struct RemoveError {
    _priv: ()
}

impl RemoveError {
    pub(crate) fn new() -> Self {
        RemoveError { _priv: () }
    }

    fn description(&self) -> &'static str {
        "remove index is out of bounds"
    }
}

#[cfg(feature="std")]
/// Requires `features="std"`.
impl Error for RemoveError {
    fn description(&self) -> &str {
        self.description()
    }
}

impl fmt::Display for RemoveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl fmt::Debug for RemoveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RemoveError: {}", self.description())
    }
}
