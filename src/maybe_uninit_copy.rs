
use array::Array;

#[derive(Copy, Clone)]
pub union MaybeUninitCopy<T>
    where T: Copy
{
    empty: (),
    value: T,
}

impl<T> MaybeUninitCopy<T>
    where T: Copy
{
    /// Create a new MaybeUninit with uninitialized interior
    pub unsafe fn uninitialized() -> Self {
        Self { empty: () }
    }

    /// Create a new MaybeUninit from the value `v`.
    pub fn from(value: T) -> Self {
        Self { value }
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

