

use array::Array;
use std::mem::ManuallyDrop;
use std::mem::uninitialized;

/// A combination of ManuallyDrop and “maybe uninitialized”;
/// this wraps a value that can be wholly or partially uninitialized;
/// it also has no drop regardless of the type of T.
#[repr(C)]
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

    // Raw pointer casts written so that we don't reference or access the
    // uninitialized interior value

    /// Return a raw pointer to the start of the interior array
    pub fn ptr(&self) -> *const T::Item
        where T: Array
    {
        self as *const _ as *const T::Item
    }

    /// Return a mut raw pointer to the start of the interior array
    pub fn ptr_mut(&mut self) -> *mut T::Item
        where T: Array
    {
        self as *mut _ as *mut T::Item
    }
}



#[test]
fn test_offset() {
    use std::ptr;

    let mut mu = MaybeUninit::from([1, 2, 3]);
    assert!(ptr::eq(mu.ptr(), &mu.0[0]));
    assert!(ptr::eq(mu.ptr_mut(), &mut mu.0[0]));
}

#[test]
#[cfg(feature = "std")]
fn test_offset_string() {
    use std::ptr;

    let s = String::from;
    let mut mu = MaybeUninit::from([s("a"), s("b")]);
    assert!(ptr::eq(mu.ptr(), &mu.0[0]));
    assert!(ptr::eq(mu.ptr_mut(), &mut mu.0[0]));
}
