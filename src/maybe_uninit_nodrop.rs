
use array::Array;
use nodrop::NoDrop;
use std::mem::uninitialized;

/// A combination of NoDrop and “maybe uninitialized”;
/// this wraps a value that can be wholly or partially uninitialized.
pub struct MaybeUninit<T>(NoDrop<T>);

impl<T> MaybeUninit<T> {
    /// Create a new MaybeUninit with uninitialized interior
    pub unsafe fn uninitialized() -> Self {
        Self::from(uninitialized())
    }

    /// Create a new MaybeUninit from the value `v`.
    pub fn from(v: T) -> Self {
        MaybeUninit(NoDrop::new(v))
    }

    /// Return a raw pointer to the start of the interior array
    pub fn ptr(&self) -> *const T::Item
        where T: Array
    {
        self.0.as_ptr()
    }

    /// Return a mut raw pointer to the start of the interior array
    pub fn ptr_mut(&mut self) -> *mut T::Item
        where T: Array
    {
        self.0.as_mut_ptr()
    }
}



