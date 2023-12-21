use std::ops::{Deref, DerefMut};
use crate::{ArrayVec, CapacityError};

// ArrayVecRef is useless because we have Deref<[T}> already

/// A mutable reference to an ArrayVec
/// It gives you all the access of a mutable reference to an ArrayVec
/// instead only a few methods exposed by ArrayVecImpl
pub struct ArrayVecRefMut<'a, T> {
    cap: usize,
    vec: &'a mut ArrayVec<T, 0>,
}

impl<'a, T> ArrayVecRefMut<'a, T> {
    pub fn new<const CAP: usize>(vec: &'a mut ArrayVec<T, CAP>) -> Self {
        unsafe {
            Self { cap: CAP, vec: std::mem::transmute(vec) }
        }
    }
    pub const fn capacity(&self) -> usize { self.cap }
    pub const fn is_full(&self) -> bool { self.vec.len() == self.capacity() }
    pub const fn remaining_capacity(&self) -> usize {
        self.capacity() - self.vec.len()
    }
    #[track_caller]
    pub fn push(&mut self, element: T) {
        self.try_push(element).unwrap()
    }

    pub fn try_push(&mut self, element: T) -> Result<(), CapacityError<T>> {
        if self.len() < self.capacity() {
            unsafe {
                self.vec.push_unchecked(element);
            }
            Ok(())
        } else {
            Err(CapacityError::new(element))
        }
    }

}


impl<'a, T> Deref for ArrayVecRefMut<'a, T> {
    type Target = ArrayVec<T, 0>;
    fn deref(&self) -> &Self::Target {
        self.vec
    }
}

impl<'a, T> DerefMut for ArrayVecRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}