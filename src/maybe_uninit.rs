

use std::mem::ManuallyDrop;
use std::mem::uninitialized;

/// A combination of ManuallyDrop and “maybe uninitialized”;
/// this wraps a value that can be wholly or partially uninitialized;
/// it also has no drop regardless of the type of T.
pub struct MaybeUninit<T>(ManuallyDrop<T>);

impl<T> MaybeUninit<T> {
    /// Create a new MaybeUninit with uninitialized interior
    pub unsafe fn uninitialized() -> Self {
        Self::from(uninitialized())
    }

    /// Create a new MaybeUninit from the value `v`.
    pub fn from(v: T) -> Self {
        MaybeUninit(ManuallyDrop::new(v))
    }

    /// Return a raw pointer to the interior
    pub fn ptr(&self) -> *const T {
        (&self.0) as *const ManuallyDrop<_> as *const T
    }

    /// Return a raw pointer to the interior (mutable)
    pub fn ptr_mut(&mut self) -> *mut T {
        (&mut self.0) as *mut ManuallyDrop<_> as *mut T
    }
}



