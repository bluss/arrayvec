

use std::mem::uninitialized;
use std::mem::ManuallyDrop;

#[repr(C)]
pub(crate) struct MaybeUninit<T>(ManuallyDrop<T>);

impl<T> MaybeUninit<T> {
    pub unsafe fn uninitialized() -> Self {
        uninitialized()
    }

    pub fn from(value: T) -> Self {
        MaybeUninit(ManuallyDrop::new(value))
    }

    pub fn ptr(&self) -> *const T {
        self as *const _ as _
    }
    pub fn ptr_mut(&mut self) -> *mut T {
        self as *mut _ as _
    }
}

